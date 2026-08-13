use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use calamine::{open_workbook_auto, Data, Reader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportPreview {
    pub entity_type: String,
    pub total_rows: usize,
    pub valid_rows: usize,
    pub errors: Vec<ImportError>,
    pub headers: Vec<String>,
    pub sample_data: Vec<Vec<String>>,
    pub mappings: Vec<FieldMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub row: usize,
    pub field: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldMapping {
    pub source_column: String,
    pub target_field: String,
    pub auto_matched: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub entity_type: String,
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub entity_type: String,
    pub file_path: String,
    pub mappings: Vec<FieldMapping>,
    pub skip_first_row: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportTemplate {
    pub entity_type: String,
    pub display_name_ar: String,
    pub description: String,
    pub columns: Vec<TemplateColumn>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateColumn {
    pub field: String,
    pub label_ar: String,
    pub required: bool,
    pub data_type: String,
    pub example: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn normalize_header(h: &str) -> String {
    h.trim().to_lowercase()
}

fn header_index_map(headers: &[String]) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(i, h)| (normalize_header(h), i))
        .collect()
}

#[allow(dead_code)]
fn first_sheet_name(file_path: &str) -> Result<String, AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::not_found(format!("الملف غير موجود: {}", file_path)));
    }
    let workbook =
        open_workbook_auto(path).map_err(|e| format!("فشل في فتح المصنف: {}", e))?;
    workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::not_found("المصنف بدون أوراق عمل"))
}

fn read_file_data(
    file_path: &str,
) -> Result<(Vec<String>, Vec<Vec<Data>>), AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::not_found(format!("الملف غير موجود: {}", file_path)));
    }
    let mut workbook =
        open_workbook_auto(path).map_err(|e| format!("فشل في فتح المصنف: {}", e))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::validation("المصنف بدون أوراق عمل"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("فشل في قراءة ورقة العمل: {}", e))?;
    let mut rows_iter = range.rows();
    let headers = match rows_iter.next() {
        Some(row) => row.iter().map(cell_to_string).collect(),
        None => return Err(AppError::validation("ورقة العمل فارغة")),
    };
    let data: Vec<Vec<Data>> = rows_iter.map(|r| r.to_vec()).collect();
    Ok((headers, data))
}

fn row_data_to_strings(row: &[Data]) -> Vec<String> {
    row.iter().map(cell_to_string).collect()
}

// ---------------------------------------------------------------------------
// Entity field definitions
// ---------------------------------------------------------------------------

const CUSTOMER_FIELDS: &[(&str, &str)] = &[
    ("name", "اسم العميل"),
    ("code", "رمز العميل"),
    ("balance", "الرصيد"),
    ("phone", "الهاتف"),
    ("email", "البريد الإلكتروني"),
    ("vat_number", "الرقم الضريبي"),
    ("address", "العنوان"),
    ("credit_limit", "حد الائتمان"),
    ("contact", "جهة الاتصال"),
    ("payment_terms", "شروط الدفع"),
];

const SUPPLIER_FIELDS: &[(&str, &str)] = &[
    ("name", "اسم المورد"),
    ("code", "رمز المورد"),
    ("balance", "الرصيد"),
    ("phone", "الهاتف"),
    ("email", "البريد الإلكتروني"),
    ("vat_number", "الرقم الضريبي"),
    ("address", "العنوان"),
    ("currency", "العملة"),
    ("contact", "جهة الاتصال"),
    ("payment_terms", "شروط الدفع"),
];

const PRODUCT_FIELDS: &[(&str, &str)] = &[
    ("name", "اسم المنتج"),
    ("code", "رمز المنتج"),
    ("price", "سعر البيع"),
    ("cost", "سعر التكلفة"),
    ("size", "المقاس"),
    ("barcode", "الباركود"),
    ("cup_type", "نوع الكوب"),
    ("cups_per_carton", "عدد الأكواب في الكرتون"),
    ("carton_type", "نوع الكرتون"),
    ("vat_pct", "نسبة الضريبة"),
    ("notes", "ملاحظات"),
];

const INVENTORY_FIELDS: &[(&str, &str)] = &[
    ("name", "اسم الصنف"),
    ("code", "رمز الصنف"),
    ("qty", "الكمية"),
    ("cost", "متوسط التكلفة"),
    ("kind", "النوع"),
    ("uom", "وحدة القياس"),
    ("reorder_level", "حد إعادة الطلب"),
    ("notes", "ملاحظات"),
];

const INVOICE_FIELDS: &[(&str, &str)] = &[
    ("date", "التاريخ"),
    ("customer", "اسم العميل"),
    ("amount", "المبلغ"),
    ("vat", "الضريبة"),
    ("product", "المنتج"),
    ("quantity", "الكمية"),
    ("unit_price", "سعر الوحدة"),
    ("payment_type", "نوع الدفع"),
    ("notes", "ملاحظات"),
];

const PURCHASE_FIELDS: &[(&str, &str)] = &[
    ("date", "التاريخ"),
    ("supplier", "اسم المورد"),
    ("amount", "المبلغ"),
    ("vat", "الضريبة"),
    ("item", "الصنف"),
    ("quantity", "الكمية"),
    ("unit_cost", "تكلفة الوحدة"),
    ("supplier_invoice_no", "رقم فاتورة المورد"),
    ("notes", "ملاحظات"),
];

const EXPENSE_FIELDS: &[(&str, &str)] = &[
    ("date", "التاريخ"),
    ("category", "الفئة"),
    ("amount", "المبلغ"),
    ("vendor", "المورد"),
    ("account_code", "رمز الحساب"),
    ("method", "طريقة الدفع"),
    ("reference", "المرجع"),
    ("vat", "الضريبة"),
    ("notes", "ملاحظات"),
];

const OPENING_BALANCE_FIELDS: &[(&str, &str)] = &[
    ("entity_type", "النوع (عميل/مورد/صنف)"),
    ("name", "الاسم"),
    ("code", "الرمز"),
    ("balance", "الرصيد الافتتاحي"),
    ("qty", "الكمية"),
    ("cost", "التكلفة"),
];

const EMPLOYEE_FIELDS: &[(&str, &str)] = &[
    ("name", "اسم الموظف"),
    ("code", "رمز الموظف"),
    ("salary", "الراتب"),
    ("job", "الوظيفة"),
    ("nationality", "الجنسية"),
    ("phone", "الهاتف"),
    ("passport_no", "رقم الجواز"),
    ("passport_expiry", "انتهاء الجواز"),
    ("residence_expiry", "انتهاء الإقامة"),
    ("joining_date", "تاريخ الالتحاق"),
    ("notes", "ملاحظات"),
];

fn get_entity_fields(entity_type: &str) -> Vec<(&'static str, &'static str)> {
    match entity_type {
        "customers" => CUSTOMER_FIELDS.to_vec(),
        "suppliers" => SUPPLIER_FIELDS.to_vec(),
        "products" => PRODUCT_FIELDS.to_vec(),
        "inventory" => INVENTORY_FIELDS.to_vec(),
        "invoices" => INVOICE_FIELDS.to_vec(),
        "purchases" => PURCHASE_FIELDS.to_vec(),
        "expenses" => EXPENSE_FIELDS.to_vec(),
        "opening_balances" => OPENING_BALANCE_FIELDS.to_vec(),
        "employees" => EMPLOYEE_FIELDS.to_vec(),
        _ => vec![],
    }
}

#[allow(dead_code)]
const CUSTOMER_HEADERS_EN: &[&str] = &[
    "name", "customer name", "phone", "email", "vat", "vat number", "credit limit",
    "address", "code", "customer code", "balance", "contact", "payment terms",
];
#[allow(dead_code)]
const CUSTOMER_HEADERS_AR: &[&str] = &[
    "الاسم", "اسم العميل", "الهاتف", "البريد الإلكتروني", "الرقم الضريبي", "الضريبة",
    "حد الائتمان", "العنوان", "الرمز", "رمز العميل", "الرصيد", "جهة الاتصال", "شروط الدفع",
];

#[allow(dead_code)]
const SUPPLIER_HEADERS_EN: &[&str] = &[
    "name", "supplier name", "phone", "email", "vat", "vat number", "address",
    "code", "supplier code", "balance", "currency", "contact", "payment terms",
];
#[allow(dead_code)]
const SUPPLIER_HEADERS_AR: &[&str] = &[
    "الاسم", "اسم المورد", "الهاتف", "البريد الإلكتروني", "الرقم الضريبي", "الضريبة",
    "العنوان", "الرمز", "رمز المورد", "الرصيد", "العملة", "جهة الاتصال", "شروط الدفع",
];

