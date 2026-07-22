use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct OvertimeRecord {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub date: String,
    pub hours: f64,
    pub rate_multiplier: f64,
    pub reason: Option<String>,
    pub approved: i64,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOvertimeInput {
    pub employee_id: i64,
    pub date: String,
    pub hours: f64,
    pub rate_multiplier: Option<f64>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveOvertimeInput {
    pub approved_by: String,
}

#[tauri::command]
pub fn list_overtime_records(
    state: State<'_, DbState>,
) -> Result<Vec<OvertimeRecord>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.employee_id, e.name, o.date, o.hours, o.rate_multiplier, o.reason, o.approved, o.approved_by, o.approved_at, o.status, o.notes, o.created_by, o.created_at FROM overtime_records o LEFT JOIN employees e ON o.employee_id=e.id ORDER BY o.date DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(OvertimeRecord {
                id: row.get(0)?,
                employee_id: row.get(1)?,
                employee_name: row.get(2)?,
                date: row.get(3)?,
                hours: row.get(4)?,
                rate_multiplier: row.get(5)?,
                reason: row.get(6)?,
                approved: row.get(7)?,
                approved_by: row.get(8)?,
                approved_at: row.get(9)?,
                status: row.get(10)?,
                notes: row.get(11)?,
                created_by: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_overtime_record(
    state: State<'_, DbState>,
    input: CreateOvertimeInput,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO overtime_records(employee_id, date, hours, rate_multiplier, reason, notes, status, created_at) VALUES(?,?,?,?,?,?, 'Pending', datetime('now'))",
        rusqlite::params![
            input.employee_id,
            input.date,
            input.hours,
            input.rate_multiplier.unwrap_or(1.5),
            input.reason,
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_overtime_record", "overtime_records", Some(id), None, None, None);
    Ok(id)
}

#[tauri::command]
pub fn approve_overtime(
    state: State<'_, DbState>,
    id: i64,
    input: ApproveOvertimeInput,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE overtime_records SET approved=1, approved_by=?, approved_at=datetime('now'), status='Approved' WHERE id=?",
        rusqlite::params![input.approved_by, id],
    )
    .map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "approve_overtime", "overtime_records", Some(id), None, Some("Approved"), None);
    Ok("Approved".to_string())
}

#[tauri::command]
pub fn reject_overtime(
    state: State<'_, DbState>,
    id: i64,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE overtime_records SET approved=0, status='Rejected' WHERE id=?",
        [id],
    )
    .map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "reject_overtime", "overtime_records", Some(id), None, Some("Rejected"), None);
    Ok("Rejected".to_string())
}
