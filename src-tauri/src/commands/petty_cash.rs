use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct PettyCashAccount {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub responsible: Option<String>,
    pub role: Option<String>,
    pub spending_limit_milli: i64,
    pub balance_milli: i64,
    pub status: Option<String>,
    pub active: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePettyCashInput {
    pub name: String,
    pub code: Option<String>,
    pub responsible: Option<String>,
    pub role: Option<String>,
    pub spending_limit_milli: Option<i64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_petty_cash_accounts(state: State<'_, DbState>) -> Result<Vec<PettyCashAccount>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, responsible, role, spending_limit_milli, balance_milli, status, active, notes
             FROM petty_cash_accounts
             WHERE active = 1
             ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(PettyCashAccount {
            id: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            responsible: row.get(3)?,
            role: row.get(4)?,
            spending_limit_milli: row.get(5)?,
            balance_milli: row.get(6)?,
            status: row.get(7)?,
            active: row.get(8)?,
            notes: row.get(9)?,
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
pub fn create_petty_cash_account(input: CreatePettyCashInput, state: State<'_, DbState>) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO petty_cash_accounts(code, name, responsible, role, spending_limit_milli, balance_milli, status, active, notes)
         VALUES(?1, ?2, ?3, ?4, ?5, 0, 'open', 1, ?6)",
        rusqlite::params![
            input.code,
            input.name,
            input.responsible,
            input.role,
            input.spending_limit_milli.unwrap_or(0),
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_petty_cash_account", "petty_cash_accounts", Some(id), None, Some(&input.name), None);
    Ok(id)
}
