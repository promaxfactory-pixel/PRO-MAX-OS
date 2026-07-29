use crate::db::DbState;
use crate::error::AppError;
use calamine::{open_workbook_auto, Data, Reader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelPreview {
    pub file_path: String,
    pub sheet_name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: usize,
    pub detected_type: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelImportInput {
    pub file_path: String,
    pub sheet_name: String,
    pub import_type: String,
    pub column_mapping: Option<ColumnMapping>,
    pub skip_first_row: bool,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub mappings: Vec<FieldMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldMapping {
    pub excel_column: String,
    pub system_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelImportResult {
    pub success: bool,
    pub total_rows: usize,
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<ImportError>,
    pub warnings: Vec<String>,
    pub summary: String,
    pub import_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub row: usize,
    pub column: String,
    pub value: String,
    pub error: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelAnalyzeInput {
    pub file_path: String,
    pub sheet_name: String,
    pub import_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelAnalysis {
    pub total_rows: usize,
    pub total_columns: usize,
    pub headers: Vec<String>,
    pub sample_data: Vec<Vec<serde_json::Value>>,
    pub detected_types: Vec<ColumnAnalysis>,
    pub validation_errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub estimated_import_time: String,
    pub requires_confirmation: bool,
    pub confirmation_reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnAnalysis {
    pub column: String,
    pub detected_type: String,
    pub null_count: usize,
    pub unique_count: usize,
    pub sample_values: Vec<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub row: usize,
    pub column: String,
    pub value: String,
    pub error_type: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportHistory {
    pub id: i64,
    pub import_type: String,
    pub file_name: String,
    pub total_rows: usize,
    pub imported: usize,
    pub skipped: usize,
    pub status: String,
    pub created_at: String,
    pub created_by: String,
}

// ---------------------------------------------------------------------------
// Constants – Arabic / English header synonyms for auto-detection
// ---------------------------------------------------------------------------

const JOURNAL_HEADERS_EN: &[&str] = &[
    "date", "account", "account code", "account_name", "description", "debit", "credit", "reference", "cost center",
];
const JOURNAL_HEADERS_AR: &[&str] = &[
    "التاريخ", "الحساب", "رمز الحساب", "اسم الحساب", "الوصف", "المدين", "الدائن", "المرجع", "مركز التكلفة",
];

const CUSTOMER_HEADERS_EN: &[&str] = &[
    "name", "customer name", "phone", "email", "vat", "vat number", "credit limit", "address", "city", "country", "code", "customer code",
];
const CUSTOMER_HEADERS_AR: &[&str] = &[
    "الاسم", "اسم العميل", "الهاتف", "البريد الإلكتروني", "الرقم الضريبي", "الضريبة", "حد الائتمان", "العنوان", "المدينة", "الدولة", "الرمز", "رمز العميل",
];

const PRODUCT_HEADERS_EN: &[&str] = &[
    "code", "product code", "name", "product name", "description", "price", "cost", "unit", "category", "size", "barcode", "sku",
];
const PRODUCT_HEADERS_AR: &[&str] = &[
    "الرمز", "رمز المنتج", "الاسم", "اسم المنتج", "الوصف", "السعر", "التكلفة", "الوحدة", "الفئة", "المقاس", "الباركود", "رمز الصنف",
];

const INVENTORY_HEADERS_EN: &[&str] = &[
    "code", "item code", "name", "item name", "quantity", "reorder level", "reorder", "cost", "location", "warehouse", "unit", "min stock", "max stock",
];
const INVENTORY_HEADERS_AR: &[&str] = &[
    "الرمز", "رمز الصنف", "الاسم", "اسم الصنف", "الكمية", "حد إعادة الطلب", "إعادة الطلب", "التكلفة", "الموقع", "المستودع", "الوحدة", "الحد الأدنى", "الحد الأقصى",
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cell_to_json(cell: &Data) -> serde_json::Value {
    match cell {
        Data::Empty => serde_json::Value::Null,
        Data::String(s) => serde_json::Value::String(s.clone()),
        Data::Float(f) => serde_json::json!(*f),
        Data::Int(i) => serde_json::json!(*i),
        Data::Bool(b) => serde_json::Value::Bool(*b),
        Data::Error(e) => serde_json::Value::String(format!("ERROR:{:?}", e)),
        Data::DateTime(dt) => serde_json::Value::String(format!("{}", dt)),
        Data::DateTimeIso(s) => serde_json::Value::String(s.clone()),
        Data::DurationIso(s) => serde_json::Value::String(s.clone()),
    }
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::Error(e) => format!("ERR:{:?}", e),
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

fn cell_to_f64(cell: &Data) -> Option<f64> {
    match cell {
        Data::Float(f) => Some(*f),
        Data::Int(i) => Some(*i as f64),
        Data::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn cell_to_i64(cell: &Data) -> Option<i64> {
    match cell {
        Data::Int(i) => Some(*i),
        Data::Float(f) => Some(*f as i64),
        Data::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn normalize_header(h: &str) -> String {
    h.trim().to_lowercase()
}

fn read_sheet(
    file_path: &str,
    sheet_name: &str,
) -> Result<(Vec<String>, Vec<Vec<Data>>), AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::not_found(format!("File not found: {}", file_path)));
    }
    let mut workbook = open_workbook_auto(path).map_err(|e| format!("Failed to open workbook: {}", e))?;
    let names = workbook.sheet_names().to_vec();
    if !names.contains(&sheet_name.to_string()) {
        return Err(AppError::not_found(format!(
            "Sheet '{}' not found. Available: {:?}",
            sheet_name, names
        )));
    }
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| format!("Failed to read sheet: {}", e))?;
    let mut rows_iter = range.rows();
    let headers = match rows_iter.next() {
        Some(row) => row.iter().map(cell_to_string).collect(),
        None => return Err(AppError::validation("Sheet is empty")),
    };
    let data: Vec<Vec<Data>> = rows_iter.map(|r| r.to_vec()).collect();
    Ok((headers, data))
}

fn read_all_rows(
    file_path: &str,
    sheet_name: &str,
) -> Result<(Vec<String>, Vec<Vec<Data>>), AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::not_found(format!("File not found: {}", file_path)));
    }
    let mut workbook = open_workbook_auto(path).map_err(|e| format!("Failed to open workbook: {}", e))?;
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| format!("Failed to read sheet: {}", e))?;
    let mut rows_iter = range.rows();
    let headers = match rows_iter.next() {
        Some(row) => row.iter().map(cell_to_string).collect(),
        None => return Err(AppError::validation("Sheet is empty")),
    };
    let data: Vec<Vec<Data>> = rows_iter.map(|r| r.to_vec()).collect();
    Ok((headers, data))
}

fn first_sheet(file_path: &str) -> Result<String, AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::not_found(format!("File not found: {}", file_path)));
    }
    let workbook = open_workbook_auto(path).map_err(|e| format!("Failed to open workbook: {}", e))?;
    let names = workbook.sheet_names();
    names
        .first()
        .cloned()
        .ok_or_else(|| AppError::validation("Workbook has no sheets"))
}

/// Count non-empty cells in a row
#[allow(dead_code)]
fn non_empty_count(row: &[Data]) -> usize {
    row.iter().filter(|c| !matches!(c, Data::Empty)).count()
}

/// Build a map: normalized header → column index
fn header_index_map(headers: &[String]) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(i, h)| (normalize_header(h), i))
        .collect()
}

/// Find column index by checking multiple possible header names (case-insensitive)
fn find_column(map: &HashMap<String, usize>, candidates: &[&str]) -> Option<usize> {
    for c in candidates {
        let key = normalize_header(c);
        if let Some(&idx) = map.get(&key) {
            return Some(idx);
        }
    }
    None
}

fn ensure_import_history_table(conn: &rusqlite::Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS import_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            import_type TEXT NOT NULL,
            file_name TEXT NOT NULL,
            total_rows INTEGER NOT NULL DEFAULT 0,
            imported INTEGER NOT NULL DEFAULT 0,
            skipped INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'completed',
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            created_by TEXT NOT NULL DEFAULT 'system'
        );",
    ).map_err(|e| AppError::migration(format!("Failed to create import_history table: {}", e)))
}

