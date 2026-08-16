use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Purchase {
    pub id: i64,
    pub pur_no: Option<String>,
    pub date: String,
    pub supplier_id: i64,
    pub supplier_name: Option<String>,
    pub supplier_invoice_no: Option<String>,
    pub vat_enabled: i64,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub total_milli: i64,
    pub paid_milli: i64,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseLine {
    pub id: i64,
    pub purchase_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub qty: f64,
    pub unit_cost_milli: i64,
    pub line_net_milli: i64,
    pub vat_pct: f64,
    pub vat_milli: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseInput {
    pub supplier_id: i64,
    pub date: String,
    pub supplier_invoice_no: Option<String>,
    pub vat_enabled: Option<bool>,
    pub notes: Option<String>,
    pub lines: Vec<CreatePurchaseLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseLineInput {
    pub item_id: i64,
    pub qty: f64,
    pub unit_cost_milli: i64,
    pub vat_pct: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSupplierPaymentInput {
    pub supplier_id: i64,
    pub date: String,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_purchases(state: State<'_, DbState>) -> Result<Vec<Purchase>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.pur_no, p.date, p.supplier_id, s.name AS supplier_name,
                    p.supplier_invoice_no, p.vat_enabled, p.net_milli, p.vat_milli,
                    p.total_milli, p.paid_milli, p.status, p.notes, p.created_by, p.created_at
             FROM purchases p
             LEFT JOIN suppliers s ON s.id = p.supplier_id
             ORDER BY p.id DESC",
        )?;

    let rows = stmt.query_map([], |row| {
        Ok(Purchase {
            id: row.get(0)?,
            pur_no: row.get(1)?,
            date: row.get(2)?,
            supplier_id: row.get(3)?,
            supplier_name: row.get(4)?,
            supplier_invoice_no: row.get(5)?,
            vat_enabled: row.get(6)?,
            net_milli: row.get(7)?,
            vat_milli: row.get(8)?,
            total_milli: row.get(9)?,
            paid_milli: row.get(10)?,
            status: row.get(11)?,
            notes: row.get(12)?,
            created_by: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_purchase(id: i64, state: State<'_, DbState>) -> Result<Purchase, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.pur_no, p.date, p.supplier_id, s.name AS supplier_name,
                    p.supplier_invoice_no, p.vat_enabled, p.net_milli, p.vat_milli,
                    p.total_milli, p.paid_milli, p.status, p.notes, p.created_by, p.created_at
             FROM purchases p
             LEFT JOIN suppliers s ON s.id = p.supplier_id
             WHERE p.id = ?1",
        )?;

    let purchase = stmt
        .query_row([id], |row| {
            Ok(Purchase {
                id: row.get(0)?,
                pur_no: row.get(1)?,
                date: row.get(2)?,
                supplier_id: row.get(3)?,
                supplier_name: row.get(4)?,
                supplier_invoice_no: row.get(5)?,
                vat_enabled: row.get(6)?,
                net_milli: row.get(7)?,
                vat_milli: row.get(8)?,
                total_milli: row.get(9)?,
                paid_milli: row.get(10)?,
                status: row.get(11)?,
                notes: row.get(12)?,
                created_by: row.get(13)?,
                created_at: row.get(14)?,
            })
        })?;

    Ok(purchase)
}

#[tauri::command]
pub fn get_purchase_lines(purchase_id: i64, state: State<'_, DbState>) -> Result<Vec<PurchaseLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT pl.id, pl.purchase_id, pl.item_id, COALESCE(i.name_ar, i.name_en, '') AS item_name,
                    pl.qty, pl.unit_cost_milli, pl.line_net_milli, pl.vat_pct, pl.vat_milli
             FROM purchase_lines pl
             LEFT JOIN inventory_items i ON i.id = pl.item_id
             WHERE pl.purchase_id = ?1",
        )?;

    let rows = stmt.query_map([purchase_id], |row| {
        Ok(PurchaseLine {
            id: row.get(0)?,
            purchase_id: row.get(1)?,
            item_id: row.get(2)?,
            item_name: row.get(3)?,
            qty: row.get(4)?,
            unit_cost_milli: row.get(5)?,
            line_net_milli: row.get(6)?,
            vat_pct: row.get(7)?,
            vat_milli: row.get(8)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_purchase(input: CreatePurchaseInput, state: State<'_, DbState>, user_id: i64) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.lines.is_empty() {
        return Err(AppError::validation("أدخل بنداً واحداً على الأقل"));
    }
    for line in &input.lines {
        if line.qty <= 0.0 {
            return Err(AppError::validation("الكمية يجب أن تكون أكبر من صفر"));
        }
        if line.unit_cost_milli < 0 {
            return Err(AppError::validation("سعر التكلفة لا يمكن أن يكون سالباً"));
        }
        if let Some(v) = line.vat_pct {
            if v < 0.0 {
                return Err(AppError::validation("نسبة الضريبة لا يمكن أن تكون سالبة"));
            }
        }
    }
    let tx = conn.transaction()?;

    let supplier_exists: i64 = tx
        .query_row("SELECT COUNT(*) FROM suppliers WHERE id=?1", [input.supplier_id], |r| r.get(0))?;
    if supplier_exists == 0 {
        return Err(AppError::not_found("المورد غير موجود"));
    }

    let year: String = tx
        .query_row("SELECT substr(?1, 1, 4)", [&input.date], |row| row.get(0))?;

    let next_num = next_sequence(&tx, "PUR", &year)?;

    let pur_no = format!("PUR-{}-{:04}", year, next_num);
    let vat_enabled: i64 = if input.vat_enabled.unwrap_or(false) { 1 } else { 0 };

    let mut net_milli: i64 = 0;
    let mut vat_milli: i64 = 0;
    for line in &input.lines {
        let line_vat = ((line.unit_cost_milli as f64) * (line.qty) * (line.vat_pct.unwrap_or(0.0)) / 100.0).round() as i64;
        let line_net = ((line.unit_cost_milli as f64) * (line.qty)).round() as i64;
        net_milli += line_net;
        vat_milli += line_vat;
    }
    let total_milli = net_milli + vat_milli;

    tx.execute(
        "INSERT INTO purchases(pur_no, date, supplier_id, supplier_invoice_no, vat_enabled, net_milli, vat_milli, total_milli, paid_milli, status, notes)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 'draft', ?9)",
        rusqlite::params![
            pur_no,
            input.date,
            input.supplier_id,
            input.supplier_invoice_no,
            vat_enabled,
            net_milli,
            vat_milli,
            total_milli,
            input.notes,
        ],
    )?;

    let purchase_id: i64 = tx.last_insert_rowid();

    for line in &input.lines {
        let line_vat = ((line.unit_cost_milli as f64) * line.qty * (line.vat_pct.unwrap_or(0.0)) / 100.0).round() as i64;
        let line_net = ((line.unit_cost_milli as f64) * line.qty).round() as i64;

        tx.execute(
            "INSERT INTO purchase_lines(purchase_id, item_id, qty, unit_cost_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                purchase_id,
                line.item_id,
                line.qty,
                line.unit_cost_milli,
                line_net,
                line.vat_pct.unwrap_or(0.0),
                line_vat,
            ],
        )?;
    }

    let _ = rbac::log_audit(&tx, None, None, "create_purchase", "purchases", Some(purchase_id), None, Some(&pur_no), None);
    tx.commit()?;
    Ok(purchase_id)
}

#[tauri::command]
pub fn post_purchase(id: i64, state: State<'_, DbState>, user_id: i64) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let journal_id = post_purchase_inner(&tx, id)?;
    let _ = rbac::log_audit(&tx, Some(user_id), None, "post_purchase", "purchases", Some(id), None, Some(&format!("journal: {}", journal_id)), None);
    tx.commit()?;
    Ok(journal_id)
}

pub(crate) fn post_purchase_inner(conn: &rusqlite::Connection, id: i64) -> Result<i64, AppError> {
    let (date, net_milli, vat_milli, total_milli, status, journal_id): (String, i64, i64, i64, String, Option<i64>) = conn
        .query_row(
            "SELECT date, net_milli, vat_milli, total_milli, status, journal_id FROM purchases WHERE id=?1",
            [id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )?;

    if journal_id.is_some() {
        return Err(AppError::validation("تم ترحيل هذه المشتريات مسبقاً"));
    }
    if status == "Void" || status == "void" {
        return Err(AppError::validation("لا يمكن ترحيل مشتريات ملغاة"));
    }

    let mut lines: Vec<(String, i64, i64, Option<String>)> = vec![
        ("1400".to_string(), net_milli, 0, Some("شراء بضاعة".to_string())),
    ];
    if vat_milli > 0 {
        lines.push(("2100".to_string(), vat_milli, 0, Some("ضريبة مشتريات".to_string())));
    }
    lines.push(("2200".to_string(), 0, total_milli, Some("مستحق للمورد".to_string())));

    let journal_id = crate::commands::accounting::post_to_journal(
        conn,
        "purchase",
        id,
        &date,
        "فاتورة مشتريات",
        &lines,
        "system",
    )?;

    conn.execute(
        "UPDATE purchases SET status='Posted', journal_id=?1 WHERE id=?2",
        rusqlite::params![journal_id, id],
    )?;

    // Keep the supplier's running balance in sync: a posted purchase increases
    // what is owed. Payments (create_supplier_payment) decrease it again.
    let supplier_id: i64 = conn.query_row(
        "SELECT supplier_id FROM purchases WHERE id=?1",
        [id],
        |r| r.get(0),
    )?;
    conn.execute(
        "UPDATE suppliers SET balance_milli = COALESCE(balance_milli, 0) + ?1 WHERE id = ?2",
        rusqlite::params![total_milli, supplier_id],
    )?;

    // Bring purchased quantities into stock and recompute the weighted-average cost
    // so physical stock stays in sync with the Inventory (1400) ledger account.
    {
        let mut stmt = conn.prepare(
            "SELECT item_id, qty, unit_cost_milli FROM purchase_lines WHERE purchase_id=?1",
        )?;
        let rows = stmt.query_map([id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, i64>(2)?))
        })?;
        let line_rows: Vec<(i64, f64, i64)> = rows.collect::<Result<_, _>>()?;
        for (item_id, qty, unit_cost_milli) in line_rows {
            let (old_qty, old_avg): (f64, i64) = conn.query_row(
                "SELECT qty_on_hand, COALESCE(avg_cost_milli, 0) FROM inventory_items WHERE id=?1",
                [item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let new_qty = old_qty + qty;
            let new_avg = if new_qty > 0.0 {
                ((old_qty * old_avg as f64 + qty * unit_cost_milli as f64) / new_qty).round() as i64
            } else {
                unit_cost_milli
            };
            conn.execute(
                "UPDATE inventory_items SET qty_on_hand=?1, avg_cost_milli=?2 WHERE id=?3",
                rusqlite::params![new_qty, new_avg, item_id],
            )?;
            conn.execute(
                "INSERT INTO inventory_movements(ts, item_id, mtype, qty_in, unit_cost_milli, ref_type, ref_id, notes)
                 VALUES(datetime('now'), ?1, 'purchase', ?2, ?3, 'purchase', ?4, 'استلام مشتريات')",
                rusqlite::params![item_id, qty, unit_cost_milli, id],
            )?;
        }
    }

    Ok(journal_id)
}

#[tauri::command]
pub fn list_suppliers_for_select(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare("SELECT id, name FROM suppliers ORDER BY name")?;

    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
        }))
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_supplier_payment(input: CreateSupplierPaymentInput, state: State<'_, DbState>, user_id: i64) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("المبلغ يجب أن يكون أكبر من صفر"));
    }
    let tx = conn.transaction()?;

    let supplier_exists: i64 = tx
        .query_row("SELECT COUNT(*) FROM suppliers WHERE id=?1", [input.supplier_id], |r| r.get(0))?;
    if supplier_exists == 0 {
        return Err(AppError::not_found("المورد غير موجود"));
    }

    let method = input.method.clone().unwrap_or_else(|| "cash".into());
    let year: String = tx
        .query_row("SELECT substr(?1, 1, 4)", [&input.date], |row| row.get(0))?;
    let seq = next_sequence(&tx, "PAY", &year)?;
    let pay_no = format!("PAY-{}-{:04}", year, seq);
    let created_by: String = tx
        .query_row("SELECT username FROM users WHERE id=?", [user_id], |r| r.get(0))
        .unwrap_or_else(|_| "user".into());
    tx.execute(
        "INSERT INTO supplier_payments(pay_no, supplier_id, date, amount_milli, method, reference, notes, created_by, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
        rusqlite::params![
            pay_no,
            input.supplier_id,
            input.date.clone(),
            input.amount_milli,
            method.clone(),
            input.reference,
            input.notes,
            created_by,
        ],
    )?;

    let payment_id: i64 = tx.last_insert_rowid();

    // Allocate the payment across the supplier's open (posted, unpaid) purchases,
    // oldest first (FIFO), so per-purchase paid/outstanding amounts stay accurate.
    // Any amount beyond the outstanding total remains as an on-account credit.
    allocate_payment_fifo(&tx, input.supplier_id, input.amount_milli, None)?;

    // Keep the supplier's running balance in sync: a payment reduces what is owed.
    tx.execute(
        "UPDATE suppliers SET balance_milli = COALESCE(balance_milli, 0) - ?1 WHERE id = ?2",
        rusqlite::params![input.amount_milli, input.supplier_id],
    )?;

    let cash_account = crate::commands::accounting::resolve_cash_account(&tx, None, &method)?;
    let lines: Vec<(String, i64, i64, Option<String>)> = vec![
        ("2200".to_string(), input.amount_milli, 0, Some("سند صرف".to_string())),
        (cash_account, 0, input.amount_milli, None),
    ];
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx,
        "supplier_payment",
        payment_id,
        &input.date,
        "سند دفع مورد",
        &lines,
        "system",
    )?;
    tx.execute(
        "UPDATE supplier_payments SET journal_id=?1 WHERE id=?2",
        rusqlite::params![journal_id, payment_id],
    )?;

    let _ = rbac::log_audit(&tx, Some(user_id), None, "create_supplier_payment", "supplier_payments", Some(payment_id), None, None, None);
    tx.commit()?;
    Ok(payment_id)
}

/// Spreads `amount` across the supplier's open (posted, unpaid) purchases, oldest
/// first (FIFO). `exclude_id` is skipped (used when redistributing the payments of
/// a purchase being voided). Returns the leftover that no open purchase absorbed.
pub(crate) fn allocate_payment_fifo(
    conn: &rusqlite::Connection,
    supplier_id: i64,
    mut amount: i64,
    exclude_id: Option<i64>,
) -> Result<i64, AppError> {    let mut stmt = conn.prepare(
        "SELECT id, total_milli, paid_milli FROM purchases
         WHERE supplier_id=?1 AND LOWER(status) NOT IN ('void','draft') AND total_milli > paid_milli
         ORDER BY date ASC, id ASC",
    )?;
    let rows = stmt.query_map([supplier_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
    })?;
    let open_purchases: Vec<(i64, i64, i64)> = rows.collect::<Result<_, _>>()?;
    drop(stmt);

    for (pid, total, paid) in open_purchases {
        if amount <= 0 {
            break;
        }
        if Some(pid) == exclude_id {
            continue;
        }
        let outstanding = total - paid;
        let apply = amount.min(outstanding);
        if apply > 0 {
            conn.execute(
                "UPDATE purchases SET paid_milli = paid_milli + ?1 WHERE id = ?2",
                rusqlite::params![apply, pid],
            )?;
            amount -= apply;
        }
    }
    Ok(amount)
}

/// Reverses the stock receipt recorded at posting time (`post_purchase_inner`):
/// removes the purchased quantity and rolls back the weighted-average cost.
pub(crate) fn reverse_purchase_stock(conn: &rusqlite::Connection, purchase_id: i64) -> Result<(), AppError> {
    let line_rows: Vec<(i64, f64, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT item_id, qty, unit_cost_milli FROM purchase_lines WHERE purchase_id=?1",
        )?;
        let rows = stmt.query_map([purchase_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, i64>(2)?))
        })?;
        rows.collect::<Result<_, _>>()?
    };

    for (item_id, qty, unit_cost_milli) in line_rows {
        let (cur_qty, cur_avg): (f64, i64) = conn.query_row(
            "SELECT qty_on_hand, COALESCE(avg_cost_milli, 0) FROM inventory_items WHERE id=?1",
            [item_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let new_qty = cur_qty - qty;
        if new_qty < -0.0001 {
            return Err(AppError::validation(format!(
                "لا يمكن إلغاء المشتريات: الكمية المتاحة للمخزون ({}) أقل من الكمية المطلوب ردها ({})",
                cur_qty, qty
            )));
        }
        let cur_value = cur_qty * cur_avg as f64;
        let purchase_value = qty * unit_cost_milli as f64;
        let new_value = cur_value - purchase_value;
        let new_avg = if new_qty > 0.0 {
            (new_value / new_qty).round() as i64
        } else {
            0
        };
        conn.execute(
            "UPDATE inventory_items SET qty_on_hand=?1, avg_cost_milli=?2 WHERE id=?3",
            rusqlite::params![new_qty, new_avg, item_id],
        )?;
        conn.execute(
            "INSERT INTO inventory_movements(ts, item_id, mtype, qty_out, unit_cost_milli, ref_type, ref_id, notes)
             VALUES(datetime('now'), ?1, 'purchase_reversal', ?2, ?3, 'purchase', ?4, 'إلغاء مشتريات')",
            rusqlite::params![item_id, qty, unit_cost_milli, purchase_id],
        )?;
    }
    Ok(())
}

#[tauri::command]
pub fn void_purchase(state: State<'_, DbState>, user_id: i64, id: i64, reason: Option<String>) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    void_purchase_inner(&mut conn, user_id, id, reason)
}

pub(crate) fn void_purchase_inner(conn: &mut rusqlite::Connection, user_id: i64, id: i64, reason: Option<String>) -> Result<String, AppError> {
    let (status, journal_id): (String, Option<i64>) = conn
        .query_row(
            "SELECT status, journal_id FROM purchases WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| AppError::not_found(format!("Purchase not found: {}", e)))?;

    let status_lower = status.to_lowercase();
    if status_lower != "draft" && status_lower != "posted" {
        return Err(AppError::validation("يمكن إلغاء المشتريات المسودة أو المرحلة فقط"));
    }

    // Every mutation below must be atomic: a failure part-way through would
    // otherwise leave stock, ledger, payments and supplier balances inconsistent.
    let tx = conn.transaction()?;

    if status_lower == "posted" {
        let (supplier_id, paid_milli, total_milli): (i64, i64, i64) = tx.query_row(
            "SELECT supplier_id, COALESCE(paid_milli, 0), COALESCE(total_milli, 0) FROM purchases WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        // Reverse the posted journal entry (mirror of void_invoice).
        if let Some(jid) = journal_id {
            let already_reversed: Option<i64> = tx.query_row(
                "SELECT reversed_by FROM journal_entries WHERE id=?",
                [jid],
                |r| r.get(0),
            ).unwrap_or(None);
            if already_reversed.is_none() {
                let inv_date: String = tx.query_row(
                    "SELECT COALESCE(date, date('now')) FROM purchases WHERE id=?",
                    [id],
                    |r| r.get(0),
                ).unwrap_or_else(|_| "".to_string());
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
                    "purchase_reversal",
                    id,
                    &inv_date,
                    "إلغاء مشتريات",
                    &lines,
                    "system",
                )?;
                tx.execute(
                    "UPDATE journal_entries SET reversed_by=? WHERE id=?",
                    rusqlite::params![rev_id, jid],
                )?;
            }
        }

        // Return the purchased goods to (out of) stock and roll back the average cost.
        reverse_purchase_stock(&tx, id)?;

        // The purchase is no longer owed, so the supplier's running balance drops
        // by the full invoice total (payments were already deducted when made).
        tx.execute(
            "UPDATE suppliers SET balance_milli = COALESCE(balance_milli, 0) - ?1 WHERE id = ?2",
            rusqlite::params![total_milli, supplier_id],
        )?;

        // Payments that had been allocated to this purchase now cover the supplier's
        // remaining open purchases (FIFO); any surplus stays as an on-account credit.
        if paid_milli > 0 {
            allocate_payment_fifo(&tx, supplier_id, paid_milli, Some(id))?;
            tx.execute(
                "UPDATE purchases SET paid_milli = 0 WHERE id=?",
                [id],
            )?;
        }
    }

    let notes_addon = reason.unwrap_or_default();
    if notes_addon.is_empty() {
        tx.execute("UPDATE purchases SET status='Void' WHERE id=?", [id])?;
    } else {
        tx.execute(
            "UPDATE purchases SET status='Void', notes=COALESCE(notes,'') || '\n[إلغاء] ' || ? WHERE id=?",
            rusqlite::params![notes_addon, id],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "void_purchase", "purchases", Some(id), None, Some("Void"), Some(&notes_addon));
    tx.commit()?;
    Ok("تم إلغاء المشتريات".to_string())
}
