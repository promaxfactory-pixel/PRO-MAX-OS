use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::error::AppError;

fn fetch_item(conn: &rusqlite::Connection, id: i64) -> Result<InventoryItem, AppError> {
    Ok(conn.query_row(
        "SELECT ii.id, ii.code, ii.name_ar, ii.name_en, ii.kind, ii.uom, ii.product_id, ii.qty_on_hand, ii.avg_cost_milli, ii.reorder_level, ii.supplier_id, ii.notes, ii.active FROM inventory_items ii WHERE ii.id=?1",
        params![id],
        |row| {
            Ok(InventoryItem {
                id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_en: row.get(3)?,
                kind: row.get(4)?, uom: row.get(5)?, product_id: row.get(6)?,
                qty_on_hand: row.get(7)?, avg_cost_milli: row.get(8)?, reorder_level: row.get(9)?,
                supplier_id: row.get(10)?, notes: row.get(11)?, active: row.get(12)?,
            })
        },
    )?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: i64,
    pub code: Option<String>,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub kind: String,
    pub uom: String,
    pub product_id: Option<i64>,
    pub qty_on_hand: f64,
    pub avg_cost_milli: i64,
    pub reorder_level: f64,
    pub supplier_id: Option<i64>,
    pub notes: Option<String>,
    pub active: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryMovement {
    pub id: i64,
    pub ts: String,
    pub item_id: i64,
    pub product_name: Option<String>,
    pub mtype: String,
    pub qty_in: f64,
    pub qty_out: f64,
    pub unit_cost_milli: i64,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemInput {
    pub code: Option<String>,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub kind: Option<String>,
    pub uom: Option<String>,
    pub product_id: Option<i64>,
    pub qty_on_hand: Option<f64>,
    pub avg_cost_milli: Option<i64>,
    pub reorder_level: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemInput {
    pub code: Option<String>,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub kind: Option<String>,
    pub uom: Option<String>,
    pub product_id: Option<i64>,
    pub reorder_level: Option<f64>,
    pub supplier_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdjustStockInput {
    pub item_id: i64,
    pub qty_change: f64,
    pub unit_cost_milli: Option<i64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_inventory_items(state: State<'_, DbState>) -> Result<Vec<InventoryItem>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT ii.id, ii.code, ii.name_ar, ii.name_en, ii.kind, ii.uom, ii.product_id, ii.qty_on_hand, ii.avg_cost_milli, ii.reorder_level, ii.supplier_id, ii.notes, ii.active
             FROM inventory_items ii WHERE ii.active = 1 ORDER BY ii.name_ar",
        )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(InventoryItem {
                id: row.get(0)?,
                code: row.get(1)?,
                name_ar: row.get(2)?,
                name_en: row.get(3)?,
                kind: row.get(4)?,
                uom: row.get(5)?,
                product_id: row.get(6)?,
                qty_on_hand: row.get(7)?,
                avg_cost_milli: row.get(8)?,
                reorder_level: row.get(9)?,
                supplier_id: row.get(10)?,
                notes: row.get(11)?,
                active: row.get(12)?,
            })
        })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_inventory_item(state: State<'_, DbState>, id: i64) -> Result<InventoryItem, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT ii.id, ii.code, ii.name_ar, ii.name_en, ii.kind, ii.uom, ii.product_id, ii.qty_on_hand, ii.avg_cost_milli, ii.reorder_level, ii.supplier_id, ii.notes, ii.active
         FROM inventory_items ii WHERE ii.id = ?1",
        params![id],
        |row| {
            Ok(InventoryItem {
                id: row.get(0)?,
                code: row.get(1)?,
                name_ar: row.get(2)?,
                name_en: row.get(3)?,
                kind: row.get(4)?,
                uom: row.get(5)?,
                product_id: row.get(6)?,
                qty_on_hand: row.get(7)?,
                avg_cost_milli: row.get(8)?,
                reorder_level: row.get(9)?,
                supplier_id: row.get(10)?,
                notes: row.get(11)?,
                active: row.get(12)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn create_inventory_item(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateItemInput,
) -> Result<InventoryItem, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    conn.execute(
        "INSERT INTO inventory_items (code, name_ar, name_en, kind, uom, product_id, qty_on_hand, avg_cost_milli, reorder_level, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            input.code,
            input.name_ar,
            input.name_en,
            input.kind.unwrap_or_else(|| "raw".to_string()),
            input.uom.unwrap_or_else(|| "pcs".to_string()),
            input.product_id,
            input.qty_on_hand.unwrap_or(0.0),
            input.avg_cost_milli.unwrap_or(0),
            input.reorder_level.unwrap_or(0.0),
            input.notes,
        ],
    )?;

    let id = conn.last_insert_rowid();
    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "create_inventory_item", "inventory_items", Some(id), None, input.name_ar.as_deref().or(input.name_en.as_deref()), None);
    fetch_item(&conn, id)
}

#[tauri::command]
pub fn update_inventory_item(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
    input: UpdateItemInput,
) -> Result<InventoryItem, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let mut sets = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.code {
        sets.push("code = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.name_ar {
        sets.push("name_ar = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.name_en {
        sets.push("name_en = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.kind {
        sets.push("kind = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.uom {
        sets.push("uom = ?");
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = input.product_id {
        sets.push("product_id = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = input.reorder_level {
        sets.push("reorder_level = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = input.supplier_id {
        sets.push("supplier_id = ?");
        values.push(Box::new(v));
    }
    if let Some(v) = &input.notes {
        sets.push("notes = ?");
        values.push(Box::new(v.clone()));
    }

    if sets.is_empty() {
        return fetch_item(&conn, id);
    }

    let sql = format!(
        "UPDATE inventory_items SET {} WHERE id = ?",
        sets.join(", ")
    );
    values.push(Box::new(id));

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, params_ref.as_slice())?;

    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "update_inventory_item", "inventory_items", Some(id), None, None, None);
    fetch_item(&conn, id)
}

#[tauri::command]
pub fn adjust_stock(
    state: State<'_, DbState>,
    user_id: i64,
    input: AdjustStockInput,
) -> Result<InventoryItem, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let item = adjust_stock_inner(&conn, &input)?;

    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "adjust_stock", "inventory_movements", Some(input.item_id), None, Some(&format!("qty_change={}", input.qty_change)), None);
    Ok(item)
}

pub(crate) fn adjust_stock_inner(
    conn: &rusqlite::Connection,
    input: &AdjustStockInput,
) -> Result<InventoryItem, AppError> {
    let (qty_in, qty_out) = if input.qty_change >= 0.0 {
        (input.qty_change, 0.0)
    } else {
        (0.0, input.qty_change.abs())
    };

    let (old_qty, old_avg): (f64, i64) = conn
        .query_row(
            "SELECT qty_on_hand, avg_cost_milli FROM inventory_items WHERE id = ?1",
            params![input.item_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::not_found("العنصر غير موجود"))?;

    // When a unit cost is supplied on an increase, merge it into the running
    // weighted-average cost instead of leaving avg_cost_milli untouched.
    let mut new_avg = old_avg;
    if input.qty_change > 0.0 {
        if let Some(cost) = input.unit_cost_milli {
            if cost > 0 {
                let new_qty = old_qty + input.qty_change;
                if new_qty > 0.0 {
                    new_avg = ((old_qty * old_avg as f64 + input.qty_change * cost as f64) / new_qty)
                        .round() as i64;
                }
            }
        }
    }

    conn.execute(
        "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?1, avg_cost_milli = ?2 WHERE id = ?3",
        params![input.qty_change, new_avg, input.item_id],
    )?;

    conn.execute(
        "INSERT INTO inventory_movements (ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, notes)
         VALUES (datetime('now'), ?1, 'adjustment', ?2, ?3, ?4, ?5)",
        params![
            input.item_id,
            qty_in,
            qty_out,
            input.unit_cost_milli.unwrap_or(old_avg),
            input.notes,
        ],
    )?;

    // Persist the adjustment in its own table for an audit trail.
    let seq: i64 = conn
        .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM inventory_adjustments", [], |row| row.get(0))
        ?;
    let adj_no = format!("ADJ-{:04}", seq);
    let direction = if input.qty_change >= 0.0 { "in" } else { "out" };
    conn.execute(
        "INSERT INTO inventory_adjustments (adj_no, date, item_id, direction, qty, unit_cost_milli, reason, status, created_at)
         VALUES (?1, date('now'), ?2, ?3, ?4, ?5, ?6, 'Approved', datetime('now'))",
        params![
            adj_no,
            input.item_id,
            direction,
            input.qty_change.abs(),
            input.unit_cost_milli.unwrap_or(old_avg),
            input.notes,
        ],
    )?;

    fetch_item(conn, input.item_id)
}

#[tauri::command]
pub fn get_inventory_movements(
    state: State<'_, DbState>,
    item_id: i64,
) -> Result<Vec<InventoryMovement>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT im.id, im.ts, im.item_id, p.name_ar, im.mtype, im.qty_in, im.qty_out, im.unit_cost_milli, im.ref_type, im.ref_id, im.location, im.notes
             FROM inventory_movements im
             LEFT JOIN inventory_items ii ON im.item_id = ii.id
             LEFT JOIN products p ON ii.product_id = p.id
             WHERE im.item_id = ?1
             ORDER BY im.id DESC",
        )?;

    let rows = stmt
        .query_map(params![item_id], |row| {
            Ok(InventoryMovement {
                id: row.get(0)?,
                ts: row.get(1)?,
                item_id: row.get(2)?,
                product_name: row.get(3)?,
                mtype: row.get(4)?,
                qty_in: row.get(5)?,
                qty_out: row.get(6)?,
                unit_cost_milli: row.get(7)?,
                ref_type: row.get(8)?,
                ref_id: row.get(9)?,
                location: row.get(10)?,
                notes: row.get(11)?,
            })
        })?;

    let mut movements = Vec::new();
    for row in rows {
        movements.push(row?);
    }
    Ok(movements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn.execute(
            "INSERT INTO users(username, password_hash, salt, role) VALUES('admin', 'x', 'y', 'admin')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO inventory_items(id, code, name_ar, kind, qty_on_hand, avg_cost_milli) VALUES(1, 'RM1', 'بكرة ورق', 'raw', 10, 500)",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn adjust_stock_merges_cost_when_provided() {
        let conn = test_db();
        let input = AdjustStockInput {
            item_id: 1,
            qty_change: 10.0,
            unit_cost_milli: Some(1000),
            notes: Some("إدخال أولي".into()),
        };
        let item = adjust_stock_inner(&conn, &input).unwrap();
        assert!((item.qty_on_hand - 20.0).abs() < 1e-9);
        // (10*500 + 10*1000) / 20 = 750
        assert_eq!(item.avg_cost_milli, 750);

        let (adj_qty, direction, status): (f64, String, String) = conn
            .query_row(
                "SELECT qty, direction, status FROM inventory_adjustments WHERE item_id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert!((adj_qty - 10.0).abs() < 1e-9);
        assert_eq!(direction, "in");
        assert_eq!(status, "Approved");
    }

    #[test]
    fn adjust_stock_down_keeps_cost() {
        let conn = test_db();
        let input = AdjustStockInput {
            item_id: 1,
            qty_change: -4.0,
            unit_cost_milli: None,
            notes: None,
        };
        let item = adjust_stock_inner(&conn, &input).unwrap();
        assert!((item.qty_on_hand - 6.0).abs() < 1e-9);
        assert_eq!(item.avg_cost_milli, 500);

        let direction: String = conn
            .query_row("SELECT direction FROM inventory_adjustments WHERE item_id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(direction, "out");
    }
}