fn insert_import_history(
    conn: &rusqlite::Connection,
    import_type: &str,
    file_name: &str,
    total_rows: usize,
    imported: usize,
    skipped: usize,
    status: &str,
) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO import_history (import_type, file_name, total_rows, imported, skipped, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![import_type, file_name, total_rows as i64, imported as i64, skipped as i64, status],
    )
    .map_err(|e| format!("Failed to insert import history: {}", e))?;
    Ok(conn.last_insert_rowid())
}

// ---------------------------------------------------------------------------
// 1. excel_read_preview
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_read_preview(file_path: String, sheet_name: Option<String>) -> Result<ExcelPreview, AppError> {
    let sheet = match sheet_name {
        Some(s) => s,
        None => first_sheet(&file_path)?,
    };
    let (headers, data) = read_sheet(&file_path, &sheet)?;
    let total_rows = data.len();

    let preview_rows: Vec<Vec<serde_json::Value>> = data
        .iter()
        .take(20)
        .map(|row| row.iter().map(cell_to_json).collect())
        .collect();

    let hmap = header_index_map(&headers);
    let (detected_type, suggestions) = detect_import_type_and_suggestions(&headers, &data, &hmap);

    Ok(ExcelPreview {
        file_path,
        sheet_name: sheet,
        headers,
        rows: preview_rows,
        total_rows,
        detected_type,
        suggestions,
    })
}

fn detect_import_type_and_suggestions(
    headers: &[String],
    data: &[Vec<Data>],
    hmap: &HashMap<String, usize>,
) -> (String, Vec<String>) {
    let mut scores: HashMap<&str, usize> = HashMap::new();
    scores.insert("journal", 0);
    scores.insert("customers", 0);
    scores.insert("products", 0);
    scores.insert("inventory", 0);

    for h in headers {
        let n = normalize_header(h);
        if JOURNAL_HEADERS_EN.iter().any(|jh| normalize_header(jh) == n) || JOURNAL_HEADERS_AR.iter().any(|jh| normalize_header(jh) == n) {
            if let Some(v) = scores.get_mut("journal") { *v += 1; }
        }
        if CUSTOMER_HEADERS_EN.iter().any(|ch| normalize_header(ch) == n) || CUSTOMER_HEADERS_AR.iter().any(|ch| normalize_header(ch) == n) {
            if let Some(v) = scores.get_mut("customers") { *v += 1; }
        }
        if PRODUCT_HEADERS_EN.iter().any(|ph| normalize_header(ph) == n) || PRODUCT_HEADERS_AR.iter().any(|ph| normalize_header(ph) == n) {
            if let Some(v) = scores.get_mut("products") { *v += 1; }
        }
        if INVENTORY_HEADERS_EN.iter().any(|ih| normalize_header(ih) == n) || INVENTORY_HEADERS_AR.iter().any(|ih| normalize_header(ih) == n) {
            if let Some(v) = scores.get_mut("inventory") { *v += 1; }
        }
    }

    // Boost: if debit+credit columns exist strongly prefer journal
    let has_debit = hmap.contains_key("debit") || hmap.contains_key("المدين");
    let has_credit = hmap.contains_key("credit") || hmap.contains_key("الدائن");
    if has_debit && has_credit {
        if let Some(v) = scores.get_mut("journal") { *v += 5; }
    }

    let best = scores.iter().max_by_key(|(_, v)| *v).unwrap_or((&"unknown", &0));
    let detected = if *best.1 >= 2 {
        best.0.to_string()
    } else {
        "unknown".to_string()
    };

    let mut suggestions = Vec::new();
    match detected.as_str() {
        "journal" => {
            suggestions.push("Detected journal entries. Ensure debit and credit columns are balanced.".into());
            if !has_debit {
                suggestions.push("Warning: 'debit' column not found. Map it in column mapping.".into());
            }
            if !has_credit {
                suggestions.push("Warning: 'credit' column not found. Map it in column mapping.".into());
            }
        }
        "customers" => {
            suggestions.push("Detected customer data. Ensure customer names are unique.".into());
        }
        "products" => {
            suggestions.push("Detected product data. Ensure product codes are unique.".into());
        }
        "inventory" => {
            suggestions.push("Detected inventory data. Quantities and reorder levels should be numeric.".into());
        }
        _ => {
            suggestions.push("Could not auto-detect import type. Please specify the import type manually.".into());
            suggestions.push("You can also provide column mappings to map Excel columns to system fields.".into());
        }
    }

    // General suggestions
    if data.is_empty() {
        suggestions.push("The sheet contains no data rows.".into());
    } else if data.len() > 1000 {
        suggestions.push(format!("Large dataset ({} rows). Import may take a while.", data.len()));
    }

    (detected, suggestions)
}

