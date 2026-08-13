use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperatingAdvance {
    pub id: i64,
    pub advance_no: String,
    pub date: String,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub department: Option<String>,
    pub purpose: String,
    pub description: Option<String>,
    pub amount_milli: i64,
    pub currency: String,
    pub exchange_rate: f64,
    pub status: String,
    pub approval_status: String,
    pub approved_by: Option<i64>,
    pub approved_at: Option<String>,
    pub disbursed_by: Option<i64>,
    pub disbursed_at: Option<String>,
    pub source_account_code: Option<String>,
    pub advance_gl_account_code: String,
    pub default_expense_account_code: Option<String>,
    pub expected_return_date: Option<String>,
    pub actual_return_date: Option<String>,
    pub total_spent_milli: i64,
    pub total_returned_milli: i64,
    pub balance_milli: i64,
    pub notes: Option<String>,
    pub created_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvanceTransaction {
    pub id: i64,
    pub advance_id: i64,
    pub ts: String,
    pub ttype: String,
    pub amount_milli: i64,
    pub balance_after_milli: i64,
    pub account_code: Option<String>,
    pub category: Option<String>,
    pub vendor_name: Option<String>,
    pub invoice_no: Option<String>,
    pub invoice_date: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub attachment_ids: Option<String>,
    pub journal_id: Option<i64>,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvanceReceipt {
    pub id: i64,
    pub advance_id: i64,
    pub transaction_id: Option<i64>,
    pub receipt_no: String,
    pub date: String,
    pub vendor_name: Option<String>,
    pub amount_milli: i64,
    pub vat_milli: i64,
    pub net_milli: i64,
    pub category: Option<String>,
    pub account_code: Option<String>,
    pub description: Option<String>,
    pub attachment_ids: Option<String>,
    pub status: String,
    pub approved_by: Option<i64>,
    pub approved_at: Option<String>,
    pub journal_id: Option<i64>,
    pub created_by: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdvanceInput {
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub department: Option<String>,
    pub purpose: String,
    pub description: Option<String>,
    pub amount_milli: i64,
    pub currency: Option<String>,
    pub exchange_rate: Option<f64>,
    pub source_account_code: Option<String>,
    pub advance_gl_account_code: Option<String>,
    pub default_expense_account_code: Option<String>,
    pub expected_return_date: Option<String>,
    pub notes: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct DisburseAdvanceInput {
    pub advance_id: i64,
    pub source_account_code: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordSpendInput {
    pub advance_id: i64,
    pub amount_milli: i64,
    pub account_code: Option<String>,
    pub category: Option<String>,
    pub vendor_name: Option<String>,
    pub invoice_no: Option<String>,
    pub invoice_date: Option<String>,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub attachment_ids: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitReceiptInput {
    pub advance_id: i64,
    pub receipt_no: String,
    pub date: String,
    pub vendor_name: Option<String>,
    pub amount_milli: i64,
    pub vat_milli: i64,
    pub net_milli: i64,
    pub category: Option<String>,
    pub account_code: Option<String>,
    pub description: Option<String>,
    pub attachment_ids: Option<String>,
    #[allow(dead_code)]
    pub notes: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct ReturnAdvanceInput {
    pub advance_id: i64,
    pub amount_milli: i64,
    pub source_account_code: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReconcileAdvanceInput {
    pub advance_id: i64,
    pub physical_amount_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveAdvanceInput {
    pub advance_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct RejectAdvanceInput {
    pub advance_id: i64,
    #[allow(dead_code)]
    pub rejected_by: i64,
    pub reason: String,
}

fn gen_advance_no(conn: &rusqlite::Connection) -> Result<String, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM operating_advances WHERE date = date('now')",
        [],
        |r| r.get(0),
    )?;
    Ok(format!("ADV-{}-{:04}", chrono::Utc::now().format("%Y%m%d"), count + 1))
}

fn generate_journal_no(conn: &rusqlite::Connection) -> Result<String, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE date = date('now')",
        [],
        |r| r.get(0),
    )?;
    Ok(format!("JE-{}-{:04}", chrono::Utc::now().format("%Y%m%d"), count + 1))
}

fn row_to_advance(row: &rusqlite::Row) -> Result<OperatingAdvance, rusqlite::Error> {
    Ok(OperatingAdvance {
        id: row.get(0)?, advance_no: row.get(1)?, date: row.get(2)?,
        employee_id: row.get(3)?, employee_name: row.get(4)?, department: row.get(5)?,
        purpose: row.get(6)?, description: row.get(7)?, amount_milli: row.get(8)?,
        currency: row.get(9)?, exchange_rate: row.get(10)?, status: row.get(11)?,
        approval_status: row.get(12)?, approved_by: row.get(13)?, approved_at: row.get(14)?,
        disbursed_by: row.get(15)?, disbursed_at: row.get(16)?,
        source_account_code: row.get(17)?, advance_gl_account_code: row.get(18)?,
        default_expense_account_code: row.get(19)?, expected_return_date: row.get(20)?,
        actual_return_date: row.get(21)?, total_spent_milli: row.get(22)?,
        total_returned_milli: row.get(23)?, balance_milli: row.get(24)?,
        notes: row.get(25)?, created_by: row.get(26)?, created_at: row.get(27)?,
        updated_at: row.get(28)?,
    })
}

#[tauri::command]
pub fn list_operating_advances(
    state: State<'_, DbState>,
    status_filter: Option<String>,
    employee_id: Option<i64>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<Vec<OperatingAdvance>, AppError> {
    let conn = state.0.lock()?;
    let sql = "SELECT id, advance_no, date, employee_id, employee_name, department, purpose,
         description, amount_milli, currency, exchange_rate, status, approval_status,
         approved_by, approved_at, disbursed_by, disbursed_at, source_account_code,
         advance_gl_account_code, default_expense_account_code, expected_return_date,
         actual_return_date, total_spent_milli, total_returned_milli, balance_milli,
         notes, created_by, created_at, updated_at
         FROM operating_advances WHERE 1=1";
    let s = status_filter.as_deref();
    let e = employee_id;
    let f = from_date.as_deref();
    let t = to_date.as_deref();
    let mut full_sql = sql.to_string();
    if s.is_some() { full_sql.push_str(" AND status = ?1"); }
    if e.is_some() { full_sql.push_str(&format!(" AND employee_id = ?{}", if s.is_some() { 2 } else { 1 })); }
    if f.is_some() { full_sql.push_str(&format!(" AND date >= ?{}", [s.is_some() as i32, e.is_some() as i32].iter().sum::<i32>() + 1)); }
    if t.is_some() { full_sql.push_str(&format!(" AND date <= ?{}", [s.is_some() as i32, e.is_some() as i32, f.is_some() as i32].iter().sum::<i32>() + 1)); }
    full_sql.push_str(" ORDER BY date DESC, id DESC");
    let mut stmt = conn.prepare(&full_sql)?;
    let rows = match (s, e, f, t) {
        (Some(s), Some(e), Some(f), Some(t)) => stmt.query_map(params![s, e, f, t], row_to_advance)?,
        (Some(s), Some(e), Some(f), None) => stmt.query_map(params![s, e, f], row_to_advance)?,
        (Some(s), Some(e), None, None) => stmt.query_map(params![s, e], row_to_advance)?,
        (Some(s), None, None, None) => stmt.query_map(params![s], row_to_advance)?,
        (None, Some(e), Some(f), Some(t)) => stmt.query_map(params![e, f, t], row_to_advance)?,
        (None, Some(e), Some(f), None) => stmt.query_map(params![e, f], row_to_advance)?,
        (None, Some(e), None, None) => stmt.query_map(params![e], row_to_advance)?,
        (None, None, Some(f), Some(t)) => stmt.query_map(params![f, t], row_to_advance)?,
        (None, None, Some(f), None) => stmt.query_map(params![f], row_to_advance)?,
        (None, None, None, Some(t)) => stmt.query_map(params![t], row_to_advance)?,
        (None, None, None, None) => stmt.query_map([], row_to_advance)?,
        _ => stmt.query_map([], row_to_advance)?,
    };
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

#[tauri::command]
pub fn get_operating_advance(state: State<'_, DbState>, id: i64) -> Result<OperatingAdvance, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, advance_no, date, employee_id, employee_name, department, purpose,
         description, amount_milli, currency, exchange_rate, status, approval_status,
         approved_by, approved_at, disbursed_by, disbursed_at, source_account_code,
         advance_gl_account_code, default_expense_account_code, expected_return_date,
         actual_return_date, total_spent_milli, total_returned_milli, balance_milli,
         notes, created_by, created_at, updated_at
         FROM operating_advances WHERE id = ?1",
        params![id],
        row_to_advance,
    ).map_err(AppError::from)
}

#[tauri::command]
pub fn create_operating_advance(
    state: State<'_, DbState>,
    input: CreateAdvanceInput,
) -> Result<OperatingAdvance, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;
    let advance_no = gen_advance_no(&tx)?;
    let currency = input.currency.unwrap_or_else(|| "OMR".to_string());
    let _exchange_rate = input.exchange_rate.unwrap_or(1.0);
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    tx.execute(
        "INSERT INTO operating_advances (advance_no, date, employee_id, employee_name,
         department, purpose, description, amount_milli, currency, exchange_rate,
         status, approval_status, source_account_code, advance_gl_account_code,
         default_expense_account_code, expected_return_date, notes, created_by, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'draft','pending',?10,?11,?12,?13,?14,?15,datetime('now'))",
        params![advance_no, date_str, input.employee_id, input.employee_name, input.department,
                input.purpose, input.description, input.amount_milli, currency,
                input.source_account_code, input.advance_gl_account_code,
                input.default_expense_account_code, input.expected_return_date, input.notes, input.created_by],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    let _ = rbac::log_audit(&conn, None, None, "create_advance", "operating_advances", Some(id), None, Some(&input.purpose), None);
    drop(conn);
    get_operating_advance(state, id)
}

#[tauri::command]
pub fn approve_advance(
    state: State<'_, DbState>,
    user_id: i64,
    input: ApproveAdvanceInput,
) -> Result<OperatingAdvance, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let current: String = tx.query_row(
        "SELECT status FROM operating_advances WHERE id = ?1",
        params![input.advance_id],
        |r| r.get(0),
    )?;
    if current != "draft" {
        return Err(AppError::validation("لا يمكن اعتماد السلف غير المسودة"));
    }
    tx.execute(
        "UPDATE operating_advances SET approval_status = 'approved', approved_by = ?1,
         approved_at = datetime('now'), status = 'approved', updated_at = datetime('now')
         WHERE id = ?2",
        params![user_id, input.advance_id],
    )?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "approve_advance", "operating_advances", Some(input.advance_id), None, None, None);
    drop(conn);
    get_operating_advance(state, input.advance_id)
}

#[tauri::command]
pub fn reject_advance(
    state: State<'_, DbState>,
    user_id: i64,
    input: RejectAdvanceInput,
) -> Result<OperatingAdvance, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE operating_advances SET approval_status = 'rejected', status = 'cancelled',
         updated_at = datetime('now'), notes = ?1 WHERE id = ?2",
        params![input.reason, input.advance_id],
    )?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "reject_advance", "operating_advances", Some(input.advance_id), None, Some(&input.reason), None);
    drop(conn);
    get_operating_advance(state, input.advance_id)
}

#[tauri::command]
pub fn disburse_advance(
    state: State<'_, DbState>,
    user_id: i64,
    input: DisburseAdvanceInput,
) -> Result<OperatingAdvance, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let (a_amount, a_gl): (i64, String) = tx.query_row(
        "SELECT amount_milli, advance_gl_account_code FROM operating_advances WHERE id = ?1 AND status = 'approved'",
        params![input.advance_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let amount_milli = a_amount;
    let je_no = generate_journal_no(&tx)?;
    let source_code = input.source_account_code.clone();
    tx.execute(
        "INSERT INTO journal_entries (entry_no, date, memo, ref_type, ref_id, created_by)
         VALUES (?1,date('now'),?2,'advance_disburse',?3,?4)",
        params![je_no, format!("Disbursement advance {}", input.advance_id), input.advance_id, user_id],
    )?;
    let je_id = tx.last_insert_rowid();
    // Disbursing cash to an employee creates an advance receivable:
    // Debit the advance (asset) account, credit the funding account.
    tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,?2,?3,0,?4)",
        params![je_id, a_gl, amount_milli, format!("Advance to employee {}", input.advance_id)])?;
    tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,?2,0,?3,?4)",
        params![je_id, source_code, amount_milli, format!("Funded from {}", source_code)])?;
    let adv_no = format!("ADV-{}", input.advance_id);
    tx.execute(
        "UPDATE operating_advances SET status = 'disbursed', disbursed_by = ?1, disbursed_at = datetime('now'),
         source_account_code = ?2, balance_milli = amount_milli, total_spent_milli = 0, total_returned_milli = 0,
         updated_at = datetime('now') WHERE id = ?3",
        params![user_id, source_code, input.advance_id],
    )?;
    tx.execute(
        "INSERT INTO advance_transactions (advance_id,ts,ttype,amount_milli,balance_after_milli,account_code,reference,notes,journal_id,created_by)
         VALUES (?1,datetime('now'),'disburse',?2,?2,?3,?4,?5,?6,?7)",
        params![input.advance_id, amount_milli, source_code, &adv_no, input.notes, je_id, user_id],
    )?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "disburse_advance", "operating_advances", Some(input.advance_id), None, None, None);
    drop(conn);
    get_operating_advance(state, input.advance_id)
}

