use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
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
pub fn list_boms(state: State<'_, DbState>) -> Result<Vec<BomEntry>, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_BOM)?;
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.product_id, COALESCE(p.name_ar, p.name_en, '') AS product_name,
                    b.item_id, COALESCE(i.name_ar, i.name_en, '') AS item_name,
                    b.qty_per_carton, b.waste_pct, b.active
             FROM bom b
             LEFT JOIN products p ON p.id = b.product_id
             LEFT JOIN inventory_items i ON i.id = b.item_id
             WHERE b.active = 1
             ORDER BY b.id DESC",
        )?;

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
        })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_bom(state: State<'_, DbState>, user_id: i64, input: CreateBomInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;
    conn.execute(
        "INSERT INTO bom (product_id, item_id, qty_per_carton, waste_pct, active)
         VALUES (?1, ?2, ?3, ?4, 1)",
        rusqlite::params![
            input.product_id,
            input.item_id,
            input.qty_per_carton,
            input.waste_pct.unwrap_or(0.0),
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_bom", "bom", Some(id), None, Some(&format!("product={}", input.product_id)), None);
    Ok(id)
}
