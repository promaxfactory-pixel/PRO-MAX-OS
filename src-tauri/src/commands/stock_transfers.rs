use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct StockTransfer {
    pub id: i64,
    pub transfer_no: Option<String>,
    pub from_warehouse: Option<String>,
    pub to_warehouse: Option<String>,
    pub item_name: Option<String>,
    pub qty: f64,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStockTransferInput {
    pub from_warehouse_id: Option<i64>,
    pub to_warehouse_id: Option<i64>,
    pub item_id: Option<i64>,
    pub qty: f64,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_stock_transfers(
    state: State<'_, DbState>,
) -> Result<Vec<StockTransfer>, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_STOCK_TRANSFERS)?;
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT st.id, st.transfer_no, mw_from.name AS from_name, mw_to.name AS to_name, COALESCE(ii.name_ar, ii.name_en, '') AS item_name, st.qty, st.status, st.notes, st.created_at
             FROM stock_transfers st
             LEFT JOIN multi_warehouse mw_from ON mw_from.id = st.from_warehouse_id
             LEFT JOIN multi_warehouse mw_to ON mw_to.id = st.to_warehouse_id
             LEFT JOIN inventory_items ii ON ii.id = st.item_id
             ORDER BY st.id DESC",
        )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(StockTransfer {
                id: row.get(0)?,
                transfer_no: row.get(1)?,
                from_warehouse: row.get(2)?,
                to_warehouse: row.get(3)?,
                item_name: row.get(4)?,
                qty: row.get(5)?,
                status: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub location: Option<String>,
    pub manager: Option<String>,
    pub active: i64,
}

#[tauri::command]
pub fn list_warehouses(state: State<'_, DbState>) -> Result<Vec<Warehouse>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, code, name, location, manager, active FROM multi_warehouse WHERE active=1 ORDER BY name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Warehouse {
            id: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            location: row.get(3)?,
            manager: row.get(4)?,
            active: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_stock_transfer(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateStockTransferInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM stock_transfers",
            [],
            |row| row.get(0),
        )?;
    let transfer_no = format!("ST-{:04}", seq);

    conn.execute(
        "INSERT INTO stock_transfers (transfer_no, from_warehouse_id, to_warehouse_id, item_id, qty, status, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'Pending', ?6, datetime('now'))",
        rusqlite::params![
            transfer_no,
            input.from_warehouse_id,
            input.to_warehouse_id,
            input.item_id,
            input.qty,
            input.notes,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_stock_transfer", "stock_transfers", Some(id), None, None, None);
    Ok(id)
}
