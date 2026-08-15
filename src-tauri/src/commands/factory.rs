use crate::commands::rbac;
use crate::commands::invoices::{CompanyPrintInfo, InvoicePrintData, SalesInvoice};
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

// ============================================================
// Cup-factory vertical
// ============================================================
// Professional quotations (عروض الأسعار) issued to any client with any
// factory data the user wants, commercial non-tax invoices (فواتير تجارية)
// printed under the factory name only, and weekly/monthly expense summaries
// grouped by source (company accounts / employee custody / owners) and
// category. Every command is a thin wrapper over a `_inner` function that
// takes an existing connection, so the real logic is unit-testable.

// ------------------------- Quotations -------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Quotation {
    pub id: i64,
    pub quote_no: Option<String>,
    pub date: String,
    pub customer_id: Option<i64>,
    pub client_name: Option<String>,
    pub client_contact: Option<String>,
    pub client_phone: Option<String>,
    pub client_email: Option<String>,
    pub client_address: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub validity_days: i64,
    pub net_milli: i64,
    pub discount_milli: i64,
    pub total_milli: i64,
    pub currency: String,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotationLine {
    pub id: i64,
    pub quote_id: i64,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub item_name: Option<String>,
    pub cup_size: Option<String>,
    pub cups_per_carton: i64,
    pub cartons: f64,
    pub unit_price_milli: i64,
    pub line_total_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuotationInput {
    pub date: Option<String>,
    pub customer_id: Option<i64>,
    pub client_name: Option<String>,
    pub client_contact: Option<String>,
    pub client_phone: Option<String>,
    pub client_email: Option<String>,
    pub client_address: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub validity_days: Option<i64>,
    pub discount_milli: Option<i64>,
    pub currency: Option<String>,
    pub status: Option<String>,
    pub lines: Vec<CreateQuotationLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuotationLineInput {
    pub product_id: Option<i64>,
    pub item_name: Option<String>,
    pub cup_size: Option<String>,
    pub cups_per_carton: Option<i64>,
    pub cartons: f64,
    pub unit_price_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotationPrintData {
    pub quotation: Quotation,
    pub lines: Vec<QuotationLine>,
    pub company: CompanyPrintInfo,
    pub client_name: String,
}

const QUOTATION_SELECT: &str = "SELECT id, quote_no, date, customer_id, client_name, client_contact,
    client_phone, client_email, client_address, title, notes, terms, validity_days,
    net_milli, discount_milli, total_milli, currency, status, created_by, created_at
    FROM quotations";

fn quotation_from_row(row: &rusqlite::Row) -> rusqlite::Result<Quotation> {
    Ok(Quotation {
        id: row.get(0)?,
        quote_no: row.get(1)?,
        date: row.get(2)?,
        customer_id: row.get(3)?,
        client_name: row.get(4)?,
        client_contact: row.get(5)?,
        client_phone: row.get(6)?,
        client_email: row.get(7)?,
        client_address: row.get(8)?,
        title: row.get(9)?,
        notes: row.get(10)?,
        terms: row.get(11)?,
        validity_days: row.get(12)?,
        net_milli: row.get(13)?,
        discount_milli: row.get(14)?,
        total_milli: row.get(15)?,
        currency: row.get(16)?,
        status: row.get(17)?,
        created_by: row.get(18)?,
        created_at: row.get(19)?,
    })
}

fn quotation_line_from_row(row: &rusqlite::Row) -> rusqlite::Result<QuotationLine> {
    Ok(QuotationLine {
        id: row.get(0)?,
        quote_id: row.get(1)?,
        product_id: row.get(2)?,
        product_name: row.get(3)?,
        item_name: row.get(4)?,
        cup_size: row.get(5)?,
        cups_per_carton: row.get(6)?,
        cartons: row.get(7)?,
        unit_price_milli: row.get(8)?,
        line_total_milli: row.get(9)?,
        notes: row.get(10)?,
    })
}

fn get_quotation_lines(conn: &rusqlite::Connection, quote_id: i64) -> Result<Vec<QuotationLine>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT ql.id, ql.quote_id, ql.product_id, COALESCE(p.name_ar, p.name_en, ql.item_name, ''), ql.item_name,
                ql.cup_size, ql.cups_per_carton, ql.cartons, ql.unit_price_milli, ql.line_total_milli, ql.notes
         FROM quotation_lines ql
         LEFT JOIN products p ON p.id = ql.product_id
         WHERE ql.quote_id = ?1
         ORDER BY ql.id",
    )?;
    let rows = stmt.query_map(params![quote_id], quotation_line_from_row)?;
    let mut lines = Vec::new();
    for row in rows {
        lines.push(row?);
    }
    Ok(lines)
}

fn compute_quote_totals(lines: &[CreateQuotationLineInput]) -> i64 {
    lines.iter().map(|l| (l.cartons * l.unit_price_milli as f64).round() as i64).sum()
}

fn quote_line_defaults(conn: &rusqlite::Connection, line: &CreateQuotationLineInput) -> (i64, String) {
    match line.product_id {
        Some(pid) => conn
            .query_row(
                "SELECT COALESCE(cups_per_carton, 1000), COALESCE(name_ar, name_en, '') FROM products WHERE id = ?1",
                params![pid],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((1000, String::new())),
        None => (1000, String::new()),
    }
}

fn insert_quote_lines(conn: &rusqlite::Connection, quote_id: i64, lines: &[CreateQuotationLineInput]) -> Result<(), AppError> {
    for line in lines {
        let (default_cpc, default_name) = quote_line_defaults(conn, line);
        let cpc = line.cups_per_carton.unwrap_or(default_cpc);
        let item_name = line.item_name.clone().unwrap_or(default_name);
        let line_total = (line.cartons * line.unit_price_milli as f64).round() as i64;
        conn.execute(
            "INSERT INTO quotation_lines(quote_id, product_id, item_name, cup_size, cups_per_carton, cartons, unit_price_milli, line_total_milli, notes)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                quote_id, line.product_id, item_name, line.cup_size, cpc, line.cartons,
                line.unit_price_milli, line_total, line.notes,
            ],
        )?;
    }
    Ok(())
}

fn current_user_name(conn: &rusqlite::Connection, user_id: i64) -> String {
    conn.query_row(
        "SELECT COALESCE(full_name, username, '') FROM users WHERE id = ?1",
        params![user_id],
        |r| r.get(0),
    )
    .unwrap_or_default()
}

// ------------------------- Quotations -------------------------

#[tauri::command]
pub fn list_quotations(state: State<'_, DbState>, status: Option<String>) -> Result<Vec<Quotation>, AppError> {
    let conn = state.0.lock()?;
    list_quotations_inner(&conn, status.as_deref())
}

pub(crate) fn list_quotations_inner(conn: &rusqlite::Connection, status: Option<&str>) -> Result<Vec<Quotation>, AppError> {
    let sql = match status {
        Some(s) if !s.is_empty() => format!("{} WHERE status = ?1 ORDER BY id DESC", QUOTATION_SELECT),
        _ => format!("{} ORDER BY id DESC", QUOTATION_SELECT),
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = match status {
        Some(s) if !s.is_empty() => stmt.query_map(params![s], quotation_from_row)?,
        _ => stmt.query_map([], quotation_from_row)?,
    };
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_quotation(state: State<'_, DbState>, id: i64) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    get_quotation_inner(&conn, id)
}

pub(crate) fn get_quotation_inner(conn: &rusqlite::Connection, id: i64) -> Result<serde_json::Value, AppError> {
    let quotation = conn
        .query_row(
            &format!("{} WHERE id = ?1", QUOTATION_SELECT),
            params![id],
            quotation_from_row,
        )
        .map_err(|_| AppError::not_found("الكوتيشن غير موجود"))?;
    let lines = get_quotation_lines(conn, id)?;
    Ok(serde_json::json!({ "quotation": quotation, "lines": lines }))
}

#[tauri::command]
pub fn create_quotation(state: State<'_, DbState>, user_id: i64, input: CreateQuotationInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    create_quotation_inner(&conn, user_id, &input)
}

pub(crate) fn create_quotation_inner(conn: &rusqlite::Connection, user_id: i64, input: &CreateQuotationInput) -> Result<i64, AppError> {
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();
    let seq = next_sequence(conn, "QUOT", &year)?;
    let quote_no = format!("QUOT-{}-{:04}", year, seq);
    let date = input.date.clone().unwrap_or(now);
    let discount = input.discount_milli.unwrap_or(0).max(0);
    let net = compute_quote_totals(&input.lines);
    let total = (net - discount).max(0);
    let created_by = current_user_name(conn, user_id);

    conn.execute(
        "INSERT INTO quotations(quote_no, date, customer_id, client_name, client_contact, client_phone,
            client_email, client_address, title, notes, terms, validity_days, net_milli, discount_milli,
            total_milli, currency, status, created_by, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, datetime('now'), datetime('now'))",
        params![
            quote_no, date, input.customer_id, input.client_name, input.client_contact, input.client_phone,
            input.client_email, input.client_address, input.title, input.notes, input.terms,
            input.validity_days.unwrap_or(7), net, discount, total,
            input.currency.clone().unwrap_or_else(|| "OMR".to_string()),
            input.status.clone().unwrap_or_else(|| "Draft".to_string()),
            created_by,
        ],
    )?;
    let quote_id = conn.last_insert_rowid();
    insert_quote_lines(conn, quote_id, &input.lines)?;

    let _ = rbac::log_audit(conn, Some(user_id), None, "create_quotation", "quotations", Some(quote_id), None, Some(&quote_no), None);
    Ok(quote_id)
}

#[tauri::command]
pub fn update_quotation(state: State<'_, DbState>, user_id: i64, id: i64, input: CreateQuotationInput) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    update_quotation_inner(&mut conn, user_id, id, &input)
}

pub(crate) fn update_quotation_inner(conn: &mut rusqlite::Connection, user_id: i64, id: i64, input: &CreateQuotationInput) -> Result<i64, AppError> {
    let tx = conn.transaction()?;
    tx.query_row("SELECT id FROM quotations WHERE id = ?1", params![id], |r| r.get::<_, i64>(0))
        .map_err(|_| AppError::not_found("الكوتيشن غير موجود"))?;

    let discount = input.discount_milli.unwrap_or(0).max(0);
    let net = compute_quote_totals(&input.lines);
    let total = (net - discount).max(0);
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

    tx.execute(
        "UPDATE quotations SET date = ?1, customer_id = ?2, client_name = ?3, client_contact = ?4,
            client_phone = ?5, client_email = ?6, client_address = ?7, title = ?8, notes = ?9, terms = ?10,
            validity_days = ?11, net_milli = ?12, discount_milli = ?13, total_milli = ?14, currency = ?15,
            status = ?16, updated_at = datetime('now')
         WHERE id = ?17",
        params![
            input.date.clone().unwrap_or(now), input.customer_id, input.client_name, input.client_contact,
            input.client_phone, input.client_email, input.client_address, input.title, input.notes, input.terms,
            input.validity_days.unwrap_or(7), net, discount, total,
            input.currency.clone().unwrap_or_else(|| "OMR".to_string()),
            input.status.clone().unwrap_or_else(|| "Draft".to_string()), id,
        ],
    )?;

    tx.execute("DELETE FROM quotation_lines WHERE quote_id = ?1", params![id])?;
    insert_quote_lines(&tx, id, &input.lines)?;

    let _ = rbac::log_audit(&tx, Some(user_id), None, "update_quotation", "quotations", Some(id), None, None, None);
    tx.commit()?;
    Ok(id)
}

#[tauri::command]
pub fn delete_quotation(state: State<'_, DbState>, user_id: i64, id: i64) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    delete_quotation_inner(&conn, user_id, id)?;
    Ok("تم حذف الكوتيشن".to_string())
}

pub(crate) fn delete_quotation_inner(conn: &rusqlite::Connection, user_id: i64, id: i64) -> Result<(), AppError> {
    conn.execute("DELETE FROM quotations WHERE id = ?1", params![id])?;
    let _ = rbac::log_audit(conn, Some(user_id), None, "delete_quotation", "quotations", Some(id), None, None, None);
    Ok(())
}

#[tauri::command]
pub fn get_quotation_for_print(state: State<'_, DbState>, id: i64) -> Result<QuotationPrintData, AppError> {
    let conn = state.0.lock()?;
    get_quotation_for_print_inner(&conn, id)
}

pub(crate) fn get_quotation_for_print_inner(conn: &rusqlite::Connection, id: i64) -> Result<QuotationPrintData, AppError> {
    let quotation = conn
        .query_row(
            &format!("{} WHERE id = ?1", QUOTATION_SELECT),
            params![id],
            quotation_from_row,
        )
        .map_err(|_| AppError::not_found("الكوتيشن غير موجود"))?;
    let mut lines = get_quotation_lines(conn, id)?;
    // Resolve product names for display even when item_name is blank.
    for line in &mut lines {
        if line.item_name.as_deref().unwrap_or("").is_empty() {
            line.item_name = line.product_name.clone();
        }
    }
    let company = crate::commands::invoices::get_company_info(conn)?;
    let client_name = quotation
        .client_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            quotation.customer_id.and_then(|cid| {
                conn.query_row("SELECT name FROM customers WHERE id = ?1", params![cid], |r| r.get(0)).ok()
            })
        })
        .unwrap_or_else(|| "—".to_string());
    Ok(QuotationPrintData { quotation, lines, company, client_name })
}