#[allow(dead_code)]
const PRODUCT_HEADERS_EN: &[&str] = &[
    "code", "product code", "name", "product name", "price", "cost", "size", "barcode",
    "cup type", "cups per carton", "carton type", "vat", "notes",
];
#[allow(dead_code)]
const PRODUCT_HEADERS_AR: &[&str] = &[
    "الرمز", "رمز المنتج", "الاسم", "اسم المنتج", "السعر", "التكلفة", "المقاس", "الباركود",
    "نوع الكوب", "عدد الأكواب في الكرتون", "نوع الكرتون", "الضريبة", "ملاحظات",
];

#[allow(dead_code)]
const INVENTORY_HEADERS_EN: &[&str] = &[
    "code", "item code", "name", "item name", "quantity", "qty", "cost", "type", "kind",
    "unit", "uom", "reorder level", "reorder", "notes",
];
#[allow(dead_code)]
const INVENTORY_HEADERS_AR: &[&str] = &[
    "الرمز", "رمز الصنف", "الاسم", "اسم الصنف", "الكمية", "الكمية", "التكلفة", "النوع", "النوع",
    "الوحدة", "وحدة القياس", "حد إعادة الطلب", "إعادة الطلب", "ملاحظات",
];

#[allow(dead_code)]
const INVOICE_HEADERS_EN: &[&str] = &[
    "date", "customer", "customer name", "amount", "vat", "total", "product", "quantity",
    "unit price", "payment type", "notes", "invoice no", "inv no",
];
#[allow(dead_code)]
const INVOICE_HEADERS_AR: &[&str] = &[
    "التاريخ", "العميل", "اسم العميل", "المبلغ", "الضريبة", "الإجمالي", "المنتج", "الكمية",
    "سعر الوحدة", "نوع الدفع", "ملاحظات", "رقم الفاتورة", "فاتورة رقم",
];

#[allow(dead_code)]
const PURCHASE_HEADERS_EN: &[&str] = &[
    "date", "supplier", "supplier name", "amount", "vat", "total", "item", "quantity",
    "unit cost", "supplier invoice no", "notes",
];
#[allow(dead_code)]
const PURCHASE_HEADERS_AR: &[&str] = &[
    "التاريخ", "المورد", "اسم المورد", "المبلغ", "الضريبة", "الإجمالي", "الصنف", "الكمية",
    "تكلفة الوحدة", "رقم فاتورة المورد", "ملاحظات",
];

#[allow(dead_code)]
const EXPENSE_HEADERS_EN: &[&str] = &[
    "date", "category", "amount", "vendor", "account code", "method", "reference",
    "vat", "notes",
];
#[allow(dead_code)]
const EXPENSE_HEADERS_AR: &[&str] = &[
    "التاريخ", "الفئة", "المبلغ", "المورد", "رمز الحساب", "طريقة الدفع", "المرجع",
    "الضريبة", "ملاحظات",
];

#[allow(dead_code)]
const EMPLOYEE_HEADERS_EN: &[&str] = &[
    "name", "employee name", "code", "salary", "job", "nationality", "phone",
    "passport no", "passport expiry", "residence expiry", "joining date", "notes",
];
#[allow(dead_code)]
const EMPLOYEE_HEADERS_AR: &[&str] = &[
    "الاسم", "اسم الموظف", "الرمز", "الراتب", "الوظيفة", "الجنسية", "الهاتف",
    "رقم الجواز", "انتهاء الجواز", "انتهاء الإقامة", "تاريخ الالتحاق", "ملاحظات",
];

fn auto_match_columns(
    headers: &[String],
    entity_type: &str,
) -> Vec<FieldMapping> {
    let hmap = header_index_map(headers);
    let entity_fields = get_entity_fields(entity_type);
    let mut mappings = Vec::new();

    for (field, _label) in &entity_fields {
        let candidates = column_candidates_for_field(field, entity_type);
        for c in &candidates {
            if let Some(&idx) = hmap.get(&normalize_header(c)) {
                mappings.push(FieldMapping {
                    source_column: headers[idx].clone(),
                    target_field: field.to_string(),
                    auto_matched: true,
                });
                break;
            }
        }
    }
    mappings
}

fn column_candidates_for_field(field: &str, entity_type: &str) -> Vec<&'static str> {
    match field {
        "name" => match entity_type {
            "customers" => vec!["name", "customer name", "customer_name", "الاسم", "اسم العميل"],
            "suppliers" => vec!["name", "supplier name", "supplier_name", "الاسم", "اسم المورد"],
            "products" => vec!["name", "product name", "product_name", "الاسم", "اسم المنتج"],
            "inventory" => vec!["name", "item name", "item_name", "الاسم", "اسم الصنف"],
            "employees" => vec!["name", "employee name", "employee_name", "الاسم", "اسم الموظف"],
            "invoices" => vec!["customer", "customer name", "العميل", "اسم العميل"],
            "purchases" => vec!["supplier", "supplier name", "المورد", "اسم المورد"],
            _ => vec!["name", "الاسم"],
        },
        "code" => match entity_type {
            "customers" => vec!["code", "customer code", "customer_code", "الرمز", "رمز العميل"],
            "suppliers" => vec!["code", "supplier code", "supplier_code", "الرمز", "رمز المورد"],
            "products" => vec!["code", "product code", "product_code", "sku", "الرمز", "رمز المنتج"],
            "inventory" => vec!["code", "item code", "item_code", "sku", "الرمز", "رمز الصنف"],
            "employees" => vec!["code", "employee code", "employee_code", "الرمز", "رمز الموظف"],
            _ => vec!["code", "الرمز"],
        },
        "balance" => vec!["balance", "opening balance", "opening_balance", "amount", "الرصيد", "المبلغ"],
        "phone" => vec!["phone", "mobile", "tel", "الهاتف", "الجوال"],
        "email" => vec!["email", "e-mail", "البريد الإلكتروني", "البريد"],
        "vat_number" => vec!["vat", "vat number", "vat_number", "tax_id", "الرقم الضريبي", "الضريبة"],
        "address" => vec!["address", "العنوان"],
        "credit_limit" => vec!["credit limit", "credit_limit", "حد الائتمان"],
        "contact" => vec!["contact", "جهة الاتصال"],
        "payment_terms" => vec!["payment terms", "payment_terms", "شروط الدفع"],
        "currency" => vec!["currency", "العملة"],
        "price" => vec!["price", "sale price", "selling_price", "السعر", "سعر البيع"],
        "cost" => vec!["cost", "cost price", "cost_price", "unit cost", "unit_cost", "التكلفة", "سعر التكلفة"],
        "size" => vec!["size", "dimension", "المقاس"],
        "barcode" => vec!["barcode", "bar code", "upc", "الباركود"],
        "cup_type" => vec!["cup type", "cup_type", "نوع الكوب"],
        "cups_per_carton" => vec!["cups per carton", "cups_per_carton", "عدد الأكواب في الكرتون"],
        "carton_type" => vec!["carton type", "carton_type", "نوع الكرتون"],
        "vat_pct" => vec!["vat", "vat %", "vat_pct", "vat percent", "الضريبة", "نسبة الضريبة"],
        "qty" | "quantity" => vec!["quantity", "qty", "stock", "الكمية", "المخزون"],
        "kind" => vec!["type", "kind", "category", "النوع", "الفئة"],
        "uom" => vec!["unit", "uom", "unit of measure", "الوحدة", "وحدة القياس"],
        "reorder_level" => vec!["reorder level", "reorder", "reorder_level", "حد إعادة الطلب"],
        "date" => vec!["date", "التاريخ"],
        "amount" => vec!["amount", "total", "total_milli", "المبلغ", "الإجمالي"],
        "product" => vec!["product", "product name", "المنتج", "اسم المنتج"],
        "unit_price" => vec!["unit price", "unit_price", "سعر الوحدة"],
        "unit_cost" => vec!["unit cost", "unit_cost", "تكلفة الوحدة"],
        "supplier_invoice_no" => vec!["supplier invoice no", "supplier_invoice_no", "invoice no", "رقم فاتورة المورد"],
        "category" => vec!["category", "الفئة"],
        "vendor" => vec!["vendor", "supplier", "المورد", "الجهة"],
        "account_code" => vec!["account code", "account_code", "الحساب", "رمز الحساب"],
        "method" => vec!["method", "payment method", "طريقة الدفع"],
        "reference" => vec!["reference", "ref", "المرجع"],
        "vat" => vec!["vat", "vat amount", "الضريبة"],
        "job" => vec!["job", "title", "position", "الوظيفة", "المسمى الوظيفي"],
        "nationality" => vec!["nationality", "الجنسية"],
        "passport_no" => vec!["passport no", "passport_no", "رقم الجواز"],
        "passport_expiry" => vec!["passport expiry", "passport_expiry", "انتهاء الجواز"],
        "residence_expiry" => vec!["residence expiry", "residence_expiry", "انتهاء الإقامة"],
        "joining_date" => vec!["joining date", "joining_date", "تاريخ الالتحاق"],
        "notes" => vec!["notes", "ملاحظات"],
        "entity_type" => vec!["entity type", "entity_type", "النوع", "نوع الكيان"],
        _ => vec![],
    }
}