#[tauri::command]
pub fn record_advance_spend(
    state: State<'_, DbState>,
    user_id: i64,
    input: RecordSpendInput,
) -> Result<AdvanceTransaction, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let (a_balance, a_status, a_gl): (i64, String, String) = tx.query_row(
        "SELECT balance_milli, status, advance_gl_account_code FROM operating_advances WHERE id = ?1",
        params![input.advance_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    if a_status != "disbursed" && a_status != "partially_spent" {
        return Err(AppError::validation("يجب صرف السلفة قبل الصرف منها"));
    }
    if a_balance < input.amount_milli {
        return Err(AppError::validation("رصيد السلفة غير كافٍ"));
    }
    let new_balance = a_balance - input.amount_milli;
    let acct_code = input.account_code.clone().unwrap_or_else(|| a_gl.clone());
    let receipt_no = format!("RCP-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
    tx.execute(
        "INSERT INTO advance_transactions (advance_id,ts,ttype,amount_milli,balance_after_milli,account_code,category,vendor_name,invoice_no,invoice_date,reference,notes,created_by)
         VALUES (?1,datetime('now'),'spend',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![input.advance_id, input.amount_milli, new_balance, acct_code,
                input.category, input.vendor_name, input.invoice_no, input.invoice_date,
                input.reference, input.notes, user_id.to_string()],
    )?;
    let txn_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO advance_receipts (advance_id,transaction_id,receipt_no,date,vendor_name,amount_milli,vat_milli,net_milli,category,account_code,description,attachment_ids,status,created_by,created_at)
         VALUES (?1,?2,?3,datetime('now'),?4,?5,0,?6,?7,?8,?9,?10,'submitted',?11,datetime('now'))",
        params![input.advance_id, txn_id, receipt_no, input.vendor_name,
                input.amount_milli, input.amount_milli, input.category, acct_code,
                input.description, input.attachment_ids, user_id.to_string()],
    )?;
    tx.execute(
        "UPDATE operating_advances SET total_spent_milli = total_spent_milli + ?1,
         balance_milli = ?2, status = CASE WHEN ?2 = 0 THEN 'reconciled' ELSE 'partially_spent' END,
         updated_at = datetime('now') WHERE id = ?3",
        params![input.amount_milli, new_balance, input.advance_id],
    )?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "advance_spend", "advance_transactions", Some(txn_id), None, None, None);
    Ok(AdvanceTransaction {
        id: txn_id, advance_id: input.advance_id,
        ts: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ttype: "spend".to_string(), amount_milli: input.amount_milli,
        balance_after_milli: new_balance, account_code: Some(acct_code),
        category: input.category, vendor_name: input.vendor_name,
        invoice_no: input.invoice_no, invoice_date: input.invoice_date,
        reference: input.reference, notes: input.notes,
        attachment_ids: input.attachment_ids, journal_id: None,
        created_by: user_id.to_string(),
    })
}

#[tauri::command]
pub fn submit_receipt(
    state: State<'_, DbState>,
    input: SubmitReceiptInput,
) -> Result<AdvanceReceipt, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "INSERT INTO advance_receipts (advance_id,transaction_id,receipt_no,date,vendor_name,amount_milli,vat_milli,net_milli,category,account_code,description,attachment_ids,status,created_by,created_at)
         VALUES (?1,NULL,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'submitted',?12,datetime('now'))"
    )?;
    stmt.execute(params![input.advance_id, input.receipt_no, input.date, input.vendor_name,
        input.amount_milli, input.vat_milli, input.net_milli, input.category,
        input.account_code, input.description, input.attachment_ids, input.created_by])?;
    let id = conn.last_insert_rowid();
    let result = conn.query_row(
        "SELECT id, advance_id, transaction_id, receipt_no, date, vendor_name, amount_milli,
         vat_milli, net_milli, category, account_code, description, attachment_ids, status,
         approved_by, approved_at, journal_id, created_by, created_at
         FROM advance_receipts WHERE id = ?1",
        params![id],
        |row| Ok(AdvanceReceipt {
            id: row.get(0)?, advance_id: row.get(1)?, transaction_id: row.get(2)?,
            receipt_no: row.get(3)?, date: row.get(4)?, vendor_name: row.get(5)?,
            amount_milli: row.get(6)?, vat_milli: row.get(7)?, net_milli: row.get(8)?,
            category: row.get(9)?, account_code: row.get(10)?, description: row.get(11)?,
            attachment_ids: row.get(12)?, status: row.get(13)?, approved_by: row.get(14)?,
            approved_at: row.get(15)?, journal_id: row.get(16)?, created_by: row.get(17)?,
            created_at: row.get(18)?,
        }),
    )?;
    let _ = rbac::log_audit(&conn, None, None, "submit_receipt", "advance_receipts", Some(id), None, None, None);
    Ok(result)
}

