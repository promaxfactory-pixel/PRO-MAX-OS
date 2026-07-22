use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Renewal {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
    pub authority: Option<String>,
    pub issue_date: Option<String>,
    pub expiry_date: Option<String>,
    pub cost_milli: i64,
    pub responsible: Option<String>,
    pub alert_days: i64,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRenewalInput {
    pub name: String,
    pub category: Option<String>,
    pub authority: Option<String>,
    pub issue_date: Option<String>,
    pub expiry_date: Option<String>,
    pub cost_milli: Option<i64>,
    pub responsible: Option<String>,
    pub alert_days: Option<i64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_renewals(state: State<'_, DbState>) -> Result<Vec<Renewal>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, authority, issue_date, expiry_date, cost_milli, responsible, alert_days, status, notes FROM renewals ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Renewal {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                authority: row.get(3)?,
                issue_date: row.get(4)?,
                expiry_date: row.get(5)?,
                cost_milli: row.get(6)?,
                responsible: row.get(7)?,
                alert_days: row.get(8)?,
                status: row.get(9)?,
                notes: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_renewal(
    state: State<'_, DbState>,
    input: CreateRenewalInput,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO renewals(name, category, authority, issue_date, expiry_date, cost_milli, responsible, alert_days, status, notes) VALUES(?,?,?,?,?,?,?,?,'Active',?)",
        rusqlite::params![
            input.name,
            input.category,
            input.authority,
            input.issue_date,
            input.expiry_date,
            input.cost_milli.unwrap_or(0),
            input.responsible,
            input.alert_days.unwrap_or(30),
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}