// ---------------------------------------------------------------------------
// 2. excel_list_sheets
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_list_sheets(file_path: String) -> Result<Vec<String>, AppError> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(AppError::not_found(format!("File not found: {}", file_path)));
    }
    let workbook = open_workbook_auto(path).map_err(|e| format!("Failed to open workbook: {}", e))?;
    Ok(workbook.sheet_names().to_vec())
}

// ---------------------------------------------------------------------------
// 3. excel_import_journal
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_import_journal(
    state: State<'_, DbState>,
    input: ExcelImportInput,
) -> Result<ExcelImportResult, AppError> {
    let (headers, data) = read_all_rows(&input.file_path, &input.sheet_name)?;
    let start_idx = if input.skip_first_row { 1 } else { 0 };
    let data_rows = &data[start_idx..];

    let mut hmap = header_index_map(&headers);

    // Apply custom column mapping if provided
    if let Some(ref mapping) = input.column_mapping {
        for fm in &mapping.mappings {
            let normalized_excel = normalize_header(&fm.excel_column);
            if let Some(&col_idx) = hmap.get(&normalized_excel) {
                hmap.insert(normalize_header(&fm.system_field), col_idx);
            }
        }
    }

    let date_col = find_column(&hmap, &["date", "التاريخ", "entry_date", "journal_date"]);
    let account_col = find_column(&hmap, &["account", "الحساب", "account_name", "اسم الحساب", "account_code", "رمز الحساب"]);
    let desc_col = find_column(&hmap, &["description", "الوصف", "desc", "memo", "التفاصيل"]);
    let debit_col = find_column(&hmap, &["debit", "المدين", "debit_amount", "مبلغ مدين"]);
    let credit_col = find_column(&hmap, &["credit", "الدائن", "credit_amount", "مبلغ دائن"]);
    let ref_col = find_column(&hmap, &["reference", "المرجع", "ref", "doc_no"]);
    let _cost_center_col = find_column(&hmap, &["cost center", "مركز التكلفة", "cost_center"]);

    if debit_col.is_none() && credit_col.is_none() {
        return Err("No debit or credit column found. Please map columns correctly.".into());
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let file_name = Path::new(&input.file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Group rows by date to form journal entries
    let mut entry_groups: Vec<(String, Vec<usize>)> = Vec::new();
    let mut current_date: Option<String> = None;
    let mut current_indices: Vec<usize> = Vec::new();

    for (i, row) in data_rows.iter().enumerate() {
        let date_val = date_col
            .and_then(|ci| row.get(ci))
            .map(cell_to_string)
            .unwrap_or_default();

        if !date_val.is_empty() && current_date.as_ref() != Some(&date_val) {
            if let Some(d) = current_date.take() {
                if !current_indices.is_empty() {
                    entry_groups.push((d, std::mem::take(&mut current_indices)));
                }
            }
            current_date = Some(date_val);
        }
        current_indices.push(i);
    }
    if let Some(d) = current_date {
        if !current_indices.is_empty() {
            entry_groups.push((d, current_indices));
        }
    }

    // If no date grouping worked, treat all as one entry
    if entry_groups.is_empty() && !data_rows.is_empty() {
        entry_groups.push((
            "2025-01-01".to_string(),
            (0..data_rows.len()).collect(),
        ));
    }

    let conn = state.0.lock()?;
    ensure_import_history_table(&conn)?;

    for (date, indices) in &entry_groups {
        let mut total_debit = 0.0f64;
        let mut total_credit = 0.0f64;
        let mut lines: Vec<(String, String, f64, f64, String)> = Vec::new(); // (account, desc, debit, credit, ref)

        for &idx in indices {
            let row = &data_rows[idx];
            let excel_row = idx + if input.skip_first_row { 2 } else { 1 };

            let account = account_col
                .and_then(|ci| row.get(ci))
                .map(cell_to_string)
                .unwrap_or_default();

            if account.is_empty() {
                errors.push(ImportError {
                    row: excel_row,
                    column: "account".into(),
                    value: String::new(),
                    error: "Account is required".into(),
                    suggestion: "Ensure each row has an account name or code.".into(),
                });
                skipped += 1;
                continue;
            }

            let description = desc_col
                .and_then(|ci| row.get(ci))
                .map(cell_to_string)
                .unwrap_or_default();

            let debit = debit_col
                .and_then(|ci| row.get(ci))
                .and_then(cell_to_f64)
                .unwrap_or(0.0);

            let credit = credit_col
                .and_then(|ci| row.get(ci))
                .and_then(cell_to_f64)
                .unwrap_or(0.0);

            let reference = ref_col
                .and_then(|ci| row.get(ci))
                .map(cell_to_string)
                .unwrap_or_default();

            if debit < 0.0 {
                errors.push(ImportError {
                    row: excel_row,
                    column: "debit".into(),
                    value: format!("{}", debit),
                    error: "Debit cannot be negative".into(),
                    suggestion: "Use positive values for debit amounts.".into(),
                });
            }
            if credit < 0.0 {
                errors.push(ImportError {
                    row: excel_row,
                    column: "credit".into(),
                    value: format!("{}", credit),
                    error: "Credit cannot be negative".into(),
                    suggestion: "Use positive values for credit amounts.".into(),
                });
            }
            if debit == 0.0 && credit == 0.0 {
                warnings.push(format!(
                    "Row {}: both debit and credit are zero, skipping line.",
                    excel_row
                ));
                skipped += 1;
                continue;
            }
            if debit > 0.0 && credit > 0.0 {
                warnings.push(format!(
                    "Row {}: both debit and credit are non-zero. Treating as separate lines.",
                    excel_row
                ));
            }

            total_debit += debit;
            total_credit += credit;
            lines.push((account, description, debit, credit, reference));
        }

        // Validate debits == credits
        let diff = (total_debit - total_credit).abs();
        if diff > 0.01 {
            warnings.push(format!(
                "Journal entry for {}: debits ({}) ≠ credits ({}). Difference: {}",
                date, total_debit, total_credit, diff
            ));
        }

        if input.dry_run {
            imported += lines.len();
            continue;
        }

        // Insert the journal entry
        let entry_desc = format!("Imported from {}", file_name);
        let entry_no = format!("IMP-{}", file_name);
        conn.execute(
            "INSERT INTO journal_entries (entry_no, date, memo, ref_type, ref_id, created_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![entry_no, date, entry_desc, "import", Option::<i64>::None, "system"],
        )
        .map_err(|e| format!("Failed to insert journal entry: {}", e))?;
        let entry_id = conn.last_insert_rowid();

        for (account, desc, debit, credit, _reference) in &lines {
            conn.execute(
                "INSERT INTO journal_entry_lines (entry_id, account_code, debit_milli, credit_milli, memo) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![entry_id, account, (debit * 1000.0) as i64, (credit * 1000.0) as i64, desc],
            )
            .map_err(|e| format!("Failed to insert journal line: {}", e))?;
            imported += 1;
        }
    }

    let status = if errors.is_empty() {
        "completed"
    } else {
        "completed_with_errors"
    };
    let history_id = insert_import_history(
        &conn,
        "journal",
        &file_name,
        data_rows.len(),
        imported,
        skipped,
        status,
    )?;
    drop(conn);

    let summary = format!(
        "Journal import: {} rows processed, {} lines imported, {} skipped, {} errors.",
        data_rows.len(),
        imported,
        skipped,
        errors.len()
    );

    Ok(ExcelImportResult {
        success: errors.is_empty(),
        total_rows: data_rows.len(),
        imported,
        skipped,
        errors,
        warnings,
        summary,
        import_id: Some(history_id),
    })
}

// ---------------------------------------------------------------------------
// 4. excel_import_customers
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_import_customers(
    state: State<'_, DbState>,
    input: ExcelImportInput,
) -> Result<ExcelImportResult, AppError> {
    let (headers, data) = read_all_rows(&input.file_path, &input.sheet_name)?;
    let data_rows = if input.skip_first_row { &data[0..] } else { &data[..] };
    let mut hmap = header_index_map(&headers);

    if let Some(ref mapping) = input.column_mapping {
        for fm in &mapping.mappings {
            let normalized_excel = normalize_header(&fm.excel_column);
            if let Some(&col_idx) = hmap.get(&normalized_excel) {
                hmap.insert(normalize_header(&fm.system_field), col_idx);
            }
        }
    }

    let name_col = find_column(&hmap, &["name", "customer name", "الاسم", "اسم العميل", "customer_name"])
        .ok_or("Customer name column not found")?;
    let phone_col = find_column(&hmap, &["phone", "الهاتف", "mobile", "tel", "الجوال", "الموبايل"]);
    let email_col = find_column(&hmap, &["email", "البريد الإلكتروني", "البريد", "e-mail"]);
    let vat_col = find_column(&hmap, &["vat", "vat number", "الرقم الضريبي", "الضريبة", "tax_id", "vat_number"]);
    let credit_col = find_column(&hmap, &["credit limit", "حد الائتمان", "credit_limit", "الحد الأقصى"]);
    let address_col = find_column(&hmap, &["address", "العنوان", "العنوان التفصيلي"]);
    let city_col = find_column(&hmap, &["city", "المدينة"]);
    let country_col = find_column(&hmap, &["country", "الدولة", "البلد"]);
    let code_col = find_column(&hmap, &["code", "customer code", "الرمز", "رمز العميل", "customer_code"]);

    let conn = state.0.lock()?;
    ensure_import_history_table(&conn)?;

    // Load existing customer names for duplicate check
    let mut existing_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT name FROM customers") {
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                existing_names.insert(r.to_lowercase());
            }
        }
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let file_name = Path::new(&input.file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for (i, row) in data_rows.iter().enumerate() {
        let excel_row = i + if input.skip_first_row { 2 } else { 1 };

        let name = row.get(name_col).map(cell_to_string).unwrap_or_default();
        if name.is_empty() {
            errors.push(ImportError {
                row: excel_row,
                column: "name".into(),
                value: String::new(),
                error: "Customer name is required".into(),
                suggestion: "Ensure the name column is populated.".into(),
            });
            skipped += 1;
            continue;
        }

        // Duplicate check
        let lower_name = name.to_lowercase();
        if existing_names.contains(&lower_name) {
            warnings.push(format!("Row {}: customer '{}' already exists, skipping.", excel_row, name));
            skipped += 1;
            continue;
        }

        let phone = phone_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let email = email_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let vat_number = vat_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let credit_limit = credit_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let address = address_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let _city = city_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let _country = country_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let code = code_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();

        // Validate email format if present
        if !email.is_empty() && !email.contains('@') {
            errors.push(ImportError {
                row: excel_row,
                column: "email".into(),
                value: email.clone(),
                error: "Invalid email format".into(),
                suggestion: "Email should contain '@'.".into(),
            });
        }

        if input.dry_run {
            existing_names.insert(lower_name);
            imported += 1;
            continue;
        }

        let customer_code = if code.is_empty() {
            format!("CUST-{}", 1000 + imported)
        } else {
            code
        };

        conn.execute(
            "INSERT INTO customers (code, name, phone, email, vat_number, credit_limit_milli, address, active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            rusqlite::params![customer_code, name, phone, email, vat_number, (credit_limit * 1000.0) as i64, address],
        )
        .map_err(|e| format!("Failed to insert customer '{}': {}", name, e))?;

        existing_names.insert(lower_name);
        imported += 1;
    }

    let status = if errors.is_empty() { "completed" } else { "completed_with_errors" };
    let history_id = insert_import_history(&conn, "customers", &file_name, data_rows.len(), imported, skipped, status)?;
    drop(conn);

    Ok(ExcelImportResult {
        success: errors.is_empty(),
        total_rows: data_rows.len(),
        imported,
        skipped,
        errors,
        warnings,
        summary: format!("Customer import: {} processed, {} imported, {} skipped.", data_rows.len(), imported, skipped),
        import_id: Some(history_id),
    })
}

// ---------------------------------------------------------------------------
// 5. excel_import_products
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_import_products(
    state: State<'_, DbState>,
    input: ExcelImportInput,
) -> Result<ExcelImportResult, AppError> {
    let (headers, data) = read_all_rows(&input.file_path, &input.sheet_name)?;
    let data_rows = if input.skip_first_row { &data[0..] } else { &data[..] };
    let mut hmap = header_index_map(&headers);

    if let Some(ref mapping) = input.column_mapping {
        for fm in &mapping.mappings {
            let normalized_excel = normalize_header(&fm.excel_column);
            if let Some(&col_idx) = hmap.get(&normalized_excel) {
                hmap.insert(normalize_header(&fm.system_field), col_idx);
            }
        }
    }

    let code_col = find_column(&hmap, &["code", "product code", "الرمز", "رمز المنتج", "sku", "product_code"]);
    let name_col = find_column(&hmap, &["name", "product name", "الاسم", "اسم المنتج", "product_name"])
        .ok_or("Product name column not found")?;
    let desc_col = find_column(&hmap, &["description", "الوصف", "desc"]);
    let price_col = find_column(&hmap, &["price", "السعر", "sale price", "selling_price", "سعر البيع"]);
    let cost_col = find_column(&hmap, &["cost", "التكلفة", "cost price", "cost_price", "سعر التكلفة"]);
    let unit_col = find_column(&hmap, &["unit", "الوحدة", "unit of measure", "uom"]);
    let category_col = find_column(&hmap, &["category", "الفئة", "type", "النوع", "group"]);
    let size_col = find_column(&hmap, &["size", "المقاس", "المقاس/اللون", "dimension"]);
    let barcode_col = find_column(&hmap, &["barcode", "الباركود", "bar code", "upc"]);

    let conn = state.0.lock()?;
    ensure_import_history_table(&conn)?;

    let mut existing_codes: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT code FROM products") {
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                existing_codes.insert(r.to_lowercase());
            }
        }
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let file_name = Path::new(&input.file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for (i, row) in data_rows.iter().enumerate() {
        let excel_row = i + if input.skip_first_row { 2 } else { 1 };

        let name = row.get(name_col).map(cell_to_string).unwrap_or_default();
        if name.is_empty() {
            errors.push(ImportError {
                row: excel_row,
                column: "name".into(),
                value: String::new(),
                error: "Product name is required".into(),
                suggestion: "Ensure the name column is populated.".into(),
            });
            skipped += 1;
            continue;
        }

        let code = code_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let lower_code = code.to_lowercase();
        if !code.is_empty() && existing_codes.contains(&lower_code) {
            warnings.push(format!("Row {}: product code '{}' already exists, skipping.", excel_row, code));
            skipped += 1;
            continue;
        }

        let _description = desc_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let price = price_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let cost = cost_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let _unit = unit_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_else(|| "piece".into());
        let _category = category_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let size = size_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let barcode = barcode_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();

        if price < 0.0 {
            errors.push(ImportError {
                row: excel_row,
                column: "price".into(),
                value: format!("{}", price),
                error: "Price cannot be negative".into(),
                suggestion: "Use a non-negative value for price.".into(),
            });
        }
        if cost < 0.0 {
            errors.push(ImportError {
                row: excel_row,
                column: "cost".into(),
                value: format!("{}", cost),
                error: "Cost cannot be negative".into(),
                suggestion: "Use a non-negative value for cost.".into(),
            });
        }
        if cost > 0.0 && price > 0.0 && cost > price {
            warnings.push(format!("Row {}: cost ({}) is higher than price ({}).", excel_row, cost, price));
        }

        if input.dry_run {
            if !code.is_empty() {
                existing_codes.insert(lower_code);
            }
            imported += 1;
            continue;
        }

        let product_code = if code.is_empty() {
            format!("PRD-{}", 1000 + imported)
        } else {
            code.clone()
        };

        conn.execute(
            "INSERT INTO products (code, name_en, default_price_milli, default_cost_milli, size, barcode, active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            rusqlite::params![product_code, name, (price * 1000.0) as i64, (cost * 1000.0) as i64, size, barcode],
        )
        .map_err(|e| format!("Failed to insert product '{}': {}", name, e))?;

        if !code.is_empty() {
            existing_codes.insert(lower_code);
        }
        imported += 1;
    }

    let status = if errors.is_empty() { "completed" } else { "completed_with_errors" };
    let history_id = insert_import_history(&conn, "products", &file_name, data_rows.len(), imported, skipped, status)?;
    drop(conn);

    Ok(ExcelImportResult {
        success: errors.is_empty(),
        total_rows: data_rows.len(),
        imported,
        skipped,
        errors,
        warnings,
        summary: format!("Product import: {} processed, {} imported, {} skipped.", data_rows.len(), imported, skipped),
        import_id: Some(history_id),
    })
}