#[tauri::command]
pub fn approve_receipt(
    state: State<'_, DbState>,
    user_id: i64,
    receipt_id: i64,
) -> Result<AdvanceReceipt, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let (r_advance_id, r_net): (i64, i64) = tx.query_row(
        "SELECT advance_id, net_milli FROM advance_receipts WHERE id = ?1 AND status = 'submitted'",
        params![receipt_id], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (expense_code, a_gl): (String, String) = tx.query_row(
        "SELECT COALESCE(ar.account_code, oa.default_expense_account_code), oa.advance_gl_account_code FROM advance_receipts ar
         JOIN operating_advances oa ON oa.id = ar.advance_id WHERE ar.id = ?1",
        params![receipt_id], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let je_no = generate_journal_no(&tx)?;
    tx.execute("INSERT INTO journal_entries (entry_no,date,memo,ref_type,ref_id,created_by) VALUES (?1,date('now'),?2,'advance_receipt',?3,?4)",
        params![je_no, format!("Receipt {} approved", receipt_id), receipt_id, user_id])?;
    let je_id = tx.last_insert_rowid();
    tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,?2,?3,0,?4)",
        params![je_id, expense_code, r_net, format!("Expense receipt {} approved", receipt_id)])?;
    tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,?2,0,?3,?4)",
        params![je_id, a_gl, r_net, format!("Advance account reduced for receipt {}", receipt_id)])?;
    tx.execute("UPDATE advance_receipts SET status = 'approved', approved_by = ?1, approved_at = datetime('now'), journal_id = ?2 WHERE id = ?3",
        params![user_id, je_id, receipt_id])?;
    tx.execute("UPDATE operating_advances SET total_spent_milli = total_spent_milli + ?1, balance_milli = balance_milli - ?1, updated_at = datetime('now') WHERE id = ?2",
        params![r_net, r_advance_id])?;
    tx.execute("UPDATE operating_advances SET status = CASE WHEN balance_milli = 0 THEN 'reconciled' ELSE 'partially_spent' END WHERE id = ?1",
        params![r_advance_id])?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, None, None, "approve_receipt", "advance_receipts", Some(receipt_id), None, None, None);
    drop(conn);
    conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, advance_id, transaction_id, receipt_no, date, vendor_name, amount_milli,
         vat_milli, net_milli, category, account_code, description, attachment_ids, status,
         approved_by, approved_at, journal_id, created_by, created_at
         FROM advance_receipts WHERE id = ?1",
        params![receipt_id],
        |row| Ok(AdvanceReceipt {
            id: row.get(0)?, advance_id: row.get(1)?, transaction_id: row.get(2)?,
            receipt_no: row.get(3)?, date: row.get(4)?, vendor_name: row.get(5)?,
            amount_milli: row.get(6)?, vat_milli: row.get(7)?, net_milli: row.get(8)?,
            category: row.get(9)?, account_code: row.get(10)?, description: row.get(11)?,
            attachment_ids: row.get(12)?, status: row.get(13)?, approved_by: row.get(14)?,
            approved_at: row.get(15)?, journal_id: row.get(16)?, created_by: row.get(17)?,
            created_at: row.get(18)?,
        }),
    ).map_err(AppError::from)
}

