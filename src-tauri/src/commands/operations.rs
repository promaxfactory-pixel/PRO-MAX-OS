use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationsSheet {
    pub id: i64,
    pub sheet_no: Option<String>,
    pub date: String,
    pub shift: Option<String>,
    pub supervisor_name: Option<String>,
    pub worker_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub normal_hours: f64,
    pub overtime_hours: f64,
    pub cartons_produced: f64,
    pub total_cups: f64,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOperationsSheetLineInput {
    pub worker_id: Option<i64>,
    pub worker_name: Option<String>,
    pub attendance: Option<String>,
    pub overtime_hours: Option<f64>,
    pub production_qty: Option<f64>,
    pub quality_grade: Option<String>,
    pub safety_incident: Option<bool>,
    pub safety_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOperationsSheetInput {
    pub date: String,
    pub shift: Option<String>,
    pub supervisor_name: Option<String>,
    pub worker_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub normal_hours: Option<f64>,
    pub overtime_hours: Option<f64>,
    pub cartons_produced: Option<f64>,
    pub total_cups: Option<f64>,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub entries: Option<Vec<CreateOperationsSheetLineInput>>,
}

#[tauri::command]
pub fn list_operations_sheets(state: State<'_, DbState>) -> Result<Vec<OperationsSheet>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, sheet_no, date, shift, supervisor_name, worker_name, start_time, end_time, normal_hours, overtime_hours, cartons_produced, total_cups, status, notes, created_by, created_at
             FROM operations_daily_sheets ORDER BY id DESC",
        )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(OperationsSheet {
                id: row.get(0)?,
                sheet_no: row.get(1)?,
                date: row.get(2)?,
                shift: row.get(3)?,
                supervisor_name: row.get(4)?,
                worker_name: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                normal_hours: row.get(8)?,
                overtime_hours: row.get(9)?,
                cartons_produced: row.get(10)?,
                total_cups: row.get(11)?,
                status: row.get(12)?,
                notes: row.get(13)?,
                created_by: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?;

    let mut sheets = Vec::new();
    for row in rows {
        sheets.push(row?);
    }
    Ok(sheets)
}

#[tauri::command]
pub fn get_operations_sheet(
    state: State<'_, DbState>,
    id: i64,
) -> Result<OperationsSheet, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT id, sheet_no, date, shift, supervisor_name, worker_name, start_time, end_time, normal_hours, overtime_hours, cartons_produced, total_cups, status, notes, created_by, created_at
         FROM operations_daily_sheets WHERE id = ?1",
        params![id],
        |row| {
            Ok(OperationsSheet {
                id: row.get(0)?,
                sheet_no: row.get(1)?,
                date: row.get(2)?,
                shift: row.get(3)?,
                supervisor_name: row.get(4)?,
                worker_name: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                normal_hours: row.get(8)?,
                overtime_hours: row.get(9)?,
                cartons_produced: row.get(10)?,
                total_cups: row.get(11)?,
                status: row.get(12)?,
                notes: row.get(13)?,
                created_by: row.get(14)?,
                created_at: row.get(15)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn create_operations_sheet(
    state: State<'_, DbState>,
    input: CreateOperationsSheetInput,
) -> Result<OperationsSheet, AppError> {
    let conn = state.0.lock()?;

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM operations_daily_sheets",
            [],
            |row| row.get(0),
        )?;
    let sheet_no = format!("OPS-{:04}", seq);

    conn.execute(
        "INSERT INTO operations_daily_sheets (sheet_no, date, shift, supervisor_name, worker_name, start_time, end_time, normal_hours, overtime_hours, cartons_produced, total_cups, status, notes, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'Draft', ?12, ?13)",
        params![
            sheet_no,
            input.date,
            input.shift,
            input.supervisor_name,
            input.worker_name,
            input.start_time,
            input.end_time,
            input.normal_hours.unwrap_or(0.0),
            input.overtime_hours.unwrap_or(0.0),
            input.cartons_produced.unwrap_or(0.0),
            input.total_cups.unwrap_or(0.0),
            input.notes,
            input.created_by,
        ],
    )?;

    let id = conn.last_insert_rowid();

    if let Some(entries) = &input.entries {
        for e in entries {
            conn.execute(
                "INSERT INTO operations_sheet_lines (sheet_id, worker_id, worker_name, attendance, overtime_hours, production_qty, quality_grade, safety_incident, safety_note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    e.worker_id,
                    e.worker_name,
                    e.attendance.clone().unwrap_or_else(|| "present".into()),
                    e.overtime_hours.unwrap_or(0.0),
                    e.production_qty.unwrap_or(0.0),
                    e.quality_grade.clone().unwrap_or_else(|| "A".into()),
                    e.safety_incident.unwrap_or(false) as i64,
                    e.safety_note,
                ],
            )?;
        }
    }

    Ok(conn.query_row(
        "SELECT id, sheet_no, date, shift, supervisor_name, worker_name, start_time, end_time, normal_hours, overtime_hours, cartons_produced, total_cups, status, notes, created_by, created_at FROM operations_daily_sheets WHERE id=?1",
        params![id],
        |row| {
            Ok(OperationsSheet {
                id: row.get(0)?, sheet_no: row.get(1)?, date: row.get(2)?, shift: row.get(3)?,
                supervisor_name: row.get(4)?, worker_name: row.get(5)?, start_time: row.get(6)?,
                end_time: row.get(7)?, normal_hours: row.get(8)?, overtime_hours: row.get(9)?,
                cartons_produced: row.get(10)?, total_cups: row.get(11)?, status: row.get(12)?,
                notes: row.get(13)?, created_by: row.get(14)?, created_at: row.get(15)?,
            })
        },
    )?)
}
