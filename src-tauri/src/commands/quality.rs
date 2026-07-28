use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityInspection {
    pub id: i64,
    pub production_line_id: Option<i64>,
    pub date: Option<String>,
    pub inspector: Option<String>,
    pub result: Option<String>,
    pub defect_type: Option<String>,
    pub defect_qty: i64,
    pub notes: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQualityInspectionInput {
    pub date: Option<String>,
    pub inspector: Option<String>,
    pub production_line_id: Option<i64>,
    pub result: Option<String>,
    pub defect_type: Option<String>,
    pub defect_qty: Option<i64>,
    pub notes: Option<String>,
    pub status: Option<String>,
}

#[tauri::command]
pub fn list_quality_inspections(
    state: State<'_, DbState>,
) -> Result<Vec<QualityInspection>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, production_line_id, date, inspector, result, defect_type, defect_qty, notes, status
             FROM quality_inspections ORDER BY id DESC",
        )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(QualityInspection {
                id: row.get(0)?,
                production_line_id: row.get(1)?,
                date: row.get(2)?,
                inspector: row.get(3)?,
                result: row.get(4)?,
                defect_type: row.get(5)?,
                defect_qty: row.get(6)?,
                notes: row.get(7)?,
                status: row.get(8)?,
            })
        })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_quality_inspection(
    state: State<'_, DbState>,
    input: CreateQualityInspectionInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    conn.execute(
        "INSERT INTO quality_inspections (production_line_id, date, inspector, result, defect_type, defect_qty, notes, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            input.production_line_id,
            input.date,
            input.inspector,
            input.result,
            input.defect_type,
            input.defect_qty.unwrap_or(0),
            input.notes,
            input.status,
        ],
    )?;
    let insp_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_quality_inspection", "quality_inspections", Some(insp_id), None, None, None);
    Ok(insp_id)
}
