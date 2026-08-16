use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use rusqlite::params;
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

#[tauri::command]
pub fn complete_stock_transfer(
    state: State<'_, DbState>,
    user_id: i64,
    transfer_id: i64,
) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;
    let tx = conn.transaction()?;
    let res = complete_stock_transfer_inner(&tx, transfer_id)?;
    let _ = rbac::log_audit(&tx, Some(user_id), None, "complete_stock_transfer", "stock_transfers", Some(transfer_id), None, None, None);
    tx.commit()?;
    Ok(res)
}

/// Completes a pending warehouse-to-warehouse transfer: validates availability,
/// records the movement, and reassigns the item's current warehouse.
///
/// Global `qty_on_hand` is unchanged (a transfer moves stock between locations,
/// it does not add or remove quantity), so the movement records equal in/out
/// quantities and the item's `warehouse_id` is set to the destination.
pub(crate) fn complete_stock_transfer_inner(
    conn: &rusqlite::Connection,
    transfer_id: i64,
) -> Result<String, AppError> {
    let (status, item_id, qty, from_wh, to_wh, _transfer_no): (String, i64, f64, Option<i64>, Option<i64>, String) =
        conn.query_row(
            "SELECT COALESCE(status, 'Draft'), item_id, qty, from_warehouse_id, to_warehouse_id, COALESCE(transfer_no, '')
             FROM stock_transfers WHERE id = ?1",
            params![transfer_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|_| AppError::not_found("التحويل غير موجود"))?;

    if status == "Completed" {
        return Err(AppError::validation("التحويل مكتمل مسبقاً"));
    }
    if qty <= 0.0 {
        return Err(AppError::validation("كمية التحويل غير صالحة"));
    }

    let (on_hand, avg_cost, item_name): (f64, i64, String) = conn
        .query_row(
            "SELECT ii.qty_on_hand, ii.avg_cost_milli, COALESCE(ii.name_ar, ii.name_en, '')
             FROM inventory_items ii WHERE ii.id = ?1",
            params![item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| AppError::not_found("العنصر غير موجود"))?;
    if on_hand + 1e-9 < qty {
        return Err(AppError::validation(format!(
            "رصيد غير كافٍ لـ'{}': المتاح {:.3}، المطلوب {:.3}",
            item_name, on_hand, qty
        )));
    }

    let location = match (from_wh, to_wh) {
        (Some(f), Some(t)) => {
            let from_name: Option<String> = conn
                .query_row("SELECT name FROM multi_warehouse WHERE id = ?1", params![f], |row| row.get(0))
                .ok();
            let to_name: Option<String> = conn
                .query_row("SELECT name FROM multi_warehouse WHERE id = ?1", params![t], |row| row.get(0))
                .ok();
            format!(
                "من {} إلى {}",
                from_name.unwrap_or_default(),
                to_name.unwrap_or_default()
            )
        }
        _ => "تحويل مخزني".to_string(),
    };

    conn.execute(
        "INSERT INTO inventory_movements (ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, location, notes)
         VALUES (datetime('now'), ?1, 'transfer', ?2, ?2, ?3, 'stock_transfer', ?4, ?5, 'تحويل بين المستودعات')",
        params![item_id, qty, avg_cost, transfer_id, location],
    )?;

    conn.execute(
        "UPDATE inventory_items SET warehouse_id = ?1 WHERE id = ?2",
        params![to_wh, item_id],
    )?;

    conn.execute(
        "UPDATE stock_transfers SET status = 'Completed', completed_at = datetime('now') WHERE id = ?1",
        params![transfer_id],
    )?;

    Ok("تم إتمام التحويل".to_string())
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
        conn.execute("INSERT INTO multi_warehouse(id, code, name) VALUES(1, 'W1', 'المخزن الرئيسي')", []).unwrap();
        conn.execute("INSERT INTO multi_warehouse(id, code, name) VALUES(2, 'W2', 'مخزن الإنتاج')", []).unwrap();
        conn.execute(
            "INSERT INTO inventory_items(id, code, name_ar, kind, qty_on_hand, avg_cost_milli) VALUES(1, 'RM1', 'بكرة ورق', 'raw', 50, 1000)",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn complete_transfer_moves_item_and_records_movement() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO stock_transfers(id, transfer_no, from_warehouse_id, to_warehouse_id, item_id, qty, status) VALUES(1, 'ST-0001', 1, 2, 1, 10, 'Pending')",
            [],
        )
        .unwrap();

        let res = complete_stock_transfer_inner(&conn, 1).unwrap();
        assert!(res.contains("تم"));

        let status: String = conn
            .query_row("SELECT status FROM stock_transfers WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "Completed");

        // item now lives in destination warehouse; global qty unchanged
        let wh: Option<i64> = conn
            .query_row("SELECT warehouse_id FROM inventory_items WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(wh, Some(2));
        let qty: f64 = conn
            .query_row("SELECT qty_on_hand FROM inventory_items WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert!((qty - 50.0).abs() < 1e-9);

        let (mov_in, mov_out): (f64, f64) = conn
            .query_row("SELECT qty_in, qty_out FROM inventory_movements WHERE item_id=1 AND mtype='transfer'", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert!((mov_in - 10.0).abs() < 1e-9);
        assert!((mov_out - 10.0).abs() < 1e-9);
    }

    #[test]
    fn complete_transfer_rejects_shortage() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO stock_transfers(id, transfer_no, from_warehouse_id, to_warehouse_id, item_id, qty, status) VALUES(1, 'ST-0001', 1, 2, 1, 500, 'Pending')",
            [],
        )
        .unwrap();

        let err = complete_stock_transfer_inner(&conn, 1).unwrap_err();
        assert!(err.to_string().contains("غير كافٍ"));

        let status: String = conn
            .query_row("SELECT status FROM stock_transfers WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "Pending");
    }

    #[test]
    fn complete_transfer_twice_is_rejected() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO stock_transfers(id, transfer_no, from_warehouse_id, to_warehouse_id, item_id, qty, status) VALUES(1, 'ST-0001', 1, 2, 1, 10, 'Completed')",
            [],
        )
        .unwrap();
        let err = complete_stock_transfer_inner(&conn, 1).unwrap_err();
        assert!(err.to_string().contains("مكتمل"));
    }
}
