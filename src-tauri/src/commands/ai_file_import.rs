use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use super::ai_providers::{chat_with_failover_json, load_provider_config, is_provider_ready};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiDocumentInput {
    pub path: String,
    pub doc_type: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiExtractionRecord {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub doc_type: String,
    pub provider: String,
    pub model: String,
    pub raw_text: String,
    pub extracted_json: String,
    pub fields_json: String,
    pub confidence: f64,
    pub status: String,
    pub target_table: Option<String>,
    pub target_id: Option<i64>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiExtractionSummary {
    pub id: i64,
    pub file_name: String,
    pub doc_type: String,
    pub provider: String,
    pub confidence: f64,
    pub status: String,
    pub created_at: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiCommitResult {
    pub success: bool,
    pub target_table: String,
    pub target_id: i64,
    pub ref_no: String,
    pub created: Vec<String>,
    pub resolved: Vec<String>,
    pub warnings: Vec<String>,
    pub message: String,
}

fn get_extension(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn milli_from(value: Option<f64>) -> i64 {
    (value.unwrap_or(0.0) * 1000.0).round() as i64
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn num_field(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(|x| x.as_f64())
}

// ─── File text extraction (unified) ───────────────────────────────

fn extract_text_from_any_file(path: &str) -> Result<(String, String), AppError> {
    let ext = get_extension(path);
    let file_name = std::path::Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let text = match ext.as_str() {
        "pdf" => {
            let bytes = std::fs::read(path)
                .map_err(|e| AppError::business(format!("Failed to read PDF: {e}")))?;
            pdf_extract::extract_text_from_mem(&bytes)
                .map_err(|e| AppError::business(format!("PDF extraction failed: {e}")))?
        }
        "png" | "jpg" | "jpeg" | "bmp" | "gif" | "tiff" | "tif" | "webp" => {
            super::ocr::extract_text_from_image(path).0
        }
        "xlsx" | "xls" => {
            let ss = super::file_reader::file_read_spreadsheet(path.to_string())?;
            let mut content = String::new();
            for sheet in &ss.sheets {
                content.push_str(&format!("=== {} ===\n", sheet.name));
                content.push_str(&sheet.headers.join(" | "));
                content.push('\n');
                content.push_str(&"-".repeat(60));
                content.push('\n');
                for row in &sheet.rows {
                    content.push_str(&row.join(" | "));
                    content.push('\n');
                }
                content.push('\n');
            }
            content
        }
        "docx" | "txt" | "log" | "md" | "csv" | "json" | "xml" | "toml" | "yaml" | "yml" | "ini" => {
            let fc = super::file_reader::file_read_any(path.to_string())?;
            fc.content
        }
        _ => {
            let fc = super::file_reader::file_read_any(path.to_string())?;
            fc.content
        }
    };

    if text.trim().is_empty() {
        return Err(AppError::business(format!(
            "No text could be extracted from {}. Try a PDF/image with clear text, or install Tesseract OCR.",
            file_name
        )));
    }

    Ok((text, ext))
}

fn detect_document_type(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut best: Option<(&'static str, i32)> = None;
    let mut add = |kw: &[&'static str], name: &'static str, base: i32| {
        let count = kw
            .iter()
            .filter(|k| lower.contains(&k.to_lowercase()))
            .count() as i32;
        if count > 0 {
            let s = base + count * 5;
            if best.is_none_or(|(_, b)| s > b) {
                best = Some((name, s));
            }
        }
    };

    add(&["invoice", "bill", "فاتورة", "فاتورة ضريبية", "invoice number", "inv no"], "invoice", 0);
    add(&["purchase order", "شراء", "purchase", "مشتريات", "أمر شراء"], "purchase", -10);
    add(&["customer", "client", "عميل", "اسم العميل", "زبون", "عملاء"], "customer", -10);
    add(&["product", "item", "منتج", "صنف", "صنف صناعي", "barcode", "sku"], "product", -15);
    add(&["expense", "مصروف", "نفقة", "مصاريف", "expense report"], "expense", -15);
    add(&["supplier", "vendor", "مورد", "المورد", "تاجر"], "supplier", -15);
    add(&["salary", "employee", "payroll", "راتب", "موظف", "الموظفين"], "hr", -20);
    add(&["stock", "inventory", "warehouse", "مخزون", "مستودع", "جرد"], "inventory", -20);

    best.map(|(n, _)| n.to_string()).unwrap_or_else(|| "invoice".to_string())
}

fn build_extraction_prompt(doc_type: &str, text: &str) -> String {
    let (schema, note) = match doc_type {
        "purchase" => (
            r#"{
  "doc_type": "purchase",
  "fields": {
    "invoice_number": "supplier invoice no",
    "date": "YYYY-MM-DD",
    "party_name": "supplier name",
    "party_phone": "",
    "subtotal": 0.0,
    "vat": 0.0,
    "total": 0.0,
    "currency": "OMR",
    "notes": ""
  },
  "items": [
    {"description": "item name", "quantity": 0.0, "unit_price": 0.0, "total": 0.0, "unit": "carton"}
  ]
}"#,
            "This is a supplier/purchase invoice.",
        ),
        "customer" => (
            r#"{
  "doc_type": "customer",
  "items": [
    {"name": "", "phone": "", "email": "", "address": "", "vat_number": "", "credit_limit": 0.0}
  ]
}"#,
            "This is a customer list / client registry.",
        ),
        "product" => (
            r#"{
  "doc_type": "product",
  "items": [
    {"name": "", "code": "", "size": "", "barcode": "", "default_price": 0.0, "default_cost": 0.0, "unit": "carton", "quantity": 0.0, "reorder_level": 0.0}
  ]
}"#,
            "This is a product / price list.",
        ),
        "expense" => (
            r#"{
  "doc_type": "expense",
  "fields": {
    "date": "YYYY-MM-DD",
    "category": "",
    "amount": 0.0,
    "vat": 0.0,
    "method": "cash",
    "vendor": "",
    "reference": "",
    "notes": ""
  }
}"#,
            "This is an expense document.",
        ),
        "supplier" => (
            r#"{
  "doc_type": "supplier",
  "items": [
    {"name": "", "phone": "", "email": "", "address": "", "vat_number": "", "currency": "OMR"}
  ]
}"#,
            "This is a supplier registry.",
        ),
        "hr" => (
            r#"{
  "doc_type": "hr",
  "items": [
    {"name": "", "nationality": "", "job": "", "salary": 0.0, "phone": "", "passport_no": "", "joining_date": ""}
  ]
}"#,
            "This is an employee/payroll document.",
        ),
        "inventory" => (
            r#"{
  "doc_type": "inventory",
  "items": [
    {"name": "", "code": "", "quantity": 0.0, "unit": "carton", "avg_cost": 0.0, "reorder_level": 0.0}
  ]
}"#,
            "This is a stock/inventory list.",
        ),
        _ => (
            r#"{
  "doc_type": "invoice",
  "fields": {
    "invoice_number": "",
    "date": "YYYY-MM-DD",
    "party_name": "customer name",
    "party_phone": "",
    "subtotal": 0.0,
    "vat": 0.0,
    "total": 0.0,
    "currency": "OMR",
    "notes": ""
  },
  "items": [
    {"description": "item name", "quantity": 0.0, "unit_price": 0.0, "total": 0.0, "unit": "carton"}
  ]
}"#,
            "This is a sales invoice.",
        ),
    };

    format!(
        "You are a precise document extraction engine for a manufacturing ERP (paper cups/packaging).\
\n\n{note}\n\nExtract ALL data from the following document text.\n\
Return ONLY valid JSON matching this EXACT schema (use null or empty string when a field is absent):\n\
{schema}\n\
\nRules:\n\
- Convert amounts to numbers (no currency symbols). VAT-inclusive vs exclusive: use the printed subtotal/vat/total.\n\
- Dates must be YYYY-MM-DD.\n\
- The 'party_name' is the trading partner (customer for invoice, supplier for purchase).\n\
- Extract every line item you can find; never invent items.\n\
- Respond in the same language as the document where relevant.\n\
\nDocument text:\n\
------------------\n\
{text}"
    )
}