#[tauri::command]
pub fn return_advance(
    state: State<'_, DbState>,
    user_id: i64,
    input: ReturnAdvanceInput,
) -> Result<AdvanceTransaction, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let advance_balance: i64 = tx.query_row(
        "SELECT balance_milli FROM operating_advances WHERE id = ?1",
        params![input.advance_id], |r| r.get(0),
    )?;
    if advance_balance < input.amount_milli {
        return Err(AppError::validation("قيمة الرد تتجاوز رصيد السلفة"));
    }
    let a_gl: String = tx.query_row(
        "SELECT advance_gl_account_code FROM operating_advances WHERE id = ?1",
        params![input.advance_id], |r| r.get(0),
    )?;
    let je_no = generate_journal_no(&tx)?;
    tx.execute("INSERT INTO journal_entries (entry_no,date,memo,ref_type,ref_id,created_by) VALUES (?1,date('now'),?2,'advance_return',?3,?4)",
        params![je_no, format!("Employee returned advance {}", input.advance_id), input.advance_id, user_id.to_string()])?;
    let je_id = tx.last_insert_rowid();
    tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,?2,?3,0,?4)",
        params![je_id, input.source_account_code, input.amount_milli, format!("Cash received back {}", input.advance_id)])?;
    tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,?2,0,?3,?4)",
        params![je_id, a_gl, input.amount_milli, format!("Advance account reduced {}", input.advance_id)])?;
    let new_balance = advance_balance - input.amount_milli;
    tx.execute(
        "UPDATE operating_advances SET total_returned_milli = total_returned_milli + ?1, balance_milli = ?2,
         status = CASE WHEN ?2 = 0 THEN 'closed' ELSE 'partially_spent' END,
         actual_return_date = CASE WHEN ?2 = 0 THEN date('now') ELSE actual_return_date END,
         updated_at = datetime('now') WHERE id = ?3",
        params![input.amount_milli, new_balance, input.advance_id],
    )?;
    tx.execute(
        "INSERT INTO advance_transactions (advance_id,ts,ttype,amount_milli,balance_after_milli,account_code,reference,notes,journal_id,created_by)
         VALUES (?1,datetime('now'),'return',?2,?3,?4,?5,?6,?7,?8)",
        params![input.advance_id, input.amount_milli, new_balance, input.source_account_code,
                &format!("Return {}", input.advance_id), input.notes, je_id, user_id.to_string()],
    )?;
    let txn_id = tx.last_insert_rowid();
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "return_advance", "advance_transactions", Some(txn_id), None, None, None);
    drop(conn);
    conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, advance_id, ts, ttype, amount_milli, balance_after_milli, account_code,
         category, vendor_name, invoice_no, invoice_date, reference, notes, attachment_ids, journal_id, created_by
         FROM advance_transactions WHERE id = ?1",
        params![txn_id],
        |row| Ok(AdvanceTransaction {
            id: row.get(0)?, advance_id: row.get(1)?, ts: row.get(2)?, ttype: row.get(3)?,
            amount_milli: row.get(4)?, balance_after_milli: row.get(5)?, account_code: row.get(6)?,
            category: row.get(7)?, vendor_name: row.get(8)?, invoice_no: row.get(9)?,
            invoice_date: row.get(10)?, reference: row.get(11)?, notes: row.get(12)?,
            attachment_ids: row.get(13)?, journal_id: row.get(14)?, created_by: row.get(15)?,
        }),
    ).map_err(AppError::from)
}

