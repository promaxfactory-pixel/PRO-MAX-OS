use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CashbankAccount {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub atype: Option<String>,
    pub balance_milli: i64,
    pub active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateCashbankInput {
    pub name: String,
    pub code: Option<String>,
    pub atype: Option<String>,
}

#[tauri::command]
pub fn list_cashbank_accounts(
    state: State<'_, DbState>,
) -> Result<Vec<CashbankAccount>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, atype, balance_milli, active FROM cashbank_accounts WHERE active=1 ORDER BY name",
        )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CashbankAccount {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                atype: row.get(3)?,
                balance_milli: row.get(4)?,
                active: row.get(5)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_cashbank_account(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateCashbankInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;

    conn.execute(
        "INSERT INTO cashbank_accounts(code, name, atype, balance_milli, active) VALUES(?,?,?,0,1)",
        rusqlite::params![
            input.code,
            input.name,
            input.atype,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_cashbank_account", "cashbank_accounts", Some(id), None, Some(&input.name), None);
    Ok(id)
}