fn compute_confidence(parsed: &Value) -> f64 {
    let mut score = 0.0;
    let mut checked = 0;
    if let Some(fields) = parsed.get("fields").and_then(|f| f.as_object()) {
        for (_key, val) in fields {
            checked += 1;
            let filled = match val {
                Value::String(s) => !s.is_empty(),
                Value::Number(n) => n.as_f64().map(|v| v != 0.0).unwrap_or(false),
                _ => false,
            };
            if filled {
                score += 1.0;
            }
        }
    }
    let items = parsed.get("items").and_then(|i| i.as_array()).map(|a| a.len()).unwrap_or(0);
    if items > 0 {
        score += 1.0;
        checked += 1;
    }
    if checked == 0 {
        return 0.0;
    }
    (score / checked as f64 * 100.0).min(100.0)
}

// ─── Commands ─────────────────────────────────────────────────────

#[tauri::command]
pub fn ai_list_extractions(state: State<'_, DbState>) -> Result<Vec<AiExtractionSummary>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, file_name, doc_type, provider, confidence, status, created_at,
                substr(extracted_json, 1, 400)
         FROM ai_extractions
         ORDER BY id DESC
         LIMIT 200",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AiExtractionSummary {
            id: row.get(0)?,
            file_name: row.get(1)?,
            doc_type: row.get(2)?,
            provider: row.get(3)?,
            confidence: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
            summary: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[tauri::command]
pub fn ai_get_extraction(state: State<'_, DbState>, id: i64) -> Result<AiExtractionRecord, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, file_path, file_name, file_type, doc_type, provider, model, raw_text,
                extracted_json, fields_json, confidence, status, target_table, target_id, created_at, updated_at
         FROM ai_extractions WHERE id=?",
        [id],
        |row| {
            Ok(AiExtractionRecord {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_type: row.get(3)?,
                doc_type: row.get(4)?,
                provider: row.get(5)?,
                model: row.get(6)?,
                raw_text: row.get(7)?,
                extracted_json: row.get(8)?,
                fields_json: row.get(9)?,
                confidence: row.get(10)?,
                status: row.get(11)?,
                target_table: row.get(12)?,
                target_id: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        },
    )
    .map_err(|_| AppError::not_found("سجل الاستخراج غير موجود"))
}

#[tauri::command]
pub fn ai_delete_extraction(state: State<'_, DbState>, user_id: i64, id: i64) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;
    conn.execute("DELETE FROM ai_extractions WHERE id=?", [id])?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "ai_delete_extraction", "ai_extractions", Some(id), None, None, None);
    Ok("Extraction record deleted".to_string())
}

#[tauri::command]
pub fn ai_update_extraction(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
    doc_type: String,
    extracted_json: String,
) -> Result<AiExtractionRecord, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;
    let parsed: Value = serde_json::from_str(&extracted_json)
        .map_err(|e| AppError::validation(format!("Invalid extraction JSON: {e}")))?;
    let confidence = compute_confidence(&parsed);
    let now = now_str();
    conn.execute(
        "UPDATE ai_extractions SET doc_type=?, extracted_json=?, fields_json=?, confidence=?, updated_at=? WHERE id=?",
        rusqlite::params![doc_type, extracted_json, extracted_json, confidence, now, id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "ai_update_extraction", "ai_extractions", Some(id), None, Some(&doc_type), None);
    drop(conn);
    ai_get_extraction(state, id)
}

#[tauri::command]
pub async fn ai_analyze_document(
    state: State<'_, DbState>,
    user_id: i64,
    input: AiDocumentInput,
) -> Result<AiExtractionRecord, AppError> {
    if input.path.trim().is_empty() {
        return Err(AppError::validation("مسار الملف لا يمكن أن يكون فارغاً"));
    }
    if !std::path::Path::new(&input.path).exists() {
        return Err(AppError::not_found(format!("File does not exist: {}", input.path)));
    }

    let (raw_text, file_type) = extract_text_from_any_file(&input.path)?;

    let doc_type = match input.doc_type {
        Some(dt) if !dt.is_empty() => dt,
        _ => detect_document_type(&raw_text),
    };

    let prompt = build_extraction_prompt(&doc_type, &raw_text);

    let (json_out, provider, model) = if let Some(prov) = &input.provider {
        let cfg = {
            let conn = state.0.lock()?;
            load_provider_config(&conn, prov)?
        };
        if !is_provider_ready(&cfg) {
            return Err(AppError::validation(format!(
                "Provider '{}' is not configured. Set its API key in Settings > AI.",
                cfg.label
            )));
        }
        let resp = super::ai_providers::call_provider(&cfg, "", &prompt, 4096, 0.1, true).await?;
        let parsed = super::ai_providers::parse_json_response(&resp.text)?;
        (parsed, resp.provider, resp.model)
    } else {
        let parsed = chat_with_failover_json(state.clone(), "You are a precise extraction engine.", &prompt, 4096).await?;
        (parsed, "auto".to_string(), "failover".to_string())
    };

    let confidence = compute_confidence(&json_out);
    let file_name = std::path::Path::new(&input.path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let extracted_str = json_out.to_string();

    let conn = state.0.lock()?;
    conn.execute(
        "INSERT INTO ai_extractions(file_path, file_name, file_type, doc_type, provider, model,
                raw_text, extracted_json, fields_json, confidence, status, created_by)
         VALUES(?,?,?,?,?,?,?,?,?,?,'draft',NULL)",
        rusqlite::params![
            input.path, file_name, file_type, doc_type, provider, model,
            raw_text, extracted_str, extracted_str, confidence
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "ai_analyze_document", "ai_extractions", Some(id), None, Some(&doc_type), None);
    drop(conn);

    ai_get_extraction(state, id)
}

// ─── Commit logic (write extracted data to correct tables) ────────

fn resolve_or_create_customer(
    conn: &rusqlite::Connection,
    name: &str,
    phone: &str,
) -> Result<i64, AppError> {
    if name.trim().is_empty() {
        return Err(AppError::validation("اسم العميل مطلوب"));
    }
    if let Ok(id) = conn.query_row(
        "SELECT id FROM customers WHERE name=? COLLATE NOCASE LIMIT 1",
        [name],
        |r| r.get::<_, i64>(0),
    ) {
        return Ok(id);
    }
    let code = format!("C-{:05}", conn.query_row("SELECT COALESCE(MAX(id),0)+1 FROM customers", [], |r| r.get::<_, i64>(0)).unwrap_or(1));
    conn.execute(
        "INSERT INTO customers(code, name, ctype, phone, address, notes, balance_milli)
         VALUES(?,?,'credit',?,?,'Created via AI file import',0)",
        rusqlite::params![code, name, phone, ""],
    )?;
    Ok(conn.last_insert_rowid())
}

fn resolve_or_create_supplier(
    conn: &rusqlite::Connection,
    name: &str,
    phone: &str,
) -> Result<i64, AppError> {
    if name.trim().is_empty() {
        return Err(AppError::validation("اسم المورد مطلوب"));
    }
    if let Ok(id) = conn.query_row(
        "SELECT id FROM suppliers WHERE name=? COLLATE NOCASE LIMIT 1",
        [name],
        |r| r.get::<_, i64>(0),
    ) {
        return Ok(id);
    }
    let code = format!("S-{:05}", conn.query_row("SELECT COALESCE(MAX(id),0)+1 FROM suppliers", [], |r| r.get::<_, i64>(0)).unwrap_or(1));
    conn.execute(
        "INSERT INTO suppliers(code, name, phone, address, notes, balance_milli)
         VALUES(?,?,?,'','Created via AI file import',0)",
        rusqlite::params![code, name, phone],
    )?;
    Ok(conn.last_insert_rowid())
}

fn resolve_or_create_product(conn: &rusqlite::Connection, desc: &str, unit_price: f64) -> Result<i64, AppError> {
    if desc.trim().is_empty() {
        return Err(AppError::validation("وصف المنتج مطلوب"));
    }
    if let Ok(id) = conn.query_row(
        "SELECT id FROM products WHERE name_en=? COLLATE NOCASE OR name_ar=? COLLATE NOCASE LIMIT 1",
        rusqlite::params![desc, desc],
        |r| r.get::<_, i64>(0),
    ) {
        return Ok(id);
    }
    let code = format!("P-{:05}", conn.query_row("SELECT COALESCE(MAX(id),0)+1 FROM products", [], |r| r.get::<_, i64>(0)).unwrap_or(1));
    conn.execute(
        "INSERT INTO products(code, name_ar, name_en, default_price_milli, default_cost_milli, notes, active)
         VALUES(?1, ?2, ?2, ?3, 0, 'Created via AI file import', 1)",
        rusqlite::params![code, desc, milli_from(Some(unit_price))],
    )?;
    Ok(conn.last_insert_rowid())
}

fn commit_invoice(conn: &rusqlite::Connection, parsed: &Value) -> Result<AiCommitResult, AppError> {
    let fields = parsed.get("fields").cloned().unwrap_or(json!({}));
    let party = str_field(&fields, "party_name");
    let phone = str_field(&fields, "party_phone");
    let customer_id = resolve_or_create_customer(conn, &party, &phone)?;
    let date = str_field(&fields, "date");
    let date = if date.is_empty() { chrono::Local::now().format("%Y-%m-%d").to_string() } else { date };

    let items = parsed.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
    if items.is_empty() {
        return Err(AppError::validation("لا توجد بنود مستخرجة لإنشاء الفاتورة"));
    }

    let year = date.get(0..4).unwrap_or("").to_string();
    let seq = next_sequence(conn, "INV", &year)?;
    let inv_no = format!("INV-{}-{:04}", year, seq);

    let mut net: i64 = 0;
    let mut vat: i64 = 0;
    for it in &items {
        let price = num_field(it, "unit_price").unwrap_or(0.0);
        let qty = num_field(it, "quantity").unwrap_or(0.0);
        let line_total = num_field(it, "total").unwrap_or(price * qty);
        net += (line_total * 1000.0).round() as i64;
        let vat_amt = num_field(it, "vat").unwrap_or(0.0);
        vat += (vat_amt * 1000.0).round() as i64;
    }
    let fields_total = milli_from(num_field(&fields, "total"));
    let fields_vat = milli_from(num_field(&fields, "vat"));
    if fields_total > 0 { net = net.saturating_sub(fields_vat); }
    if fields_total > 0 { net = fields_total.saturating_sub(vat); }

    conn.execute(
        "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, vat_enabled, net_milli, vat_milli, total_milli, status, notes, created_by)
         VALUES(?,?,?,'credit',1,?,?,?, 'Draft', ?, 'AI Import')",
        rusqlite::params![inv_no, date, customer_id, net, vat, net + vat, str_field(&fields, "notes")],
    )?;
    let invoice_id = conn.last_insert_rowid();

    let mut created = Vec::new();
    let mut resolved = Vec::new();
    for it in &items {
        let desc = str_field(it, "description");
        let price = num_field(it, "unit_price").unwrap_or(0.0);
        let qty = num_field(it, "quantity").unwrap_or(0.0);
        let product_id = resolve_or_create_product(conn, &desc, price)?;
        if product_id > 0 {
            let existing: bool = conn.query_row(
                "SELECT COUNT(*) FROM products WHERE id=? AND name_en=?" ,
                rusqlite::params![product_id, desc],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
            if existing { resolved.push(desc.clone()); } else { created.push(desc.clone()); }
        }
        let cups_per_carton: i64 = conn.query_row(
            "SELECT COALESCE(cups_per_carton, 1000) FROM products WHERE id=?",
            [product_id],
            |r| r.get(0),
        ).unwrap_or(1000);
        let line_net = (qty * price * 1000.0).round() as i64;
        let vat_pct: f64 = conn.query_row(
            "SELECT COALESCE(vat_pct, 5) FROM products WHERE id=?",
            [product_id],
            |r| r.get(0),
        ).unwrap_or(5.0);
        let line_vat = (line_net as f64 * vat_pct / 100.0).round() as i64;
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?,?,?,?,?,?,?,?,?)",
            rusqlite::params![invoice_id, product_id, qty, cups_per_carton, qty * cups_per_carton as f64, milli_from(Some(price)), line_net, vat_pct, line_vat],
        )?;
    }

    // Recompute totals from lines to keep GL-consistent
    let (sum_net, sum_vat): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(line_net_milli),0), COALESCE(SUM(vat_milli),0) FROM sales_invoice_lines WHERE invoice_id=?",
        [invoice_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    conn.execute(
        "UPDATE sales_invoices SET net_milli=?, vat_milli=?, total_milli=? WHERE id=?",
        rusqlite::params![sum_net, sum_vat, sum_net + sum_vat, invoice_id],
    )?;

    let message = format!("Invoice {inv_no} created for customer '{party}'");
    Ok(AiCommitResult {
        success: true,
        target_table: "sales_invoices".into(),
        target_id: invoice_id,
        ref_no: inv_no,
        created,
        resolved,
        warnings: Vec::new(),
        message,
    })
}

fn commit_purchase(conn: &rusqlite::Connection, parsed: &Value) -> Result<AiCommitResult, AppError> {
    let fields = parsed.get("fields").cloned().unwrap_or(json!({}));
    let party = str_field(&fields, "party_name");
    let phone = str_field(&fields, "party_phone");
    let supplier_id = resolve_or_create_supplier(conn, &party, &phone)?;
    let date = str_field(&fields, "date");
    let date = if date.is_empty() { chrono::Local::now().format("%Y-%m-%d").to_string() } else { date };
    let supplier_invoice_no = str_field(&fields, "invoice_number");

    let items = parsed.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
    if items.is_empty() {
        return Err(AppError::validation("لا توجد بنود مستخرجة لإنشاء المشتريات"));
    }

    let year = date.get(0..4).unwrap_or("").to_string();
    let seq = next_sequence(conn, "PUR", &year)?;
    let pur_no = format!("PUR-{}-{:04}", year, seq);

    conn.execute(
        "INSERT INTO purchases(pur_no, date, supplier_id, supplier_invoice_no, vat_enabled, net_milli, vat_milli, total_milli, status, notes, created_by)
         VALUES(?,?,?,?,1,0,0,0,'draft',?,'AI Import')",
        rusqlite::params![pur_no, date, supplier_id, supplier_invoice_no, str_field(&fields, "notes")],
    )?;
    let purchase_id = conn.last_insert_rowid();

    let mut created = Vec::new();
    let mut resolved = Vec::new();
    for it in &items {
        let desc = str_field(it, "description");
        let price = num_field(it, "unit_price").unwrap_or(0.0);
        let qty = num_field(it, "quantity").unwrap_or(0.0);
        let item_id = resolve_or_create_product(conn, &desc, price)?;
        let is_new: bool = conn.query_row(
            "SELECT COUNT(*) FROM products WHERE id=? AND notes='Created via AI file import'",
            [item_id],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if is_new { created.push(desc.clone()); } else { resolved.push(desc.clone()); }
        let vat_pct: f64 = conn.query_row(
            "SELECT COALESCE(vat_pct, 5) FROM products WHERE id=?",
            [item_id],
            |r| r.get(0),
        ).unwrap_or(5.0);
        conn.execute(
            "INSERT INTO purchase_lines(purchase_id, item_id, qty, unit_cost_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?,?,?,?,?,?,?)",
            rusqlite::params![purchase_id, item_id, qty, milli_from(Some(price)), (qty * price * 1000.0).round() as i64, vat_pct, (qty * price * 1000.0 * vat_pct / 100.0).round() as i64],
        )?;
    }

    let (sum_net, sum_vat): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(line_net_milli),0), COALESCE(SUM(vat_milli),0) FROM purchase_lines WHERE purchase_id=?",
        [purchase_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    conn.execute(
        "UPDATE purchases SET net_milli=?, vat_milli=?, total_milli=? WHERE id=?",
        rusqlite::params![sum_net, sum_vat, sum_net + sum_vat, purchase_id],
    )?;

    let message = format!("Purchase {pur_no} created for supplier '{party}'");
    Ok(AiCommitResult {
        success: true,
        target_table: "purchases".into(),
        target_id: purchase_id,
        ref_no: pur_no,
        created,
        resolved,
        warnings: Vec::new(),
        message,
    })
}

fn commit_customers(conn: &rusqlite::Connection, parsed: &Value) -> Result<AiCommitResult, AppError> {
    let items = parsed.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
    if items.is_empty() {
        return Err(AppError::validation("لا يوجد عملاء مستخرجون للإنشاء"));
    }
    let mut created = Vec::new();
    let mut resolved = Vec::new();
    for it in &items {
        let name = str_field(it, "name");
        if name.is_empty() { continue; }
        let id = resolve_or_create_customer(conn, &name, &str_field(it, "phone"))?;
        conn.execute(
            "UPDATE customers SET phone=?, email=?, address=?, vat_number=?, credit_limit_milli=?
             WHERE id=? AND (phone='' OR ?!='')",
            rusqlite::params![str_field(it, "phone"), str_field(it, "email"), str_field(it, "address"), str_field(it, "vat_number"), milli_from(num_field(it, "credit_limit")), id, str_field(it, "phone")],
        )?;
        if id > 0 {
            let is_new: bool = conn.query_row(
                "SELECT COUNT(*) FROM customers WHERE id=? AND notes='Created via AI file import'",
                [id],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
            if is_new { created.push(name.clone()); } else { resolved.push(name.clone()); }
        }
    }
    let created_count = created.len();
    let resolved_count = resolved.len();
    let message = format!("{created_count} customers created, {resolved_count} resolved");
    Ok(AiCommitResult {
        success: true,
        target_table: "customers".into(),
        target_id: 0,
        ref_no: String::new(),
        created,
        resolved,
        warnings: Vec::new(),
        message,
    })
}

fn commit_products(conn: &rusqlite::Connection, parsed: &Value) -> Result<AiCommitResult, AppError> {
    let items = parsed.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
    if items.is_empty() {
        return Err(AppError::validation("لا توجد منتجات مستخرجة للإنشاء"));
    }
    let mut created = Vec::new();
    let mut resolved = Vec::new();
    for it in &items {
        let name = str_field(it, "name");
        if name.is_empty() { continue; }
        let price = num_field(it, "default_price").unwrap_or(0.0);
        let cost = num_field(it, "default_cost").unwrap_or(0.0);
        let qty = num_field(it, "quantity");
        if let Ok(id) = conn.query_row(
            "SELECT id FROM products WHERE name_en=? COLLATE NOCASE OR name_ar=? COLLATE NOCASE LIMIT 1",
            rusqlite::params![name, name],
            |r| r.get::<_, i64>(0),
        ) {
            resolved.push(name.clone());
            conn.execute(
                "UPDATE products SET default_price_milli=?, default_cost_milli=?, barcode=?, size=?
                 WHERE id=?",
                rusqlite::params![milli_from(Some(price)), milli_from(Some(cost)), str_field(it, "barcode"), str_field(it, "size"), id],
            )?;
            if let Some(q) = qty {
                record_inventory_movement(conn, id, &name, q, cost)?;
            }
        } else {
            let code = format!("P-{:05}", conn.query_row("SELECT COALESCE(MAX(id),0)+1 FROM products", [], |r| r.get::<_, i64>(0)).unwrap_or(1));
            conn.execute(
                "INSERT INTO products(code, name_ar, name_en, size, barcode, default_price_milli, default_cost_milli, notes, active)
                 VALUES(?1,?2,?2,?3,?4,?5,?6,'Created via AI file import',1)",
                rusqlite::params![code, name, str_field(it, "size"), str_field(it, "barcode"), milli_from(Some(price)), milli_from(Some(cost))],
            )?;
            let pid = conn.last_insert_rowid();
            created.push(name.clone());
            if let Some(q) = qty {
                record_inventory_movement(conn, pid, &name, q, cost)?;
            }
        }
    }
    let created_count = created.len();
    let resolved_count = resolved.len();
    let message = format!("{created_count} products created, {resolved_count} updated");
    Ok(AiCommitResult {
        success: true,
        target_table: "products".into(),
        target_id: 0,
        ref_no: String::new(),
        created,
        resolved,
        warnings: Vec::new(),
        message,
    })
}

fn record_inventory_movement(
    conn: &rusqlite::Connection,
    product_id: i64,
    name: &str,
    qty: f64,
    cost: f64,
) -> Result<(), AppError> {
    let item_id = conn
        .query_row(
            "SELECT id FROM inventory_items WHERE product_id=? LIMIT 1",
            [product_id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or_else(|_| {
            let _ = conn.execute(
                "INSERT INTO inventory_items(code, name_ar, name_en, kind, uom, product_id, qty_on_hand, avg_cost_milli, notes)
                 VALUES(?,'','','raw','carton',?,0,0,'Created via AI file import')",
                rusqlite::params![format!("P-{:05}", product_id), product_id],
            );
            conn.last_insert_rowid()
        });
    conn.execute(
        "INSERT INTO inventory_movements(ts, item_id, mtype, qty_in, unit_cost_milli, ref_type, notes)
         VALUES(datetime('now'),?, 'purchase', ?, ?, 'AI_IMPORT', 'AI file import opening stock')",
        rusqlite::params![item_id, qty, milli_from(Some(cost))],
    )?;
    conn.execute(
        "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ? WHERE id=?",
        rusqlite::params![qty, item_id],
    )?;
    let _ = name;
    Ok(())
}

fn commit_suppliers(conn: &rusqlite::Connection, parsed: &Value) -> Result<AiCommitResult, AppError> {
    let items = parsed.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
    if items.is_empty() {
        return Err(AppError::validation("لا يوجد موردون مستخرجون للإنشاء"));
    }
    let mut created = Vec::new();
    let mut resolved = Vec::new();
    for it in &items {
        let name = str_field(it, "name");
        if name.is_empty() { continue; }
        let id = resolve_or_create_supplier(conn, &name, &str_field(it, "phone"))?;
        conn.execute(
            "UPDATE suppliers SET phone=?, email=?, address=?, vat_number=? WHERE id=? AND (phone='' OR ?!='')",
            rusqlite::params![str_field(it, "phone"), str_field(it, "email"), str_field(it, "address"), str_field(it, "vat_number"), id, str_field(it, "phone")],
        )?;
        if id > 0 {
            let is_new: bool = conn.query_row(
                "SELECT COUNT(*) FROM suppliers WHERE id=? AND notes='Created via AI file import'",
                [id],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;
            if is_new { created.push(name.clone()); } else { resolved.push(name.clone()); }
        }
    }
    let created_count = created.len();
    let resolved_count = resolved.len();
    let message = format!("{created_count} suppliers created, {resolved_count} resolved");
    Ok(AiCommitResult {
        success: true,
        target_table: "suppliers".into(),
        target_id: 0,
        ref_no: String::new(),
        created,
        resolved,
        warnings: Vec::new(),
        message,
    })
}

fn commit_expense(conn: &rusqlite::Connection, parsed: &Value) -> Result<AiCommitResult, AppError> {
    let fields = parsed.get("fields").cloned().unwrap_or(json!({}));
    let date = str_field(&fields, "date");
    let date = if date.is_empty() { chrono::Local::now().format("%Y-%m-%d").to_string() } else { date };
    let amount = milli_from(num_field(&fields, "amount"));
    let vat = milli_from(num_field(&fields, "vat"));
    if amount <= 0 {
        return Err(AppError::validation("قيمة المصروف مطلوبة"));
    }
    let year = date.get(0..4).unwrap_or("").to_string();
    let seq = next_sequence(conn, "EXP", &year)?;
    let exp_no = format!("EXP-{}-{:04}", year, seq);
    conn.execute(
        "INSERT INTO expenses(exp_no, date, category, account_code, amount_milli, vat_milli, method, vendor, reference, notes, approval_status, created_by)
         VALUES(?,?,'General','5100',?,?,'cash',?,?,'Created via AI file import','pending','AI Import')",
        rusqlite::params![exp_no, date, amount, vat, str_field(&fields, "vendor"), str_field(&fields, "reference")],
    )?;
    let expense_id = conn.last_insert_rowid();
    let message = format!("Expense {exp_no} ({amount} baisa) recorded");
    Ok(AiCommitResult {
        success: true,
        target_table: "expenses".into(),
        target_id: expense_id,
        ref_no: exp_no,
        created: vec![str_field(&fields, "category")],
        resolved: Vec::new(),
        warnings: Vec::new(),
        message,
    })
}

#[tauri::command]
pub fn ai_commit_extraction(state: State<'_, DbState>, user_id: i64, id: i64) -> Result<AiCommitResult, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;

    let (doc_type, extracted_json): (String, String) = tx.query_row(
        "SELECT doc_type, extracted_json FROM ai_extractions WHERE id=?",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .map_err(|_| AppError::not_found("سجل الاستخراج غير موجود"))?;

    let parsed: Value = serde_json::from_str(&extracted_json)
        .map_err(|e| AppError::validation(format!("Stored extraction is invalid JSON: {e}")))?;

    let result = match doc_type.as_str() {
        "purchase" => commit_purchase(&tx, &parsed)?,
        "customer" => commit_customers(&tx, &parsed)?,
        "product" => commit_products(&tx, &parsed)?,
        "supplier" => commit_suppliers(&tx, &parsed)?,
        "expense" => commit_expense(&tx, &parsed)?,
        _ => commit_invoice(&tx, &parsed)?,
    };

    tx.execute(
        "UPDATE ai_extractions SET status='committed', target_table=?, target_id=?, updated_at=? WHERE id=?",
        rusqlite::params![result.target_table, result.target_id, now_str(), id],
    )?;

    tx.commit()?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "ai_commit_extraction", &result.target_table, Some(result.target_id), None, Some(&result.target_table), None);

    Ok(result)
}

#[tauri::command]
pub fn ai_duplicate_check(state: State<'_, DbState>, extracted_json: String) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    let parsed: Value = serde_json::from_str(&extracted_json)
        .map_err(|e| AppError::validation(format!("Invalid JSON: {e}")))?;
    let fields = parsed.get("fields").cloned().unwrap_or(json!({}));
    let inv_no = str_field(&fields, "invoice_number");
    let party = str_field(&fields, "party_name");
    let total = milli_from(num_field(&fields, "total"));

    let mut matches = Vec::new();
    if !inv_no.is_empty() {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT 'sales_invoices' AS tbl, inv_no, date, total_milli, customer_id FROM sales_invoices WHERE inv_no LIKE ? OR notes LIKE ?
             UNION ALL
             SELECT 'purchases', pur_no, date, total_milli, supplier_id FROM purchases WHERE supplier_invoice_no=?",
        ) {
            let rows = stmt.query_map(
                rusqlite::params![format!("%{inv_no}%"), format!("%{inv_no}%"), inv_no],
                |r| Ok(json!({"table": r.get::<_, String>(0)?, "ref": r.get::<_, String>(1)?, "date": r.get::<_, String>(2)?, "total": r.get::<_, i64>(3)?})),
            );
            if let Ok(rows) = rows {
                for r in rows.flatten() {
                    matches.push(r);
                }
            }
        }
    }
    if matches.is_empty() && !party.is_empty() && total > 0 {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT 'sales_invoices', inv_no, date, total_milli FROM sales_invoices si JOIN customers c ON si.customer_id=c.id WHERE c.name LIKE ? AND si.total_milli=?",
        ) {
            let rows = stmt.query_map(rusqlite::params![format!("%{party}%"), total], |r| Ok(json!({"table": r.get::<_, String>(0)?, "ref": r.get::<_, String>(1)?, "date": r.get::<_, String>(2)?, "total": r.get::<_, i64>(3)?})));
            if let Ok(rows) = rows {
                for r in rows.flatten() {
                    matches.push(r);
                }
            }
        }
    }

    Ok(json!({
        "duplicates": matches,
        "count": matches.len(),
        "suspected_duplicate": !matches.is_empty(),
    }))
}
