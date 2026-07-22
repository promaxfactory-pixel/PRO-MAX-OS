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
}

#[tauri::command]
pub fn list_expenses(state: State<'_, DbState>) -> Result<Vec<Expense>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, exp_no, date, category, account_code, amount_milli, vat_milli,
                    method, vendor, reference, notes, approval_status, created_by, created_at
             FROM expenses
             ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;

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
            created_by: row.get(12)?,
            created_at: row.get(13)?,
        })
    })
    .map_err(|e| e.to_string())?;

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

    conn.execute(
        "INSERT INTO expenses(exp_no, date, category, account_code, amount_milli, vat_milli, method, vendor, reference, notes, approval_status)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending')",
        rusqlite::params![
            exp_no,
            input.date,
            input.category,
            input.account_code,
            input.amount_milli,
            input.vat_milli.unwrap_or(0),
            input.method,
            input.vendor,
            input.reference,
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;

    let exp_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_expense", "expenses", Some(exp_id), None, Some(&input.notes.unwrap_or_default()), None);
    Ok(exp_id)
}