fn validate_preview(
    rows: &[Vec<String>],
    mappings: &[FieldMapping],
    entity_type: &str,
    all_headers: &[String],
) -> (usize, Vec<ImportError>) {
    let mut errors = Vec::new();
    let hmap = header_index_map(all_headers);

    let required_fields: Vec<&str> = match entity_type {
        "customers" => vec!["name"],
        "suppliers" => vec!["name"],
        "products" => vec!["name"],
        "inventory" => vec!["name"],
        "invoices" => vec!["date", "customer"],
        "purchases" => vec!["date", "supplier"],
        "expenses" => vec!["date", "amount"],
        "employees" => vec!["name"],
        "opening_balances" => vec!["entity_type", "name", "balance"],
        _ => vec![],
    };

    let mapped_fields: std::collections::HashSet<String> =
        mappings.iter().map(|m| m.target_field.clone()).collect();

    for req in &required_fields {
        if !mapped_fields.contains(*req) {
            errors.push(ImportError {
                row: 0,
                field: req.to_string(),
                message: format!("الحقل المطلوب '{}' غير مطابق في الأعمدة", req),
                severity: "error".to_string(),
            });
        }
    }

    let mut valid = 0usize;
    for (i, row) in rows.iter().enumerate() {
        let row_num = i + 2;
        let mut row_has_error = false;

        for req in &required_fields {
            if let Some(mapping) = mappings.iter().find(|m| m.target_field == *req) {
                if let Some(&col_idx) = hmap.get(&normalize_header(&mapping.source_column)) {
                    let val = row.get(col_idx).map(|s| s.trim()).unwrap_or("");
                    if val.is_empty() {
                        errors.push(ImportError {
                            row: row_num,
                            field: req.to_string(),
                            message: format!("الحقل المطلوب '{}' فارغ في الصف {}", req, row_num),
                            severity: "warning".to_string(),
                        });
                        row_has_error = true;
                    }
                }
            }
        }

        for mapping in mappings {
            if let Some(&col_idx) = hmap.get(&normalize_header(&mapping.source_column)) {
                let val = row.get(col_idx).map(|s| s.trim()).unwrap_or("");
                if val.is_empty() {
                    continue;
                }
                match mapping.target_field.as_str() {
                    "balance" | "amount" | "price" | "cost" | "unit_price" | "unit_cost"
                    | "salary" | "credit_limit" | "vat" | "vat_pct" => {
                        if val.parse::<f64>().is_err() {
                            errors.push(ImportError {
                                row: row_num,
                                field: mapping.target_field.clone(),
                                message: format!("القيمة '{}' ليست رقماً صحيحاً في الحقل '{}'", val, mapping.target_field),
                                severity: "error".to_string(),
                            });
                            row_has_error = true;
                        }
                    }
                    "quantity" | "qty" | "cups_per_carton" => {
                        if val.parse::<f64>().is_err() {
                            errors.push(ImportError {
                                row: row_num,
                                field: mapping.target_field.clone(),
                                message: format!("القيمة '{}' ليست رقماً صحيحاً في الحقل '{}'", val, mapping.target_field),
                                severity: "error".to_string(),
                            });
                            row_has_error = true;
                        }
                    }
                    "date"
                        if parse_date_flexible(val).is_none() => {
                            errors.push(ImportError {
                                row: row_num,
                                field: "date".to_string(),
                                message: format!("صيغة التاريخ '{}' غير صحيحة", val),
                                severity: "warning".to_string(),
                            });
                        }
                    _ => {}
                }
            }
        }

        if !row_has_error {
            valid += 1;
        }
    }

    (valid, errors)
}

fn headers_from_mappings(mappings: &[FieldMapping]) -> Vec<String> {
    mappings.iter().map(|m| m.source_column.clone()).collect()
}

fn find_value_in_row(
    row: &[String],
    mappings: &[FieldMapping],
    target_field: &str,
    _all_headers: &[String],
) -> Option<String> {
    if let Some(mapping) = mappings.iter().find(|m| m.target_field == target_field) {
        let col_name_lower = normalize_header(&mapping.source_column);
        for (i, h) in _all_headers.iter().enumerate() {
            if normalize_header(h) == col_name_lower {
                return row.get(i).cloned();
            }
        }
    }
    None
}

fn parse_date_flexible(s: &str) -> Option<String> {
    let s = s.trim();
    // Try YYYY-MM-DD
    if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        return Some(s.to_string());
    }
    // Try DD/MM/YYYY or DD-MM-YYYY
    for sep in &['/', '-', '.'] {
        let parts: Vec<&str> = s.split(*sep).collect();
        if parts.len() == 3 {
            if let (Ok(d), Ok(m), Ok(y)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                let (day, month, year) = if y > 31 {
                    (d, m, y)
                } else {
                    (m, d, y)
                };
                if (1..=12).contains(&month) && (1..=31).contains(&day) && year >= 2000 {
                    return Some(format!("{:04}-{:02}-{:02}", year, month, day));
                }
            }
        }
    }
    // Try YYYY/MM/DD
    {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            if let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                if y > 31 && month_in_range(m) && day_in_range(d) {
                    return Some(format!("{:04}-{:02}-{:02}", y, m, d));
                }
            }
        }
    }
    None
}

fn month_in_range(m: u32) -> bool {
    (1..=12).contains(&m)
}

fn day_in_range(d: u32) -> bool {
    (1..=31).contains(&d)
}

fn parse_amount_to_milli(s: &str) -> i64 {
    let s = s.trim().replace(',', "");
    if let Ok(f) = s.parse::<f64>() {
        (f * 1000.0).round() as i64
    } else {
        0
    }
}

