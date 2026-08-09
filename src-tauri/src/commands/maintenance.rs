use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceSheet {
    pub id: i64,
    pub sheet_no: Option<String>,
    pub date: String,
    pub shift: Option<String>,
    pub maintenance_supervisor: Option<String>,
    pub machine_id: Option<i64>,
    pub area: Option<String>,
    pub fault_title: Option<String>,
    pub fault_description: Option<String>,
    pub severity: Option<String>,
    pub machine_stopped: i64,
    pub downtime_minutes: i64,
    pub repair_status: Option<String>,
    pub total_repair_cost_milli: i64,
    pub root_cause: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub approved_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenanceSheetInput {
    pub date: String,
    pub shift: Option<String>,
    pub maintenance_supervisor: Option<String>,
    pub machine_id: Option<i64>,
    pub area: Option<String>,
    pub fault_title: Option<String>,
    pub fault_description: Option<String>,
    pub severity: Option<String>,
    pub notes: Option<String>,
    pub machine_stopped: Option<i64>,
    pub downtime_start: Option<String>,
    pub downtime_end: Option<String>,
    pub downtime_minutes: Option<i64>,
    pub repair_action: Option<String>,
    pub parts_changed: Option<String>,
    pub spare_parts_cost_milli: Option<i64>,
    pub labor_cost_milli: Option<i64>,
    pub other_cost_milli: Option<i64>,
    pub root_cause: Option<String>,
    pub preventive_action: Option<String>,
    pub next_followup_date: Option<String>,
}

#[tauri::command]
pub fn list_maintenance_sheets(state: State<'_, DbState>) -> Result<Vec<MaintenanceSheet>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, sheet_no, date, shift, maintenance_supervisor, machine_id, area, fault_title, fault_description, severity, machine_stopped, downtime_minutes, repair_status, total_repair_cost_milli, root_cause, status, notes, created_by, created_at, approved_by
             FROM maintenance_daily_sheets ORDER BY id DESC",
        )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MaintenanceSheet {
                id: row.get(0)?,
                sheet_no: row.get(1)?,
                date: row.get(2)?,
                shift: row.get(3)?,
                maintenance_supervisor: row.get(4)?,
                machine_id: row.get(5)?,
                area: row.get(6)?,
                fault_title: row.get(7)?,
                fault_description: row.get(8)?,
                severity: row.get(9)?,
                machine_stopped: row.get(10)?,
                downtime_minutes: row.get(11)?,
                repair_status: row.get(12)?,
                total_repair_cost_milli: row.get(13)?,
                root_cause: row.get(14)?,
                status: row.get(15)?,
                notes: row.get(16)?,
                created_by: row.get(17)?,
                created_at: row.get(18)?,
                approved_by: row.get(19)?,
            })
        })?;

    let mut sheets = Vec::new();
    for row in rows {
        sheets.push(row?);
    }
    Ok(sheets)
}

#[tauri::command]
pub fn get_maintenance_sheet(
    state: State<'_, DbState>,
    id: i64,
) -> Result<MaintenanceSheet, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT id, sheet_no, date, shift, maintenance_supervisor, machine_id, area, fault_title, fault_description, severity, machine_stopped, downtime_minutes, repair_status, total_repair_cost_milli, root_cause, status, notes, created_by, created_at, approved_by
         FROM maintenance_daily_sheets WHERE id = ?1",
        params![id],
        |row| {
            Ok(MaintenanceSheet {
                id: row.get(0)?,
                sheet_no: row.get(1)?,
                date: row.get(2)?,
                shift: row.get(3)?,
                maintenance_supervisor: row.get(4)?,
                machine_id: row.get(5)?,
                area: row.get(6)?,
                fault_title: row.get(7)?,
                fault_description: row.get(8)?,
                severity: row.get(9)?,
                machine_stopped: row.get(10)?,
                downtime_minutes: row.get(11)?,
                repair_status: row.get(12)?,
                total_repair_cost_milli: row.get(13)?,
                root_cause: row.get(14)?,
                status: row.get(15)?,
                notes: row.get(16)?,
                created_by: row.get(17)?,
                created_at: row.get(18)?,
                approved_by: row.get(19)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn create_maintenance_sheet(
    state: State<'_, DbState>,
    input: CreateMaintenanceSheetInput,
) -> Result<MaintenanceSheet, AppError> {
    let conn = state.0.lock()?;

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM maintenance_daily_sheets",
            [],
            |row| row.get(0),
        )?;
    let sheet_no = format!("MNT-{:04}", seq);

    let spare = input.spare_parts_cost_milli.unwrap_or(0);
    let labor = input.labor_cost_milli.unwrap_or(0);
    let other = input.other_cost_milli.unwrap_or(0);

    conn.execute(
        "INSERT INTO maintenance_daily_sheets (sheet_no, date, shift, maintenance_supervisor, machine_id, area, fault_title, fault_description, severity, notes, machine_stopped, downtime_start, downtime_end, downtime_minutes, repair_action, parts_changed, spare_parts_cost_milli, labor_cost_milli, other_cost_milli, total_repair_cost_milli, root_cause, preventive_action, next_followup_date, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, 'Open', datetime('now'))",
        params![
            sheet_no,
            input.date,
            input.shift,
            input.maintenance_supervisor,
            input.machine_id,
            input.area,
            input.fault_title,
            input.fault_description,
            input.severity,
            input.notes,
            input.machine_stopped.unwrap_or(0),
            input.downtime_start,
            input.downtime_end,
            input.downtime_minutes.unwrap_or(0),
            input.repair_action,
            input.parts_changed,
            spare,
            labor,
            other,
            spare + labor + other,
            input.root_cause,
            input.preventive_action,
            input.next_followup_date,
        ],
    )?;

    let sheet_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_maintenance_sheet", "maintenance_daily_sheets", Some(sheet_id), None, None, None);

    Ok(conn.query_row(
        "SELECT id, sheet_no, date, shift, maintenance_supervisor, machine_id, area, fault_title, fault_description, severity, machine_stopped, downtime_minutes, repair_status, total_repair_cost_milli, root_cause, status, notes, created_by, created_at, approved_by FROM maintenance_daily_sheets WHERE id=?1",
        params![sheet_id],
        |row| {
            Ok(MaintenanceSheet {
                id: row.get(0)?, sheet_no: row.get(1)?, date: row.get(2)?, shift: row.get(3)?,
                maintenance_supervisor: row.get(4)?, machine_id: row.get(5)?, area: row.get(6)?,
                fault_title: row.get(7)?, fault_description: row.get(8)?, severity: row.get(9)?,
                machine_stopped: row.get(10)?, downtime_minutes: row.get(11)?,
                repair_status: row.get(12)?, total_repair_cost_milli: row.get(13)?,
                root_cause: row.get(14)?, status: row.get(15)?, notes: row.get(16)?,
                created_by: row.get(17)?, created_at: row.get(18)?, approved_by: row.get(19)?,
            })
        },
    )?)
}