#[tauri::command]
pub fn reconcile_advance(
    state: State<'_, DbState>,
    user_id: i64,
    input: ReconcileAdvanceInput,
) -> Result<OperatingAdvance, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let (_a_balance, a_spent): (i64, i64) = tx.query_row(
        "SELECT balance_milli, total_spent_milli FROM operating_advances WHERE id = ?1",
        params![input.advance_id], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let variance = input.physical_amount_milli - a_spent;
    if variance != 0 {
        let je_no = generate_journal_no(&tx)?;
        tx.execute("INSERT INTO journal_entries (entry_no,date,memo,ref_type,ref_id,created_by) VALUES (?1,date('now'),?2,'advance_reconciliation',?3,?4)",
            params![je_no, format!("Reconciliation variance {} OMR", variance as f64 / 1000.0), input.advance_id, user_id.to_string()])?;
        let je_id = tx.last_insert_rowid();
        if variance > 0 {
            tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,'2000',?2,0,?3)",
                params![je_id, variance, "Cash surplus detected"])?;
            tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,'5100',0,?2,?3)",
                params![je_id, variance, "Credit surplus account"])?;
        } else {
            let abs_v = -variance;
            tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,'2000',0,?2,?3)",
                params![je_id, abs_v, format!("Cash shortage {} OMR", abs_v as f64 / 1000.0)])?;
            tx.execute("INSERT INTO journal_entry_lines (entry_id,account_code,debit_milli,credit_milli,memo) VALUES (?1,'5200',?2,0,?3)",
                params![je_id, abs_v, "Debit shortage overhead"])?;
        }
    }
    tx.execute(
        "UPDATE operating_advances SET status = 'reconciled', actual_return_date = date('now'), updated_at = datetime('now'), notes = ?1 WHERE id = ?2",
        params![input.notes, input.advance_id],
    )?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "reconcile_advance", "operating_advances", Some(input.advance_id), None, None, None);
    drop(conn);
    conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, advance_no, date, employee_id, employee_name, department, purpose, description,
         amount_milli, currency, exchange_rate, status, approval_status, approved_by, approved_at,
         disbursed_by, disbursed_at, source_account_code, advance_gl_account_code,
         default_expense_account_code, expected_return_date, actual_return_date,
         total_spent_milli, total_returned_milli, balance_milli, notes, created_by, created_at, updated_at
         FROM operating_advances WHERE id = ?1",
        params![input.advance_id],
        row_to_advance,
    ).map_err(AppError::from)
}