// ---------------------------------------------------------------------------
// 6. excel_import_inventory
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_import_inventory(
    state: State<'_, DbState>,
    input: ExcelImportInput,
) -> Result<ExcelImportResult, AppError> {
    let (headers, data) = read_all_rows(&input.file_path, &input.sheet_name)?;
    let data_rows = if input.skip_first_row { &data[0..] } else { &data[..] };
    let mut hmap = header_index_map(&headers);

    if let Some(ref mapping) = input.column_mapping {
        for fm in &mapping.mappings {
            let normalized_excel = normalize_header(&fm.excel_column);
            if let Some(&col_idx) = hmap.get(&normalized_excel) {
                hmap.insert(normalize_header(&fm.system_field), col_idx);
            }
        }
    }

    let code_col = find_column(&hmap, &["code", "item code", "الرمز", "رمز الصنف", "sku", "product_code"]);
    let name_col = find_column(&hmap, &["name", "item name", "الاسم", "اسم الصنف", "product_name", "item_name"])
        .ok_or("Item name column not found")?;
    let qty_col = find_column(&hmap, &["quantity", "الكمية", "qty", "stock", "المخزون"]);
    let reorder_col = find_column(&hmap, &["reorder level", "reorder", "حد إعادة الطلب", "reorder_level", "reorder_point"]);
    let cost_col = find_column(&hmap, &["cost", "التكلفة", "unit cost", "cost_price", "average_cost"]);
    let location_col = find_column(&hmap, &["location", "الموقع", "shelf", "position"]);
    let warehouse_col = find_column(&hmap, &["warehouse", "المستودع", "depot"]);
    let unit_col = find_column(&hmap, &["unit", "الوحدة", "uom"]);
    let min_col = find_column(&hmap, &["min stock", "min", "الحد الأدنى", "minimum", "min_stock"]);
    let max_col = find_column(&hmap, &["max stock", "max", "الحد الأقصى", "maximum", "max_stock"]);

    let conn = state.0.lock()?;
    ensure_import_history_table(&conn)?;

    let mut existing_codes: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT code FROM inventory_items") {
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                existing_codes.insert(r.to_lowercase());
            }
        }
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let file_name = Path::new(&input.file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for (i, row) in data_rows.iter().enumerate() {
        let excel_row = i + if input.skip_first_row { 2 } else { 1 };

        let name = row.get(name_col).map(cell_to_string).unwrap_or_default();
        if name.is_empty() {
            errors.push(ImportError {
                row: excel_row,
                column: "name".into(),
                value: String::new(),
                error: "Item name is required".into(),
                suggestion: "Ensure the name column is populated.".into(),
            });
            skipped += 1;
            continue;
        }

        let code = code_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let lower_code = code.to_lowercase();
        if !code.is_empty() && existing_codes.contains(&lower_code) {
            warnings.push(format!("Row {}: item code '{}' already exists, skipping.", excel_row, code));
            skipped += 1;
            continue;
        }

        let quantity = qty_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let reorder_level = reorder_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let cost = cost_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let _location = location_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let _warehouse = warehouse_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let unit = unit_col.and_then(|ci| row.get(ci)).map(cell_to_string).unwrap_or_default();
        let min_stock = min_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);
        let max_stock = max_col.and_then(|ci| row.get(ci)).and_then(cell_to_f64).unwrap_or(0.0);

        if quantity < 0.0 {
            errors.push(ImportError {
                row: excel_row,
                column: "quantity".into(),
                value: format!("{}", quantity),
                error: "Quantity cannot be negative".into(),
                suggestion: "Use a non-negative value for quantity.".into(),
            });
        }
        if cost < 0.0 {
            errors.push(ImportError {
                row: excel_row,
                column: "cost".into(),
                value: format!("{}", cost),
                error: "Cost cannot be negative".into(),
                suggestion: "Use a non-negative value for cost.".into(),
            });
        }
        if max_stock > 0.0 && min_stock > max_stock {
            warnings.push(format!("Row {}: min stock ({}) > max stock ({}).", excel_row, min_stock, max_stock));
        }
        if reorder_level > 0.0 && max_stock > 0.0 && reorder_level > max_stock {
            warnings.push(format!("Row {}: reorder level ({}) > max stock ({}).", excel_row, reorder_level, max_stock));
        }
        if quantity < reorder_level && reorder_level > 0.0 {
            warnings.push(format!("Row {}: quantity ({}) is below reorder level ({}).", excel_row, quantity, reorder_level));
        }

        if input.dry_run {
            if !code.is_empty() {
                existing_codes.insert(lower_code);
            }
            imported += 1;
            continue;
        }

        let item_code = if code.is_empty() {
            format!("INV-{}", 1000 + imported)
        } else {
            code.clone()
        };

        conn.execute(
            "INSERT INTO inventory_items (code, name_en, qty_on_hand, reorder_level, avg_cost_milli, uom, active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            rusqlite::params![item_code, name, quantity, reorder_level, (cost * 1000.0) as i64, unit],
        )
        .map_err(|e| format!("Failed to insert inventory item '{}': {}", name, e))?;

        if !code.is_empty() {
            existing_codes.insert(lower_code);
        }
        imported += 1;
    }

    let status = if errors.is_empty() { "completed" } else { "completed_with_errors" };
    let history_id = insert_import_history(&conn, "inventory", &file_name, data_rows.len(), imported, skipped, status)?;
    drop(conn);

    Ok(ExcelImportResult {
        success: errors.is_empty(),
        total_rows: data_rows.len(),
        imported,
        skipped,
        errors,
        warnings,
        summary: format!("Inventory import: {} processed, {} imported, {} skipped.", data_rows.len(), imported, skipped),
        import_id: Some(history_id),
    })
}

