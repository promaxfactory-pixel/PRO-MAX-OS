use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct BomEntry {
    pub id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub qty_per_carton: f64,
    pub waste_pct: f64,
    pub active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateBomInput {
    pub product_id: i64,
    pub item_id: i64,
    pub qty_per_carton: f64,
    pub waste_pct: Option<f64>,
}

#[tauri::command]
pub fn list_boms(state: State<'_, DbState>) -> Result<Vec<BomEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.product_id, p.name AS product_name,
                    b.item_id, i.name AS item_name,
                    b.qty_per_carton, b.waste_pct, b.active
             FROM bom b
             LEFT JOIN products p ON p.id = b.product_id
             LEFT JOIN inventory_items i ON i.id = b.item_id
             WHERE b.active = 1
             ORDER BY b.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BomEntry {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                item_id: row.get(3)?,
                item_name: row.get(4)?,
                qty_per_carton: row.get(5)?,
                waste_pct: row.get(6)?,
                active: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_bom(state: State<'_, DbState>, input: CreateBomInput) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO bom (product_id, item_id, qty_per_carton, waste_pct, active)
         VALUES (?1, ?2, ?3, ?4, 1)",
        rusqlite::params![
            input.product_id,
            input.item_id,
            input.qty_per_carton,
            input.waste_pct.unwrap_or(0.0),
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}