#[tauri::command]
pub fn cancel_advance(
    state: State<'_, DbState>,
    user_id: i64,
    advance_id: i64,
) -> Result<OperatingAdvance, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;
    let current_status: String = tx.query_row(
        "SELECT status FROM operating_advances WHERE id = ?1",
        params![advance_id], |r| r.get(0),
    )?;
    if current_status != "draft" && current_status != "approved" && current_status != "disbursed" {
        return Err(AppError::validation("لا يمكن إلغاء السلفة في الحالة الحالية"));
    }
    if current_status == "disbursed" {
        let balance: i64 = tx.query_row(
            "SELECT balance_milli FROM operating_advances WHERE id = ?1",
            params![advance_id], |r| r.get(0),
        )?;
        if balance > 0 {
            return Err(AppError::validation("لا يمكن إلغاء السلفة مع وجود رصيد مستحق؛ أعد الفائض أولاً"));
        }
    }
    tx.execute(
        "UPDATE operating_advances SET status = 'cancelled', approval_status = 'rejected', updated_at = datetime('now') WHERE id = ?1",
        params![advance_id],
    )?;
    tx.commit()?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "cancel_advance", "operating_advances", Some(advance_id), None, None, None);
    drop(conn);
    conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, advance_no, date, employee_id, employee_name, department, purpose, description,
         amount_milli, currency, exchange_rate, status, approval_status, approved_by, approved_at,
         disbursed_by, disbursed_at, source_account_code, advance_gl_account_code,
         default_expense_account_code, expected_return_date, actual_return_date,
         total_spent_milli, total_returned_milli, balance_milli, notes, created_by, created_at, updated_at
         FROM operating_advances WHERE id = ?1",
        params![advance_id],
        row_to_advance,
    ).map_err(AppError::from)
}