// ---------------------------------------------------------------------------
// 7. excel_analyze_data
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_analyze_data(
    state: State<'_, DbState>,
    input: ExcelAnalyzeInput,
) -> Result<ExcelAnalysis, AppError> {
    let (headers, data) = read_all_rows(&input.file_path, &input.sheet_name)?;
    let total_rows = data.len();
    let total_columns = headers.len();

    let sample_data: Vec<Vec<serde_json::Value>> = data
        .iter()
        .take(10)
        .map(|row| row.iter().map(cell_to_json).collect())
        .collect();

    let hmap = header_index_map(&headers);
    let mut detected_types = Vec::new();
    let mut validation_errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut confirmation_reasons = Vec::new();

    for (col_idx, header) in headers.iter().enumerate() {
        let mut null_count = 0usize;
        let mut unique_vals: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut sample_values = Vec::new();
        let mut type_votes: HashMap<&str, usize> = HashMap::new();
        type_votes.insert("text", 0);
        type_votes.insert("number", 0);
        type_votes.insert("date", 0);
        type_votes.insert("currency", 0);
        type_votes.insert("id", 0);

        for row in data.iter() {
            let cell = row.get(col_idx).cloned().unwrap_or(Data::Empty);
            if cell == Data::Empty {
                null_count += 1;
                continue;
            }
            let s = cell_to_string(&cell);
            unique_vals.insert(s.to_lowercase());

            if sample_values.len() < 5 {
                sample_values.push(s.clone());
            }

            // Type detection
            if let Some(f) = cell_to_f64(&cell) {
                if f > 0.0 && f == (f as i64) as f64 && f < 100_000.0 {
                    if let Some(v) = type_votes.get_mut("id") { *v += 1; }
                }
                if f >= 0.0 {
                    if let Some(v) = type_votes.get_mut("currency") { *v += 1; }
                }
                if let Some(v) = type_votes.get_mut("number") { *v += 1; }
            } else if cell_to_i64(&cell).is_some() {
                if let Some(v) = type_votes.get_mut("number") { *v += 1; }
                if let Some(v) = type_votes.get_mut("id") { *v += 1; }
            } else {
                let lower = s.to_lowercase();
                // Date detection
                if lower.contains('/') || lower.contains('-') || lower.contains('.') {
                    let parts: Vec<&str> = lower.split(&['/', '-', '.'][..]).collect();
                    if parts.len() == 3
                        && parts.iter().all(|p| p.parse::<u32>().is_ok()) {
                            if let Some(v) = type_votes.get_mut("date") { *v += 3; }
                        }
                }
                if let Some(v) = type_votes.get_mut("text") { *v += 1; }
            }
        }

        let best_type = type_votes
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.to_string())
            .unwrap_or_else(|| "text".to_string());

        let null_pct = if total_rows > 0 {
            (null_count as f64 / total_rows as f64 * 100.0) as usize
        } else {
            0
        };

        let suggestion = if null_pct > 50 {
            Some(format!("Column is {}% empty. Consider if this column is needed.", null_pct))
        } else if best_type == "id" && unique_vals.len() == total_rows - null_count {
            Some("Values appear to be unique IDs.".into())
        } else if best_type == "currency" {
            Some("Detected as currency/amount. Ensure correct currency unit.".into())
        } else if best_type == "date" {
            Some("Detected as date. Ensure consistent date format (YYYY-MM-DD recommended).".into())
        } else {
            None
        };

        detected_types.push(ColumnAnalysis {
            column: header.clone(),
            detected_type: best_type,
            null_count,
            unique_count: unique_vals.len(),
            sample_values,
            suggestion,
        });
    }

    // Validate against import type expectations
    let conn = state.0.lock()?;
    ensure_import_history_table(&conn)?;
    drop(conn);

    match input.import_type.as_str() {
        "journal" => {
            let has_debit = hmap.contains_key("debit") || hmap.contains_key("المدين");
            let has_credit = hmap.contains_key("credit") || hmap.contains_key("الدائن");
            let has_account = find_column(&hmap, &["account", "الحساب", "account_name", "account_code", "اسم الحساب", "رمز الحساب"]);
            let has_date = find_column(&hmap, &["date", "التاريخ", "entry_date"]);

            if !has_debit {
                validation_errors.push(ValidationError {
                    row: 0,
                    column: "debit".into(),
                    value: String::new(),
                    error_type: "missing_required".into(),
                    message: "Debit column not found".into(),
                    suggestion: "Map the debit column in column mapping, or rename the column to 'debit' or 'المدين'.".into(),
                });
            }
            if !has_credit {
                validation_errors.push(ValidationError {
                    row: 0,
                    column: "credit".into(),
                    value: String::new(),
                    error_type: "missing_required".into(),
                    message: "Credit column not found".into(),
                    suggestion: "Map the credit column in column mapping, or rename the column to 'credit' or 'الدائن'.".into(),
                });
            }
            if has_account.is_none() {
                validation_errors.push(ValidationError {
                    row: 0,
                    column: "account".into(),
                    value: String::new(),
                    error_type: "missing_required".into(),
                    message: "Account column not found".into(),
                    suggestion: "Map the account column in column mapping.".into(),
                });
            }
            if has_date.is_none() {
                warnings.push("No date column detected. Entries will use the current date.".into());
            }
            if has_debit && has_credit {
                // Check balance
                let mut total_debit = 0.0f64;
                let mut total_credit = 0.0f64;
                let debit_ci = hmap.get("debit").or_else(|| hmap.get("المدين")).copied();
                let credit_ci = hmap.get("credit").or_else(|| hmap.get("الدائن")).copied();
                for row in &data {
                    if let Some(ci) = debit_ci {
                        if let Some(c) = row.get(ci) {
                            total_debit += cell_to_f64(c).unwrap_or(0.0);
                        }
                    }
                    if let Some(ci) = credit_ci {
                        if let Some(c) = row.get(ci) {
                            total_credit += cell_to_f64(c).unwrap_or(0.0);
                        }
                    }
                }
                let diff = (total_debit - total_credit).abs();
                if diff > 0.01 {
                    warnings.push(format!(
                        "Total debits ({}) ≠ total credits ({}). Difference: {}. Journal entries should balance.",
                        total_debit, total_credit, diff
                    ));
                    confirmation_reasons.push("Journal entries are not balanced. Import may create unbalanced entries.".into());
                }
            }
        }
        "customers" => {
            let has_name = find_column(&hmap, &["name", "customer name", "الاسم", "اسم العميل"]);
            if has_name.is_none() {
                validation_errors.push(ValidationError {
                    row: 0,
                    column: "name".into(),
                    value: String::new(),
                    error_type: "missing_required".into(),
                    message: "Customer name column not found".into(),
                    suggestion: "Map the name column in column mapping.".into(),
                });
            }
        }
        "products" => {
            let has_name = find_column(&hmap, &["name", "product name", "الاسم", "اسم المنتج"]);
            if has_name.is_none() {
                validation_errors.push(ValidationError {
                    row: 0,
                    column: "name".into(),
                    value: String::new(),
                    error_type: "missing_required".into(),
                    message: "Product name column not found".into(),
                    suggestion: "Map the name column in column mapping.".into(),
                });
            }
        }
        "inventory" => {
            let has_name = find_column(&hmap, &["name", "item name", "الاسم", "اسم الصنف"]);
            if has_name.is_none() {
                validation_errors.push(ValidationError {
                    row: 0,
                    column: "name".into(),
                    value: String::new(),
                    error_type: "missing_required".into(),
                    message: "Item name column not found".into(),
                    suggestion: "Map the name column in column mapping.".into(),
                });
            }
        }
        _ => {}
    }

    // Check for duplicates in key columns
    for (col_idx, header) in headers.iter().enumerate() {
        let lower_header = normalize_header(header);
        let is_key = matches!(
            lower_header.as_str(),
            "code" | "الرمز" | "product code" | "item code" | "customer code" | "barcode" | "الباركود"
        );
        if is_key {
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
            for (row_idx, row) in data.iter().enumerate() {
                if let Some(c) = row.get(col_idx) {
                    let s = cell_to_string(c).to_lowercase();
                    if !s.is_empty() && !seen.insert(s.clone()) {
                        validation_errors.push(ValidationError {
                            row: row_idx + 1,
                            column: header.clone(),
                            value: s.clone(),
                            error_type: "duplicate".into(),
                            message: format!("Duplicate value '{}' in key column", s),
                            suggestion: "Ensure all codes/IDs are unique. Duplicates will be skipped during import.".into(),
                        });
                    }
                }
            }
        }
    }

    let estimated_time = if total_rows < 50 {
        "instant".into()
    } else if total_rows < 500 {
        "a few seconds".into()
    } else if total_rows < 5000 {
        "10-30 seconds".into()
    } else {
        "1-5 minutes".into()
    };

    let requires_confirmation = !confirmation_reasons.is_empty() || validation_errors.len() > 10;
    if validation_errors.len() > 10 {
        confirmation_reasons.push(format!("{} validation errors found. Please review before importing.", validation_errors.len()));
    }

    suggestions.push(format!("Total: {} rows × {} columns.", total_rows, total_columns));
    if total_rows > 1000 {
        suggestions.push("Large dataset. Consider doing a dry run first.".into());
    }

    Ok(ExcelAnalysis {
        total_rows,
        total_columns,
        headers,
        sample_data,
        detected_types,
        validation_errors,
        warnings,
        suggestions,
        estimated_import_time: estimated_time,
        requires_confirmation,
        confirmation_reasons,
    })
}

