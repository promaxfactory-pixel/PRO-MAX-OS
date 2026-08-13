use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Expense {
    pub id: i64,
    pub exp_no: Option<String>,
    pub date: String,
    pub category: Option<String>,
    pub account_code: Option<String>,
    pub amount_milli: i64,
    pub vat_milli: i64,
    pub method: Option<String>,
    pub vendor: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub approval_status: Option<String>,
    pub paid_by_employee_id: Option<i64>,
    pub paid_by_name: Option<String>,
    pub paid_from_source: Option<String>,
    pub petty_id: Option<i64>,
    pub petty_name: Option<String>,
    pub custody_txn_id: Option<i64>,
    pub reimbursement_status: Option<String>,
    pub reimbursement_date: Option<String>,
    pub reimbursed_by: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseInput {
    pub date: String,
    pub category: Option<String>,
    pub account_code: Option<String>,
    pub amount_milli: i64,
    pub vat_milli: Option<i64>,
    pub method: Option<String>,
    pub vendor: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub paid_by_employee_id: Option<i64>,
    pub paid_from_source: Option<String>,
    pub petty_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct EmployeeSelect {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
}

const EXPENSE_SELECT: &str = "SELECT e.id, e.exp_no, e.date, e.category, e.account_code, e.amount_milli, e.vat_milli,
    e.method, e.vendor, e.reference, e.notes, e.approval_status,
    e.paid_by_employee_id, emp.name as paid_by_name, e.paid_from_source, e.petty_id,
    pca.name as petty_name, e.custody_txn_id, e.reimbursement_status, e.reimbursement_date, e.reimbursed_by,
    e.created_by, e.created_at
FROM expenses e
LEFT JOIN employees emp ON e.paid_by_employee_id = emp.id
LEFT JOIN petty_cash_accounts pca ON e.petty_id = pca.id";

#[tauri::command]
pub fn list_expenses(state: State<'_, DbState>) -> Result<Vec<Expense>, AppError> {
    let conn = state.0.lock()?;
    let sql = format!("{} ORDER BY e.id DESC", EXPENSE_SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(Expense {
            id: row.get(0)?,
            exp_no: row.get(1)?,
            date: row.get(2)?,
            category: row.get(3)?,
            account_code: row.get(4)?,
            amount_milli: row.get(5)?,
            vat_milli: row.get(6)?,
            method: row.get(7)?,
            vendor: row.get(8)?,
            reference: row.get(9)?,
            notes: row.get(10)?,
            approval_status: row.get(11)?,
            paid_by_employee_id: row.get(12)?,
            paid_by_name: row.get(13)?,
            paid_from_source: row.get(14)?,
            petty_id: row.get(15)?,
            petty_name: row.get(16)?,
            custody_txn_id: row.get(17)?,
            reimbursement_status: row.get(18)?,
            reimbursement_date: row.get(19)?,
            reimbursed_by: row.get(20)?,
            created_by: row.get(21)?,
            created_at: row.get(22)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_expense(input: CreateExpenseInput, state: State<'_, DbState>, user_id: i64) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let year: String = conn
        .query_row("SELECT substr(?1, 1, 4)", [&input.date], |row| row.get(0))?;
    let next_num = next_sequence(&conn, "EXP", &year)?;
    let exp_no = format!("EXP-{}-{:04}", year, next_num);

    let source = input.paid_from_source.unwrap_or_else(|| "company".to_string());
    let mut reimbursement = "none".to_string();
    let petty_id_val = input.petty_id;

    // If paid from custody, create custody transaction and deduct balance
    if source == "custody" {
        if let Some(pid) = petty_id_val {
        let current_balance: i64 = conn.query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id = ?1",
            [pid],
            |row| row.get(0),
        )?;
        if current_balance < input.amount_milli {
            return Err(AppError::validation("رصيد العهده غير كافٍ"));
        }
        let new_balance = current_balance - input.amount_milli;
        conn.execute("UPDATE petty_cash_accounts SET balance_milli = ?1 WHERE id = ?2", [new_balance, pid])?;
        conn.execute(
            "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category, reference, notes)
             VALUES (datetime('now'), ?1, 'Spend', 0, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![pid, input.amount_milli, new_balance, input.category, input.reference, input.notes],
        )?;
        let _ = rbac::log_audit(&conn, None, None, "custody_spend_for_expense", "petty_cash_accounts", Some(pid), None, Some(&format!("expense amount:{}", input.amount_milli)), None);
        }
    }

    // If paid personally by employee, mark as needing reimbursement
    if source == "personal" {
        reimbursement = "pending".to_string();
    }

    conn.execute(
        "INSERT INTO expenses(exp_no, date, category, account_code, amount_milli, vat_milli, method, vendor, reference, notes, approval_status,
         paid_by_employee_id, paid_from_source, petty_id, reimbursement_status)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12, ?13, ?14)",
        rusqlite::params![
            exp_no, input.date, input.category, input.account_code, input.amount_milli,
            input.vat_milli.unwrap_or(0), input.method, input.vendor, input.reference, input.notes,
            input.paid_by_employee_id, source, petty_id_val, reimbursement,
        ],
    )?;
    let exp_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_expense", "expenses", Some(exp_id), None, Some(&input.notes.unwrap_or_default()), None);
    Ok(exp_id)
}

#[tauri::command]
pub fn reimburse_expense(state: State<'_, DbState>, user_id: i64, expense_id: i64, reimbursed_by: String) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;

    let current_status: Option<String> = tx
        .query_row("SELECT reimbursement_status FROM expenses WHERE id=?1", [expense_id], |r| r.get(0))
        .unwrap_or_default();
    if current_status.as_deref() == Some("reimbursed") {
        return Ok("تم رد المبلغ سلفاً".to_string());
    }
    if current_status.as_deref() != Some("pending") {
        return Err(AppError::validation("المصروف غير مؤهل للرد"));
    }

    tx.execute(
        "UPDATE expenses SET reimbursement_status='reimbursed', reimbursement_date=date('now'), reimbursed_by=?1 WHERE id=?2",
        rusqlite::params![reimbursed_by, expense_id],
    )?;

    // Personal expenses have no journal entry yet (approval only books company-paid
    // ones). Reimbursement is when the money actually leaves the company, so book
    // the expense against the payment account now.
    let journal_id: Option<i64> = tx
        .query_row("SELECT journal_id FROM expenses WHERE id=?1", [expense_id], |r| r.get(0))
        .unwrap_or(None);
    if journal_id.is_none() {
        let (amount_milli, vat_milli, account_code, method, date, exp_no): (i64, i64, Option<String>, Option<String>, String, Option<String>) = tx
            .query_row(
                "SELECT amount_milli, vat_milli, account_code, method, date, exp_no FROM expenses WHERE id=?1",
                [expense_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .map_err(|_| AppError::not_found("المصروف غير موجود"))?;
        let expense_account = account_code
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| "5200".to_string());
        let cash_account = crate::commands::accounting::resolve_cash_account(&tx, None, &method.unwrap_or_default())?;
        let total = amount_milli + vat_milli;
        let lines: Vec<(String, i64, i64, Option<String>)> = vec![
            (expense_account, total, 0, Some("مصروف".to_string())),
            (cash_account, 0, total, None),
        ];
        let jid = crate::commands::accounting::post_to_journal(
            &tx,
            "expense_reimbursement",
            expense_id,
            &date,
            &format!("رد مصروف {}", exp_no.unwrap_or_default()),
            &lines,
            "system",
        )?;
        tx.execute(
            "UPDATE expenses SET journal_id=?1 WHERE id=?2",
            rusqlite::params![jid, expense_id],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "reimburse_expense", "expenses", Some(expense_id), None, Some(&reimbursed_by), None);
    tx.commit()?;
    Ok("تم رد المبلغ بنجاح".to_string())
}

#[tauri::command]
pub fn approve_expense(state: State<'_, DbState>, user_id: i64, expense_id: i64) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;

    let current_status: Option<String> = tx
        .query_row("SELECT approval_status FROM expenses WHERE id=?1", [expense_id], |r| r.get(0))
        .unwrap_or_default();
    if current_status.as_deref() == Some("approved") {
        return Ok("تم اعتماد المصروف مسبقاً".to_string());
    }

    tx.execute(
        "UPDATE expenses SET approval_status='approved' WHERE id=?1",
        [expense_id],
    )?;

    let journal_id: Option<i64> = tx
        .query_row("SELECT journal_id FROM expenses WHERE id=?1", [expense_id], |r| r.get(0))
        .unwrap_or(None);
    let source: Option<String> = tx
        .query_row("SELECT paid_from_source FROM expenses WHERE id=?1", [expense_id], |r| r.get(0))
        .unwrap_or_default();

    if journal_id.is_none() && source.as_deref() == Some("company") {
        let (amount_milli, vat_milli, account_code, method, date, exp_no): (i64, i64, Option<String>, Option<String>, String, Option<String>) = tx
            .query_row(
                "SELECT amount_milli, vat_milli, account_code, method, date, exp_no FROM expenses WHERE id=?1",
                [expense_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .map_err(|_| AppError::not_found("المصروف غير موجود"))?;
        let expense_account = account_code
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| "5200".to_string());
        let cash_account = crate::commands::accounting::resolve_cash_account(&tx, None, &method.unwrap_or_default())?;
        let total = amount_milli + vat_milli;
        let lines: Vec<(String, i64, i64, Option<String>)> = vec![
            (expense_account, total, 0, Some("مصروف".to_string())),
            (cash_account, 0, total, None),
        ];
        let jid = crate::commands::accounting::post_to_journal(
            &tx,
            "expense",
            expense_id,
            &date,
            &format!("مصروف {}", exp_no.unwrap_or_default()),
            &lines,
            "system",
        )?;
        tx.execute(
            "UPDATE expenses SET journal_id=?1 WHERE id=?2",
            rusqlite::params![jid, expense_id],
        )?;
    }

    let _ = rbac::log_audit(&tx, Some(user_id), None, "approve_expense", "expenses", Some(expense_id), None, None, None);
    tx.commit()?;
    Ok("تم اعتماد المصروف".to_string())
}

#[tauri::command]
pub fn list_employees_for_select(state: State<'_, DbState>) -> Result<Vec<EmployeeSelect>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, code FROM employees WHERE active=1 ORDER BY name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(EmployeeSelect { id: row.get(0)?, name: row.get(1)?, code: row.get(2)? })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn get_custody_accounts_for_select(state: State<'_, DbState>) -> Result<Vec<EmployeeSelect>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, code FROM petty_cash_accounts WHERE active=1 ORDER BY name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(EmployeeSelect { id: row.get(0)?, name: row.get(1)?, code: row.get(2)? })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}