#[tauri::command]
pub fn get_advance_transactions(
    state: State<'_, DbState>,
    advance_id: i64,
) -> Result<Vec<AdvanceTransaction>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, advance_id, ts, ttype, amount_milli, balance_after_milli, account_code,
         category, vendor_name, invoice_no, invoice_date, reference, notes, attachment_ids, journal_id, created_by
         FROM advance_transactions WHERE advance_id = ?1 ORDER BY ts ASC",
    )?;
    let rows = stmt.query_map(params![advance_id], |row| Ok(AdvanceTransaction {
        id: row.get(0)?, advance_id: row.get(1)?, ts: row.get(2)?, ttype: row.get(3)?,
        amount_milli: row.get(4)?, balance_after_milli: row.get(5)?, account_code: row.get(6)?,
        category: row.get(7)?, vendor_name: row.get(8)?, invoice_no: row.get(9)?,
        invoice_date: row.get(10)?, reference: row.get(11)?, notes: row.get(12)?,
        attachment_ids: row.get(13)?, journal_id: row.get(14)?, created_by: row.get(15)?,
    }))?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

#[tauri::command]
pub fn get_advance_receipts(
    state: State<'_, DbState>,
    advance_id: i64,
) -> Result<Vec<AdvanceReceipt>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, advance_id, transaction_id, receipt_no, date, vendor_name, amount_milli,
         vat_milli, net_milli, category, account_code, description, attachment_ids, status,
         approved_by, approved_at, journal_id, created_by, created_at
         FROM advance_receipts WHERE advance_id = ?1 ORDER BY date ASC, id ASC"
    )?;
    let rows = stmt.query_map(params![advance_id], |row| Ok(AdvanceReceipt {
        id: row.get(0)?, advance_id: row.get(1)?, transaction_id: row.get(2)?,
        receipt_no: row.get(3)?, date: row.get(4)?, vendor_name: row.get(5)?,
        amount_milli: row.get(6)?, vat_milli: row.get(7)?, net_milli: row.get(8)?,
        category: row.get(9)?, account_code: row.get(10)?, description: row.get(11)?,
        attachment_ids: row.get(12)?, status: row.get(13)?, approved_by: row.get(14)?,
        approved_at: row.get(15)?, journal_id: row.get(16)?, created_by: row.get(17)?,
        created_at: row.get(18)?,
    }))?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

