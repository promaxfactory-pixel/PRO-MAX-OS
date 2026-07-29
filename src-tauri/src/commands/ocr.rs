use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::State;

use crate::db::DbState;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub file_path: String,
    pub file_size: i64,
    pub raw_text: String,
    pub confidence: f64,
    pub suggested_fields: OcrInvoiceData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrInvoiceData {
    pub invoice_number: Option<String>,
    pub date: Option<String>,
    pub customer_name: Option<String>,
    pub total_amount: Option<f64>,
    pub vat_amount: Option<f64>,
    pub net_amount: Option<f64>,
    pub items: Vec<OcrLineItem>,
    pub raw_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrLineItem {
    pub description: String,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub total: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractionResult {
    pub fields: Vec<ExtractedField>,
    pub items: Vec<LineItemResult>,
    pub raw_text: String,
    pub vendor_name: String,
    pub invoice_number: String,
    pub date: String,
    pub subtotal: f64,
    pub vat: f64,
    pub total: f64,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractedField {
    pub key: String,
    pub label: String,
    pub value: String,
    pub confidence: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineItemResult {
    pub description: String,
    pub qty: f64,
    pub unit_price: f64,
    pub total: f64,
    pub confidence: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Suggestion {
    pub id: String,
    pub label: String,
    pub description: String,
    pub action_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrScan {
    pub id: i64,
    pub file_name: String,
    pub file_path: String,
    pub extracted_text: String,
    pub parsed_data: String,
    pub confidence: f64,
    pub status: String,
    pub created_at: String,
}

fn get_file_extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn is_image_file(path: &str) -> bool {
    let ext = get_file_extension(path);
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "gif" | "tiff" | "tif" | "webp")
}

fn read_file_as_bytes(path: &str) -> Result<Vec<u8>, AppError> {
    Ok(fs::read(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?)
}

fn extract_text_from_pdf(path: &str) -> Result<String, AppError> {
    let bytes = read_file_as_bytes(path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("PDF extraction failed: {}", e))?;
    Ok(text)
}

fn try_run_tesseract(path: &str) -> Option<String> {
    let tesseract_paths = [
        "tesseract",
        "C:\\Program Files\\Tesseract-OCR\\tesseract.exe",
        "C:\\Program Files (x86)\\Tesseract-OCR\\tesseract.exe",
    ];
    for tp in &tesseract_paths {
        if let Ok(output) = std::process::Command::new(tp)
            .args([path, "stdout", "-l", "ara+eng", "--psm", "3"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                if !text.trim().is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn extract_text_from_image(path: &str) -> (String, f64) {
    if let Some(tesseract_text) = try_run_tesseract(path) {
        if tesseract_text.trim().len() > 20 {
            return (tesseract_text, 70.0);
        }
    }

    let data = match read_file_as_bytes(path) {
        Ok(d) => d,
        Err(_) => return (String::new(), 0.0),
    };

    let ext = get_file_extension(path);
    let mut raw_text = String::new();
    let mut confidence = 0.0;

    if ext == "png" {
        if let Some(png_text) = basic_png_text_extraction(&data) {
            raw_text.push_str(&png_text);
            confidence += 40.0;
        }
    }

    if raw_text.is_empty() {
        let readable = extract_possible_text_from_binary(&data);
        if !readable.trim().is_empty() {
            raw_text.push_str(&readable);
            confidence += 20.0;
        }
    }

    if raw_text.is_empty() {
        if let Some(text_from_file) = try_read_text_from_file(path) {
            raw_text.push_str(&text_from_file);
            confidence += 15.0;
        }
    }

    if raw_text.is_empty() {
        raw_text = format!(
            "[Image file - Tesseract not available]\nPath: {}\nSize: {} bytes\nTip: Install Tesseract OCR from https://github.com/UB-Mannheim/tesseract/wiki for full OCR support.",
            path, data.len()
        );
        confidence = 5.0;
    }

    (raw_text, confidence)
}

fn basic_png_text_extraction(data: &[u8]) -> Option<String> {
    if data.len() < 8 { return None; }
    if data[0] != 0x89 || data[1] != 0x50 || data[2] != 0x4E || data[3] != 0x47 { return None; }
    let mut extracted = Vec::new();
    let mut i = 8;
    while i + 8 <= data.len() {
        let chunk_len = ((data[i] as u32) << 24)
            | ((data[i + 1] as u32) << 16)
            | ((data[i + 2] as u32) << 8)
            | (data[i + 3] as u32);
        let chunk_type = &data[i + 4..i + 8];
        if chunk_type == b"tEXt" || chunk_type == b"iTXt" || chunk_type == b"zTXt" {
            let start = i + 8;
            let end = std::cmp::min(start + chunk_len as usize, data.len());
            if let Ok(text) = String::from_utf8(data[start..end].to_vec()) {
                let cleaned: String = text.chars().filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n' || *c == '\r' || *c == '\t').collect();
                if !cleaned.trim().is_empty() {
                    extracted.push(cleaned);
                }
            }
        }
        i += 12 + chunk_len as usize;
        if chunk_len == 0 { break; }
    }
    if extracted.is_empty() { None } else { Some(extracted.join("\n")) }
}

fn extract_possible_text_from_binary(data: &[u8]) -> String {
    let mut result = String::new();
    let mut current_run = Vec::new();
    for &byte in data.iter() {
        if (0x20..0x7F).contains(&byte) || byte == b'\n' || byte == b'\r' || byte == b'\t' {
            current_run.push(byte);
        } else {
            if current_run.len() >= 4 {
                if let Ok(text) = String::from_utf8(current_run.clone()) {
                    result.push_str(&text);
                    result.push('\n');
                }
            }
            current_run.clear();
        }
    }
    if current_run.len() >= 4 {
        if let Ok(text) = String::from_utf8(current_run) {
            result.push_str(&text);
        }
    }
    result
}

fn try_read_text_from_file(path: &str) -> Option<String> {
    let data = read_file_as_bytes(path).ok()?;
    let text_chunks: Vec<String> = data
        .chunks(4096)
        .filter_map(|chunk| {
            let valid: Vec<u8> = chunk.iter()
                .filter(|&&b| (0x20..0x7F).contains(&b) || b == b'\n' || b == b'\r' || b == b'\t')
                .cloned().collect();
            if valid.len() as f64 > chunk.len() as f64 * 0.7 {
                String::from_utf8(valid).ok()
            } else { None }
        }).collect();
    if text_chunks.is_empty() { None } else { Some(text_chunks.join("")) }
}

fn strip_control_chars(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t').collect()
}

fn find_number_after_keywords(text: &str, keywords: &[&str]) -> Option<f64> {
    let lower = text.to_lowercase();
    for keyword in keywords {
        if let Some(pos) = lower.find(&keyword.to_lowercase()) {
            let after = &text[pos + keyword.len()..];
            let after_trimmed = after.trim_start();
            let mut num_str = String::new();
            let mut found_digit = false;
            let mut has_dot = false;
            for ch in after_trimmed.chars() {
                if ch.is_ascii_digit() { num_str.push(ch); found_digit = true; }
                else if ch == '.' && !has_dot && found_digit { num_str.push(ch); has_dot = true; }
                else if ch == ',' && !has_dot && found_digit { num_str.push('.'); has_dot = true; }
                else if found_digit { break; }
            }
            if let Ok(val) = num_str.parse::<f64>() { return Some(val); }
        }
    }
    None
}

fn find_number_after_line_start(text: &str, keyword: &str) -> Option<f64> {
    let keyword_lower = keyword.to_lowercase();
    for line in text.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains(&keyword_lower) {
            if let Some(pos) = line_lower.find(&keyword_lower) {
                let after = &line[pos + keyword.len()..].trim_start();
                let mut num_str = String::new();
                let mut found_digit = false;
                let mut has_dot = false;
                for ch in after.chars() {
                    if ch.is_ascii_digit() { num_str.push(ch); found_digit = true; }
                    else if ch == '.' && !has_dot && found_digit { num_str.push(ch); has_dot = true; }
                    else if ch == ',' && found_digit { num_str.push('.'); has_dot = true; }
                    else if found_digit { break; }
                }
                if let Ok(val) = num_str.parse::<f64>() { return Some(val); }
            }
        }
    }
    None
}

fn extract_invoice_number(text: &str) -> Option<String> {
    let keywords = [
        "invoice no", "invoice #", "invoice#", "inv no", "inv #", "inv#",
        "bill no", "bill #", "bill#", "receipt no", "receipt #",
        "رقم الفاتورة", "فاتورة رقم", "فاتورة", "fatura",
    ];
    let lower = text.to_lowercase();
    for keyword in &keywords {
        let kw_lower = keyword.to_lowercase();
        if let Some(pos) = lower.find(&kw_lower) {
            let after = &text[pos + keyword.len()..].trim_start();
            let mut value = String::new();
            for ch in after.chars() {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '/' || ch == '\\' { value.push(ch); }
                else if !value.is_empty() { break; }
            }
            if !value.trim().is_empty() && value.trim().len() >= 2 { return Some(value.trim().to_string()); }
        }
    }
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.len() >= 4 && trimmed.len() <= 30 {
            let looks_like_inv = trimmed.starts_with("INV") || trimmed.starts_with("inv")
                || trimmed.starts_with("INV-") || trimmed.starts_with("INV/");
            if looks_like_inv { return Some(trimmed.to_string()); }
        }
    }
    None
}

fn extract_date(text: &str) -> Option<String> {
    let date_patterns = [
        (r"[Dd]ate\s*[:\-]\s*", 20), (r"[Dd]ue\s*[Dd]ate\s*[:\-]\s*", 20),
        (r"[Ii]nvoice\s*[Dd]ate\s*[:\-]\s*", 20), (r"[Bb]ill\s*[Dd]ate\s*[:\-]\s*", 20),
        (r"[Tt]ransaction\s*[Dd]ate\s*[:\-]\s*", 20), ("تاريخ", 20),
    ];
    let lines: Vec<&str> = text.lines().collect();
    for (pattern, _) in &date_patterns {
        for line in &lines {
            let lower_line = line.to_lowercase();
            let pattern_lower = pattern.to_lowercase();
            if lower_line.contains(&pattern_lower) {
                let after_idx = lower_line.find(&pattern_lower).unwrap_or(0) + pattern.len();
                let remaining = line.get(after_idx..).unwrap_or("").trim();
                if let Some(date_str) = extract_date_from_text(remaining) { return Some(date_str); }
            }
        }
    }
    for line in &lines {
        if let Some(date_str) = extract_date_from_text(line) { return Some(date_str); }
    }
    None
}

fn extract_date_from_text(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    for i in 0..len {
        if chars[i].is_ascii_digit() {
            let mut num1 = String::new();
            let mut j = i;
            while j < len && chars[j].is_ascii_digit() { num1.push(chars[j]); j += 1; }
            if j < len && (chars[j] == '/' || chars[j] == '-' || chars[j] == '.') {
                let sep = chars[j]; j += 1;
                let mut num2 = String::new();
                while j < len && chars[j].is_ascii_digit() { num2.push(chars[j]); j += 1; }
                if j < len && chars[j] == sep {
                    j += 1;
                    let mut num3 = String::new();
                    while j < len && chars[j].is_ascii_digit() { num3.push(chars[j]); j += 1; }
                    if !num1.is_empty() && !num2.is_empty() && !num3.is_empty() {
                        let n1: u32 = num1.parse().unwrap_or(0);
                        let n2: u32 = num2.parse().unwrap_or(0);
                        let n3: u32 = num3.parse().unwrap_or(0);
                        let (year, month, day) = if n1 > 31 { (n1, n2, n3) }
                            else if n3 > 31 { (n3, if n1 > 12 { n2 } else { n1 }, if n1 > 12 { n1 } else { n2 }) }
                            else { (n3, n1, n2) };
                        if year < 2200 && (1..=12).contains(&month) && (1..=31).contains(&day) {
                            return Some(format!("{:04}-{:02}-{:02}", if year < 100 { year + 2000 } else { year }, month, day));
                        }
                    }
                }
            }
        }
    }
    let arabic_digits = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];
    let normalized: String = text.chars().map(|c| {
        if let Some(pos) = arabic_digits.iter().position(|&d| d == c) { (b'0' + pos as u8) as char } else { c }
    }).collect();
    if normalized != text { return extract_date_from_text(&normalized); }
    None
}

fn extract_customer_name(text: &str) -> Option<String> {
    let keywords = [
        "customer", "client", "bill to", "billto", "sold to", "soldto",
        "ship to", "shipto", "name", "to:", "عميل", "اسم العميل", "فاتورة على", "المشتري",
    ];
    let lower = text.to_lowercase();
    for keyword in &keywords {
        let kw_lower = keyword.to_lowercase();
        if let Some(pos) = lower.find(&kw_lower) {
            let after = text[pos + keyword.len()..].trim().trim_start_matches(':').trim_start_matches('-').trim();
            if after.is_empty() { continue; }
            let mut value = String::new();
            for ch in after.chars() {
                if ch == '\n' || ch == '\r' { break; }
                if ch.is_ascii_alphabetic() || ch == ' ' || (ch as u32) >= 0x0600 && (ch as u32) <= 0x06FF || ch == '.' || ch == '-' || ch == '\'' {
                    value.push(ch);
                } else if !value.is_empty() && value.ends_with(' ') { break; }
            }
            let trimmed = value.trim().trim_end_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '\'');
            if trimmed.len() >= 2 && trimmed.len() <= 100 { return Some(trimmed.to_string()); }
        }
    }
    None
}

fn extract_amounts(text: &str) -> (Option<f64>, Option<f64>, Option<f64>) {
    let total = find_number_after_keywords(text, &[
        "total", "grand total", "amount due", "balance due", "total amount", "net total",
        "المبلغ الإجمالي", "الإجمالي", "المجموع", "المبلغ المستحق",
    ]).or_else(|| find_number_after_line_start(text, "total"))
      .or_else(|| find_number_after_line_start(text, "المبلغ الإجمالي"));

    let vat = find_number_after_keywords(text, &[
        "vat", "tax", "sales tax", "vat amount", "tax amount",
        "ضريبة", "ضريبة القيمة المضافة", "الضريبة",
    ]).or_else(|| find_number_after_line_start(text, "vat"))
      .or_else(|| find_number_after_line_start(text, "ضريبة"));

    let net = find_number_after_keywords(text, &[
        "subtotal", "sub total", "net amount", "before tax",
        "المجموع الفرعي", "المبلغ الصافي", "قبل الضريبة",
    ]).or_else(|| find_number_after_line_start(text, "subtotal"))
      .or_else(|| find_number_after_line_start(text, "المجموع الفرعي"));

    (total, vat, net)
}

fn extract_line_items(text: &str) -> Vec<OcrLineItem> {
    let mut items = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let skip_keywords = [
        "invoice", "bill", "date", "customer", "total", "subtotal", "vat", "tax",
        "amount", "balance", "paid", "thank", "note", "payment", "address", "phone",
        "email", "mobile", "tel", "receipt", "فاتورة", "المبلغ", "الضريبة", "الإجمالي",
    ];
    let header_keywords = [
        "item", "description", "product", "service", "qty", "quantity",
        "price", "unit", "amount", "line", "البيان", "الوصف", "الكمية", "السعر", "المبلغ",
    ];
    let mut header_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 && header_keywords.iter().any(|kw| line.to_lowercase().contains(kw.to_lowercase().as_str())) {
            header_idx = Some(idx); break;
        }
    }
    let start = header_idx.unwrap_or(0);
    let mut i = start;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() { i += 1; continue; }
        if header_idx.is_some() && skip_keywords.iter().any(|kw| line.to_lowercase().starts_with(kw)) { i += 1; continue; }
        let has_digit = line.chars().any(|c| c.is_ascii_digit());
        let has_alpha = line.chars().any(|c| c.is_ascii_alphabetic() || (c as u32 >= 0x0600 && c as u32 <= 0x06FF));
        if has_digit && has_alpha {
            let parts: Vec<&str> = line.split(|c: char| c == '|' || c == '\t' || (c == ' ' && line.matches(' ').count() >= 2))
                .map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 {
                let desc = parts[0].to_string();
                let numerics: Vec<f64> = parts[1..].iter().filter_map(|p| p.replace(',', ".").parse::<f64>().ok()).collect();
                if desc.len() >= 2 {
                    let qty = numerics.first().filter(|&&v| v < 100000.0).copied();
                    let price = numerics.get(1).copied();
                    let total = if numerics.len() >= 3 { numerics.last().copied() }
                        else { qty.and_then(|q| price.map(|p| q * p)) };
                    items.push(OcrLineItem { description: desc, quantity: qty, unit_price: price, total });
                }
            }
        }
        i += 1;
    }
    items
}

fn calculate_confidence(data: &OcrInvoiceData) -> f64 {
    let mut score = 0.0;
    if data.invoice_number.is_some() { score += 15.0; }
    if data.date.is_some() { score += 15.0; }
    if data.customer_name.is_some() { score += 10.0; }
    if data.total_amount.is_some() { score += 20.0; }
    if data.vat_amount.is_some() { score += 10.0; }
    if data.net_amount.is_some() { score += 10.0; }
    if !data.items.is_empty() {
        score += 10.0;
        if data.items.iter().all(|i| i.total.is_some()) { score += 5.0; } else { score += 2.0; }
    }
    if !data.raw_text.trim().is_empty() {
        score += (data.raw_text.len() as f64 / 500.0).min(5.0);
    }
    score.min(100.0)
}

fn parse_invoice_data_from_text(text: &str) -> OcrInvoiceData {
    let raw = strip_control_chars(text);
    let invoice_number = extract_invoice_number(&raw);
    let date = extract_date(&raw);
    let customer_name = extract_customer_name(&raw);
    let (total, vat, net) = extract_amounts(&raw);
    let items = extract_line_items(&raw);
    let computed_net = net.or_else(|| total.and_then(|t| vat.map(|v| t - v)));
    let computed_vat = vat.or_else(|| total.and_then(|t| net.map(|n| t - n)));
    OcrInvoiceData { invoice_number, date, customer_name, total_amount: total, vat_amount: computed_vat, net_amount: computed_net, items, raw_text: raw }
}

fn parse_into_extraction_result(data: &OcrInvoiceData) -> ExtractionResult {
    let score = calculate_confidence(data);
    let c = |s: f64| -> String { if s >= 70.0 { "high".into() } else if s >= 40.0 { "medium".into() } else { "low".into() } };

    ExtractionResult {
        fields: vec![
            ExtractedField { key: "vendor".into(), label: "المورد".into(), value: data.customer_name.clone().unwrap_or_default(), confidence: c(if data.customer_name.is_some() { 80.0 } else { 0.0 }), source: "parsed".into() },
            ExtractedField { key: "invoice_number".into(), label: "رقم الفاتورة".into(), value: data.invoice_number.clone().unwrap_or_default(), confidence: c(if data.invoice_number.is_some() { 85.0 } else { 0.0 }), source: "parsed".into() },
            ExtractedField { key: "date".into(), label: "التاريخ".into(), value: data.date.clone().unwrap_or_default(), confidence: c(if data.date.is_some() { 80.0 } else { 0.0 }), source: "parsed".into() },
            ExtractedField { key: "total".into(), label: "الإجمالي".into(), value: data.total_amount.map(|v| format!("{:.3}", v)).unwrap_or_default(), confidence: c(if data.total_amount.is_some() { 90.0 } else { 0.0 }), source: "parsed".into() },
            ExtractedField { key: "vat".into(), label: "الضريبة".into(), value: data.vat_amount.map(|v| format!("{:.3}", v)).unwrap_or_default(), confidence: c(if data.vat_amount.is_some() { 85.0 } else { 0.0 }), source: "parsed".into() },
            ExtractedField { key: "subtotal".into(), label: "المجموع الفرعي".into(), value: data.net_amount.map(|v| format!("{:.3}", v)).unwrap_or_default(), confidence: c(if data.net_amount.is_some() { 80.0 } else { 0.0 }), source: "parsed".into() },
        ],
        items: data.items.iter().map(|i| LineItemResult {
            description: i.description.clone(),
            qty: i.quantity.unwrap_or(0.0),
            unit_price: i.unit_price.unwrap_or(0.0),
            total: i.total.unwrap_or(0.0),
            confidence: "medium".into(),
        }).collect(),
        raw_text: data.raw_text.clone(),
        vendor_name: data.customer_name.clone().unwrap_or_default(),
        invoice_number: data.invoice_number.clone().unwrap_or_default(),
        date: data.date.clone().unwrap_or_default(),
        subtotal: data.net_amount.unwrap_or_else(|| data.total_amount.map(|t| t - data.vat_amount.unwrap_or(0.0)).unwrap_or(0.0)),
        vat: data.vat_amount.unwrap_or(0.0),
        total: data.total_amount.unwrap_or(0.0),
        confidence_score: score / 100.0,
    }
}

#[tauri::command]
pub fn ocr_extract_from_file(path: String) -> Result<OcrResult, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_OCR)?;
    if path.trim().is_empty() { return Err(AppError::validation("File path cannot be empty")); }
    let path_obj = Path::new(&path);
    if !path_obj.exists() { return Err(AppError::not_found(format!("File does not exist: {}", path))); }
    if !path_obj.is_file() { return Err(AppError::validation(format!("Path is not a file: {}", path))); }
    let file_size = fs::metadata(&path).map_err(|e| format!("Failed to get file metadata: {}", e))?.len() as i64;
    let ext = get_file_extension(&path);

    let raw_text = if ext == "pdf" {
        extract_text_from_pdf(&path).unwrap_or_default()
    } else if !is_image_file(&path) {
        try_read_text_from_file(&path).unwrap_or_default()
    } else {
        extract_text_from_image(&path).0
    };

    if raw_text.trim().is_empty() {
        return Err(AppError::business(format!("No text could be extracted from .{} file", ext)));
    }
    let suggested_fields = parse_invoice_data_from_text(&raw_text);
    let confidence = calculate_confidence(&suggested_fields);
    Ok(OcrResult { file_path: path, file_size, raw_text: raw_text.clone(), confidence, suggested_fields })
}

#[tauri::command]
pub fn ocr_parse_invoice(path: String) -> Result<ExtractionResult, AppError> {
    let ext = get_file_extension(&path);
    let raw_text = if ext == "pdf" {
        extract_text_from_pdf(&path).unwrap_or_default()
    } else if !is_image_file(&path) {
        try_read_text_from_file(&path).unwrap_or_default()
    } else {
        extract_text_from_image(&path).0
    };
    if raw_text.trim().is_empty() {
        return Err(AppError::business("No text could be extracted"));
    }
    let data = parse_invoice_data_from_text(&raw_text);
    Ok(parse_into_extraction_result(&data))
}

#[tauri::command]
pub fn ocr_enhance_with_ai(
    _state: State<'_, DbState>,
    _path: String,
    raw_text: String,
) -> Result<ExtractionResult, AppError> {
    let data = parse_invoice_data_from_text(&raw_text);

    if raw_text.trim().len() < 20 {
        return Ok(parse_into_extraction_result(&data));
    }

    let ai_prompt = format!(
        "You are an OCR enhancement assistant. Extract structured invoice data from the following text. \
         Return valid JSON only with these fields: \
         invoice_number, date (YYYY-MM-DD), vendor_name, subtotal (number), vat (number), total (number). \
         If a field cannot be determined, use null.\n\nText:\n{}",
        raw_text
    );

    let result_text = prompt_ai_for_enhancement(&ai_prompt);
    if let Ok(json_str) = result_text {
        if let Ok(ai_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let mut enhanced = data.clone();
            if let Some(val) = ai_data.get("invoice_number").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                enhanced.invoice_number = Some(val.into());
            }
            if let Some(val) = ai_data.get("date").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                enhanced.date = Some(val.into());
            }
            if let Some(val) = ai_data.get("vendor_name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                enhanced.customer_name = Some(val.into());
            }
            if let Some(val) = ai_data.get("total").and_then(|v| v.as_f64()) {
                enhanced.total_amount = Some(val);
            }
            if let Some(val) = ai_data.get("vat").and_then(|v| v.as_f64()) {
                enhanced.vat_amount = Some(val);
            }
            if let Some(val) = ai_data.get("subtotal").and_then(|v| v.as_f64()) {
                enhanced.net_amount = Some(val);
            }
            return Ok(parse_into_extraction_result(&enhanced));
        }
    }

    Ok(parse_into_extraction_result(&data))
}

