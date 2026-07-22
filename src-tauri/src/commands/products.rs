use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub code: Option<String>,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub size: Option<String>,
    pub cup_type: Option<String>,
    pub cups_per_carton: i64,
    pub carton_type: Option<String>,
    pub default_price_milli: i64,
    pub default_cost_milli: i64,
    pub vat_pct: f64,
    pub barcode: Option<String>,
    pub notes: Option<String>,
    pub active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductInput {
    pub code: Option<String>,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub size: Option<String>,
    pub cup_type: Option<String>,
    pub cups_per_carton: Option<i64>,
    pub default_price_milli: Option<i64>,
    pub default_cost_milli: Option<i64>,
    pub barcode: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_products(state: State<'_, DbState>) -> Result<Vec<Product>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, code, name_ar, name_en, size, cup_type, cups_per_carton, carton_type, default_price_milli, default_cost_milli, vat_pct, barcode, notes, active FROM products WHERE active=1 ORDER BY name_ar").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(Product { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_en: row.get(3)?, size: row.get(4)?, cup_type: row.get(5)?, cups_per_carton: row.get(6)?, carton_type: row.get(7)?, default_price_milli: row.get(8)?, default_cost_milli: row.get(9)?, vat_pct: row.get(10)?, barcode: row.get(11)?, notes: row.get(12)?, active: row.get(13)? })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_product(state: State<'_, DbState>, id: i64) -> Result<Product, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row("SELECT id, code, name_ar, name_en, size, cup_type, cups_per_carton, carton_type, default_price_milli, default_cost_milli, vat_pct, barcode, notes, active FROM products WHERE id=?", [id], |row| {
        Ok(Product { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_en: row.get(3)?, size: row.get(4)?, cup_type: row.get(5)?, cups_per_carton: row.get(6)?, carton_type: row.get(7)?, default_price_milli: row.get(8)?, default_cost_milli: row.get(9)?, vat_pct: row.get(10)?, barcode: row.get(11)?, notes: row.get(12)?, active: row.get(13)? })
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_product(state: State<'_, DbState>, input: CreateProductInput) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO products(code, name_ar, name_en, size, cup_type, cups_per_carton, default_price_milli, default_cost_milli, barcode, notes) VALUES(?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![input.code, input.name_ar, input.name_en, input.size, input.cup_type, input.cups_per_carton.unwrap_or(1000), input.default_price_milli.unwrap_or(0), input.default_cost_milli.unwrap_or(0), input.barcode, input.notes],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_product", "products", Some(id), None, Some(&input.name_ar.as_deref().unwrap_or("")), None);
    Ok(id)
}

#[tauri::command]
pub fn update_product(state: State<'_, DbState>, id: i64, input: CreateProductInput) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE products SET code=?, name_ar=?, name_en=?, size=?, cup_type=?, cups_per_carton=?, default_price_milli=?, default_cost_milli=?, barcode=?, notes=? WHERE id=?",
        rusqlite::params![input.code, input.name_ar, input.name_en, input.size, input.cup_type, input.cups_per_carton, input.default_price_milli, input.default_cost_milli, input.barcode, input.notes, id],
    ).map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "update_product", "products", Some(id), None, None, None);
    Ok("تم التحديث".to_string())
}

#[tauri::command]
pub fn delete_product(state: State<'_, DbState>, id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE products SET active=0 WHERE id=?", [id]).map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "delete_product", "products", Some(id), None, None, None);
    Ok("تم الحذف".to_string())
}