// --------------------- Commercial invoices ---------------------

#[derive(Debug, Deserialize)]
pub struct CreateCommercialInvoiceInput {
    pub customer_id: i64,
    pub payment_type: Option<String>,
    pub date: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<CreateCommercialInvoiceLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommercialInvoiceLineInput {
    pub product_id: i64,
    pub cartons: f64,
    pub unit_price_milli: i64,
}

#[tauri::command]
pub fn create_commercial_invoice(state: State<'_, DbState>, user_id: i64, input: CreateCommercialInvoiceInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    create_commercial_invoice_inner(&conn, user_id, &input)
}

pub(crate) fn create_commercial_invoice_inner(conn: &rusqlite::Connection, user_id: i64, input: &CreateCommercialInvoiceInput) -> Result<i64, AppError> {
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();
    let seq = next_sequence(conn, "CINV", &year)?;
    let inv_no = format!("CINV-{}-{:04}", year, seq);
    let date = input.date.clone().unwrap_or(now);

    let mut net: i64 = 0;
    let mut product_infos: Vec<i64> = Vec::new();
    for line in &input.lines {
        let cpc: i64 = conn
            .query_row(
                "SELECT COALESCE(cups_per_carton, 1000) FROM products WHERE id = ?1",
                params![line.product_id],
                |r| r.get(0),
            )
            .map_err(|_| AppError::not_found("المنتج غير موجود"))?;
        product_infos.push(cpc);
        net += (line.cartons * line.unit_price_milli as f64).round() as i64;
    }

    conn.execute(
        "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, vat_enabled, net_milli, vat_milli, total_milli, status, notes, is_commercial)
         VALUES(?1, ?2, ?3, ?4, 0, ?5, 0, ?5, 'Draft', ?6, 1)",
        params![inv_no, date, input.customer_id, input.payment_type.clone().unwrap_or_else(|| "credit".into()), net, input.notes],
    )?;
    let inv_id = conn.last_insert_rowid();

    for (i, line) in input.lines.iter().enumerate() {
        let cpc = product_infos[i];
        let qty_cups = line.cartons * cpc as f64;
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, 0, 0)",
            params![inv_id, line.product_id, line.cartons, cpc, qty_cups, line.unit_price_milli, line.unit_price_milli],
        )?;
    }

    let _ = rbac::log_audit(conn, Some(user_id), None, "create_commercial_invoice", "sales_invoices", Some(inv_id), None, Some(&inv_no), None);
    Ok(inv_id)
}

