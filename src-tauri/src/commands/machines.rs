use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub mtype: Option<String>,
    pub supported_products: Option<String>,
    pub purchase_date: Option<String>,
    pub supplier: Option<String>,
    pub cost_milli: i64,
    pub capacity_cpm: i64,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateMachineInput {
    pub name: String,
    pub code: Option<String>,
    pub mtype: Option<String>,
    pub supported_products: Option<String>,
    pub purchase_date: Option<String>,
    pub supplier: Option<String>,
    pub cost_milli: Option<i64>,
    pub capacity_cpm: Option<i64>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMachineInput {
    pub name: Option<String>,
    pub code: Option<String>,
    pub mtype: Option<String>,
    pub supported_products: Option<String>,
    pub purchase_date: Option<String>,
    pub supplier: Option<String>,
    pub cost_milli: Option<i64>,
    pub capacity_cpm: Option<i64>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub active: Option<i64>,
}

#[tauri::command]
pub fn list_machines(state: State<'_, DbState>) -> Result<Vec<Machine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, mtype, supported_products, purchase_date, supplier, cost_milli, capacity_cpm, status, notes, active FROM machines WHERE active=1 ORDER BY name",
        )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Machine {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                mtype: row.get(3)?,
                supported_products: row.get(4)?,
                purchase_date: row.get(5)?,
                supplier: row.get(6)?,
                cost_milli: row.get(7)?,
                capacity_cpm: row.get(8)?,
                status: row.get(9)?,
                notes: row.get(10)?,
                active: row.get(11)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn get_machine(state: State<'_, DbState>, id: i64) -> Result<Machine, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT id, code, name, mtype, supported_products, purchase_date, supplier, cost_milli, capacity_cpm, status, notes, active FROM machines WHERE id=?",
        [id],
        |row| {
            Ok(Machine {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                mtype: row.get(3)?,
                supported_products: row.get(4)?,
                purchase_date: row.get(5)?,
                supplier: row.get(6)?,
                cost_milli: row.get(7)?,
                capacity_cpm: row.get(8)?,
                status: row.get(9)?,
                notes: row.get(10)?,
                active: row.get(11)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn create_machine(
    state: State<'_, DbState>,
    input: CreateMachineInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;

    conn.execute(
        "INSERT INTO machines(code, name, mtype, supported_products, purchase_date, supplier, cost_milli, capacity_cpm, status, notes, active) VALUES(?,?,?,?,?,?,?,?,?,?,1)",
        rusqlite::params![
            input.code,
            input.name,
            input.mtype,
            input.supported_products,
            input.purchase_date,
            input.supplier,
            input.cost_milli.unwrap_or(0),
            input.capacity_cpm.unwrap_or(0),
            input.status.unwrap_or_else(|| "active".to_string()),
            input.notes,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_machine", "machines", Some(id), None, Some(&input.name), None);
    Ok(id)
}

#[tauri::command]
pub fn update_machine(
    state: State<'_, DbState>,
    id: i64,
    input: UpdateMachineInput,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.name {
        sets.push("name=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.code {
        sets.push("code=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.mtype {
        sets.push("mtype=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.supported_products {
        sets.push("supported_products=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.purchase_date {
        sets.push("purchase_date=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.supplier {
        sets.push("supplier=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.cost_milli {
        sets.push("cost_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.capacity_cpm {
        sets.push("capacity_cpm=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.status {
        sets.push("status=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.notes {
        sets.push("notes=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.active {
        sets.push("active=?");
        params.push(Box::new(v));
    }

    if sets.is_empty() {
        return Err(AppError::validation("No changes provided"));
    }

    params.push(Box::new(id));
    let sql = format!("UPDATE machines SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
    let _ = rbac::log_audit(&conn, None, None, "update_machine", "machines", Some(id), None, None, None);
    Ok("Updated successfully".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemperatureLog {
    pub id: i64,
    pub machine_id: i64,
    pub machine_name: Option<String>,
    pub temperature: f64,
    pub ts: String,
    pub recorded_by: Option<String>,
}

#[tauri::command]
pub fn record_temperature(
    state: State<'_, DbState>,
    machine_id: i64,
    temperature: f64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    conn.execute(
        "INSERT INTO machine_temp_logs (machine_id, temperature) VALUES (?1, ?2)",
        params![machine_id, temperature],
    )?;
    Ok("تم تسجيل درجة الحرارة".to_string())
}

#[tauri::command]
pub fn get_machine_temperatures(
    state: State<'_, DbState>,
    machine_id: i64,
    hours: Option<i64>,
) -> Result<Vec<TemperatureLog>, AppError> {
    let conn = state.0.lock()?;
    let h = hours.unwrap_or(1);
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(h);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT mtl.id, mtl.machine_id, COALESCE(m.name, ''), mtl.temperature, mtl.ts, mtl.recorded_by
             FROM machine_temp_logs mtl
             LEFT JOIN machines m ON m.id = mtl.machine_id
             WHERE mtl.machine_id = ?1 AND mtl.ts >= ?2
             ORDER BY mtl.ts DESC",
        )?;

    let rows = stmt
        .query_map(params![machine_id, cutoff_str], |row| {
            Ok(TemperatureLog {
                id: row.get(0)?,
                machine_id: row.get(1)?,
                machine_name: row.get(2)?,
                temperature: row.get(3)?,
                ts: row.get(4)?,
                recorded_by: row.get(5)?,
            })
        })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveMachineTemp {
    pub machine_id: i64,
    pub machine_name: String,
    pub temperature: f64,
    pub ts: String,
    pub status: String,
}

#[tauri::command]
pub fn get_live_machine_temps(state: State<'_, DbState>) -> Result<Vec<LiveMachineTemp>, AppError> {
    let conn = state.0.lock()?;
    let cutoff = chrono::Utc::now() - chrono::Duration::minutes(5);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.name, mtl.temperature, mtl.ts,
                    CASE WHEN mtl.temperature > 180 THEN 'critical'
                         WHEN mtl.temperature > 150 THEN 'warning'
                         ELSE 'normal' END as status
             FROM machines m
             LEFT JOIN machine_temp_logs mtl ON mtl.id = (
                 SELECT id FROM machine_temp_logs
                 WHERE machine_id = m.id AND ts >= ?1
                 ORDER BY ts DESC LIMIT 1
             )
             WHERE m.active = 1
             ORDER BY m.name",
        )?;

    let rows = stmt
        .query_map(params![cutoff_str], |row| {
            Ok(LiveMachineTemp {
                machine_id: row.get(0)?,
                machine_name: row.get(1)?,
                temperature: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                ts: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                status: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "offline".to_string()),
            })
        })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}
