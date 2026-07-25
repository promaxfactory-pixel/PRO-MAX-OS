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
    pub brand_name: Option<String>,
    pub cup_size_ml: Option<f64>,
    pub cup_diameter_mm: Option<f64>,
    pub paper_weight_gsm: Option<f64>,
    pub lid_type: Option<String>,
    pub print_colors: Option<i64>,
    pub carton_length_cm: Option<f64>,
    pub carton_width_cm: Option<f64>,
    pub carton_height_cm: Option<f64>,
    pub color: Option<String>,
    pub material_type: Option<String>,
    pub product_type: Option<String>,
    pub family_id: Option<i64>,
    pub min_stock: Option<f64>,
    pub weight_kg: Option<f64>,
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
    pub brand_name: Option<String>,
    pub cup_size_ml: Option<f64>,
    pub cup_diameter_mm: Option<f64>,
    pub paper_weight_gsm: Option<f64>,
    pub lid_type: Option<String>,
    pub print_colors: Option<i64>,
    pub carton_length_cm: Option<f64>,
    pub carton_width_cm: Option<f64>,
    pub carton_height_cm: Option<f64>,
    pub color: Option<String>,
    pub material_type: Option<String>,
    pub product_type: Option<String>,
    pub family_id: Option<i64>,
    pub min_stock: Option<f64>,
    pub weight_kg: Option<f64>,
}