// ---------------------------------------------------------------------------
// 8. excel_get_import_history
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn excel_get_import_history(
    state: State<'_, DbState>,
) -> Result<Vec<ImportHistory>, AppError> {
    let conn = state.0.lock()?;
    ensure_import_history_table(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, import_type, file_name, total_rows, imported, skipped, status, created_at, created_by
             FROM import_history
             ORDER BY created_at DESC
             LIMIT 100",
        )
        .map_err(|e| format!("Query failed: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ImportHistory {
                id: row.get(0)?,
                import_type: row.get(1)?,
                file_name: row.get(2)?,
                total_rows: row.get::<_, i64>(3)? as usize,
                imported: row.get::<_, i64>(4)? as usize,
                skipped: row.get::<_, i64>(5)? as usize,
                status: row.get(6)?,
                created_at: row.get(7)?,
                created_by: row.get(8)?,
            })
        })
        .map_err(|e| format!("Row mapping failed: {}", e))?;

    let mut result = Vec::new();
    for row in rows {
        match row {
            Ok(r) => result.push(r),
            Err(e) => {
                warnings_log(&format!("Skipping corrupt history row: {}", e));
            }
        }
    }

    Ok(result)
}

fn warnings_log(msg: &str) {
    eprintln!("[excel_import] {}", msg);
}