#[tauri::command]
pub fn list_commercial_invoices(state: State<'_, DbState>) -> Result<Vec<SalesInvoice>, AppError> {
    let conn = state.0.lock()?;
    list_commercial_invoices_inner(&conn)
}

pub(crate) fn list_commercial_invoices_inner(conn: &rusqlite::Connection) -> Result<Vec<SalesInvoice>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at, si.is_commercial
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id
         WHERE COALESCE(si.is_commercial, 0) = 1
         ORDER BY si.id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SalesInvoice {
            id: row.get(0)?,
            inv_no: row.get(1)?,
            date: row.get(2)?,
            customer_id: row.get(3)?,
            customer_name: row.get(4)?,
            payment_type: row.get(5)?,
            vat_enabled: row.get(6)?,
            net_milli: row.get(7)?,
            vat_milli: row.get(8)?,
            discount_milli: row.get(9)?,
            total_milli: row.get(10)?,
            discount_reason: row.get(11)?,
            cogs_milli: row.get(12)?,
            paid_milli: row.get(13)?,
            status: row.get(14)?,
            notes: row.get(15)?,
            created_by: row.get(16)?,
            created_at: row.get(17)?,
            is_commercial: row.get(18)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_commercial_invoice_for_print(state: State<'_, DbState>, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let conn = state.0.lock()?;
    crate::commands::invoices::get_invoice_for_print_inner(&conn, invoice_id)
}

// ----------------------- Expense summary -----------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseSourceTotal {
    pub source: String,
    pub label: String,
    pub total_milli: i64,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseCategoryTotal {
    pub category: String,
    pub total_milli: i64,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseSummary {
    pub date_from: String,
    pub date_to: String,
    pub total_milli: i64,
    pub count: i64,
    pub by_source: Vec<ExpenseSourceTotal>,
    pub by_category: Vec<ExpenseCategoryTotal>,
    pub details: Vec<crate::commands::expenses::Expense>,
}

fn source_label(source: &str) -> &'static str {
    match source {
        "custody" => "من عهد الموظفين",
        "personal" => "من أصحاب المصنع / شخصي",
        _ => "من الحسابات الرئيسية",
    }
}

/// Total (amount + VAT) and per-source / per-category breakdown of expenses in
/// a date range, plus the detail rows. The user asked for "إجمالي وتفاصيل كل
/// أنواع المصاريف أسبوعياً وشهرياً سواء من عهد الموظفين أو من أصحاب المصنع أو
/// من الحسابات الرئيسية".
#[tauri::command]
pub fn get_expense_summary(state: State<'_, DbState>, date_from: String, date_to: String) -> Result<ExpenseSummary, AppError> {
    let conn = state.0.lock()?;
    get_expense_summary_inner(&conn, &date_from, &date_to)
}

pub(crate) fn get_expense_summary_inner(conn: &rusqlite::Connection, date_from: &str, date_to: &str) -> Result<ExpenseSummary, AppError> {
    let total_milli: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_milli + vat_milli), 0) FROM expenses WHERE date >= ?1 AND date <= ?2",
            params![date_from, date_to],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM expenses WHERE date >= ?1 AND date <= ?2",
            params![date_from, date_to],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut src_stmt = conn.prepare(
        "SELECT COALESCE(paid_from_source, 'company'), SUM(amount_milli + vat_milli), COUNT(*)
         FROM expenses WHERE date >= ?1 AND date <= ?2
         GROUP BY paid_from_source ORDER BY SUM(amount_milli + vat_milli) DESC",
    )?;
    let mut by_source = Vec::new();
    for row in src_stmt.query_map(params![date_from, date_to], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
    })? {
        let (source, sum, cnt) = row?;
        by_source.push(ExpenseSourceTotal {
            source: source.clone(),
            label: source_label(&source).to_string(),
            total_milli: sum,
            count: cnt,
        });
    }

    let mut cat_stmt = conn.prepare(
        "SELECT COALESCE(NULLIF(TRIM(category), ''), 'عام'), SUM(amount_milli + vat_milli), COUNT(*)
         FROM expenses WHERE date >= ?1 AND date <= ?2
         GROUP BY category ORDER BY SUM(amount_milli + vat_milli) DESC",
    )?;
    let mut by_category = Vec::new();
    for row in cat_stmt.query_map(params![date_from, date_to], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
    })? {
        let (category, sum, cnt) = row?;
        by_category.push(ExpenseCategoryTotal { category, total_milli: sum, count: cnt });
    }

    let details = crate::commands::expenses::expense_rows_in_range(conn, date_from, date_to)?;

    Ok(ExpenseSummary { date_from: date_from.to_string(), date_to: date_to.to_string(), total_milli, count, by_source, by_category, details })
}

