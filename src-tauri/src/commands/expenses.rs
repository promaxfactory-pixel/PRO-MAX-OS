use crate::commands::rbac;
use crate::db::DbState;
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
pub fn list_expenses(state: State<'_, DbState>) -> Result<Vec<Expense>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("{} ORDER BY e.id DESC", EXPENSE_SELECT);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
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
    }).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_expense(input: CreateExpenseInput, state: State<'_, DbState>) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let year: String = conn
        .query_row("SELECT substr(?1, 1, 4)", [&input.date], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let next_num: i64 = conn
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type=? AND year=?",
            ["EXP", &year],
            |row| row.get(0),
        )
        .unwrap_or(1);
    conn.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES(?,?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        ["EXP", &year, &next_num.to_string()],
    )
    .map_err(|e| e.to_string())?;
    let exp_no = format!("EXP-{}-{:04}", year, next_num);

    let source = input.paid_from_source.unwrap_or_else(|| "company".to_string());
    let mut reimbursement = "none".to_string();
    let mut petty_id_val = input.petty_id;

    // If paid from custody, create custody transaction and deduct balance
    if source == "custody" && petty_id_val.is_some() {
        let pid = petty_id_val.unwrap();
        let current_balance: i64 = conn.query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id = ?1",
            [pid],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        if current_balance < input.amount_milli {
            return Err("رصيد العهده غير كافٍ".to_string());
        }
        let new_balance = current_balance - input.amount_milli;
        conn.execute("UPDATE petty_cash_accounts SET balance_milli = ?1 WHERE id = ?2", [new_balance, pid]).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category, reference, notes)
             VALUES (datetime('now'), ?1, 'Spend', 0, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![pid, input.amount_milli, new_balance, input.category, input.reference, input.notes],
        ).map_err(|e| e.to_string())?;
        let _ = rbac::log_audit(&conn, None, None, "custody_spend_for_expense", "petty_cash_accounts", Some(pid), None, Some(&format!("expense amount:{}", input.amount_milli)), None);
        petty_id_val = Some(pid);
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
    ).map_err(|e| e.to_string())?;
    let exp_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_expense", "expenses", Some(exp_id), None, Some(&input.notes.unwrap_or_default()), None);
    Ok(exp_id)
}

#[tauri::command]
pub fn reimburse_expense(state: State<'_, DbState>, expense_id: i64, reimbursed_by: String) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE expenses SET reimbursement_status='reimbursed', reimbursement_date=date('now'), reimbursed_by=?1 WHERE id=?2 AND reimbursement_status='pending'",
        rusqlite::params![reimbursed_by, expense_id],
    ).map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "reimburse_expense", "expenses", Some(expense_id), None, Some(&reimbursed_by), None);
    Ok("تم رد المبلغ بنجاح".to_string())
}

#[tauri::command]
pub fn approve_expense(state: State<'_, DbState>, expense_id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE expenses SET approval_status='approved' WHERE id=?1",
        [expense_id],
    ).map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "approve_expense", "expenses", Some(expense_id), None, None, None);
    Ok("تم اعتماد المصروف".to_string())
}

#[tauri::command]
pub fn list_employees_for_select(state: State<'_, DbState>) -> Result<Vec<EmployeeSelect>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name, code FROM employees WHERE active=1 ORDER BY name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(EmployeeSelect { id: row.get(0)?, name: row.get(1)?, code: row.get(2)? })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custody_accounts_for_select(state: State<'_, DbState>) -> Result<Vec<EmployeeSelect>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name, code FROM petty_cash_accounts WHERE active=1 ORDER BY name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(EmployeeSelect { id: row.get(0)?, name: row.get(1)?, code: row.get(2)? })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
