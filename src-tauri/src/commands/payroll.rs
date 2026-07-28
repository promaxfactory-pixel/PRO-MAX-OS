use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollRun {
    pub id: i64,
    pub run_no: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub status: Option<String>,
    pub total_gross_milli: i64,
    pub total_deductions_milli: i64,
    pub total_net_milli: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollRunInput {
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeAdvance {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub amount_milli: i64,
    pub date: String,
    pub reason: Option<String>,
    pub status: Option<String>,
    pub remaining_milli: i64,
    pub deduction_per_payroll_milli: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdvanceInput {
    pub employee_id: i64,
    pub amount_milli: i64,
    pub date: String,
    pub reason: Option<String>,
    pub deduction_per_payroll_milli: Option<i64>,
}

#[tauri::command]
pub fn list_payroll_runs(state: State<'_, DbState>) -> Result<Vec<PayrollRun>, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_PAYROLL)?;
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, run_no, period_start, period_end, status, total_gross_milli, total_deductions_milli, total_net_milli, created_at FROM payroll_runs ORDER BY id DESC",
        )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PayrollRun {
                id: row.get(0)?,
                run_no: row.get(1)?,
                period_start: row.get(2)?,
                period_end: row.get(3)?,
                status: row.get(4)?,
                total_gross_milli: row.get(5)?,
                total_deductions_milli: row.get(6)?,
                total_net_milli: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_payroll_run(
    state: State<'_, DbState>,
    input: CreatePayrollRunInput,
) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq: i64 = tx
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='PR' AND year=?",
            [&year],
            |r| r.get(0),
        )
        .unwrap_or(1);
    tx.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('PR',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    ).map_err(|e| format!("Failed to increment payroll sequence: {}", e))?;
    let run_no = format!("PR-{}-{:04}", year, seq);

    tx.execute(
        "INSERT INTO payroll_runs(run_no, period_start, period_end, status, total_gross_milli, total_deductions_milli, total_net_milli, created_at) VALUES(?,?,?,'Draft',0,0,0,datetime('now'))",
        rusqlite::params![
            run_no,
            input.period_start,
            input.period_end,
        ],
    )?;
    let run_id = tx.last_insert_rowid();
    let _ = rbac::log_audit(&*tx, None, None, "create_payroll_run", "payroll_runs", Some(run_id), None, Some(&run_no), None);
    tx.commit()?;
    Ok(run_id)
}

#[tauri::command]
pub fn list_employee_advances(
    state: State<'_, DbState>,
) -> Result<Vec<EmployeeAdvance>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT ea.id, ea.employee_id, e.name, ea.amount_milli, ea.date, ea.reason, ea.status, ea.remaining_milli, ea.deduction_per_payroll_milli
             FROM employee_advances ea LEFT JOIN employees e ON ea.employee_id=e.id ORDER BY ea.id DESC",
        )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(EmployeeAdvance {
                id: row.get(0)?,
                employee_id: row.get(1)?,
                employee_name: row.get(2)?,
                amount_milli: row.get(3)?,
                date: row.get(4)?,
                reason: row.get(5)?,
                status: row.get(6)?,
                remaining_milli: row.get(7)?,
                deduction_per_payroll_milli: row.get(8)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_employee_advance(
    state: State<'_, DbState>,
    input: CreateAdvanceInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    let deduction = input.deduction_per_payroll_milli.unwrap_or(0);

    conn.execute(
        "INSERT INTO employee_advances(employee_id, amount_milli, date, reason, status, remaining_milli, deduction_per_payroll_milli) VALUES(?,?,?,?,'open',?,?)",
        rusqlite::params![
            input.employee_id,
            input.amount_milli,
            input.date,
            input.reason,
            input.amount_milli,
            deduction,
        ],
    )?;
    let adv_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_employee_advance", "employee_advances", Some(adv_id), None, None, None);
    Ok(adv_id)
}