fn prompt_ai_for_enhancement(prompt: &str) -> Result<String, AppError> {
    let api_key = std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("ANTHROPIC_API_KEY"));
    let provider = if std::env::var("OPENAI_API_KEY").is_ok() { "openai" } else { "anthropic" };
    let key = api_key.map_err(|_| AppError::validation("No AI API key configured"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        ?;

    let (url, body) = if provider == "openai" {
        let body = serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.1,
            "response_format": { "type": "json_object" }
        });
        ("https://api.openai.com/v1/chat/completions", body)
    } else {
        let body = serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 1000,
            "messages": [{"role": "user", "content": prompt}]
        });
        ("https://api.anthropic.com/v1/messages", body)
    };

    let mut req = client.post(url)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json");
    if provider == "anthropic" {
        req = req.header("anthropic-version", "2023-06-01");
    }

    let resp = req.json(&body).send()?;
    let json: serde_json::Value = resp.json()?;

    let content = if provider == "openai" {
        json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
    } else {
        json["content"][0]["text"].as_str().unwrap_or("").to_string()
    };

    // Try to extract JSON from the response
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            return Ok(content[start..=end].to_string());
        }
    }
    Ok(content)
}

#[tauri::command]
pub fn ocr_get_suggestions(
    result: ExtractionResult,
) -> Result<Vec<Suggestion>, AppError> {
    let mut suggestions = Vec::new();
    let has_invoice = !result.invoice_number.is_empty();
    let has_vendor = !result.vendor_name.is_empty();
    let has_items = !result.items.is_empty();

    if has_invoice && has_vendor && has_items {
        suggestions.push(Suggestion {
            id: uuid::Uuid::new_v4().to_string(),
            label: "إنشاء فاتورة مشتريات".into(),
            description: format!("فاتورة رقم {} بقيمة {:.3} من {}", result.invoice_number, result.total, result.vendor_name),
            action_type: "create_invoice".into(),
            data: serde_json::json!({
                "vendor_name": result.vendor_name,
                "invoice_number": result.invoice_number,
                "date": result.date,
                "subtotal": result.subtotal,
                "vat": result.vat,
                "total": result.total,
                "items": result.items,
            }),
        });
    }

    if has_vendor && !has_invoice {
        suggestions.push(Suggestion {
            id: uuid::Uuid::new_v4().to_string(),
            label: "إضافة مورد جديد".into(),
            description: format!("إضافة {} إلى قائمة الموردين", result.vendor_name),
            action_type: "add_supplier".into(),
            data: serde_json::json!({ "name": result.vendor_name }),
        });
    }

    if has_items {
        suggestions.push(Suggestion {
            id: uuid::Uuid::new_v4().to_string(),
            label: "تسجيل مصروفات".into(),
            description: format!("{} بند بمبلغ {}", result.items.len(), result.total),
            action_type: "register_expense".into(),
            data: serde_json::json!({
                "total": result.total,
                "description": format!("مستند مورد - {}", result.vendor_name),
                "date": result.date,
            }),
        });
    }

    if result.confidence_score < 0.5 {
        suggestions.push(Suggestion {
            id: uuid::Uuid::new_v4().to_string(),
            label: "تحسين بالذكاء الاصطناعي".into(),
            description: "إرسال النص المستخرج إلى AI لتحليل أفضل".into(),
            action_type: "ai_enhance".into(),
            data: serde_json::json!({ "raw_text": result.raw_text }),
        });
    }

    Ok(suggestions)
}