#[tauri::command]
pub fn get_advance_summary(state: State<'_, DbState>) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    let total_advances: i64 = conn.query_row("SELECT COALESCE(SUM(amount_milli),0) FROM operating_advances", [], |r| r.get(0))?;
    let total_disbursed: i64 = conn.query_row("SELECT COALESCE(SUM(amount_milli),0) FROM operating_advances WHERE status IN ('disbursed','partially_spent','reconciled','closed')", [], |r| r.get(0))?;
    let total_spent: i64 = conn.query_row("SELECT COALESCE(SUM(total_spent_milli),0) FROM operating_advances", [], |r| r.get(0))?;
    let total_returned: i64 = conn.query_row("SELECT COALESCE(SUM(total_returned_milli),0) FROM operating_advances", [], |r| r.get(0))?;
    let open_count: i64 = conn.query_row("SELECT COUNT(*) FROM operating_advances WHERE status IN ('disbursed','partially_spent')", [], |r| r.get(0))?;
    let pending_approval_count: i64 = conn.query_row("SELECT COUNT(*) FROM operating_advances WHERE approval_status = 'pending'", [], |r| r.get(0))?;
    let pending_receipt_count: i64 = conn.query_row("SELECT COUNT(*) FROM advance_receipts WHERE status = 'submitted'", [], |r| r.get(0))?;
    Ok(serde_json::json!({
        "total_advances_milli": total_advances,
        "total_disbursed_milli": total_disbursed,
        "total_spent_milli": total_spent,
        "total_returned_milli": total_returned,
        "outstanding_balance_milli": total_disbursed - total_spent - total_returned,
        "open_advance_count": open_count,
        "pending_approval_count": pending_approval_count,
        "pending_receipt_count": pending_receipt_count,
    }))
}

#[tauri::command]
pub fn list_pending_receipts(state: State<'_, DbState>) -> Result<Vec<AdvanceReceipt>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, advance_id, transaction_id, receipt_no, date, vendor_name, amount_milli,
         vat_milli, net_milli, category, account_code, description, attachment_ids, status,
         approved_by, approved_at, journal_id, created_by, created_at
         FROM advance_receipts WHERE status = 'submitted' ORDER BY date ASC"
    )?;
    let rows = stmt.query_map([], |row| Ok(AdvanceReceipt {
        id: row.get(0)?, advance_id: row.get(1)?, transaction_id: row.get(2)?,
        receipt_no: row.get(3)?, date: row.get(4)?, vendor_name: row.get(5)?,
        amount_milli: row.get(6)?, vat_milli: row.get(7)?, net_milli: row.get(8)?,
        category: row.get(9)?, account_code: row.get(10)?, description: row.get(11)?,
        attachment_ids: row.get(12)?, status: row.get(13)?, approved_by: row.get(14)?,
        approved_at: row.get(15)?, journal_id: row.get(16)?, created_by: row.get(17)?,
        created_at: row.get(18)?,
    }))?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}