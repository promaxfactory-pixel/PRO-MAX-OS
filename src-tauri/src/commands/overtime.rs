use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
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
    pub overtime_type: String,
    pub hourly_rate_milli: i64,
    pub estimated_cost_milli: i64,
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
    pub overtime_type: Option<String>,
    pub rate_multiplier: Option<f64>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_overtime_records(
    state: State<'_, DbState>,
) -> Result<Vec<OvertimeRecord>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT o.id, o.employee_id, e.name, o.date, o.hours, o.rate_multiplier, COALESCE(o.overtime_type, 'normal_day_day'),
                    CAST(COALESCE(e.overtime_rate_milli, 0) AS INTEGER) AS hourly_rate_milli,
                    CAST(ROUND(o.hours * o.rate_multiplier * COALESCE(e.overtime_rate_milli, 0)) AS INTEGER) AS estimated_cost_milli,
                    o.reason, o.approved, o.approved_by, o.approved_at, o.status, o.notes, o.created_by, o.created_at
             FROM overtime_records o
             LEFT JOIN employees e ON o.employee_id=e.id
             ORDER BY o.date DESC",
        )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(OvertimeRecord {
                id: row.get(0)?,
                employee_id: row.get(1)?,
                employee_name: row.get(2)?,
                date: row.get(3)?,
                hours: row.get(4)?,
                rate_multiplier: row.get(5)?,
                overtime_type: row.get(6)?,
                hourly_rate_milli: row.get(7)?,
                estimated_cost_milli: row.get(8)?,
                reason: row.get(9)?,
                approved: row.get(10)?,
                approved_by: row.get(11)?,
                approved_at: row.get(12)?,
                status: row.get(13)?,
                notes: row.get(14)?,
                created_by: row.get(15)?,
                created_at: row.get(16)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_overtime_record(
    state: State<'_, DbState>,
    input: CreateOvertimeInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    if input.hours <= 0.0 {
        return Err(AppError::validation("عدد ساعات العمل الإضافي يجب أن يكون أكبر من صفر"));
    }
    let overtime_type = input.overtime_type.unwrap_or_else(|| "normal_day_day".to_string());
    let legal_multiplier = match overtime_type.as_str() {
        "normal_day_day" => 1.25,
        "normal_day_night" => 1.50,
        "rest_or_holiday" => 2.00,
        "emergency_day" => 1.50,
        "emergency_night" => 1.75,
        "emergency_rest_or_holiday" => 3.00,
        _ => return Err(AppError::validation("نوع العمل الإضافي غير معروف")),
    };
    let multiplier = input.rate_multiplier.unwrap_or(legal_multiplier);
    if (multiplier - legal_multiplier).abs() > 0.0001 {
        return Err(AppError::validation("مضاعف الأجر لا يطابق نوع العمل الإضافي المحدد"));
    }
    conn.execute(
        "INSERT INTO overtime_records(employee_id, date, hours, rate_multiplier, overtime_type, reason, notes, status, created_at)
         VALUES(?,?,?,?,?,?,?, 'Pending', datetime('now'))",
        rusqlite::params![
            input.employee_id,
            input.date,
            input.hours,
            multiplier,
            overtime_type,
            input.reason,
            input.notes,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_overtime_record", "overtime_records", Some(id), None, None, None);
    Ok(id)
}

#[tauri::command]
pub fn approve_overtime(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "hr", "manager"])?;
    conn.execute(
        "UPDATE overtime_records SET approved=1, approved_by=?, approved_at=datetime('now'), status='Approved' WHERE id=?",
        rusqlite::params![user_id, id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "approve_overtime", "overtime_records", Some(id), None, Some("Approved"), None);
    Ok("Approved".to_string())
}

#[tauri::command]
pub fn reject_overtime(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "hr", "manager"])?;
    conn.execute(
        "UPDATE overtime_records SET approved=0, status='Rejected' WHERE id=?",
        [id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "reject_overtime", "overtime_records", Some(id), None, Some("Rejected"), None);
    Ok("Rejected".to_string())
}