#[tauri::command]
pub fn ocr_create_invoice(
    state: State<'_, DbState>,
    data: serde_json::Value,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let vendor = data["vendor_name"].as_str().unwrap_or("Unknown");
    let inv_no = data["invoice_number"].as_str().unwrap_or("");
    let date = data["date"].as_str().unwrap_or("");
    let total = data["total"].as_f64().unwrap_or(0.0);
    let vat = data["vat"].as_f64().unwrap_or(0.0);
    let subtotal = data["subtotal"].as_f64().unwrap_or(total - vat);

    let supplier_id = conn
        .query_row(
            "SELECT id FROM suppliers WHERE name LIKE ?1 LIMIT 1",
            [format!("%{}%", vendor)],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    let net_baisa = (subtotal * 1000.0) as i64;
    let vat_baisa = (vat * 1000.0) as i64;
    let total_baisa = (total * 1000.0) as i64;

    let inv_id: i64 = conn.query_row(
        "INSERT INTO purchases (pur_no, date, supplier_id, vat_enabled, net_milli, vat_milli, total_milli, status, notes, created_by)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, 'Posted', 'OCR Import', 'system')
         RETURNING id",
        rusqlite::params![inv_no, date, supplier_id, net_baisa, vat_baisa, total_baisa],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to create invoice: {}", e))?;

    Ok(format!("Purchase {} created (id: {})", inv_no, inv_id))
}

#[tauri::command]
pub fn ocr_add_supplier(
    state: State<'_, DbState>,
    data: serde_json::Value,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let name = data["name"].as_str().unwrap_or("OCR Import");
    conn.execute(
        "INSERT INTO suppliers (name) VALUES (?1)",
        [name],
    )?;
    Ok(format!("Supplier '{}' created", name))
}

#[tauri::command]
pub fn ocr_register_expense(
    state: State<'_, DbState>,
    data: serde_json::Value,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let total = data["total"].as_f64().unwrap_or(0.0);
    let desc = data["description"].as_str().unwrap_or("OCR Expense");
    let category = data["category"].as_str().unwrap_or("general");
    let date = data["date"].as_str().unwrap_or("");
    let amount_milli = (total * 1000.0) as i64;

    let seq: i64 = conn
        .query_row("SELECT COALESCE(MAX(CAST(SUBSTR(exp_no, 12) AS INTEGER)), 0) + 1 FROM expenses", [], |r| r.get(0))
        .unwrap_or(1);
    let year = chrono::Utc::now().format("%Y").to_string();
    let exp_no = format!("EXP-{}-{:04}", year, seq);
    let use_date = if date.is_empty() { chrono::Utc::now().format("%Y-%m-%d").to_string() } else { date.to_string() };

    conn.execute(
        "INSERT INTO expenses (exp_no, date, category, description, amount_milli, method, notes) VALUES (?1, ?2, ?3, ?4, ?5, 'OCR', 'OCR Scanned')",
        rusqlite::params![exp_no, use_date, category, desc, amount_milli],
    )?;
    Ok(format!("Expense {} for {:.3} OMR registered", exp_no, total))
}

#[tauri::command]
pub fn ocr_update_prices(
    state: State<'_, DbState>,
    data: serde_json::Value,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let items = data["items"].as_array().ok_or_else(|| AppError::validation("items must be an array"))?;
    let mut count = 0u32;
    for item in items {
        if let (Some(product_id), Some(price)) = (item["product_id"].as_i64(), item["new_price_milli"].as_i64()) {
            conn.execute(
                "UPDATE products SET default_price_milli = ?1 WHERE id = ?2",
                rusqlite::params![price, product_id],
            )?;
            count += 1;
        }
    }
    Ok(format!("{} prices updated via OCR", count))
}

#[tauri::command]
pub fn ocr_get_history(state: State<'_, DbState>) -> Result<Vec<OcrScan>, AppError> {
    let db = state.0.lock()?;
    let mut stmt = db
        .prepare("SELECT id, file_name, file_path, extracted_text, parsed_data, confidence, status, created_at FROM ocr_scans ORDER BY created_at DESC LIMIT 200")
        ?;
    let rows = stmt.query_map([], |row| {
        Ok(OcrScan { id: row.get(0)?, file_name: row.get(1)?, file_path: row.get(2)?, extracted_text: row.get(3)?, parsed_data: row.get(4)?, confidence: row.get(5)?, status: row.get(6)?, created_at: row.get(7)? })
    })?;
    let mut scans = Vec::new();
    for row in rows { scans.push(row?); }
    Ok(scans)
}

#[tauri::command]
pub fn ocr_save_scan(state: State<'_, DbState>, input: serde_json::Value) -> Result<i64, AppError> {
    let db = state.0.lock()?;
    let file_path = input["file_path"].as_str().unwrap_or("");
    let raw_text = input["raw_text"].as_str().unwrap_or("");
    let parsed_data = input["parsed_data"].as_str().unwrap_or("");
    let confidence = input["confidence"].as_f64().unwrap_or(0.0);
    let file_name = Path::new(file_path).file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    db.execute(
        "INSERT INTO ocr_scans (file_name, file_path, extracted_text, parsed_data, confidence, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![file_name, file_path, raw_text, parsed_data, confidence, if confidence > 50.0 { "completed" } else { "pending" }],
    )?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn ocr_detect_language(text: String) -> Result<String, AppError> {
    let arabic_chars = text.chars().filter(|c| (*c as u32) >= 0x0600 && (*c as u32) <= 0x06FF).count();
    let latin_chars = text.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let total = (arabic_chars + latin_chars) as f64;
    if total == 0.0 { return Ok("unknown".into()); }
    let arabic_pct = arabic_chars as f64 / total;
    Ok(if arabic_pct > 0.3 { "ara".into() } else { "eng".into() })
}
