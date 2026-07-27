use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub asset_no: String,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_cost_milli: i64,
    pub current_value_milli: i64,
    pub depreciation_method: String,
    pub depreciation_rate_pct: f64,
    pub useful_life_months: i32,
    pub accumulated_depreciation_milli: i64,
    pub location: Option<String>,
    pub department: Option<String>,
    pub assigned_to: Option<String>,
    pub supplier: Option<String>,
    pub warranty_expiry: Option<String>,
    pub last_maintenance: Option<String>,
    pub next_maintenance: Option<String>,
    pub condition_status: String,
    pub status: String,
    pub notes: Option<String>,
    pub active: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetMaintenanceLog {
    pub id: i64,
    pub asset_id: i64,
    pub maintenance_type: String,
    pub date: String,
    pub description: String,
    pub cost_milli: i64,
    pub performed_by: Option<String>,
    pub next_due: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetInput {
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_cost_milli: i64,
    pub depreciation_method: Option<String>,
    pub depreciation_rate_pct: Option<f64>,
    pub useful_life_months: Option<i32>,
    pub location: Option<String>,
    pub department: Option<String>,
    pub assigned_to: Option<String>,
    pub supplier: Option<String>,
    pub warranty_expiry: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenanceInput {
    pub asset_id: i64,
    pub maintenance_type: String,
    pub date: String,
    pub description: String,
    pub cost_milli: i64,
    pub performed_by: Option<String>,
    pub next_due: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_assets(state: State<'_, DbState>) -> Result<Vec<Asset>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, asset_no, name, category, description, serial_number, purchase_date, purchase_cost_milli, current_value_milli, depreciation_method, depreciation_rate_pct, useful_life_months, accumulated_depreciation_milli, location, department, assigned_to, supplier, warranty_expiry, last_maintenance, next_maintenance, condition_status, status, notes, active, created_at FROM fixed_assets WHERE active=1 ORDER BY asset_no"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Asset {
            id: row.get(0)?, asset_no: row.get(1)?, name: row.get(2)?, category: row.get(3)?,
            description: row.get(4)?, serial_number: row.get(5)?, purchase_date: row.get(6)?,
            purchase_cost_milli: row.get(7)?, current_value_milli: row.get(8)?,
            depreciation_method: row.get(9)?, depreciation_rate_pct: row.get(10)?,
            useful_life_months: row.get(11)?, accumulated_depreciation_milli: row.get(12)?,
            location: row.get(13)?, department: row.get(14)?, assigned_to: row.get(15)?,
            supplier: row.get(16)?, warranty_expiry: row.get(17)?, last_maintenance: row.get(18)?,
            next_maintenance: row.get(19)?, condition_status: row.get(20)?, status: row.get(21)?,
            notes: row.get(22)?, active: row.get(23)?, created_at: row.get(24)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
}

#[tauri::command]
pub fn get_asset(state: State<'_, DbState>, id: i64) -> Result<Asset, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, asset_no, name, category, description, serial_number, purchase_date, purchase_cost_milli, current_value_milli, depreciation_method, depreciation_rate_pct, useful_life_months, accumulated_depreciation_milli, location, department, assigned_to, supplier, warranty_expiry, last_maintenance, next_maintenance, condition_status, status, notes, active, created_at FROM fixed_assets WHERE id = ?",
        [id],
        |row| Ok(Asset {
            id: row.get(0)?, asset_no: row.get(1)?, name: row.get(2)?, category: row.get(3)?,
            description: row.get(4)?, serial_number: row.get(5)?, purchase_date: row.get(6)?,
            purchase_cost_milli: row.get(7)?, current_value_milli: row.get(8)?,
            depreciation_method: row.get(9)?, depreciation_rate_pct: row.get(10)?,
            useful_life_months: row.get(11)?, accumulated_depreciation_milli: row.get(12)?,
            location: row.get(13)?, department: row.get(14)?, assigned_to: row.get(15)?,
            supplier: row.get(16)?, warranty_expiry: row.get(17)?, last_maintenance: row.get(18)?,
            next_maintenance: row.get(19)?, condition_status: row.get(20)?, status: row.get(21)?,
            notes: row.get(22)?, active: row.get(23)?, created_at: row.get(24)?,
        }),
    ).map_err(|_| AppError::not_found("Asset not found"))
}

#[tauri::command]
pub fn create_asset(state: State<'_, DbState>, input: CreateAssetInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    let seq: i64 = conn.query_row("SELECT COALESCE(MAX(CAST(SUBSTR(asset_no, 5) AS INTEGER)), 0) + 1 FROM fixed_assets", [], |r| r.get(0)).unwrap_or(1);
    let asset_no = format!("AST-{}", seq);
    let dep_method = input.depreciation_method.unwrap_or_else(|| "straight_line".to_string());
    let dep_rate = input.depreciation_rate_pct.unwrap_or(0.0);
    let useful_life = input.useful_life_months.unwrap_or(60);

    conn.execute(
        "INSERT INTO fixed_assets (asset_no, name, category, description, serial_number, purchase_date, purchase_cost_milli, current_value_milli, depreciation_method, depreciation_rate_pct, useful_life_months, accumulated_depreciation_milli, location, department, assigned_to, supplier, warranty_expiry, condition_status, status, notes, active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?, 'good', 'active', ?, 1, datetime('now'))",
        rusqlite::params![asset_no, input.name, input.category, input.description, input.serial_number, input.purchase_date, input.purchase_cost_milli, input.purchase_cost_milli, dep_method, dep_rate, useful_life, input.location, input.department, input.assigned_to, input.supplier, input.warranty_expiry, input.notes],
    )?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_asset_maintenance(state: State<'_, DbState>, asset_id: i64) -> Result<Vec<AssetMaintenanceLog>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, asset_id, maintenance_type, date, description, cost_milli, performed_by, next_due, notes FROM asset_maintenance_logs WHERE asset_id = ? ORDER BY date DESC"
    )?;
    let rows = stmt.query_map([asset_id], |row| {
        Ok(AssetMaintenanceLog {
            id: row.get(0)?, asset_id: row.get(1)?, maintenance_type: row.get(2)?,
            date: row.get(3)?, description: row.get(4)?, cost_milli: row.get(5)?,
            performed_by: row.get(6)?, next_due: row.get(7)?, notes: row.get(8)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
}

#[tauri::command]
pub fn create_asset_maintenance(state: State<'_, DbState>, input: CreateMaintenanceInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    conn.execute(
        "INSERT INTO asset_maintenance_logs (asset_id, maintenance_type, date, description, cost_milli, performed_by, next_due, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![input.asset_id, input.maintenance_type, input.date, input.description, input.cost_milli, input.performed_by, input.next_due, input.notes],
    )?;
    conn.execute("UPDATE fixed_assets SET last_maintenance = ? WHERE id = ?", rusqlite::params![input.date, input.asset_id]).ok();
    if let Some(ref next) = input.next_due {
        conn.execute("UPDATE fixed_assets SET next_maintenance = ? WHERE id = ?", rusqlite::params![next, input.asset_id]).ok();
    }
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_asset_register_summary(state: State<'_, DbState>) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    let total_assets: i64 = conn.query_row("SELECT COUNT(*) FROM fixed_assets WHERE active=1", [], |r| r.get(0)).unwrap_or(0);
    let total_cost: i64 = conn.query_row("SELECT COALESCE(SUM(purchase_cost_milli), 0) FROM fixed_assets WHERE active=1", [], |r| r.get(0)).unwrap_or(0);
    let total_current: i64 = conn.query_row("SELECT COALESCE(SUM(current_value_milli), 0) FROM fixed_assets WHERE active=1", [], |r| r.get(0)).unwrap_or(0);
    let total_depreciation: i64 = conn.query_row("SELECT COALESCE(SUM(accumulated_depreciation_milli), 0) FROM fixed_assets WHERE active=1", [], |r| r.get(0)).unwrap_or(0);
    let maintenance_due: i64 = conn.query_row("SELECT COUNT(*) FROM fixed_assets WHERE active=1 AND next_maintenance IS NOT NULL AND next_maintenance <= date('now')", [], |r| r.get(0)).unwrap_or(0);
    let warranty_expiring: i64 = conn.query_row("SELECT COUNT(*) FROM fixed_assets WHERE active=1 AND warranty_expiry IS NOT NULL AND warranty_expiry BETWEEN date('now') AND date('now', '+90 days')", [], |r| r.get(0)).unwrap_or(0);

    Ok(serde_json::json!({
        "total_assets": total_assets,
        "total_cost_milli": total_cost,
        "total_current_value_milli": total_current,
        "total_depreciation_milli": total_depreciation,
        "maintenance_due": maintenance_due,
        "warranty_expiring_90d": warranty_expiring,
    }))
}

#[tauri::command]
pub fn calculate_depreciation(state: State<'_, DbState>, asset_id: i64, months: i32) -> Result<serde_json::Value, AppError> {
    let asset = get_asset(state, asset_id)?;
    let monthly_rate = asset.depreciation_rate_pct / 100.0 / 12.0;
    let mut value = asset.purchase_cost_milli as f64;
    let mut schedule: Vec<serde_json::Value> = Vec::new();
    for m in 1..=months {
        let dep = (value * monthly_rate).round() as i64;
        value -= dep as f64;
        schedule.push(serde_json::json!({
            "month": m,
            "opening_value_milli": (value + dep as f64) as i64,
            "depreciation_milli": dep,
            "closing_value_milli": value as i64,
        }));
    }
    Ok(serde_json::json!({
        "asset_id": asset_id,
        "asset_name": asset.name,
        "purchase_cost_milli": asset.purchase_cost_milli,
        "monthly_rate_pct": (monthly_rate * 100.0 * 100.0).round() / 100.0,
        "months": months,
        "schedule": schedule,
    }))
}