// ---------------------------- Tests ----------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;

    fn setup() -> (std::path::PathBuf, rusqlite::Connection) {
        let db_path = std::env::temp_dir().join(format!("promax_factory_{}.db", uuid::Uuid::new_v4()));
        let conn = init_database(&db_path).expect("fresh db");
        conn.execute(
            "INSERT INTO company_settings(name, factory_name, address, phone, email, vat_number, cr_number, default_vat_pct) VALUES('شركة التجربة','مصنع بهلاء للأكواب','بهلاء، سلطنة عمان','24560000','x@y.com','OM123456789','CR-2026-0001',5.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, notes) VALUES('C1','مؤسسة التجارة','credit',NULL,'99001122',NULL,NULL,NULL,0,'net',30,NULL)",
            [],
        )
        .unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct, cup_size_ml) VALUES('CUP200','كوب 200 مل','Cup 200ml',1000,15000,5.0,200)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct) VALUES('CUPJ','كوب عصير','Juice Cup',2000,18000,5.0)", []).unwrap();
        let _ = cid;
        (db_path, conn)
    }

    fn cleanup(db_path: &std::path::Path) {
        let _ = std::fs::remove_file(db_path);
        let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
        let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    }

    fn quote_line(product_id: i64, cartons: f64, price: i64) -> CreateQuotationLineInput {
        CreateQuotationLineInput {
            product_id: Some(product_id),
            item_name: None,
            cup_size: Some("200 مل".to_string()),
            cups_per_carton: None,
            cartons,
            unit_price_milli: price,
            notes: None,
        }
    }

    #[test]
    fn quotation_flow_create_update_delete_print() {
        let (db_path, mut conn) = setup();
        // admin user id = 1
        let input = CreateQuotationInput {
            date: Some("2026-08-16".to_string()),
            customer_id: None,
            client_name: Some("هايبر السيب".to_string()),
            client_contact: Some("أحمد".to_string()),
            client_phone: Some("91234567".to_string()),
            client_email: None,
            client_address: Some("السيب".to_string()),
            title: Some("عرض سعر كؤوس".to_string()),
            notes: Some("أسعار شاملة التغليف".to_string()),
            terms: Some("الدفع عند الاستلام".to_string()),
            validity_days: Some(14),
            discount_milli: Some(1000),
            currency: None,
            status: None,
            lines: vec![quote_line(1, 500.0, 12000), quote_line(2, 300.0, 18000)],
        };

        let quote_id = create_quotation_inner(&conn, 1, &input).expect("create quote");
        // 500*12000 + 300*18000 = 6,000,000 + 5,400,000 = 11,400,000; minus 1000 discount.
        let quotes = list_quotations_inner(&conn, None).unwrap();
        assert_eq!(quotes.len(), 1);
        assert!(quotes[0].quote_no.as_deref().unwrap().starts_with("QUOT-"));
        assert_eq!(quotes[0].net_milli, 11_400_000);
        assert_eq!(quotes[0].discount_milli, 1000);
        assert_eq!(quotes[0].total_milli, 11_399_000);
        assert_eq!(quotes[0].client_name.as_deref(), Some("هايبر السيب"));

        let data = get_quotation_for_print_inner(&conn, quote_id).unwrap();
        assert_eq!(data.lines.len(), 2);
        assert_eq!(data.lines[0].product_name.as_deref(), Some("كوب 200 مل"));
        assert_eq!(data.lines[0].cups_per_carton, 1000);
        assert_eq!(data.client_name, "هايبر السيب");
        assert_eq!(data.company.factory_name.as_deref(), Some("مصنع بهلاء للأكواب"));

        // Update: switch product 1 to free-form text item + new price.
        let upd = CreateQuotationInput {
            date: Some("2026-08-16".to_string()),
            customer_id: None,
            client_name: Some("هايبر السيب".to_string()),
            client_contact: None,
            client_phone: None,
            client_email: None,
            client_address: None,
            title: None,
            notes: Some("تعديل".to_string()),
            terms: None,
            validity_days: None,
            discount_milli: Some(0),
            currency: None,
            status: Some("Sent".to_string()),
            lines: vec![CreateQuotationLineInput {
                product_id: None,
                item_name: Some("كوب مخصص باسم العميل".to_string()),
                cup_size: Some("300 مل".to_string()),
                cups_per_carton: Some(1500),
                cartons: 100.0,
                unit_price_milli: 20000,
                notes: None,
            }],
        };
        update_quotation_inner(&mut conn, 1, quote_id, &upd).unwrap();
        let data = get_quotation_for_print_inner(&conn, quote_id).unwrap();
        assert_eq!(data.lines.len(), 1);
        assert_eq!(data.lines[0].item_name.as_deref(), Some("كوب مخصص باسم العميل"));
        assert_eq!(data.lines[0].cups_per_carton, 1500);
        assert_eq!(data.lines[0].line_total_milli, 2_000_000);
        assert_eq!(data.quotation.status, "Sent");
        assert_eq!(data.quotation.total_milli, 2_000_000);

        delete_quotation_inner(&conn, 1, quote_id).unwrap();
        assert!(list_quotations_inner(&conn, None).unwrap().is_empty());
        cleanup(&db_path);
    }

    #[test]
    fn commercial_invoice_is_non_vat_and_printable() {
        let (db_path, conn) = setup();
        let input = CreateCommercialInvoiceInput {
            customer_id: 1,
            payment_type: Some("credit".to_string()),
            date: Some("2026-08-16".to_string()),
            notes: Some("فاتورة تجارية".to_string()),
            lines: vec![CreateCommercialInvoiceLineInput { product_id: 1, cartons: 250.0, unit_price_milli: 15000 }],
        };
        let inv_id = create_commercial_invoice_inner(&conn, 1, &input).unwrap();
        let list = list_commercial_invoices_inner(&conn).unwrap();
        assert_eq!(list.len(), 1);
        let inv = &list[0];
        assert!(inv.inv_no.as_deref().unwrap().starts_with("CINV-"), "CINV prefix");
        assert_eq!(inv.vat_enabled, 0);
        assert_eq!(inv.is_commercial, 1);
        assert_eq!(inv.vat_milli, 0);
        assert_eq!(inv.net_milli, 3_750_000);
        assert_eq!(inv.total_milli, 3_750_000);
        assert_eq!(inv.status, "Draft");

        let print_data = crate::commands::invoices::get_invoice_for_print_inner(&conn, inv_id).unwrap();
        assert_eq!(print_data.invoice.is_commercial, 1);
        assert_eq!(print_data.lines.len(), 1);
        assert_eq!(print_data.lines[0].vat_pct, 0.0);
        assert_eq!(print_data.lines[0].vat_milli, 0);
        cleanup(&db_path);
    }

    #[test]
    fn expense_summary_groups_by_source_and_category() {
        let (db_path, conn) = setup();
        for (date, category, amount, source) in [
            ("2026-08-10", "ورق خام", 500_000, "company"),
            ("2026-08-11", "صيانة", 120_000, "custody"),
            ("2026-08-12", "وقود", 80_000, "personal"),
            ("2026-08-12", "ورق خام", 200_000, "custody"),
        ] {
            conn.execute(
                "INSERT INTO expenses(exp_no, date, category, amount_milli, vat_milli, approval_status, paid_from_source)
                 VALUES(?1, ?2, ?3, ?4, 0, 'approved', ?5)",
                params![format!("EXP-2026-{:04}", date), date, category, amount, source],
            )
            .unwrap();
        }
        let summary = get_expense_summary_inner(&conn, "2026-08-01", "2026-08-31").unwrap();
        assert_eq!(summary.total_milli, 900_000);
        assert_eq!(summary.count, 4);
        assert_eq!(summary.details.len(), 4);

        let src = summary.by_source.iter().find(|s| s.source == "custody").unwrap();
        assert_eq!(src.total_milli, 320_000);
        assert_eq!(src.label, "من عهد الموظفين");
        let comp = summary.by_source.iter().find(|s| s.source == "company").unwrap();
        assert_eq!(comp.total_milli, 500_000);
        assert_eq!(comp.label, "من الحسابات الرئيسية");

        let cat = summary.by_category.iter().find(|c| c.category == "ورق خام").unwrap();
        assert_eq!(cat.total_milli, 700_000);

        // Range filter excludes the 08-10 row.
        let week = get_expense_summary_inner(&conn, "2026-08-11", "2026-08-17").unwrap();
        assert_eq!(week.total_milli, 400_000);
        assert_eq!(week.count, 3);
        cleanup(&db_path);
    }

    #[test]
    fn commercial_invoices_are_skipped_by_auto_enqueue() {
        let (db_path, conn) = setup();
        conn.execute(
            "INSERT INTO companies(code, name_ar, default_vat_pct) VALUES('MAIN', 'شركة التجربة', 5.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO einvoice_settings(company_id, active, submit_on_post) VALUES(1, 1, 1)",
            [],
        )
        .unwrap();
        let input = CreateCommercialInvoiceInput {
            customer_id: 1,
            payment_type: Some("credit".to_string()),
            date: Some("2026-08-16".to_string()),
            notes: None,
            lines: vec![CreateCommercialInvoiceLineInput { product_id: 1, cartons: 10.0, unit_price_milli: 15000 }],
        };
        let inv_id = create_commercial_invoice_inner(&conn, 1, &input).unwrap();
        crate::commands::einvoice::auto_enqueue_on_post(&conn, inv_id).unwrap();
        let queued: i64 = conn
            .query_row("SELECT COUNT(*) FROM einvoice_queue WHERE invoice_id = ?1", params![inv_id], |r| r.get(0))
            .unwrap();
        assert_eq!(queued, 0, "commercial invoices must never be enqueued for e-invoicing");
        cleanup(&db_path)
    }
}