fn parse_amount_to_f64(s: &str) -> f64 {
    let s = s.trim().replace(',', "");
    s.parse::<f64>().unwrap_or(0.0)
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn preview_import(
    file_path: String,
    entity_type: String,
) -> Result<ImportPreview, AppError> {
    let valid_entity_types = [
        "customers", "suppliers", "products", "inventory",
        "invoices", "purchases", "expenses", "opening_balances", "employees",
    ];
    if !valid_entity_types.contains(&entity_type.as_str()) {
        return Err(AppError::validation(format!("نوع الكيان '{}' غير معروف", entity_type)));
    }

    let (headers, raw_data) = read_file_data(&file_path)?;
    let string_rows: Vec<Vec<String>> = raw_data.iter().map(|row| row_data_to_strings(row)).collect();

    let mappings = auto_match_columns(&headers, &entity_type);
    let total_rows = string_rows.len();

    let sample_data: Vec<Vec<String>> = string_rows.iter().take(10).cloned().collect();

    let (valid_rows, errors) = validate_preview(&string_rows, &mappings, &entity_type, &headers);

    Ok(ImportPreview {
        entity_type,
        total_rows,
        valid_rows,
        errors,
        headers,
        sample_data,
        mappings,
    })
}

#[tauri::command]
pub fn execute_import(
    state: State<'_, DbState>,
    user_id: i64,
    input: ImportRequest,
) -> Result<ImportResult, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;
    let skip_header = input.skip_first_row.unwrap_or(true);
    let (_, raw_data) = read_file_data(&input.file_path)?;
    let mut data: Vec<Vec<String>> = raw_data.iter().map(|row| row_data_to_strings(row)).collect();
    let total_rows = data.len();
    if skip_header && !data.is_empty() {
        data.remove(0);
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<ImportError> = Vec::new();

    match input.entity_type.as_str() {
        "customers" => {
            let mut existing: std::collections::HashSet<String> =
                load_existing_names(&conn, "SELECT name FROM customers WHERE active=1")
                    .unwrap_or_default();
            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let name = find_value_in_row(row, &input.mappings, "name", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                if name.trim().is_empty() {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: "اسم العميل مطلوب".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                }
                let lower_name = name.trim().to_lowercase();
                if existing.contains(&lower_name) {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: format!("العميل '{}' موجود مسبقاً", name), severity: "warning".into() });
                    skipped += 1;
                    continue;
                }
                let code = find_value_in_row(row, &input.mappings, "code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let balance = find_value_in_row(row, &input.mappings, "balance", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let phone = find_value_in_row(row, &input.mappings, "phone", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let email = find_value_in_row(row, &input.mappings, "email", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vat_number = find_value_in_row(row, &input.mappings, "vat_number", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let address = find_value_in_row(row, &input.mappings, "address", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let contact = find_value_in_row(row, &input.mappings, "contact", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let credit_limit = find_value_in_row(row, &input.mappings, "credit_limit", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let payment_terms = find_value_in_row(row, &input.mappings, "payment_terms", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let bal_milli = parse_amount_to_milli(&balance);
                let cl_milli = parse_amount_to_milli(&credit_limit);
                let customer_code = if code.trim().is_empty() {
                    auto_generate_code(&conn, "CUST")?
                } else {
                    code.trim().to_string()
                };

                conn.execute(
                    "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, opening_balance_milli, balance_milli) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        customer_code, name.trim(), "credit", if contact.is_empty() { None } else { Some(&contact) },
                        if phone.is_empty() { None } else { Some(&phone) },
                        if email.is_empty() { None } else { Some(&email) },
                        if address.is_empty() { None } else { Some(&address) },
                        if vat_number.is_empty() { None } else { Some(&vat_number) },
                        cl_milli,
                        if payment_terms.is_empty() { None } else { Some(&payment_terms) },
                        bal_milli, bal_milli,
                    ],
                ).map_err(|e| format!("فشل إدراج العميل '{}': {}", name.trim(), e))?;

                existing.insert(lower_name);
                imported += 1;
            }
        }
        "suppliers" => {
            let mut existing: std::collections::HashSet<String> =
                load_existing_names(&conn, "SELECT name FROM suppliers WHERE active=1")
                    .unwrap_or_default();
            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let name = find_value_in_row(row, &input.mappings, "name", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                if name.trim().is_empty() {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: "اسم المورد مطلوب".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                }
                let lower_name = name.trim().to_lowercase();
                if existing.contains(&lower_name) {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: format!("المورد '{}' موجود مسبقاً", name), severity: "warning".into() });
                    skipped += 1;
                    continue;
                }
                let code = find_value_in_row(row, &input.mappings, "code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let balance = find_value_in_row(row, &input.mappings, "balance", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let phone = find_value_in_row(row, &input.mappings, "phone", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let email = find_value_in_row(row, &input.mappings, "email", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vat_number = find_value_in_row(row, &input.mappings, "vat_number", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let address = find_value_in_row(row, &input.mappings, "address", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let contact = find_value_in_row(row, &input.mappings, "contact", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let currency = find_value_in_row(row, &input.mappings, "currency", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let payment_terms = find_value_in_row(row, &input.mappings, "payment_terms", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let bal_milli = parse_amount_to_milli(&balance);
                let supplier_code = if code.trim().is_empty() {
                    auto_generate_code(&conn, "SUP")?
                } else {
                    code.trim().to_string()
                };

                conn.execute(
                    "INSERT INTO suppliers(code, name, contact, phone, email, address, vat_number, currency, payment_terms, opening_balance_milli, balance_milli) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        supplier_code, name.trim(),
                        if contact.is_empty() { None } else { Some(&contact) },
                        if phone.is_empty() { None } else { Some(&phone) },
                        if email.is_empty() { None } else { Some(&email) },
                        if address.is_empty() { None } else { Some(&address) },
                        if vat_number.is_empty() { None } else { Some(&vat_number) },
                        if currency.is_empty() { "OMR".to_string() } else { currency },
                        if payment_terms.is_empty() { None } else { Some(&payment_terms) },
                        bal_milli, bal_milli,
                    ],
                ).map_err(|e| format!("فشل إدراج المورد '{}': {}", name.trim(), e))?;

                existing.insert(lower_name);
                imported += 1;
            }
        }
        "products" => {
            let mut existing: std::collections::HashSet<String> =
                load_existing_names(&conn, "SELECT code FROM products WHERE active=1")
                    .unwrap_or_default();
            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let name = find_value_in_row(row, &input.mappings, "name", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                if name.trim().is_empty() {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: "اسم المنتج مطلوب".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                }
                let code = find_value_in_row(row, &input.mappings, "code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let lower_code = code.trim().to_lowercase();
                if !code.trim().is_empty() && existing.contains(&lower_code) {
                    errors.push(ImportError { row: row_num, field: "code".into(), message: format!("رمز المنتج '{}' موجود مسبقاً", code), severity: "warning".into() });
                    skipped += 1;
                    continue;
                }
                let price = find_value_in_row(row, &input.mappings, "price", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let cost = find_value_in_row(row, &input.mappings, "cost", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let size = find_value_in_row(row, &input.mappings, "size", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let barcode = find_value_in_row(row, &input.mappings, "barcode", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let cup_type = find_value_in_row(row, &input.mappings, "cup_type", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let cups_per_carton = find_value_in_row(row, &input.mappings, "cups_per_carton", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let carton_type = find_value_in_row(row, &input.mappings, "carton_type", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vat_pct = find_value_in_row(row, &input.mappings, "vat_pct", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let notes = find_value_in_row(row, &input.mappings, "notes", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let price_milli = parse_amount_to_milli(&price);
                let cost_milli = parse_amount_to_milli(&cost);
                let cups: i64 = cups_per_carton.trim().parse().unwrap_or(1000);
                let vat: f64 = vat_pct.trim().parse().unwrap_or(15.0);
                let product_code = if code.trim().is_empty() {
                    auto_generate_code(&conn, "PRD")?
                } else {
                    code.trim().to_string()
                };

                conn.execute(
                    "INSERT INTO products(code, name_ar, name_en, size, cup_type, cups_per_carton, carton_type, default_price_milli, default_cost_milli, vat_pct, barcode, notes) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        product_code,
                        name.trim(),
                        name.trim(),
                        if size.is_empty() { None } else { Some(&size) },
                        if cup_type.is_empty() { None } else { Some(&cup_type) },
                        cups,
                        if carton_type.is_empty() { None } else { Some(&carton_type) },
                        price_milli, cost_milli, vat,
                        if barcode.is_empty() { None } else { Some(&barcode) },
                        if notes.is_empty() { None } else { Some(&notes) },
                    ],
                ).map_err(|e| format!("فشل إدراج المنتج '{}': {}", name.trim(), e))?;

                if !code.trim().is_empty() {
                    existing.insert(lower_code);
                }
                imported += 1;
            }
        }
        "inventory" => {
            let mut existing: std::collections::HashSet<String> =
                load_existing_names(&conn, "SELECT code FROM inventory_items WHERE active=1")
                    .unwrap_or_default();
            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let name = find_value_in_row(row, &input.mappings, "name", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                if name.trim().is_empty() {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: "اسم الصنف مطلوب".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                }
                let code = find_value_in_row(row, &input.mappings, "code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let lower_code = code.trim().to_lowercase();
                if !code.trim().is_empty() && existing.contains(&lower_code) {
                    errors.push(ImportError { row: row_num, field: "code".into(), message: format!("رمز الصنف '{}' موجود مسبقاً", code), severity: "warning".into() });
                    skipped += 1;
                    continue;
                }
                let qty = find_value_in_row(row, &input.mappings, "qty", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let cost = find_value_in_row(row, &input.mappings, "cost", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let kind = find_value_in_row(row, &input.mappings, "kind", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let uom = find_value_in_row(row, &input.mappings, "uom", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let reorder_level = find_value_in_row(row, &input.mappings, "reorder_level", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let notes = find_value_in_row(row, &input.mappings, "notes", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let qty_val: f64 = qty.trim().parse().unwrap_or(0.0);
                let cost_val: i64 = (parse_amount_to_f64(&cost) * 1000.0).round() as i64;
                let reorder: f64 = reorder_level.trim().parse().unwrap_or(0.0);
                let item_code = if code.trim().is_empty() {
                    auto_generate_code(&conn, "INV")?
                } else {
                    code.trim().to_string()
                };

                conn.execute(
                    "INSERT INTO inventory_items(code, name_ar, name_en, kind, uom, qty_on_hand, avg_cost_milli, reorder_level, notes) VALUES(?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        item_code, name.trim(), name.trim(),
                        if kind.is_empty() { "raw".to_string() } else { kind },
                        if uom.is_empty() { "pcs".to_string() } else { uom },
                        qty_val, cost_val, reorder,
                        if notes.is_empty() { None } else { Some(&notes) },
                    ],
                ).map_err(|e| format!("فشل إدراج الصنف '{}': {}", name.trim(), e))?;

                if !code.trim().is_empty() {
                    existing.insert(lower_code);
                }
                imported += 1;
            }
        }
        "invoices" => {
            let mut customer_cache: HashMap<String, i64> = HashMap::new();
            {
                let mut stmt = conn
                    .prepare("SELECT id, name FROM customers WHERE active=1")
                    ?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, i64>(0)?)))
                    ?;
                for r in rows {
                    let (name, id) = r?;
                    customer_cache.insert(name.to_lowercase(), id);
                }
            }
            let mut product_cache: HashMap<String, i64> = HashMap::new();
            {
                let mut stmt = conn
                    .prepare("SELECT id, COALESCE(name_ar,'') FROM products WHERE active=1")
                    ?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, i64>(0)?)))
                    ?;
                for r in rows {
                    let (name, id) = r?;
                    product_cache.insert(name.to_lowercase(), id);
                }
            }
            let year = chrono::Utc::now().format("%Y").to_string();

            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let date_str = find_value_in_row(row, &input.mappings, "date", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let customer_name = find_value_in_row(row, &input.mappings, "customer", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let amount_str = find_value_in_row(row, &input.mappings, "amount", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vat_str = find_value_in_row(row, &input.mappings, "vat", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let product_name = find_value_in_row(row, &input.mappings, "product", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let qty_str = find_value_in_row(row, &input.mappings, "quantity", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let unit_price_str = find_value_in_row(row, &input.mappings, "unit_price", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let payment_type = find_value_in_row(row, &input.mappings, "payment_type", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let notes = find_value_in_row(row, &input.mappings, "notes", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let date = parse_date_flexible(&date_str).unwrap_or_else(|| "2000-01-01".to_string());

                let customer_id = if !customer_name.trim().is_empty() {
                    customer_cache.get(&customer_name.trim().to_lowercase()).copied().unwrap_or_else(|| {
                        let _cid = conn.last_insert_rowid();
                        let _ = conn.execute(
                            "INSERT INTO customers(code, name, ctype) VALUES(?1, ?2, 'credit')",
                            rusqlite::params![auto_generate_code(&conn, "CUST").unwrap_or_default(), customer_name.trim()],
                        );
                        let new_id = conn.last_insert_rowid();
                        customer_cache.insert(customer_name.trim().to_lowercase(), new_id);
                        new_id
                    })
                } else {
                    errors.push(ImportError { row: row_num, field: "customer".into(), message: "اسم العميل مطلوب للفاتورة".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                };

                let net_milli = parse_amount_to_milli(&amount_str);
                let vat_milli = parse_amount_to_milli(&vat_str);
                let total_milli = net_milli + vat_milli;

                let inv_no = format!("INV-{}-{:04}", year, next_sequence(&conn, "INV", &year)?);
                let ptype = if payment_type.trim().is_empty() { "credit".to_string() } else { payment_type };

                conn.execute(
                    "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, vat_enabled, net_milli, vat_milli, total_milli, status, notes) VALUES(?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        inv_no, date, customer_id, ptype,
                        if vat_milli > 0 { 1i64 } else { 0i64 },
                        net_milli, vat_milli, total_milli, "Posted",
                        if notes.is_empty() { None } else { Some(&notes) },
                    ],
                ).map_err(|e| format!("فشل إدراج الفاتورة: {}", e))?;
                let inv_id = conn.last_insert_rowid();

                if !product_name.trim().is_empty() {
                    let product_id = product_cache.get(&product_name.trim().to_lowercase()).copied().unwrap_or(0);
                    if product_id > 0 {
                        let qty: f64 = qty_str.trim().parse().unwrap_or(1.0);
                        let unit_price = parse_amount_to_milli(&unit_price_str);
                        let line_net = ((qty * unit_price as f64).round()) as i64;
                        let cups_per: i64 = conn
                            .query_row("SELECT cups_per_carton FROM products WHERE id=?", [product_id], |r| r.get(0))
                            .unwrap_or(1000);
                        let vat_pct: f64 = conn
                            .query_row("SELECT vat_pct FROM products WHERE id=?", [product_id], |r| r.get(0))
                            .unwrap_or(15.0);
                        let line_vat = ((line_net as f64 * vat_pct / 100.0).round()) as i64;
                        let qty_cups = qty * cups_per as f64;

                        conn.execute(
                            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,?,?,?,?,?,?)",
                            rusqlite::params![inv_id, product_id, qty, cups_per, qty_cups, unit_price, line_net, vat_pct, line_vat],
                        ).map_err(|e| format!("فشل إدراج بند الفاتورة: {}", e))?;
                    }
                }

                imported += 1;
            }
        }
        "purchases" => {
            let mut supplier_cache: HashMap<String, i64> = HashMap::new();
            {
                let mut stmt = conn
                    .prepare("SELECT id, name FROM suppliers WHERE active=1")
                    ?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, i64>(0)?)))
                    ?;
                for r in rows {
                    let (name, id) = r?;
                    supplier_cache.insert(name.to_lowercase(), id);
                }
            }
            let mut item_cache: HashMap<String, i64> = HashMap::new();
            {
                let mut stmt = conn
                    .prepare("SELECT id, COALESCE(name_ar,'') FROM inventory_items WHERE active=1")
                    ?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, i64>(0)?)))
                    ?;
                for r in rows {
                    let (name, id) = r?;
                    item_cache.insert(name.to_lowercase(), id);
                }
            }
            let year = chrono::Utc::now().format("%Y").to_string();

            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let date_str = find_value_in_row(row, &input.mappings, "date", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let supplier_name = find_value_in_row(row, &input.mappings, "supplier", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let amount_str = find_value_in_row(row, &input.mappings, "amount", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vat_str = find_value_in_row(row, &input.mappings, "vat", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let item_name = find_value_in_row(row, &input.mappings, "item", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let qty_str = find_value_in_row(row, &input.mappings, "quantity", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let unit_cost_str = find_value_in_row(row, &input.mappings, "unit_cost", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let supplier_inv_no = find_value_in_row(row, &input.mappings, "supplier_invoice_no", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let notes = find_value_in_row(row, &input.mappings, "notes", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let date = parse_date_flexible(&date_str).unwrap_or_else(|| "2000-01-01".to_string());

                let supplier_id = if !supplier_name.trim().is_empty() {
                    supplier_cache.get(&supplier_name.trim().to_lowercase()).copied().unwrap_or_else(|| {
                        let _ = conn.execute(
                            "INSERT INTO suppliers(code, name) VALUES(?1, ?2)",
                            rusqlite::params![auto_generate_code(&conn, "SUP").unwrap_or_default(), supplier_name.trim()],
                        );
                        let new_id = conn.last_insert_rowid();
                        supplier_cache.insert(supplier_name.trim().to_lowercase(), new_id);
                        new_id
                    })
                } else {
                    errors.push(ImportError { row: row_num, field: "supplier".into(), message: "اسم المورد مطلوب للشراء".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                };

                let net_milli = parse_amount_to_milli(&amount_str);
                let vat_milli = parse_amount_to_milli(&vat_str);
                let total_milli = net_milli + vat_milli;

                let pur_no = format!("PUR-{}-{:04}", year, next_sequence(&conn, "PUR", &year)?);

                conn.execute(
                    "INSERT INTO purchases(pur_no, date, supplier_id, supplier_invoice_no, vat_enabled, net_milli, vat_milli, total_milli, status, notes) VALUES(?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        pur_no, date, supplier_id,
                        if supplier_inv_no.is_empty() { None } else { Some(&supplier_inv_no) },
                        if vat_milli > 0 { 1i64 } else { 0i64 },
                        net_milli, vat_milli, total_milli, "posted",
                        if notes.is_empty() { None } else { Some(&notes) },
                    ],
                ).map_err(|e| format!("فشل إدراج أمر الشراء: {}", e))?;
                let purchase_id = conn.last_insert_rowid();

                if !item_name.trim().is_empty() {
                    let item_id = item_cache.get(&item_name.trim().to_lowercase()).copied().unwrap_or(0);
                    if item_id > 0 {
                        let qty: f64 = qty_str.trim().parse().unwrap_or(1.0);
                        let unit_cost = parse_amount_to_milli(&unit_cost_str);
                        let line_net = ((qty * unit_cost as f64).round()) as i64;
                        let vat_pct: f64 = vat_str.trim().parse().unwrap_or(15.0);
                        let line_vat = ((line_net as f64 * vat_pct / 100.0).round()) as i64;

                        conn.execute(
                            "INSERT INTO purchase_lines(purchase_id, item_id, qty, unit_cost_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,?,?,?,?)",
                            rusqlite::params![purchase_id, item_id, qty, unit_cost, line_net, vat_pct, line_vat],
                        ).map_err(|e| format!("فشل إدراج بند الشراء: {}", e))?;
                    }
                }

                imported += 1;
            }
        }
        "expenses" => {
            let year = chrono::Utc::now().format("%Y").to_string();

            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let date_str = find_value_in_row(row, &input.mappings, "date", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let category = find_value_in_row(row, &input.mappings, "category", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let amount_str = find_value_in_row(row, &input.mappings, "amount", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vendor = find_value_in_row(row, &input.mappings, "vendor", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let account_code = find_value_in_row(row, &input.mappings, "account_code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let method = find_value_in_row(row, &input.mappings, "method", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let reference = find_value_in_row(row, &input.mappings, "reference", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let vat_str = find_value_in_row(row, &input.mappings, "vat", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let notes = find_value_in_row(row, &input.mappings, "notes", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let date = parse_date_flexible(&date_str).unwrap_or_else(|| "2000-01-01".to_string());
                let amount_milli = parse_amount_to_milli(&amount_str);
                let vat_milli = parse_amount_to_milli(&vat_str);

                if amount_milli == 0 && !amount_str.trim().is_empty() {
                    errors.push(ImportError { row: row_num, field: "amount".into(), message: format!("قيمة المبلغ '{}' غير صحيحة", amount_str), severity: "error".into() });
                    skipped += 1;
                    continue;
                }

                let exp_no = format!("EXP-{}-{:04}", year, next_sequence(&conn, "EXP", &year)?);

                conn.execute(
                    "INSERT INTO expenses(exp_no, date, category, account_code, amount_milli, vat_milli, method, vendor, reference, notes, approval_status) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        exp_no, date,
                        if category.is_empty() { None } else { Some(&category) },
                        if account_code.is_empty() { None } else { Some(&account_code) },
                        amount_milli, vat_milli,
                        if method.is_empty() { None } else { Some(&method) },
                        if vendor.is_empty() { None } else { Some(&vendor) },
                        if reference.is_empty() { None } else { Some(&reference) },
                        if notes.is_empty() { None } else { Some(&notes) },
                        "approved",
                    ],
                ).map_err(|e| format!("فشل إدراج المصروف: {}", e))?;

                imported += 1;
            }
        }
        "opening_balances" => {
            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let entity = find_value_in_row(row, &input.mappings, "entity_type", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let name = find_value_in_row(row, &input.mappings, "name", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let code = find_value_in_row(row, &input.mappings, "code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let balance_str = find_value_in_row(row, &input.mappings, "balance", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let qty_str = find_value_in_row(row, &input.mappings, "qty", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let cost_str = find_value_in_row(row, &input.mappings, "cost", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let bal_milli = parse_amount_to_milli(&balance_str);
                let _lower_name = name.trim().to_lowercase();
                let entity_lower = entity.trim().to_lowercase();

                if entity_lower.contains("عميل") || entity_lower == "customer" || entity_lower == "customers" {
                    let found = if code.trim().is_empty() { {
                        conn.query_row("SELECT id FROM customers WHERE name=? AND active=1", [name.trim()], |r| r.get::<_, i64>(0))
                    } } else { {
                        conn.query_row("SELECT id FROM customers WHERE code=? AND active=1", [code.trim()], |r| r.get::<_, i64>(0))
                    } };
                    match found {
                        Ok(id) => {
                            conn.execute("UPDATE customers SET opening_balance_milli=?1, balance_milli=?1 WHERE id=?2", rusqlite::params![bal_milli, id])
                                .map_err(|e| format!("فشل تحديث رصيد العميل '{}': {}", name, e))?;
                            imported += 1;
                        }
                        Err(_) => {
                            errors.push(ImportError { row: row_num, field: "name".into(), message: format!("العميل '{}' غير موجود", name), severity: "error".into() });
                            skipped += 1;
                        }
                    }
                } else if entity_lower.contains("مورد") || entity_lower == "supplier" || entity_lower == "suppliers" {
                    let found = if code.trim().is_empty() { {
                        conn.query_row("SELECT id FROM suppliers WHERE name=? AND active=1", [name.trim()], |r| r.get::<_, i64>(0))
                    } } else { {
                        conn.query_row("SELECT id FROM suppliers WHERE code=? AND active=1", [code.trim()], |r| r.get::<_, i64>(0))
                    } };
                    match found {
                        Ok(id) => {
                            conn.execute("UPDATE suppliers SET opening_balance_milli=?1, balance_milli=?1 WHERE id=?2", rusqlite::params![bal_milli, id])
                                .map_err(|e| format!("فشل تحديث رصيد المورد '{}': {}", name, e))?;
                            imported += 1;
                        }
                        Err(_) => {
                            errors.push(ImportError { row: row_num, field: "name".into(), message: format!("المورد '{}' غير موجود", name), severity: "error".into() });
                            skipped += 1;
                        }
                    }
                } else if entity_lower.contains("صنف") || entity_lower.contains("مخزون") || entity_lower == "inventory" || entity_lower == "inventories" {
                    let qty_val: f64 = qty_str.trim().parse().unwrap_or(0.0);
                    let cost_val: i64 = (parse_amount_to_f64(&cost_str) * 1000.0).round() as i64;
                    let found = if code.trim().is_empty() { {
                        conn.query_row("SELECT id FROM inventory_items WHERE name_ar=? AND active=1", [name.trim()], |r| r.get::<_, i64>(0))
                    } } else { {
                        conn.query_row("SELECT id FROM inventory_items WHERE code=? AND active=1", [code.trim()], |r| r.get::<_, i64>(0))
                    } };
                    match found {
                        Ok(id) => {
                            if qty_val != 0.0 {
                                conn.execute("UPDATE inventory_items SET qty_on_hand=?1, avg_cost_milli=?2 WHERE id=?3", rusqlite::params![qty_val, cost_val, id])
                                    .map_err(|e| format!("فشل تحديث صنف '{}': {}", name, e))?;
                            }
                            imported += 1;
                        }
                        Err(_) => {
                            errors.push(ImportError { row: row_num, field: "name".into(), message: format!("الصنف '{}' غير موجود", name), severity: "error".into() });
                            skipped += 1;
                        }
                    }
                } else {
                    errors.push(ImportError { row: row_num, field: "entity_type".into(), message: format!("نوع الكيان '{}' غير معروف. استخدم: عميل/مورد/صنف", entity), severity: "error".into() });
                    skipped += 1;
                }
            }
        }
        "employees" => {
            let mut existing: std::collections::HashSet<String> =
                load_existing_names(&conn, "SELECT name FROM employees WHERE active=1")
                    .unwrap_or_default();
            for (i, row) in data.iter().enumerate() {
                let row_num = i + 2;
                let name = find_value_in_row(row, &input.mappings, "name", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                if name.trim().is_empty() {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: "اسم الموظف مطلوب".into(), severity: "error".into() });
                    skipped += 1;
                    continue;
                }
                let lower_name = name.trim().to_lowercase();
                if existing.contains(&lower_name) {
                    errors.push(ImportError { row: row_num, field: "name".into(), message: format!("الموظف '{}' موجود مسبقاً", name), severity: "warning".into() });
                    skipped += 1;
                    continue;
                }
                let code = find_value_in_row(row, &input.mappings, "code", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let salary_str = find_value_in_row(row, &input.mappings, "salary", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let job = find_value_in_row(row, &input.mappings, "job", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let nationality = find_value_in_row(row, &input.mappings, "nationality", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let phone = find_value_in_row(row, &input.mappings, "phone", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let passport_no = find_value_in_row(row, &input.mappings, "passport_no", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let passport_expiry = find_value_in_row(row, &input.mappings, "passport_expiry", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let residence_expiry = find_value_in_row(row, &input.mappings, "residence_expiry", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let joining_date = find_value_in_row(row, &input.mappings, "joining_date", &headers_from_mappings(&input.mappings)).unwrap_or_default();
                let notes = find_value_in_row(row, &input.mappings, "notes", &headers_from_mappings(&input.mappings)).unwrap_or_default();

                let salary_milli = parse_amount_to_milli(&salary_str);
                let join_date = parse_date_flexible(&joining_date);
                let emp_code = if code.trim().is_empty() {
                    auto_generate_code(&conn, "EMP")?
                } else {
                    code.trim().to_string()
                };

                conn.execute(
                    "INSERT INTO employees(code, name, nationality, job, salary_milli, phone, passport_no, passport_expiry, residence_expiry, joining_date, notes) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
                    rusqlite::params![
                        emp_code, name.trim(),
                        if nationality.is_empty() { None } else { Some(&nationality) },
                        if job.is_empty() { None } else { Some(&job) },
                        salary_milli,
                        if phone.is_empty() { None } else { Some(&phone) },
                        if passport_no.is_empty() { None } else { Some(&passport_no) },
                        if passport_expiry.is_empty() { None } else { Some(&passport_expiry) },
                        if residence_expiry.is_empty() { None } else { Some(&residence_expiry) },
                        join_date,
                        if notes.is_empty() { None } else { Some(&notes) },
                    ],
                ).map_err(|e| format!("فشل إدراج الموظف '{}': {}", name.trim(), e))?;

                existing.insert(lower_name);
                imported += 1;
            }
        }
        _ => {
            return Err(AppError::validation(format!("نوع الكيان '{}' غير مدعوم للتنفيذ", input.entity_type)));
        }
    }

    let file_name = Path::new(&input.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let _ = conn.execute(
        "INSERT INTO import_history(import_type, file_name, total_rows, imported, skipped, status, created_by) VALUES(?1,?2,?3,?4,?5,'completed',datetime('now','localtime'))",
        rusqlite::params![input.entity_type, file_name, total_rows as i64, imported as i64, skipped as i64],
    );

    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "execute_import", "import_history", None, None, Some(&input.entity_type), Some(&file_name));

    Ok(ImportResult {
        entity_type: input.entity_type,
        imported,
        skipped,
        errors,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportHistoryEntry {
    pub id: i64,
    pub entity_type: String,
    pub file_name: String,
    pub rows_imported: i64,
    pub rows_skipped: i64,
    pub created_at: String,
}

#[tauri::command]
pub fn import_get_history(state: State<'_, DbState>) -> Result<Vec<ImportHistoryEntry>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, import_type, file_name, imported, skipped, COALESCE(created_at, '') FROM import_history ORDER BY id DESC LIMIT 100"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ImportHistoryEntry {
            id: row.get(0)?,
            entity_type: row.get(1)?,
            file_name: row.get(2)?,
            rows_imported: row.get(3)?,
            rows_skipped: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn get_import_templates() -> Result<Vec<ImportTemplate>, AppError> {
    Ok(vec![
        ImportTemplate {
            entity_type: "customers".into(),
            display_name_ar: "العملاء".into(),
            description: "استيراد بيانات العملاء من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "name".into(), label_ar: "اسم العميل".into(), required: true, data_type: "text".into(), example: "شركة الأهرام للتجارة".into() },
                TemplateColumn { field: "code".into(), label_ar: "رمز العميل".into(), required: false, data_type: "text".into(), example: "CUST-2025-0001".into() },
                TemplateColumn { field: "balance".into(), label_ar: "الرصيد الافتتاحي".into(), required: false, data_type: "number".into(), example: "1500.500".into() },
                TemplateColumn { field: "phone".into(), label_ar: "الهاتف".into(), required: false, data_type: "text".into(), example: "+968 99123456".into() },
                TemplateColumn { field: "email".into(), label_ar: "البريد الإلكتروني".into(), required: false, data_type: "text".into(), example: "info@example.com".into() },
                TemplateColumn { field: "vat_number".into(), label_ar: "الرقم الضريبي".into(), required: false, data_type: "text".into(), example: "OM123456789".into() },
                TemplateColumn { field: "address".into(), label_ar: "العنوان".into(), required: false, data_type: "text".into(), example: "مسقط، سلطنة عمان".into() },
                TemplateColumn { field: "credit_limit".into(), label_ar: "حد الائتمان".into(), required: false, data_type: "number".into(), example: "50000.000".into() },
                TemplateColumn { field: "contact".into(), label_ar: "جهة الاتصال".into(), required: false, data_type: "text".into(), example: "محمد العلي".into() },
                TemplateColumn { field: "payment_terms".into(), label_ar: "شروط الدفع".into(), required: false, data_type: "text".into(), example: "30 يوم".into() },
            ],
        },
        ImportTemplate {
            entity_type: "suppliers".into(),
            display_name_ar: "الموردون".into(),
            description: "استيراد بيانات الموردين من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "name".into(), label_ar: "اسم المورد".into(), required: true, data_type: "text".into(), example: "مصنع العُمان للبلاستيك".into() },
                TemplateColumn { field: "code".into(), label_ar: "رمز المورد".into(), required: false, data_type: "text".into(), example: "SUP-2025-0001".into() },
                TemplateColumn { field: "balance".into(), label_ar: "الرصيد الافتتاحي".into(), required: false, data_type: "number".into(), example: "3200.000".into() },
                TemplateColumn { field: "phone".into(), label_ar: "الهاتف".into(), required: false, data_type: "text".into(), example: "+968 24123456".into() },
                TemplateColumn { field: "email".into(), label_ar: "البريد الإلكتروني".into(), required: false, data_type: "text".into(), example: "sales@omanplastic.com".into() },
                TemplateColumn { field: "vat_number".into(), label_ar: "الرقم الضريبي".into(), required: false, data_type: "text".into(), example: "OM987654321".into() },
                TemplateColumn { field: "address".into(), label_ar: "العنوان".into(), required: false, data_type: "text".into(), example: "صلالة، سلطنة عمان".into() },
                TemplateColumn { field: "currency".into(), label_ar: "العملة".into(), required: false, data_type: "text".into(), example: "OMR".into() },
                TemplateColumn { field: "contact".into(), label_ar: "جهة الاتصال".into(), required: false, data_type: "text".into(), example: "أحمد الراشدي".into() },
                TemplateColumn { field: "payment_terms".into(), label_ar: "شروط الدفع".into(), required: false, data_type: "text".into(), example: "60 يوم".into() },
            ],
        },
        ImportTemplate {
            entity_type: "products".into(),
            display_name_ar: "المنتجات".into(),
            description: "استيراد بيانات المنتجات من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "name".into(), label_ar: "اسم المنتج".into(), required: true, data_type: "text".into(), example: "كوب بلاستيك 8 أونصة".into() },
                TemplateColumn { field: "code".into(), label_ar: "رمز المنتج".into(), required: false, data_type: "text".into(), example: "PRD-2025-0001".into() },
                TemplateColumn { field: "price".into(), label_ar: "سعر البيع".into(), required: false, data_type: "number".into(), example: "2.500".into() },
                TemplateColumn { field: "cost".into(), label_ar: "سعر التكلفة".into(), required: false, data_type: "number".into(), example: "1.200".into() },
                TemplateColumn { field: "size".into(), label_ar: "المقاس".into(), required: false, data_type: "text".into(), example: "8 أونصة".into() },
                TemplateColumn { field: "barcode".into(), label_ar: "الباركود".into(), required: false, data_type: "text".into(), example: "6281234567890".into() },
                TemplateColumn { field: "cup_type".into(), label_ar: "نوع الكوب".into(), required: false, data_type: "text".into(), example: "بلاستيك شفاف".into() },
                TemplateColumn { field: "cups_per_carton".into(), label_ar: "عدد الأكواب في الكرتون".into(), required: false, data_type: "number".into(), example: "1000".into() },
                TemplateColumn { field: "carton_type".into(), label_ar: "نوع الكرتون".into(), required: false, data_type: "text".into(), example: "كرتون مقوّى".into() },
                TemplateColumn { field: "vat_pct".into(), label_ar: "نسبة الضريبة (%)".into(), required: false, data_type: "number".into(), example: "15".into() },
                TemplateColumn { field: "notes".into(), label_ar: "ملاحظات".into(), required: false, data_type: "text".into(), example: "منتج جديد".into() },
            ],
        },
        ImportTemplate {
            entity_type: "inventory".into(),
            display_name_ar: "المخزون".into(),
            description: "استيراد بيانات بنود المخزون من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "name".into(), label_ar: "اسم الصنف".into(), required: true, data_type: "text".into(), example: "خام بلاستيك PE".into() },
                TemplateColumn { field: "code".into(), label_ar: "رمز الصنف".into(), required: false, data_type: "text".into(), example: "INV-2025-0001".into() },
                TemplateColumn { field: "qty".into(), label_ar: "الكمية".into(), required: false, data_type: "number".into(), example: "5000".into() },
                TemplateColumn { field: "cost".into(), label_ar: "متوسط التكلفة".into(), required: false, data_type: "number".into(), example: "0.850".into() },
                TemplateColumn { field: "kind".into(), label_ar: "النوع".into(), required: false, data_type: "text".into(), example: "خام".into() },
                TemplateColumn { field: "uom".into(), label_ar: "وحدة القياس".into(), required: false, data_type: "text".into(), example: "كيلو".into() },
                TemplateColumn { field: "reorder_level".into(), label_ar: "حد إعادة الطلب".into(), required: false, data_type: "number".into(), example: "1000".into() },
                TemplateColumn { field: "notes".into(), label_ar: "ملاحظات".into(), required: false, data_type: "text".into(), example: "".into() },
            ],
        },
        ImportTemplate {
            entity_type: "invoices".into(),
            display_name_ar: "فواتير المبيعات".into(),
            description: "استيراد فواتير المبيعات من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "date".into(), label_ar: "التاريخ".into(), required: true, data_type: "date".into(), example: "2025-01-15".into() },
                TemplateColumn { field: "customer".into(), label_ar: "اسم العميل".into(), required: true, data_type: "text".into(), example: "شركة الأهرام للتجارة".into() },
                TemplateColumn { field: "amount".into(), label_ar: "المبلغ".into(), required: false, data_type: "number".into(), example: "1500.000".into() },
                TemplateColumn { field: "vat".into(), label_ar: "الضريبة".into(), required: false, data_type: "number".into(), example: "225.000".into() },
                TemplateColumn { field: "product".into(), label_ar: "المنتج".into(), required: false, data_type: "text".into(), example: "كوب بلاستيك 8 أونصة".into() },
                TemplateColumn { field: "quantity".into(), label_ar: "الكمية".into(), required: false, data_type: "number".into(), example: "10".into() },
                TemplateColumn { field: "unit_price".into(), label_ar: "سعر الوحدة".into(), required: false, data_type: "number".into(), example: "150.000".into() },
                TemplateColumn { field: "payment_type".into(), label_ar: "نوع الدفع".into(), required: false, data_type: "text".into(), example: "credit".into() },
                TemplateColumn { field: "notes".into(), label_ar: "ملاحظات".into(), required: false, data_type: "text".into(), example: "".into() },
            ],
        },
        ImportTemplate {
            entity_type: "purchases".into(),
            display_name_ar: "أوامر الشراء".into(),
            description: "استيراد أوامر الشراء من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "date".into(), label_ar: "التاريخ".into(), required: true, data_type: "date".into(), example: "2025-01-15".into() },
                TemplateColumn { field: "supplier".into(), label_ar: "اسم المورد".into(), required: true, data_type: "text".into(), example: "مصنع العُمان للبلاستيك".into() },
                TemplateColumn { field: "amount".into(), label_ar: "المبلغ".into(), required: false, data_type: "number".into(), example: "8000.000".into() },
                TemplateColumn { field: "vat".into(), label_ar: "الضريبة".into(), required: false, data_type: "number".into(), example: "1200.000".into() },
                TemplateColumn { field: "item".into(), label_ar: "الصنف".into(), required: false, data_type: "text".into(), example: "خام بلاستيك PE".into() },
                TemplateColumn { field: "quantity".into(), label_ar: "الكمية".into(), required: false, data_type: "number".into(), example: "10000".into() },
                TemplateColumn { field: "unit_cost".into(), label_ar: "تكلفة الوحدة".into(), required: false, data_type: "number".into(), example: "0.800".into() },
                TemplateColumn { field: "supplier_invoice_no".into(), label_ar: "رقم فاتورة المورد".into(), required: false, data_type: "text".into(), example: "INV-SUP-001".into() },
                TemplateColumn { field: "notes".into(), label_ar: "ملاحظات".into(), required: false, data_type: "text".into(), example: "".into() },
            ],
        },
        ImportTemplate {
            entity_type: "expenses".into(),
            display_name_ar: "المصروفات".into(),
            description: "استيراد المصروفات من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "date".into(), label_ar: "التاريخ".into(), required: true, data_type: "date".into(), example: "2025-01-15".into() },
                TemplateColumn { field: "category".into(), label_ar: "الفئة".into(), required: false, data_type: "text".into(), example: "إيجار".into() },
                TemplateColumn { field: "amount".into(), label_ar: "المبلغ".into(), required: true, data_type: "number".into(), example: "1200.000".into() },
                TemplateColumn { field: "vendor".into(), label_ar: "المورد/الجهة".into(), required: false, data_type: "text".into(), example: "شركة المباني العمانية".into() },
                TemplateColumn { field: "account_code".into(), label_ar: "رمز الحساب".into(), required: false, data_type: "text".into(), example: "6100".into() },
                TemplateColumn { field: "method".into(), label_ar: "طريقة الدفع".into(), required: false, data_type: "text".into(), example: "تحويل بنكي".into() },
                TemplateColumn { field: "reference".into(), label_ar: "المرجع".into(), required: false, data_type: "text".into(), example: "REF-2025-001".into() },
                TemplateColumn { field: "vat".into(), label_ar: "الضريبة".into(), required: false, data_type: "number".into(), example: "180.000".into() },
                TemplateColumn { field: "notes".into(), label_ar: "ملاحظات".into(), required: false, data_type: "text".into(), example: "إيجار شهر يناير".into() },
            ],
        },
        ImportTemplate {
            entity_type: "opening_balances".into(),
            display_name_ar: "الأرصدة الافتتاحية".into(),
            description: "استيراد الأرصدة الافتتاحية للعملاء والموردين والمخزون".into(),
            columns: vec![
                TemplateColumn { field: "entity_type".into(), label_ar: "النوع (عميل/مورد/صنف)".into(), required: true, data_type: "text".into(), example: "عميل".into() },
                TemplateColumn { field: "name".into(), label_ar: "الاسم".into(), required: true, data_type: "text".into(), example: "شركة الأهرام للتجارة".into() },
                TemplateColumn { field: "code".into(), label_ar: "الرمز".into(), required: false, data_type: "text".into(), example: "CUST-2025-0001".into() },
                TemplateColumn { field: "balance".into(), label_ar: "الرصيد الافتتاحي".into(), required: true, data_type: "number".into(), example: "5000.000".into() },
                TemplateColumn { field: "qty".into(), label_ar: "الكمية (للمخزون)".into(), required: false, data_type: "number".into(), example: "500".into() },
                TemplateColumn { field: "cost".into(), label_ar: "التكلفة (للمخزون)".into(), required: false, data_type: "number".into(), example: "0.850".into() },
            ],
        },
        ImportTemplate {
            entity_type: "employees".into(),
            display_name_ar: "الموظفون".into(),
            description: "استيراد بيانات الموظفين من ملف Excel أو CSV".into(),
            columns: vec![
                TemplateColumn { field: "name".into(), label_ar: "اسم الموظف".into(), required: true, data_type: "text".into(), example: "خالد بن سعيد الراشدي".into() },
                TemplateColumn { field: "code".into(), label_ar: "رمز الموظف".into(), required: false, data_type: "text".into(), example: "EMP-2025-0001".into() },
                TemplateColumn { field: "salary".into(), label_ar: "الراتب الأساسي".into(), required: false, data_type: "number".into(), example: "1200.000".into() },
                TemplateColumn { field: "job".into(), label_ar: "الوظيفة".into(), required: false, data_type: "text".into(), example: "فني إنتاج".into() },
                TemplateColumn { field: "nationality".into(), label_ar: "الجنسية".into(), required: false, data_type: "text".into(), example: "عماني".into() },
                TemplateColumn { field: "phone".into(), label_ar: "الهاتف".into(), required: false, data_type: "text".into(), example: "+968 99123456".into() },
                TemplateColumn { field: "passport_no".into(), label_ar: "رقم الجواز".into(), required: false, data_type: "text".into(), example: "A12345678".into() },
                TemplateColumn { field: "passport_expiry".into(), label_ar: "انتهاء صلاحية الجواز".into(), required: false, data_type: "date".into(), example: "2028-06-15".into() },
                TemplateColumn { field: "residence_expiry".into(), label_ar: "انتهاء الإقامة".into(), required: false, data_type: "date".into(), example: "2026-12-31".into() },
                TemplateColumn { field: "joining_date".into(), label_ar: "تاريخ الالتحاق".into(), required: false, data_type: "date".into(), example: "2020-03-01".into() },
                TemplateColumn { field: "notes".into(), label_ar: "ملاحظات".into(), required: false, data_type: "text".into(), example: "".into() },
            ],
        },
    ])
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn load_existing_names(
    conn: &rusqlite::Connection,
    query: &str,
) -> Result<std::collections::HashSet<String>, AppError> {
    let mut set = std::collections::HashSet::new();
    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for name in rows.flatten() {
        set.insert(name.to_lowercase());
    }
    Ok(set)
}

fn auto_generate_code(conn: &rusqlite::Connection, prefix: &str) -> Result<String, AppError> {
    let year = chrono::Utc::now().format("%Y").to_string();
    let seq = next_sequence(conn, prefix, &year)?;
    Ok(format!("{}-{}-{:04}", prefix, year, seq))
}
