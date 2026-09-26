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
    pub source_account_code: Option<String>,
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
    pub source_account_code: Option<String>,
    pub cashbank_id: Option<i64>,
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
    e.paid_by_employee_id, emp.name as paid_by_name, e.paid_from_source, e.source_account_code, e.petty_id,
    pca.name as petty_name, e.custody_txn_id, e.reimbursement_status, e.reimbursement_date, e.reimbursed_by,
    e.created_by, e.created_at
FROM expenses e
LEFT JOIN employees emp ON e.paid_by_employee_id = emp.id
LEFT JOIN petty_cash_accounts pca ON e.petty_id = pca.id";

#[tauri::command]
pub fn list_expenses(state: State<'_, DbState>) -> Result<Vec<Expense>, AppError> {
    let conn = state.0.lock()?;
    let sql = format!("{} ORDER BY e.id DESC", EXPENSE_SELECT);
    expense_rows_from_sql(&conn, &sql, &[])
}

/// Shared row mapper for the EXPENSE_SELECT projection.
fn expense_rows_from_sql(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::types::ToSql],
) -> Result<Vec<Expense>, AppError> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params, |row| {
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
            source_account_code: row.get(15)?,
            petty_id: row.get(16)?,
            petty_name: row.get(17)?,
            custody_txn_id: row.get(18)?,
            reimbursement_status: row.get(19)?,
            reimbursement_date: row.get(20)?,
            reimbursed_by: row.get(21)?,
            created_by: row.get(22)?,
            created_at: row.get(23)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Expenses whose date falls inside [date_from, date_to], newest first.
/// An optional `approval_status` narrows the rows (e.g. "approved").
pub(crate) fn expense_rows_in_range(
    conn: &rusqlite::Connection,
    date_from: &str,
    date_to: &str,
    approval_status: Option<&str>,
) -> Result<Vec<Expense>, AppError> {
    let sql = format!("{} WHERE e.date >= ?1 AND e.date <= ?2 ORDER BY e.date DESC, e.id DESC", EXPENSE_SELECT);
    let mut rows = expense_rows_from_sql(
        conn,
        &sql,
        &[&date_from as &dyn rusqlite::types::ToSql, &date_to as &dyn rusqlite::types::ToSql],
    )?;
    if let Some(s) = approval_status.filter(|s| !s.is_empty()) {
        rows.retain(|e| e.approval_status.as_deref() == Some(s));
    }
    Ok(rows)
}

#[tauri::command]
pub fn create_expense(input: CreateExpenseInput, state: State<'_, DbState>, user_id: i64) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("المبلغ يجب أن يكون أكبر من صفر"));
    }
    if input.vat_milli.unwrap_or(0) < 0 {
        return Err(AppError::validation("قيمة الضريبة لا يمكن أن تكون سالبة"));
    }
    let tx = conn.transaction()?;
    let year: String = tx
        .query_row("SELECT substr(?1, 1, 4)", [&input.date], |row| row.get(0))?;
    let next_num = next_sequence(&tx, "EXP", &year)?;
    let exp_no = format!("EXP-{}-{:04}", year, next_num);

    let source = input.paid_from_source.unwrap_or_else(|| "company".to_string()).to_lowercase();
    let mut reimbursement = "none".to_string();
    let petty_id_val = input.petty_id;
    let total_cash_milli = input.amount_milli
        .checked_add(input.vat_milli.unwrap_or(0))
        .ok_or_else(|| AppError::validation("إجمالي المصروف يتجاوز الحد المسموح"))?;
    let mut custody_txn_id: Option<i64> = None;

    if source == "personal" && input.paid_by_employee_id.is_none() {
        return Err(AppError::validation("حدد الموظف الذي دفع المصروف من ماله"));
    }

    // Resolve the balance-sheet account that actually funded the expense.
    // This separates the economic expense from who physically paid it.
    let source_account = if let Some(code) = input.source_account_code.clone().filter(|x| !x.trim().is_empty()) {
        let exists: i64 = tx.query_row("SELECT COUNT(*) FROM accounts WHERE code=?1", [&code], |r| r.get(0)).unwrap_or(0);
        if exists == 0 {
            return Err(AppError::validation("حساب مصدر الدفع غير موجود"));
        }
        code
    } else {
        match source.as_str() {
            "custody" => {
                let pid = petty_id_val.ok_or_else(|| AppError::validation("حدد حساب العهدة"))?;
                tx.query_row(
                    "SELECT COALESCE(NULLIF(trim(account_code),''),'1110') FROM petty_cash_accounts WHERE id=?1",
                    [pid],
                    |r| r.get::<_, String>(0),
                ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?
            }
            "personal" => "2260".to_string(),
            "owner_saif" => "2310".to_string(),
            "owner_abu_saif" => "2320".to_string(),
            "company" => crate::commands::accounting::resolve_cash_account(
                &tx,
                input.cashbank_id,
                input.method.as_deref().unwrap_or("cash"),
            )?,
            _ => return Err(AppError::validation("مصدر الدفع غير معروف")),
        }
    };

    // Cash leaves the custody subledger immediately when the expense happens.
    if source == "custody" {
        let pid = petty_id_val.ok_or_else(|| AppError::validation("حدد حساب العهدة"))?;
        let current_balance: i64 = tx.query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id = ?1",
            [pid],
            |row| row.get(0),
        )?;
        if current_balance < total_cash_milli {
            return Err(AppError::validation("رصيد العهدة غير كافٍ"));
        }
        let new_balance = current_balance - total_cash_milli;
        tx.execute("UPDATE petty_cash_accounts SET balance_milli = ?1 WHERE id = ?2", [new_balance, pid])?;
        tx.execute(
            "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category, account_code, reference, notes, user_id)
             VALUES (?1, ?2, 'Spend', 0, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                format!("{} 12:00:00", input.date),
                pid,
                total_cash_milli,
                new_balance,
                input.category,
                input.account_code,
                input.reference,
                input.notes,
                user_id
            ],
        )?;
        custody_txn_id = Some(tx.last_insert_rowid());
        let _ = rbac::log_audit(&tx, Some(user_id), None, "custody_spend_for_expense", "petty_cash_accounts", Some(pid), None, Some(&format!("expense amount:{}", total_cash_milli)), None);
    }

    if source == "personal" {
        reimbursement = "pending".to_string();
    }

    tx.execute(
        "INSERT INTO expenses(exp_no, date, category, account_code, amount_milli, vat_milli, method, cashbank_id, vendor, reference, notes, approval_status,
         paid_by_employee_id, paid_from_source, source_account_code, petty_id, custody_txn_id, reimbursement_status)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'pending', ?12, ?13, ?14, ?15, ?16, ?17)",
        rusqlite::params![
            exp_no, input.date, input.category, input.account_code, input.amount_milli,
            input.vat_milli.unwrap_or(0), input.method, input.cashbank_id, input.vendor, input.reference, input.notes,
            input.paid_by_employee_id, source, source_account, petty_id_val, custody_txn_id, reimbursement,
        ],
    )?;
    let exp_id = tx.last_insert_rowid();
    if let Some(txn_id) = custody_txn_id {
        tx.execute(
            "UPDATE petty_cash_transactions SET expense_id=?1 WHERE id=?2",
            rusqlite::params![exp_id, txn_id],
        )?;
    }
    let _ = rbac::log_audit(&tx, None, None, "create_expense", "expenses", Some(exp_id), None, Some(&input.notes.unwrap_or_default()), None);
    tx.commit()?;
    Ok(exp_id)
}

