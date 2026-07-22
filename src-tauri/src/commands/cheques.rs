use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cheque {
    pub id: i64,
    pub kind: String,
    pub cheque_no: Option<String>,
    pub bank: Option<String>,
    pub party: Option<String>,
    pub amount_milli: i64,
    pub due_date: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChequeInput {
    pub kind: String,
    pub cheque_no: Option<String>,
    pub bank: Option<String>,
    pub party: Option<String>,
    pub amount_milli: Option<i64>,
    pub due_date: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_cheques(state: State<'_, DbState>) -> Result<Vec<Cheque>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, cheque_no, bank, party, amount_milli, due_date, status, notes FROM cheques ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Cheque {
                id: row.get(0)?,
                kind: row.get(1)?,
                cheque_no: row.get(2)?,
                bank: row.get(3)?,
                party: row.get(4)?,
                amount_milli: row.get(5)?,
                due_date: row.get(6)?,
                status: row.get(7)?,
                notes: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_cheque(
    state: State<'_, DbState>,
    input: CreateChequeInput,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO cheques(kind, cheque_no, bank, party, amount_milli, due_date, status, notes) VALUES(?,?,?,?,?,?,'issued',?)",
        rusqlite::params![
            input.kind,
            input.cheque_no,
            input.bank,
            input.party,
            input.amount_milli.unwrap_or(0),
            input.due_date,
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_cheque", "cheques", Some(id), None, None, None);
    Ok(id)
}
