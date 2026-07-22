use crate::db::DbState;
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
) -> Result<Vec<StockTransfer>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT st.id, st.transfer_no, mw_from.name AS from_name, mw_to.name AS to_name, ii.name AS item_name, st.qty, st.status, st.notes, st.created_at
             FROM stock_transfers st
             LEFT JOIN multi_warehouse mw_from ON mw_from.id = st.from_warehouse_id
             LEFT JOIN multi_warehouse mw_to ON mw_to.id = st.to_warehouse_id
             LEFT JOIN inventory_items ii ON ii.id = st.item_id
             ORDER BY st.id DESC",
        )
        .map_err(|e| e.to_string())?;

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
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_stock_transfer(
    state: State<'_, DbState>,
    input: CreateStockTransferInput,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM stock_transfers",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
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
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}