#[tauri::command]
pub fn reimburse_expense(
    state: State<'_, DbState>,
    user_id: i64,
    expense_id: i64,
    reimbursed_by: String,
    method: Option<String>,
    cashbank_id: Option<i64>,
) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;

    let (current_status, approval_status, source, total, expense_date): (Option<String>, Option<String>, Option<String>, i64, String) = tx
        .query_row(
            "SELECT reimbursement_status, approval_status, paid_from_source,
                    amount_milli + COALESCE(vat_milli,0), date
             FROM expenses WHERE id=?1",
            [expense_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|_| AppError::not_found("المصروف غير موجود"))?;

    if current_status.as_deref() == Some("reimbursed") {
        return Ok("تم رد المبلغ سلفاً".to_string());
    }
    if source.as_deref() != Some("personal") || current_status.as_deref() != Some("pending") {
        return Err(AppError::validation("المصروف غير مؤهل لرد مصروف موظف"));
    }
    if approval_status.as_deref() != Some("approved") {
        return Err(AppError::validation("يجب اعتماد المصروف قبل رد المبلغ للموظف"));
    }

    let pay_method = method.unwrap_or_else(|| "cash".to_string());
    let cash_account = crate::commands::accounting::resolve_cash_account(&tx, cashbank_id, &pay_method)?;
    let lines = vec![
        ("2260".to_string(), total, 0, Some("تسوية مستحق رد مصروف موظف".to_string())),
        (cash_account, 0, total, Some(format!("رد بواسطة {}", reimbursed_by))),
    ];
    let settlement_id = crate::commands::accounting::post_to_journal(
        &tx,
        "expense_reimbursement",
        expense_id,
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        "تسوية رد مصروف موظف",
        &lines,
        &user_id.to_string(),
    )?;

    tx.execute(
        "UPDATE expenses SET reimbursement_status='reimbursed', reimbursement_date=date('now'), reimbursed_by=?1 WHERE id=?2",
        rusqlite::params![reimbursed_by, expense_id],
    )?;

    let _ = rbac::log_audit(
        &tx,
        Some(user_id),
        None,
        "reimburse_expense",
        "expenses",
        Some(expense_id),
        Some(&format!("expense_date={}", expense_date)),
        Some(&format!("settlement_journal={}", settlement_id)),
        None,
    );
    tx.commit()?;
    Ok("تم رد المبلغ وتسوية مستحق الموظف".to_string())
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

    if journal_id.is_none() {
        let (amount_milli, vat_milli, account_code, source_account_code, date, exp_no):
            (i64, i64, Option<String>, Option<String>, String, Option<String>) = tx
            .query_row(
                "SELECT amount_milli, COALESCE(vat_milli,0), account_code, source_account_code, date, exp_no
                 FROM expenses WHERE id=?1",
                [expense_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .map_err(|_| AppError::not_found("المصروف غير موجود"))?;
        let expense_account = account_code
            .filter(|x| !x.trim().is_empty())
            .unwrap_or_else(|| "5200".to_string());
        let source_account = source_account_code
            .filter(|x| !x.trim().is_empty())
            .ok_or_else(|| AppError::validation("حساب مصدر دفع المصروف غير محدد"))?;
        let total = amount_milli
            .checked_add(vat_milli)
            .ok_or_else(|| AppError::validation("إجمالي المصروف يتجاوز الحد المسموح"))?;
        let mut lines: Vec<(String, i64, i64, Option<String>)> = vec![
            (expense_account, amount_milli, 0, Some("صافي المصروف".to_string())),
        ];
        if vat_milli > 0 {
            // 2100 is the net VAT control account: input VAT is a debit and
            // output VAT is a credit, producing the net amount payable/refundable.
            lines.push(("2100".to_string(), vat_milli, 0, Some("ضريبة مدخلات".to_string())));
        }
        lines.push((source_account, 0, total, Some("مصدر تمويل المصروف".to_string())));
        let jid = crate::commands::accounting::post_to_journal(
            &tx,
            "expense",
            expense_id,
            &date,
            &format!("مصروف {}", exp_no.unwrap_or_default()),
            &lines,
            &user_id.to_string(),
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