#[tauri::command]
pub fn list_products(state: State<'_, DbState>) -> Result<Vec<Product>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, code, name_ar, name_en, size, cup_type, cups_per_carton, carton_type, default_price_milli, default_cost_milli, vat_pct, barcode, notes, active, brand_name, cup_size_ml, cup_diameter_mm, paper_weight_gsm, lid_type, print_colors, carton_length_cm, carton_width_cm, carton_height_cm, color, material_type, product_type, family_id, min_stock, weight_kg FROM products WHERE active=1 ORDER BY name_ar",
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(Product {
            id: row.get(0)?,
            code: row.get(1)?,
            name_ar: row.get(2)?,
            name_en: row.get(3)?,
            size: row.get(4)?,
            cup_type: row.get(5)?,
            cups_per_carton: row.get(6)?,
            carton_type: row.get(7)?,
            default_price_milli: row.get(8)?,
            default_cost_milli: row.get(9)?,
            vat_pct: row.get(10)?,
            barcode: row.get(11)?,
            notes: row.get(12)?,
            active: row.get(13)?,
            brand_name: row.get(14)?,
            cup_size_ml: row.get(15)?,
            cup_diameter_mm: row.get(16)?,
            paper_weight_gsm: row.get(17)?,
            lid_type: row.get(18)?,
            print_colors: row.get(19)?,
            carton_length_cm: row.get(20)?,
            carton_width_cm: row.get(21)?,
            carton_height_cm: row.get(22)?,
            color: row.get(23)?,
            material_type: row.get(24)?,
            product_type: row.get(25)?,
            family_id: row.get(26)?,
            min_stock: row.get(27)?,
            weight_kg: row.get(28)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSelectItem {
    pub id: i64,
    pub name_ar: Option<String>,
    pub code: Option<String>,
    pub cups_per_carton: i64,
    pub default_price_milli: i64,
}

#[tauri::command]
pub fn list_products_for_select(state: State<'_, DbState>) -> Result<Vec<ProductSelectItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name_ar, code, cups_per_carton, default_price_milli FROM products WHERE active=1 ORDER BY name_ar",
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(ProductSelectItem {
            id: row.get(0)?,
            name_ar: row.get(1)?,
            code: row.get(2)?,
            cups_per_carton: row.get(3)?,
            default_price_milli: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_product(state: State<'_, DbState>, id: i64) -> Result<Product, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, code, name_ar, name_en, size, cup_type, cups_per_carton, carton_type, default_price_milli, default_cost_milli, vat_pct, barcode, notes, active, brand_name, cup_size_ml, cup_diameter_mm, paper_weight_gsm, lid_type, print_colors, carton_length_cm, carton_width_cm, carton_height_cm, color, material_type, product_type, family_id, min_stock, weight_kg FROM products WHERE id=?",
        [id],
        |row| {
            Ok(Product {
                id: row.get(0)?,
                code: row.get(1)?,
                name_ar: row.get(2)?,
                name_en: row.get(3)?,
                size: row.get(4)?,
                cup_type: row.get(5)?,
                cups_per_carton: row.get(6)?,
                carton_type: row.get(7)?,
                default_price_milli: row.get(8)?,
                default_cost_milli: row.get(9)?,
                vat_pct: row.get(10)?,
                barcode: row.get(11)?,
                notes: row.get(12)?,
                active: row.get(13)?,
                brand_name: row.get(14)?,
                cup_size_ml: row.get(15)?,
                cup_diameter_mm: row.get(16)?,
                paper_weight_gsm: row.get(17)?,
                lid_type: row.get(18)?,
                print_colors: row.get(19)?,
                carton_length_cm: row.get(20)?,
                carton_width_cm: row.get(21)?,
                carton_height_cm: row.get(22)?,
                color: row.get(23)?,
                material_type: row.get(24)?,
                product_type: row.get(25)?,
                family_id: row.get(26)?,
                min_stock: row.get(27)?,
                weight_kg: row.get(28)?,
            })
        },
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_product(state: State<'_, DbState>, input: CreateProductInput) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO products(code, name_ar, name_en, size, cup_type, cups_per_carton, default_price_milli, default_cost_milli, barcode, notes, brand_name, cup_size_ml, cup_diameter_mm, paper_weight_gsm, lid_type, print_colors, carton_length_cm, carton_width_cm, carton_height_cm, color, material_type, product_type, family_id, min_stock, weight_kg) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            input.code,
            input.name_ar,
            input.name_en,
            input.size,
            input.cup_type,
            input.cups_per_carton.unwrap_or(1000),
            input.default_price_milli.unwrap_or(0),
            input.default_cost_milli.unwrap_or(0),
            input.barcode,
            input.notes,
            input.brand_name,
            input.cup_size_ml,
            input.cup_diameter_mm,
            input.paper_weight_gsm,
            input.lid_type,
            input.print_colors,
            input.carton_length_cm,
            input.carton_width_cm,
            input.carton_height_cm,
            input.color,
            input.material_type,
            input.product_type,
            input.family_id,
            input.min_stock,
            input.weight_kg,
        ],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_product", "products", Some(id), None, Some(&input.name_ar.as_deref().unwrap_or("")), None);
    Ok(id)
}

#[tauri::command]
pub fn update_product(state: State<'_, DbState>, id: i64, input: CreateProductInput) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE products SET code=?, name_ar=?, name_en=?, size=?, cup_type=?, cups_per_carton=?, default_price_milli=?, default_cost_milli=?, barcode=?, notes=?, brand_name=?, cup_size_ml=?, cup_diameter_mm=?, paper_weight_gsm=?, lid_type=?, print_colors=?, carton_length_cm=?, carton_width_cm=?, carton_height_cm=?, color=?, material_type=?, product_type=?, family_id=?, min_stock=?, weight_kg=? WHERE id=?",
        rusqlite::params![
            input.code,
            input.name_ar,
            input.name_en,
            input.size,
            input.cup_type,
            input.cups_per_carton,
            input.default_price_milli,
            input.default_cost_milli,
            input.barcode,
            input.notes,
            input.brand_name,
            input.cup_size_ml,
            input.cup_diameter_mm,
            input.paper_weight_gsm,
            input.lid_type,
            input.print_colors,
            input.carton_length_cm,
            input.carton_width_cm,
            input.carton_height_cm,
            input.color,
            input.material_type,
            input.product_type,
            input.family_id,
            input.min_stock,
            input.weight_kg,
            id,
        ],
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
