use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesInvoice {
    pub id: i64,
    pub inv_no: Option<String>,
    pub date: String,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub payment_type: Option<String>,
    pub vat_enabled: i64,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub discount_milli: i64,
    pub total_milli: i64,
    pub discount_reason: Option<String>,
    pub cogs_milli: i64,
    pub paid_milli: i64,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub is_commercial: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: i64,
    pub invoice_id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub cartons: f64,
    pub cups_per_carton: i64,
    pub qty_cups: f64,
    pub unit_price_milli: i64,
    pub customs_price_milli: i64,
    pub line_net_milli: i64,
    pub vat_pct: f64,
    pub vat_milli: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceInput {
    pub customer_id: i64,
    pub payment_type: Option<String>,
    pub date: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<CreateInvoiceLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceLineInput {
    pub product_id: i64,
    pub cartons: f64,
    pub unit_price_milli: i64,
    pub customs_price_milli: Option<i64>,
}

struct ProductInfo {
    cups_per_carton: i64,
    vat_pct: f64,
}

#[tauri::command]
pub fn list_invoices(state: State<'_, DbState>) -> Result<Vec<SalesInvoice>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at, si.is_commercial FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id ORDER BY si.id DESC"
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
    
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn get_invoice(state: State<'_, DbState>, id: i64) -> Result<SalesInvoice, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at, si.is_commercial FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id WHERE si.id=?",
        [id],
        |row| {
            Ok(SalesInvoice {
                id: row.get(0)?, inv_no: row.get(1)?, date: row.get(2)?, customer_id: row.get(3)?,
                customer_name: row.get(4)?, payment_type: row.get(5)?, vat_enabled: row.get(6)?,
                net_milli: row.get(7)?, vat_milli: row.get(8)?, discount_milli: row.get(9)?,
                total_milli: row.get(10)?, discount_reason: row.get(11)?, cogs_milli: row.get(12)?,
                paid_milli: row.get(13)?, status: row.get(14)?, notes: row.get(15)?,
                created_by: row.get(16)?, created_at: row.get(17)?,
                is_commercial: row.get(18)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn get_invoice_lines(state: State<'_, DbState>, invoice_id: i64) -> Result<Vec<InvoiceLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT sil.id, sil.invoice_id, sil.product_id, p.name_ar, sil.cartons, sil.cups_per_carton, sil.qty_cups, sil.unit_price_milli, COALESCE(sil.customs_price_milli, 0), sil.line_net_milli, sil.vat_pct, sil.vat_milli FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id=p.id WHERE sil.invoice_id=?"
    )?;
    
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceLine {
            id: row.get(0)?, invoice_id: row.get(1)?, product_id: row.get(2)?,
            product_name: row.get(3)?, cartons: row.get(4)?, cups_per_carton: row.get(5)?,
            qty_cups: row.get(6)?, unit_price_milli: row.get(7)?,
            customs_price_milli: row.get(8)?,
            line_net_milli: row.get(9)?, vat_pct: row.get(10)?, vat_milli: row.get(11)?,
        })
    })?;
    
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub(crate) fn validate_invoice_lines(lines: &[CreateInvoiceLineInput]) -> Result<(), AppError> {
    if lines.is_empty() {
        return Err(AppError::validation("أدخل بنداً واحداً على الأقل"));
    }
    for line in lines {
        if line.cartons <= 0.0 {
            return Err(AppError::validation("الكمية يجب أن تكون أكبر من صفر"));
        }
        if line.unit_price_milli < 0 {
            return Err(AppError::validation("السعر لا يمكن أن يكون سالباً"));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_invoice(state: State<'_, DbState>, user_id: i64, input: CreateInvoiceInput) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    validate_invoice_lines(&input.lines)?;
    let tx = conn.transaction()?;
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();
    
    let seq = next_sequence(&tx, "INV", &year)?;
    let inv_no = format!("INV-{}-{:04}", year, seq);
    let invoice_date = input.date.clone().unwrap_or_else(|| now.clone());
    
    let mut net: i64 = 0;
    let mut vat: i64 = 0;
    let mut product_infos: Vec<ProductInfo> = Vec::new();

    for line in &input.lines {
        let info: ProductInfo = tx.query_row(
            "SELECT cups_per_carton, vat_pct FROM products WHERE id=?",
            [line.product_id],
            |row| Ok(ProductInfo {
                cups_per_carton: row.get(0)?,
                vat_pct: row.get(1)?,
            }),
        ).map_err(|e| format!("Product {} not found: {}", line.product_id, e))?;
        product_infos.push(info);
    }

    for (i, line) in input.lines.iter().enumerate() {
        let info = &product_infos[i];
        let line_net = (line.cartons * line.unit_price_milli as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        net += line_net;
        vat += line_vat;
    }
    let total = net + vat;
    
    tx.execute(
        "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status, notes) VALUES(?,?,?,?,?,?,?,'Draft',?)",
        rusqlite::params![inv_no, invoice_date, input.customer_id, input.payment_type.unwrap_or_else(|| "credit".into()), net, vat, total, input.notes],
    )?;
    let inv_id = tx.last_insert_rowid();
    
    for (i, line) in input.lines.iter().enumerate() {
        let info = &product_infos[i];
        let qty_cups = line.cartons * info.cups_per_carton as f64;
        let line_net = (line.cartons * line.unit_price_milli as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        let customs = line.customs_price_milli.unwrap_or(line.unit_price_milli);
        tx.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,?,?,?,?,?,?,?)",
            rusqlite::params![inv_id, line.product_id, line.cartons, info.cups_per_carton, qty_cups, line.unit_price_milli, customs, line_net, info.vat_pct, line_vat],
        )?;
    }

    let _ = rbac::log_audit(&tx, None, None, "create_invoice", "sales_invoices", Some(inv_id), None, Some(&inv_no), None);

    tx.commit()?;
    
    Ok(inv_id)
}

#[tauri::command]
pub fn post_invoice(state: State<'_, DbState>, user_id: i64, id: i64) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    post_invoice_inner(&mut conn, user_id, id)
}

pub(crate) fn post_invoice_inner(conn: &mut rusqlite::Connection, user_id: i64, id: i64) -> Result<String, AppError> {
    let tx = conn.transaction()?;

    let current_status: String = tx
        .query_row("SELECT status FROM sales_invoices WHERE id=?", [id], |r| r.get(0))
        .map_err(|_| AppError::not_found("الفاتورة غير موجودة"))?;
    if current_status != "Draft" {
        return Err(AppError::validation("يمكن ترحيل الفواتير المسودة فقط"));
    }

    let (payment_type, inv_date, inv_no, net_milli, vat_milli, total_milli, customer_id): (String, String, String, i64, i64, i64, i64) = tx
        .query_row(
            "SELECT COALESCE(payment_type,'credit'), date, inv_no, net_milli, vat_milli, total_milli, customer_id FROM sales_invoices WHERE id=?",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Enforce the customer's credit limit for credit sales: posting would raise
    // the outstanding receivable, so reject when it would exceed the limit.
    // A limit of 0 (or negative) means unlimited.
    if payment_type == "credit" {
        let (credit_limit, balance): (i64, i64) = tx
            .query_row(
                "SELECT COALESCE(credit_limit_milli, 0), COALESCE(balance_milli, 0) FROM customers WHERE id=?",
                [customer_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| AppError::not_found("العميل غير موجود"))?;
        if credit_limit > 0 && balance + total_milli > credit_limit {
            return Err(AppError::validation(format!(
                "تجاوز الحد الائتماني للعميل (الحد: {}، الرصيد الحالي + الفاتورة: {})",
                credit_limit,
                balance + total_milli
            )));
        }
    }

    let total_cogs = deduct_invoice_stock(&tx, id)?;

    let mut lines: Vec<(String, i64, i64, Option<String>)> = Vec::new();
    let cash_account = match payment_type.as_str() {
        "cash" => "1100",
        "cheque" => "1101",
        _ => "1200",
    };
    lines.push((cash_account.to_string(), total_milli, 0, Some("فاتورة مبيعات".to_string())));
    lines.push(("4100".to_string(), 0, net_milli, None));
    if vat_milli > 0 {
        lines.push(("2100".to_string(), 0, vat_milli, None));
    }
    if total_cogs > 0 {
        lines.push(("5100".to_string(), total_cogs, 0, Some("تكلفة البضاعة المباعة".to_string())));
        lines.push(("1400".to_string(), 0, total_cogs, None));
    }
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx,
        "invoice",
        id,
        &inv_date,
        &format!("فاتورة مبيعات {}", inv_no),
        &lines,
        "system",
    )?;

    tx.execute(
        "UPDATE sales_invoices SET status='Posted', cogs_milli=?1, journal_id=?2 WHERE id=?3",
        rusqlite::params![total_cogs, journal_id, id],
    )
    ?;

    // Credit sales create an AR receivable: keep the denormalized customer balance in sync.
    if payment_type == "credit" {
        tx.execute(
            "UPDATE customers SET balance_milli = COALESCE(balance_milli,0) + ?1 WHERE id=?2",
            rusqlite::params![total_milli, customer_id],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "post_invoice", "sales_invoices", Some(id), None, Some(&format!("COGS: {} mil, status: Posted, journal: {}", total_cogs, journal_id)), None);

    tx.commit()?;
    let _ = crate::commands::einvoice::auto_enqueue_on_post(conn, id);
    Ok("تم ترحيل الفاتورة بنجاح".to_string())
}

/// Deducts sold quantities from stock at posting time and computes COGS in milli.
/// Mirrored by `restore_invoice_stock` when a posted invoice is voided.
pub(crate) fn deduct_invoice_stock(conn: &rusqlite::Connection, invoice_id: i64) -> Result<i64, AppError> {
    let mut total_cogs: i64 = 0;
    let lines: Vec<(i64, f64)> = {
        let mut stmt = conn
            .prepare("SELECT product_id, cartons FROM sales_invoice_lines WHERE invoice_id=?")?;
        let rows = stmt.query_map([invoice_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };
    for (product_id, cartons) in &lines {
        let inv_items: Vec<(i64, i64)> = {
            let mut stmt = conn
                .prepare("SELECT id, CAST(avg_cost_milli AS INTEGER) FROM inventory_items WHERE product_id=? AND active=1 ORDER BY kind DESC LIMIT 1")?;
            let rows = stmt.query_map([product_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r?);
            }
            v
        };
        for (inv_id, avg_cost) in &inv_items {
            let qty_to_deduct = *cartons;
            let cost_milli = (qty_to_deduct * *avg_cost as f64).round() as i64;
            total_cogs += cost_milli;

            conn.execute(
                "UPDATE inventory_items SET qty_on_hand = qty_on_hand - ?1 WHERE id = ?2",
                rusqlite::params![qty_to_deduct, inv_id],
            )?;

            conn.execute(
                "INSERT INTO inventory_movements(ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, notes)
                 VALUES(datetime('now'), ?1, 'sale', 0, ?2, ?3, 'invoice', ?4, 'فاتورة مبيعات')",
                rusqlite::params![inv_id, qty_to_deduct, avg_cost, invoice_id],
            )?;
        }
    }
    Ok(total_cogs)
}

/// Returns sold quantities back to stock for a voided (previously posted) invoice.
/// Mirrors the deduction performed by `deduct_invoice_stock` so on-hand quantities stay in sync.
pub(crate) fn restore_invoice_stock(conn: &rusqlite::Connection, invoice_id: i64) -> Result<(), AppError> {
    let line_rows: Vec<(i64, f64)> = {
        let mut stmt = conn.prepare(
            "SELECT product_id, cartons FROM sales_invoice_lines WHERE invoice_id=?",
        )?;
        let rows = stmt.query_map([invoice_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };
    for (product_id, cartons) in line_rows {
        let inv_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM inventory_items WHERE product_id=? AND active=1 ORDER BY kind DESC LIMIT 1",
                [product_id],
                |row| row.get(0),
            )
            .ok();
        if let Some(inv_id) = inv_id {
            conn.execute(
                "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?1 WHERE id = ?2",
                rusqlite::params![cartons, inv_id],
            )?;
            let _ = conn.execute(
                "INSERT INTO inventory_movements(ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, notes)
                 VALUES(datetime('now'), ?1, 'sale_reversal', ?2, 0, 0, 'invoice', ?3, 'إلغاء فاتورة مبيعات')",
                rusqlite::params![inv_id, cartons, invoice_id],
            );
        }
    }
    Ok(())
}

#[tauri::command]
pub fn void_invoice(state: State<'_, DbState>, user_id: i64, id: i64, reason: Option<String>) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    void_invoice_inner(&mut conn, user_id, id, reason)
}

pub(crate) fn void_invoice_inner(conn: &mut rusqlite::Connection, user_id: i64, id: i64, reason: Option<String>) -> Result<String, AppError> {
    let tx = conn.transaction()?;

    let (status, inv_date, payment_type, total_milli, customer_id, paid_milli): (String, String, String, i64, i64, i64) = tx
        .query_row(
            "SELECT status, COALESCE(date, date('now')), COALESCE(payment_type,'credit'), total_milli, customer_id, COALESCE(paid_milli,0) FROM sales_invoices WHERE id=?",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .map_err(|e| AppError::not_found(format!("Invoice not found: {}", e)))?;

    if status != "Draft" && status != "Posted" {
        return Err(AppError::validation("يمكن إلغاء الفواتير المسودة أو المرحلة فقط"));
    }

    if status == "Posted" {
        let existing_journal: Option<i64> = tx.query_row(
            "SELECT journal_id FROM sales_invoices WHERE id=?",
            [id],
            |r| r.get(0),
        ).unwrap_or(None);
        if let Some(jid) = existing_journal {
            let already_reversed: Option<i64> = tx.query_row(
                "SELECT reversed_by FROM journal_entries WHERE id=?",
                [jid],
                |r| r.get(0),
            ).unwrap_or(None);
            if already_reversed.is_none() {
                let mut lines: Vec<(String, i64, i64, Option<String>)> = Vec::new();
                let mut stmt = tx.prepare(
                    "SELECT account_code, debit_milli, credit_milli FROM journal_entry_lines WHERE entry_id=?",
                )?;
                let rows = stmt.query_map([jid], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
                })?;
                for r in rows {
                    let (code, d, c) = r?;
                    lines.push((code, c, d, None));
                }
                let rev_id = crate::commands::accounting::post_to_journal(
                    &tx,
                    "invoice_reversal",
                    id,
                    &inv_date,
                    "إلغاء فاتورة مبيعات",
                    &lines,
                    "system",
                )?;
                tx.execute(
                    "UPDATE journal_entries SET reversed_by=? WHERE id=?",
                    rusqlite::params![rev_id, jid],
                )?;
            }
        }

        // Restore inventory deducted at posting time (mirror of post_invoice).
        restore_invoice_stock(&tx, id)?;

        // Reverse the AR receivable created at posting time.
        if payment_type == "credit" {
            tx.execute(
                "UPDATE customers SET balance_milli = COALESCE(balance_milli,0) - ?1 WHERE id=?2",
                rusqlite::params![total_milli, customer_id],
            )?;
        }

        // Payments that had been allocated to this invoice now cover the customer's
        // remaining open invoices (FIFO); any surplus stays as an on-account credit.
        if paid_milli > 0 {
            crate::commands::customers::allocate_customer_payment_fifo(&tx, customer_id, paid_milli, Some(id))?;
            tx.execute(
                "UPDATE sales_invoices SET paid_milli = 0 WHERE id=?",
                [id],
            )?;
        }
    }

    let notes_addon = reason.unwrap_or_default();
    if notes_addon.is_empty() {
        tx.execute("UPDATE sales_invoices SET status='Void' WHERE id=?", [id])?;
    } else {
        tx.execute(
            "UPDATE sales_invoices SET status='Void', notes=COALESCE(notes,'') || '\n[إلغاء] ' || ? WHERE id=?",
            rusqlite::params![notes_addon, id],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "void_invoice", "sales_invoices", Some(id), None, Some("Void"), Some(&notes_addon));
    tx.commit()?;
    Ok("تم إلغاء الفاتورة".to_string())
}

#[tauri::command]
pub fn duplicate_invoice(state: State<'_, DbState>, user_id: i64, id: i64) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    duplicate_invoice_inner(&mut conn, user_id, id)
}

pub(crate) fn duplicate_invoice_inner(conn: &mut rusqlite::Connection, user_id: i64, id: i64) -> Result<i64, AppError> {
    let tx = conn.transaction()?;

    let inv: SalesInvoice = tx.query_row(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at, si.is_commercial FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id WHERE si.id=?",
        [id],
        |row| {
            Ok(SalesInvoice {
                id: row.get(0)?, inv_no: row.get(1)?, date: row.get(2)?, customer_id: row.get(3)?,
                customer_name: row.get(4)?, payment_type: row.get(5)?, vat_enabled: row.get(6)?,
                net_milli: row.get(7)?, vat_milli: row.get(8)?, discount_milli: row.get(9)?,
                total_milli: row.get(10)?, discount_reason: row.get(11)?, cogs_milli: row.get(12)?,
                paid_milli: row.get(13)?, status: row.get(14)?, notes: row.get(15)?,
                created_by: row.get(16)?, created_at: row.get(17)?,
                is_commercial: row.get(18)?,
            })
        },
    ).map_err(|e| format!("Invoice not found: {}", e))?;

    let source_lines: Vec<(i64, f64, i64, i64)> = {
        let mut stmt = tx.prepare(
            "SELECT product_id, cartons, unit_price_milli, COALESCE(customs_price_milli, unit_price_milli) FROM sales_invoice_lines WHERE invoice_id=?"
        )?;
        let rows = stmt.query_map([id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq = next_sequence(&tx, "INV", &year)?;
    let inv_no = format!("INV-{}-{:04}", year, seq);

    let mut net: i64 = 0;
    let mut vat: i64 = 0;
    let mut line_data: Vec<(i64, f64, i64, ProductInfo, i64)> = Vec::new();

    for (product_id, cartons, unit_price, customs) in &source_lines {
        let info: ProductInfo = tx.query_row(
            "SELECT cups_per_carton, vat_pct FROM products WHERE id=?",
            [*product_id],
            |row| Ok(ProductInfo {
                cups_per_carton: row.get(0)?,
                vat_pct: row.get(1)?,
            }),
        ).map_err(|e| format!("Product {} not found: {}", product_id, e))?;
        let line_net = (*cartons * *unit_price as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        net += line_net;
        vat += line_vat;
        line_data.push((*product_id, *cartons, *unit_price, info, *customs));
    }
    let total = net + vat;

    let note = format!("نسخة من {}", inv.inv_no.unwrap_or_default());

    tx.execute(
        "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, vat_enabled, net_milli, vat_milli, total_milli, status, notes, is_commercial) VALUES(?,?,?,?,?,?,?,?,'Draft',?,?)",
        rusqlite::params![inv_no, now, inv.customer_id, inv.payment_type.unwrap_or_else(|| "credit".into()), inv.vat_enabled, net, vat, total, note, inv.is_commercial],
    )?;
    let new_id = tx.last_insert_rowid();

    for (product_id, cartons, unit_price, info, customs) in &line_data {
        let qty_cups = *cartons * info.cups_per_carton as f64;
        let line_net = (*cartons * *unit_price as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        tx.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,?,?,?,?,?,?,?)",
            rusqlite::params![new_id, product_id, cartons, info.cups_per_carton, qty_cups, unit_price, customs, line_net, info.vat_pct, line_vat],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "duplicate_invoice", "sales_invoices", Some(new_id), None, Some(&format!("source_id={}", id)), None);
    tx.commit()?;
    Ok(new_id)
}

#[tauri::command]
pub fn update_invoice(state: State<'_, DbState>, user_id: i64, id: i64, notes: Option<String>) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if let Some(n) = notes {
        conn.execute("UPDATE sales_invoices SET notes=? WHERE id=?", rusqlite::params![n, id])?;
    }
    let _ = rbac::log_audit(&conn, Some(user_id), None, "update_invoice", "sales_invoices", Some(id), None, Some("notes"), None);
    Ok("تم التحديث".to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateCreditNoteInput {
    pub invoice_id: i64,
    pub date: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<CreateCreditNoteLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCreditNoteLineInput {
    pub product_id: i64,
    pub cartons: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNoteSummary {
    pub id: i64,
    pub cn_no: Option<String>,
    pub date: String,
    pub invoice_id: i64,
    pub invoice_no: Option<String>,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub total_milli: i64,
    pub reason: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
}

/// Creates a sales credit note against a posted invoice: returns goods to stock,
/// reverses the invoice journal (revenue/VAT/AR, and inventory/COGS), and reduces
/// the customer's AR balance for credit sales. Price/VAT always come from the
/// original invoice lines (never trusted from the client).
#[tauri::command]
pub fn create_credit_note(state: State<'_, DbState>, user_id: i64, input: CreateCreditNoteInput) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    create_credit_note_inner(&mut conn, user_id, input)
}

pub(crate) fn create_credit_note_inner(conn: &mut rusqlite::Connection, user_id: i64, input: CreateCreditNoteInput) -> Result<i64, AppError> {
    let tx = conn.transaction()?;

    if input.lines.is_empty() {
        return Err(AppError::validation("أدخل بنداً واحداً على الأقل"));
    }

    let (status, inv_no, payment_type, customer_id): (String, Option<String>, String, i64) = tx
        .query_row(
            "SELECT status, inv_no, COALESCE(payment_type, 'credit'), customer_id FROM sales_invoices WHERE id=?",
            [input.invoice_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e| AppError::not_found(format!("الفاتورة غير موجودة: {}", e)))?;
    if status != "Posted" {
        return Err(AppError::validation("يمكن عمل إشعار خصم على الفواتير المرحلة فقط"));
    }

    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();
    let cn_date = input.date.clone().unwrap_or_else(|| now.clone());

    let mut net: i64 = 0;
    let mut vat: i64 = 0;
    let mut cogs: i64 = 0;
    // (product_id, cartons, unit_price_milli, line_net_milli, vat_pct, line_vat_milli)
    let mut processed: Vec<(i64, f64, i64, i64, f64, i64)> = Vec::new();

    for line in &input.lines {
        if line.cartons <= 0.0 {
            return Err(AppError::validation("الكمية يجب أن تكون أكبر من صفر"));
        }
        let orig: Vec<(f64, i64, f64)> = {
            let mut stmt = tx.prepare(
                "SELECT cartons, unit_price_milli, vat_pct FROM sales_invoice_lines WHERE invoice_id=? AND product_id=?",
            )?;
            let rows = stmt.query_map(
                rusqlite::params![input.invoice_id, line.product_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r?);
            }
            v
        };
        if orig.is_empty() {
            return Err(AppError::validation(format!("المنتج {} غير موجود في الفاتورة الأصلية", line.product_id)));
        }
        let orig_cartons: f64 = orig.iter().map(|o| o.0).sum();
        let unit_price: i64 = orig.iter().map(|o| o.1).max().unwrap_or(0);
        let vat_pct: f64 = orig.iter().map(|o| o.2).fold(0.0f64, f64::max);

        let credited: f64 = tx
            .query_row(
                "SELECT COALESCE(SUM(cnl.cartons), 0) FROM credit_note_lines cnl JOIN credit_notes cn ON cnl.cn_id=cn.id WHERE cn.invoice_id=? AND cnl.product_id=? AND cn.status != 'Void'",
                rusqlite::params![input.invoice_id, line.product_id],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        if credited + line.cartons > orig_cartons + 1e-9 {
            return Err(AppError::validation(format!(
                "الكمية المردودة تتجاوز كمية الفاتورة (المتبقي: {:.2} كرتونة)",
                orig_cartons - credited
            )));
        }

        let line_net = (line.cartons * unit_price as f64).round() as i64;
        let line_vat = (line_net as f64 * vat_pct / 100.0).round() as i64;
        let line_cogs = credit_note_line_cogs(&tx, line.product_id, line.cartons)?;
        net += line_net;
        vat += line_vat;
        cogs += line_cogs;
        processed.push((line.product_id, line.cartons, unit_price, line_net, vat_pct, line_vat));
    }

    let total = net + vat;

    let seq = next_sequence(&tx, "CN", &year)?;
    let cn_no = format!("CN-{}-{:04}", year, seq);

    tx.execute(
        "INSERT INTO credit_notes(cn_no, date, customer_id, invoice_id, net_milli, vat_milli, total_milli, cogs_milli, reason, status, notes, created_by) VALUES(?,?,?,?,?,?,?,?,?,'Posted',?,?)",
        rusqlite::params![cn_no, cn_date, customer_id, input.invoice_id, net, vat, total, cogs, input.reason, input.notes, user_id],
    )?;
    let cn_id = tx.last_insert_rowid();

    for (product_id, cartons, unit_price, line_net, vat_pct, line_vat) in &processed {
        tx.execute(
            "INSERT INTO credit_note_lines(cn_id, product_id, cartons, qty_cups, unit_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,0,?,?,?,?)",
            rusqlite::params![cn_id, *product_id, *cartons, *unit_price, *line_net, *vat_pct, *line_vat],
        )?;
        let inv_item: Option<(i64, i64)> = tx
            .query_row(
                "SELECT id, CAST(avg_cost_milli AS INTEGER) FROM inventory_items WHERE product_id=? AND active=1 ORDER BY kind DESC LIMIT 1",
                [*product_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        if let Some((inv_id, avg_cost)) = inv_item {
            tx.execute(
                "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?1 WHERE id=?2",
                rusqlite::params![*cartons, inv_id],
            )?;
            tx.execute(
                "INSERT INTO inventory_movements(ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, notes) VALUES(datetime('now'),?1,'credit_note',?2,0,?3,'credit_note',?4,'إشعار خصم')",
                rusqlite::params![inv_id, *cartons, avg_cost, cn_id],
            )?;
        }
    }

    // Reverse the posted invoice journal exactly (mirror of post_invoice),
    // crediting the same account the original invoice debited.
    let mut lines: Vec<(String, i64, i64, Option<String>)> = Vec::new();
    lines.push(("4100".to_string(), net, 0, Some("إشعار خصم مبيعات".to_string())));
    if vat > 0 {
        lines.push(("2100".to_string(), vat, 0, None));
    }
    let reversal_account = match payment_type.as_str() {
        "cash" => "1100",
        "cheque" => "1101",
        _ => "1200",
    };
    lines.push((reversal_account.to_string(), 0, total, None));
    if cogs > 0 {
        lines.push(("1400".to_string(), cogs, 0, None));
        lines.push(("5100".to_string(), 0, cogs, None));
    }
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx,
        "credit_note",
        cn_id,
        &cn_date,
        &format!("إشعار خصم {} على فاتورة {}", cn_no, inv_no.unwrap_or_default()),
        &lines,
        "system",
    )?;
    tx.execute("UPDATE credit_notes SET journal_id=? WHERE id=?", rusqlite::params![journal_id, cn_id])?;

    if payment_type == "credit" {
        tx.execute(
            "UPDATE customers SET balance_milli = COALESCE(balance_milli,0) - ?1 WHERE id=?2",
            rusqlite::params![total, customer_id],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "create_credit_note", "credit_notes", Some(cn_id), None, Some(&format!("total: {} mil, journal: {}", total, journal_id)), None);

    tx.commit()?;
    Ok(cn_id)
}

fn credit_note_line_cogs(conn: &rusqlite::Connection, product_id: i64, cartons: f64) -> Result<i64, AppError> {
    let inv_item: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, CAST(avg_cost_milli AS INTEGER) FROM inventory_items WHERE product_id=? AND active=1 ORDER BY kind DESC LIMIT 1",
            [product_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();
    Ok(match inv_item {
        Some((_, avg_cost)) => (cartons * avg_cost as f64).round() as i64,
        None => 0,
    })
}

#[tauri::command]
pub fn list_credit_notes(state: State<'_, DbState>) -> Result<Vec<CreditNoteSummary>, AppError> {    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT cn.id, cn.cn_no, cn.date, cn.invoice_id, si.inv_no, cn.customer_id, c.name, cn.net_milli, cn.vat_milli, cn.total_milli, cn.reason, cn.status, cn.created_at FROM credit_notes cn LEFT JOIN sales_invoices si ON cn.invoice_id=si.id LEFT JOIN customers c ON cn.customer_id=c.id ORDER BY cn.id DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CreditNoteSummary {
            id: row.get(0)?,
            cn_no: row.get(1)?,
            date: row.get(2)?,
            invoice_id: row.get(3)?,
            invoice_no: row.get(4)?,
            customer_id: row.get(5)?,
            customer_name: row.get(6)?,
            net_milli: row.get(7)?,
            vat_milli: row.get(8)?,
            total_milli: row.get(9)?,
            reason: row.get(10)?,
            status: row.get(11)?,
            created_at: row.get(12)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceCreditRemaining {
    pub product_id: i64,
    pub product_name: Option<String>,
    pub original_cartons: f64,
    pub credited_cartons: f64,
}

#[tauri::command]
pub fn get_invoice_credit_remaining(state: State<'_, DbState>, invoice_id: i64) -> Result<Vec<InvoiceCreditRemaining>, AppError> {
    let conn = state.0.lock()?;
    get_invoice_credit_remaining_inner(&conn, invoice_id)
}

fn get_invoice_credit_remaining_inner(conn: &rusqlite::Connection, invoice_id: i64) -> Result<Vec<InvoiceCreditRemaining>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT sil.product_id, p.name_ar, SUM(sil.cartons) AS orig, COALESCE((SELECT SUM(cnl.cartons) FROM credit_note_lines cnl JOIN credit_notes cn ON cnl.cn_id=cn.id WHERE cn.invoice_id=sil.invoice_id AND cnl.product_id=sil.product_id AND cn.status != 'Void'), 0) AS credited FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id=p.id WHERE sil.invoice_id=? GROUP BY sil.product_id"
    )?;
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceCreditRemaining {
            product_id: row.get(0)?,
            product_name: row.get(1)?,
            original_cartons: row.get(2)?,
            credited_cartons: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierPaymentPrintInfo {
    pub id: i64,
    pub receipt_no: Option<String>,
    pub date: String,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierPrintInfo {
    pub id: i64,
    pub name: String,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierReceiptPrintData {
    pub payment: SupplierPaymentPrintInfo,
    pub supplier: SupplierPrintInfo,
    pub company: CompanyPrintInfo,
}

#[tauri::command]
pub fn get_supplier_receipt_for_print(state: State<'_, DbState>, payment_id: i64) -> Result<SupplierReceiptPrintData, AppError> {
    let conn = state.0.lock()?;
    get_supplier_receipt_for_print_inner(&conn, payment_id)
}

fn get_supplier_receipt_for_print_inner(conn: &rusqlite::Connection, payment_id: i64) -> Result<SupplierReceiptPrintData, AppError> {
    let payment = conn.query_row(
        "SELECT sp.id, sp.pay_no, sp.date, sp.amount_milli, sp.method, sp.reference, sp.notes FROM supplier_payments sp WHERE sp.id=?",
        [payment_id],
        |row| Ok(SupplierPaymentPrintInfo {
            id: row.get(0)?,
            receipt_no: row.get(1)?,
            date: row.get(2)?,
            amount_milli: row.get(3)?,
            method: row.get(4)?,
            reference: row.get(5)?,
            notes: row.get(6)?,
        }),
    ).map_err(|e| format!("Payment not found: {}", e))?;
    let supplier_id: i64 = conn
        .query_row("SELECT supplier_id FROM supplier_payments WHERE id=?", [payment_id], |r| r.get(0))
        .unwrap_or(0);
    let supplier = conn
        .query_row(
            "SELECT id, name, address, vat_number, phone FROM suppliers WHERE id=?",
            [supplier_id],
            |row| Ok(SupplierPrintInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                address: row.get(2)?,
                vat_number: row.get(3)?,
                phone: row.get(4)?,
            }),
        )
        .map_err(|e| format!("Supplier not found: {}", e))?;
    let company = get_company_info(conn)?;
    Ok(SupplierReceiptPrintData { payment, supplier, company })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyPrintInfo {
    pub name: Option<String>,
    pub factory_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub vat_number: Option<String>,
    pub cr_number: Option<String>,
    pub logo_path: Option<String>,
    pub stamp_path: Option<String>,
    pub signature_path: Option<String>,
    pub footer_notes: Option<String>,
    pub bank_details: Option<String>,
    pub default_vat_pct: f64,
    pub currency: String,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub bank_iban: Option<String>,
    pub bank_swift: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPrintInfo {
    pub id: i64,
    pub name: String,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoicePrintData {
    pub invoice: SalesInvoice,
    pub customer: CustomerPrintInfo,
    pub lines: Vec<InvoiceLine>,
    pub company: CompanyPrintInfo,
    pub qr_data_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentPrintInfo {
    pub id: i64,
    pub receipt_no: Option<String>,
    pub date: String,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptPrintData {
    pub payment: PaymentPrintInfo,
    pub customer: CustomerPrintInfo,
    pub company: CompanyPrintInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryNoteData {
    pub invoice: SalesInvoice,
    pub customer: CustomerPrintInfo,
    pub lines: Vec<InvoiceLine>,
    pub company: CompanyPrintInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNoteInfo {
    pub id: i64,
    pub cn_no: Option<String>,
    pub date: String,
    pub invoice_no: Option<String>,
    pub reason: Option<String>,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub total_milli: i64,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNoteLineInfo {
    pub id: i64,
    pub product_name: Option<String>,
    pub cartons: f64,
    pub qty_cups: f64,
    pub unit_price_milli: i64,
    pub line_net_milli: i64,
    pub vat_pct: f64,
    pub vat_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNotePrintData {
    pub credit_note: CreditNoteInfo,
    pub customer: CustomerPrintInfo,
    pub lines: Vec<CreditNoteLineInfo>,
    pub company: CompanyPrintInfo,
}

pub(crate) fn get_company_info(conn: &rusqlite::Connection) -> Result<CompanyPrintInfo, AppError> {
    conn.query_row(
        "SELECT name, factory_name, address, phone, email, vat_number, cr_number, logo_path, stamp_path, signature_path, footer_notes, bank_details, default_vat_pct, currency, bank_name, bank_account_no, bank_iban, bank_swift FROM company_settings LIMIT 1",
        [],
        |row| Ok(CompanyPrintInfo {
            name: row.get(0)?,
            factory_name: row.get(1)?,
            address: row.get(2)?,
            phone: row.get(3)?,
            email: row.get(4)?,
            vat_number: row.get(5)?,
            cr_number: row.get(6)?,
            logo_path: row.get(7)?,
            stamp_path: row.get(8)?,
            signature_path: row.get(9)?,
            footer_notes: row.get(10)?,
            bank_details: row.get(11)?,
            default_vat_pct: row.get(12)?,
            currency: row.get(13)?,
            bank_name: row.get(14)?,
            bank_account_no: row.get(15)?,
            bank_iban: row.get(16)?,
            bank_swift: row.get(17)?,
        }),
    ).map_err(|e| AppError::business(format!("Company settings not found: {}", e)))
}

pub(crate) fn get_customer_info(conn: &rusqlite::Connection, customer_id: i64) -> Result<CustomerPrintInfo, AppError> {
    conn.query_row(
        "SELECT id, name, address, vat_number, phone FROM customers WHERE id=?",
        [customer_id],
        |row| Ok(CustomerPrintInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            address: row.get(2)?,
            vat_number: row.get(3)?,
            phone: row.get(4)?,
        }),
    ).map_err(|e| AppError::business(format!("Customer not found: {}", e)))
}

fn build_invoice_qr(company: &CompanyPrintInfo, invoice: &SalesInvoice) -> Option<String> {
    if invoice.total_milli <= 0 {
        return None;
    }
    let seller = company
        .name
        .as_deref()
        .or(company.factory_name.as_deref())
        .unwrap_or("PRO MAX OS");
    let vat_number = company.vat_number.as_deref().unwrap_or("");
    let timestamp = crate::zatca::to_iso8601(&invoice.date, &invoice.created_at);
    let payload = crate::zatca::build_zatca_payload(&crate::zatca::ZatcaQrFields {
        seller_name: seller,
        vat_number,
        timestamp: &timestamp,
        total_units: invoice.total_milli as f64 / 1000.0,
        vat_units: invoice.vat_milli as f64 / 1000.0,
    });
    crate::zatca::qr_png_data_url(&payload).ok()
}

#[tauri::command]
pub fn get_invoice_for_print(state: State<'_, DbState>, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let conn = state.0.lock()?;
    get_invoice_for_print_inner(&conn, invoice_id)
}

pub(crate) fn get_invoice_for_print_inner(conn: &rusqlite::Connection, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let invoice = get_invoice_by_conn(conn, invoice_id)?;
    let customer = get_customer_info(conn, invoice.customer_id)?;
    let lines = get_invoice_lines_internal(conn, invoice_id)?;
    let company = get_company_info(conn)?;
    let qr_data_url = build_invoice_qr(&company, &invoice);
    Ok(InvoicePrintData { invoice, customer, lines, company, qr_data_url })
}

#[tauri::command]
pub fn get_invoice_for_print_customs(state: State<'_, DbState>, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let conn = state.0.lock()?;
    get_invoice_for_print_customs_inner(&conn, invoice_id)
}

fn get_invoice_for_print_customs_inner(conn: &rusqlite::Connection, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let invoice = get_invoice_by_conn(conn, invoice_id)?;
    let customer = get_customer_info(conn, invoice.customer_id)?;
    let mut lines = get_invoice_lines_internal(conn, invoice_id)?;
    let company = get_company_info(conn)?;

    // Swap real prices with customs prices for customs clearance printing
    let mut new_net: i64 = 0;
    let mut new_vat: i64 = 0;
    for line in &mut lines {
        let customs_price = if line.customs_price_milli > 0 {
            line.customs_price_milli
        } else {
            line.unit_price_milli
        };
        let _qty_cups = line.cartons * line.cups_per_carton as f64;
        let line_net = (line.cartons * customs_price as f64).round() as i64;
        let line_vat = (line_net as f64 * line.vat_pct / 100.0).round() as i64;
        line.unit_price_milli = customs_price;
        line.line_net_milli = line_net;
        line.vat_milli = line_vat;
        new_net += line_net;
        new_vat += line_vat;
    }

    // Return modified print data with customs pricing
    let mut inv = invoice;
    inv.net_milli = new_net;
    inv.vat_milli = new_vat;
    inv.total_milli = new_net + new_vat;

    let qr_data_url = build_invoice_qr(&company, &inv);
    Ok(InvoicePrintData { invoice: inv, customer, lines, company, qr_data_url })
}

#[tauri::command]
pub fn get_receipt_for_print(state: State<'_, DbState>, payment_id: i64) -> Result<ReceiptPrintData, AppError> {
    let conn = state.0.lock()?;
    let payment = conn.query_row(
        "SELECT cp.id, cp.rec_no, cp.date, cp.amount_milli, cp.method, cp.reference, cp.notes FROM customer_payments cp WHERE cp.id=?",
        [payment_id],
        |row| Ok(PaymentPrintInfo {
            id: row.get(0)?,
            receipt_no: row.get(1)?,
            date: row.get(2)?,
            amount_milli: row.get(3)?,
            method: row.get(4)?,
            reference: row.get(5)?,
            status: None,
            notes: row.get(6)?,
        }),
    ).map_err(|e| format!("Payment not found: {}", e))?;
    let customer_id: i64 = conn.query_row(
        "SELECT cp.customer_id FROM customer_payments cp WHERE cp.id=?",
        [payment_id],
        |r| r.get(0),
    ).unwrap_or(0);
    let customer = get_customer_info(&conn, customer_id)?;
    let company = get_company_info(&conn)?;
    Ok(ReceiptPrintData { payment, customer, company })
}

#[tauri::command]
pub fn get_delivery_note_for_print(state: State<'_, DbState>, invoice_id: i64) -> Result<DeliveryNoteData, AppError> {
    let conn = state.0.lock()?;
    let invoice = get_invoice_by_conn(&conn, invoice_id)?;
    let customer = get_customer_info(&conn, invoice.customer_id)?;
    let lines = get_invoice_lines_internal(&conn, invoice_id)?;
    let company = get_company_info(&conn)?;
    Ok(DeliveryNoteData { invoice, customer, lines, company })
}

#[tauri::command]
pub fn get_credit_note_for_print(state: State<'_, DbState>, credit_note_id: i64) -> Result<CreditNotePrintData, AppError> {
    let conn = state.0.lock()?;
    let cn = conn.query_row(
        "SELECT cn.id, cn.cn_no, cn.date, si.inv_no, cn.reason, cn.net_milli, cn.vat_milli, cn.total_milli, cn.status, cn.notes FROM credit_notes cn LEFT JOIN sales_invoices si ON cn.invoice_id=si.id WHERE cn.id=?",
        [credit_note_id],
        |row| Ok(CreditNoteInfo {
            id: row.get(0)?,
            cn_no: row.get(1)?,
            date: row.get(2)?,
            invoice_no: row.get(3)?,
            reason: row.get(4)?,
            net_milli: row.get(5)?,
            vat_milli: row.get(6)?,
            total_milli: row.get(7)?,
            status: row.get(8)?,
            notes: row.get(9)?,
        }),
    ).map_err(|e| format!("Credit note not found: {}", e))?;
    let customer_id: i64 = conn.query_row(
        "SELECT customer_id FROM credit_notes WHERE id=?",
        [credit_note_id],
        |r| r.get(0),
    ).unwrap_or(0);
    let customer = get_customer_info(&conn, customer_id)?;
    let company = get_company_info(&conn)?;
    let mut stmt = conn.prepare(
        "SELECT cnl.id, p.name_ar, cnl.cartons, cnl.qty_cups, cnl.unit_price_milli, cnl.line_net_milli, cnl.vat_pct, cnl.vat_milli FROM credit_note_lines cnl LEFT JOIN products p ON cnl.product_id=p.id WHERE cnl.cn_id=?"
    )?;
    let lines = stmt.query_map([credit_note_id], |row| {
        Ok(CreditNoteLineInfo {
            id: row.get(0)?,
            product_name: row.get(1)?,
            cartons: row.get(2)?,
            qty_cups: row.get(3)?,
            unit_price_milli: row.get(4)?,
            line_net_milli: row.get(5)?,
            vat_pct: row.get(6)?,
            vat_milli: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(CreditNotePrintData { credit_note: cn, customer, lines, company })
}

// Internal helpers that work with an existing connection (not State)
fn get_invoice_by_conn(conn: &rusqlite::Connection, id: i64) -> Result<SalesInvoice, AppError> {
    conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at, si.is_commercial FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id WHERE si.id=?",
        [id],
        |row| {
            Ok(SalesInvoice {
                id: row.get(0)?, inv_no: row.get(1)?, date: row.get(2)?, customer_id: row.get(3)?,
                customer_name: row.get(4)?, payment_type: row.get(5)?, vat_enabled: row.get(6)?,
                net_milli: row.get(7)?, vat_milli: row.get(8)?, discount_milli: row.get(9)?,
                total_milli: row.get(10)?, discount_reason: row.get(11)?, cogs_milli: row.get(12)?,
                paid_milli: row.get(13)?, status: row.get(14)?, notes: row.get(15)?,
                created_by: row.get(16)?, created_at: row.get(17)?,
                is_commercial: row.get(18)?,
            })
        },
    ).map_err(|e| AppError::not_found(format!("Invoice not found: {}", e)))
}

fn get_invoice_lines_internal(conn: &rusqlite::Connection, invoice_id: i64) -> Result<Vec<InvoiceLine>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT sil.id, sil.invoice_id, sil.product_id, p.name_ar, sil.cartons, sil.cups_per_carton, sil.qty_cups, sil.unit_price_milli, COALESCE(sil.customs_price_milli, 0), sil.line_net_milli, sil.vat_pct, sil.vat_milli FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id=p.id WHERE sil.invoice_id=?"
    )?;
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceLine {
            id: row.get(0)?, invoice_id: row.get(1)?, product_id: row.get(2)?,
            product_name: row.get(3)?, cartons: row.get(4)?, cups_per_carton: row.get(5)?,
            qty_cups: row.get(6)?, unit_price_milli: row.get(7)?,
            customs_price_milli: row.get(8)?,
            line_net_milli: row.get(9)?, vat_pct: row.get(10)?, vat_milli: row.get(11)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use crate::db::init_database;

    fn build_test_data() -> (std::path::PathBuf, rusqlite::Connection, i64) {
        let db_path = std::env::temp_dir().join(format!("promax_qr_{}.db", uuid::Uuid::new_v4()));
        let conn = init_database(&db_path).expect("fresh db");
        conn.execute(
            "INSERT INTO company_settings(name, factory_name, address, phone, email, vat_number, default_vat_pct) VALUES('شركة التجربة','مصنع التجربة','الكويت','12345678','x@y.com','300012345600003',5.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, notes) VALUES('C1','عميل تجربة','credit',NULL,'99001122',NULL,NULL,NULL,0,'net',30,NULL)",
            [],
        )
        .unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status, notes) VALUES('INV-2026-0001','2026-08-13',?1,'credit',100000,5000,105000,'Posted','note')",
            [cid],
        )
        .unwrap();
        let iid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct) VALUES('P1','كوب تجربة','Test Cup',5,10000,5.0)",
            [],
        )
        .unwrap();
        let pid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?1,?2,10,5,50,10000,20000,100000,5.0,5000)",
            rusqlite::params![iid, pid],
        )
        .unwrap();
        (db_path, conn, iid)
    }

    fn cleanup(db_path: &std::path::Path) {
        let _ = std::fs::remove_file(db_path);
        let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
        let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    }

    #[test]
    fn invoice_print_includes_zatca_qr() {
        let (db_path, conn, iid) = build_test_data();
        let data = get_invoice_for_print_inner(&conn, iid).expect("print data");
        let qr = data.qr_data_url.expect("QR must be generated");
        assert!(qr.starts_with("data:image/png;base64,"));
        let b64 = &qr["data:image/png;base64,".len()..];
        let png = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .expect("png base64 decodes");
        assert_eq!(&png[0..4], b"\x89PNG");
        cleanup(&db_path);
    }

    #[test]
    fn invoice_print_customs_uses_customs_totals_in_qr() {
        let (db_path, conn, iid) = build_test_data();
        let data = get_invoice_for_print_customs_inner(&conn, iid).expect("customs print data");
        // 10 cartons * 20000 (customs price) = 200000 net + 5% VAT = 10000 -> 210000 total.
        assert_eq!(data.invoice.net_milli, 200000);
        assert_eq!(data.invoice.vat_milli, 10000);
        assert_eq!(data.invoice.total_milli, 210000);
        assert!(data.qr_data_url.is_some());
        cleanup(&db_path);
    }

    #[test]
    fn credit_note_reverses_invoice_and_returns_stock() {
        let db_path = std::env::temp_dir().join(format!("promax_cn_{}.db", uuid::Uuid::new_v4()));
        let mut conn = init_database(&db_path).expect("fresh db");
        conn.execute("INSERT INTO company_settings(name, vat_number, default_vat_pct) VALUES('شركة','OM0000000000000',5.0)", []).unwrap();
        conn.execute(
            "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, notes) VALUES('C1','عميل','credit',NULL,'999',NULL,NULL,NULL,0,'net',30,NULL)",
            [],
        ).unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct) VALUES('P1','كوب','Cup',5,10000,5.0)",
            [],
        ).unwrap();
        let pid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_items(product_id, kind, qty_on_hand, avg_cost_milli) VALUES(?1,'main',100,6000)",
            [pid],
        ).unwrap();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, cogs_milli, status) VALUES('INV-2026-0001','2026-08-13',?1,'credit',100000,5000,105000,60000,'Posted')",
            [cid],
        ).unwrap();
        let iid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?1,?2,10,5,50,10000,20000,100000,5.0,5000)",
            rusqlite::params![iid, pid],
        ).unwrap();
        conn.execute("UPDATE customers SET balance_milli=105000 WHERE id=?", [cid]).unwrap();

        let cn_id = create_credit_note_inner(&mut conn, 1, CreateCreditNoteInput {
            invoice_id: iid,
            date: Some("2026-08-14".to_string()),
            reason: Some("مرتجعات".to_string()),
            notes: None,
            lines: vec![CreateCreditNoteLineInput { product_id: pid, cartons: 4.0 }],
        }).expect("credit note created");

        let (cn_no, net, vat, total, cogs, status): (String, i64, i64, i64, i64, String) = conn
            .query_row(
                "SELECT cn_no, net_milli, vat_milli, total_milli, cogs_milli, status FROM credit_notes WHERE id=?",
                [cn_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .unwrap();
        assert!(cn_no.starts_with("CN-2026-"), "cn_no = {}", cn_no);
        assert_eq!(net, 40000);
        assert_eq!(vat, 2000);
        assert_eq!(total, 42000);
        assert_eq!(cogs, 24000);
        assert_eq!(status, "Posted");

        // 100 on-hand + 4 returned = 104.
        let qty: f64 = conn.query_row("SELECT qty_on_hand FROM inventory_items WHERE product_id=?", [pid], |r| r.get(0)).unwrap();
        assert_eq!(qty, 104.0);

        // 105000 - 42000 = 63000 AR balance.
        let bal: i64 = conn.query_row("SELECT balance_milli FROM customers WHERE id=?", [cid], |r| r.get(0)).unwrap();
        assert_eq!(bal, 63000);

        // Remaining returnable quantity is 10 - 4 = 6 cartons.
        let remaining = get_invoice_credit_remaining_inner(&conn, iid).expect("remaining");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].original_cartons, 10.0);
        assert_eq!(remaining[0].credited_cartons, 4.0);

        // Reversal journal is balanced and references the credit note.
        let jid: i64 = conn.query_row("SELECT journal_id FROM credit_notes WHERE id=?", [cn_id], |r| r.get(0)).unwrap();
        let (d, c): (i64, i64) = conn
            .query_row(
                "SELECT SUM(debit_milli), SUM(credit_milli) FROM journal_entry_lines WHERE entry_id=?",
                [jid],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(d > 0 && d == c);

        // Over-return is rejected (4 credited + 7 > 10 cartons).
        let err = create_credit_note_inner(&mut conn, 1, CreateCreditNoteInput {
            invoice_id: iid,
            date: None,
            reason: None,
            notes: None,
            lines: vec![CreateCreditNoteLineInput { product_id: pid, cartons: 7.0 }],
        }).unwrap_err();
        assert!(err.to_string().contains("تتجاوز"), "err = {}", err);

        // Draft invoices cannot be credited.
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status) VALUES('INV-2026-0002','2026-08-13',?1,'credit',0,0,0,'Draft')",
            [cid],
        ).unwrap();
        let did = conn.last_insert_rowid();
        let err2 = create_credit_note_inner(&mut conn, 1, CreateCreditNoteInput {
            invoice_id: did,
            date: None,
            reason: None,
            notes: None,
            lines: vec![CreateCreditNoteLineInput { product_id: pid, cartons: 1.0 }],
        }).unwrap_err();
        assert!(err2.to_string().contains("المرحلة"), "err2 = {}", err2);

        cleanup(&db_path);
    }

    #[test]
    fn supplier_receipt_print_returns_payment_and_supplier() {
        let db_path = std::env::temp_dir().join(format!("promax_srp_{}.db", uuid::Uuid::new_v4()));
        let conn = init_database(&db_path).expect("fresh db");
        conn.execute("INSERT INTO company_settings(name, vat_number, default_vat_pct) VALUES('شركة','OM0000000000000',5.0)", []).unwrap();
        conn.execute(
            "INSERT INTO suppliers(code, name, contact, phone, email, address, vat_number, currency, payment_terms, notes) VALUES('S1','مورد','Ali','998','a@b.com','نزوى','OM123','OMR','net',NULL)",
            [],
        ).unwrap();
        let sid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO supplier_payments(pay_no, date, supplier_id, amount_milli, method, reference, notes) VALUES('PAY-2026-0001','2026-08-13',?1,50000,'bank_transfer','REF1','دفع')",
            [sid],
        ).unwrap();
        let pay_id = conn.last_insert_rowid();

        let data = get_supplier_receipt_for_print_inner(&conn, pay_id).expect("print data");
        assert_eq!(data.payment.amount_milli, 50000);
        assert_eq!(data.payment.receipt_no.as_deref(), Some("PAY-2026-0001"));
        assert_eq!(data.supplier.name, "مورد");
        assert_eq!(data.supplier.vat_number.as_deref(), Some("OM123"));
        assert!(data.company.name.is_some());

        cleanup(&db_path);
    }

    #[test]
    fn validate_invoice_lines_rejects_empty_and_bad_quantities() {
        assert!(validate_invoice_lines(&[]).is_err());

        let bad_qty = CreateInvoiceLineInput { product_id: 1, cartons: 0.0, unit_price_milli: 1000, customs_price_milli: None };
        let err = validate_invoice_lines(&[bad_qty]).unwrap_err();
        assert!(err.to_string().contains("الكمية"), "err = {}", err);

        let bad_qty = CreateInvoiceLineInput { product_id: 1, cartons: -5.0, unit_price_milli: 1000, customs_price_milli: None };
        assert!(validate_invoice_lines(&[bad_qty]).is_err());

        let bad_price = CreateInvoiceLineInput { product_id: 1, cartons: 10.0, unit_price_milli: -1, customs_price_milli: None };
        let err = validate_invoice_lines(&[bad_price]).unwrap_err();
        assert!(err.to_string().contains("السعر"), "err = {}", err);

        let ok = CreateInvoiceLineInput { product_id: 1, cartons: 10.0, unit_price_milli: 1500, customs_price_milli: Some(2000) };
        assert!(validate_invoice_lines(&[ok]).is_ok());
    }

    #[test]
    fn duplicate_invoice_preserves_flags_payment_type_and_customs_price() {
        let db_path = std::env::temp_dir().join(format!("promax_dup_{}.db", uuid::Uuid::new_v4()));
        let mut conn = init_database(&db_path).expect("fresh db");
        conn.execute("INSERT INTO company_settings(name, vat_number, default_vat_pct) VALUES('شركة','OM0000000000000',5.0)", []).unwrap();
        conn.execute(
            "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, notes) VALUES('C1','عميل','credit',NULL,'999',NULL,NULL,NULL,0,'net',30,NULL)",
            [],
        ).unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct) VALUES('P1','كوب','Cup',5,10000,5.0)",
            [],
        ).unwrap();
        let pid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, vat_enabled, net_milli, vat_milli, total_milli, status, is_commercial) VALUES('CINV-2026-0001','2026-08-13',?1,'cash',0,100000,0,100000,'Posted',1)",
            [cid],
        ).unwrap();
        let iid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?1,?2,10,5,50,10000,20000,100000,0.0,0)",
            rusqlite::params![iid, pid],
        ).unwrap();

        let new_id = duplicate_invoice_inner(&mut conn, 1, iid).expect("duplicate");
        assert!(new_id != iid);

        let (inv_no, payment_type, vat_enabled, is_commercial, status, note): (String, String, i64, i64, String, String) = conn
            .query_row(
                "SELECT inv_no, COALESCE(payment_type,'credit'), vat_enabled, is_commercial, status, notes FROM sales_invoices WHERE id=?",
                [new_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .unwrap();
        assert!(inv_no.starts_with("INV-2026-"), "inv_no = {}", inv_no);
        assert_eq!(payment_type, "cash");
        assert_eq!(vat_enabled, 0);
        assert_eq!(is_commercial, 1);
        assert_eq!(status, "Draft");
        assert!(note.contains("نسخة من"), "note = {}", note);

        let (cartons, unit, customs, line_net): (f64, i64, i64, i64) = conn
            .query_row(
                "SELECT cartons, unit_price_milli, customs_price_milli, line_net_milli FROM sales_invoice_lines WHERE invoice_id=?",
                [new_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(cartons, 10.0);
        assert_eq!(unit, 10000);
        assert_eq!(customs, 20000, "customs price must be copied from the source line");
        assert_eq!(line_net, 100000);
        cleanup(&db_path);
    }

    #[test]
    fn post_credit_invoice_enforces_credit_limit() {
        let db_path = std::env::temp_dir().join(format!("promax_cl_{}.db", uuid::Uuid::new_v4()));
        let mut conn = init_database(&db_path).expect("fresh db");
        conn.execute("INSERT INTO company_settings(name, vat_number, default_vat_pct) VALUES('شركة','OM0000000000000',5.0)", []).unwrap();
        conn.execute(
            "INSERT INTO customers(code, name, ctype, credit_limit_milli, payment_terms, payment_terms_days) VALUES('C1','عميل','credit',10000,'net',30)",
            [],
        ).unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct) VALUES('P1','كوب','Cup',5,10000,5.0)",
            [],
        ).unwrap();
        let pid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_items(product_id, kind, qty_on_hand, avg_cost_milli) VALUES(?1,'main',1000,6000)",
            [pid],
        ).unwrap();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status) VALUES('INV-2026-0001','2026-08-13',?1,'credit',100000,5000,105000,'Draft')",
            [cid],
        ).unwrap();
        let iid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?1,?2,10,5,50,10000,10000,100000,5.0,5000)",
            rusqlite::params![iid, pid],
        ).unwrap();

        // Limit 10000 < total 105000: posting must be rejected and nothing changed.
        let err = post_invoice_inner(&mut conn, 1, iid).unwrap_err();
        assert!(err.to_string().contains("الحد الائتماني"), "err = {}", err);
        let status: String = conn.query_row("SELECT status FROM sales_invoices WHERE id=?", [iid], |r| r.get(0)).unwrap();
        assert_eq!(status, "Draft", "rejected post must leave the invoice as Draft");

        // Raising the limit above the total allows the post.
        conn.execute("UPDATE customers SET credit_limit_milli=200000 WHERE id=?", [cid]).unwrap();
        post_invoice_inner(&mut conn, 1, iid).expect("post with sufficient limit");
        let status: String = conn.query_row("SELECT status FROM sales_invoices WHERE id=?", [iid], |r| r.get(0)).unwrap();
        assert_eq!(status, "Posted");
        let bal: i64 = conn.query_row("SELECT balance_milli FROM customers WHERE id=?", [cid], |r| r.get(0)).unwrap();
        assert_eq!(bal, 105000);

        // A limit of 0 means unlimited: a new invoice beyond the old limit posts.
        conn.execute("UPDATE customers SET credit_limit_milli=0 WHERE id=?", [cid]).unwrap();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status) VALUES('INV-2026-0002','2026-08-13',?1,'credit',5000000,0,5000000,'Draft')",
            [cid],
        ).unwrap();
        let iid2 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?1,?2,500,5,2500,10000,10000,5000000,0.0,0)",
            rusqlite::params![iid2, pid],
        ).unwrap();
        post_invoice_inner(&mut conn, 1, iid2).expect("limit 0 means unlimited");
        let status: String = conn.query_row("SELECT status FROM sales_invoices WHERE id=?", [iid2], |r| r.get(0)).unwrap();
        assert_eq!(status, "Posted");
        cleanup(&db_path);
    }

    #[test]
    fn credit_note_reversal_credits_the_invoice_payment_account() {
        let db_path = std::env::temp_dir().join(format!("promax_cnacct_{}.db", uuid::Uuid::new_v4()));
        let mut conn = init_database(&db_path).expect("fresh db");
        conn.execute("INSERT INTO company_settings(name, vat_number, default_vat_pct) VALUES('شركة','OM0000000000000',5.0)", []).unwrap();
        conn.execute(
            "INSERT INTO customers(code, name, ctype, credit_limit_milli) VALUES('C1','عميل','credit',0)",
            [],
        ).unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO products(code, name_ar, name_en, cups_per_carton, default_price_milli, vat_pct) VALUES('P1','كوب','Cup',5,10000,5.0)",
            [],
        ).unwrap();
        let pid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_items(product_id, kind, qty_on_hand, avg_cost_milli) VALUES(?1,'main',100,6000)",
            [pid],
        ).unwrap();

        for (suffix, payment_type, expected_account) in [("A", "cash", "1100"), ("B", "cheque", "1101")] {
            let inv_no = format!("INV-2026-{}", suffix);
            conn.execute(
                "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status) VALUES(?1,'2026-08-13',?2,?3,100000,5000,105000,'Posted')",
                rusqlite::params![inv_no, cid, payment_type],
            ).unwrap();
            let iid = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?1,?2,10,5,50,10000,10000,100000,5.0,5000)",
                rusqlite::params![iid, pid],
            ).unwrap();

            let cn_id = create_credit_note_inner(&mut conn, 1, CreateCreditNoteInput {
                invoice_id: iid,
                date: Some("2026-08-14".to_string()),
                reason: Some("مرتجع".to_string()),
                notes: None,
                lines: vec![CreateCreditNoteLineInput { product_id: pid, cartons: 10.0 }],
            }).expect("credit note");

            let jid: i64 = conn.query_row("SELECT journal_id FROM credit_notes WHERE id=?", [cn_id], |r| r.get(0)).unwrap();
            let credited: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(credit_milli),0) FROM journal_entry_lines WHERE entry_id=? AND account_code=?",
                    rusqlite::params![jid, expected_account],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(credited, 105000, "{} invoice reversal must credit {}", payment_type, expected_account);
            let wrong_account: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(credit_milli),0) FROM journal_entry_lines WHERE entry_id=? AND account_code='1200'",
                    [jid],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(wrong_account, 0, "credit-sales account must not be used for {} payment", payment_type);
            let (d, c): (i64, i64) = conn
                .query_row(
                    "SELECT SUM(debit_milli), SUM(credit_milli) FROM journal_entry_lines WHERE entry_id=?",
                    [jid],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert!(d > 0 && d == c);
        }
        cleanup(&db_path);
    }
}
