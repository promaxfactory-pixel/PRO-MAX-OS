use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct BarterExchange {
    pub id: i64,
    pub exchange_no: Option<String>,
    pub date: Option<String>,
    pub local_supplier_id: i64,
    pub supplier_name: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub cartons_given: f64,
    pub carton_value_milli: i64,
    pub received_item_id: Option<i64>,
    pub received_item_name: Option<String>,
    pub bags_received: f64,
    pub bag_value_milli: i64,
    pub net_value_milli: i64,
    pub balance_milli: i64,
    pub settlement_status: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBarterInput {
    pub local_supplier_id: i64,
    pub product_id: Option<i64>,
    pub cartons_given: Option<f64>,
    pub carton_value_milli: Option<i64>,
    pub received_item_id: Option<i64>,
    pub bags_received: Option<f64>,
    pub bag_value_milli: Option<i64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

const BARTER_COLUMNS: &str = "e.id, e.exchange_no, e.date, e.local_supplier_id, sp.name AS supplier_name, e.product_id, COALESCE(p.name_ar, p.name_en, '') AS product_name, e.cartons_given, e.carton_value_milli, e.received_item_id, COALESCE(ii.name_ar, ii.name_en, '') AS received_item_name, e.bags_received, e.bag_value_milli, e.net_value_milli, e.balance_milli, e.settlement_status, e.reference, e.notes, e.status, e.created_by, e.created_at";

#[tauri::command]
pub fn list_barter_exchanges(state: State<'_, DbState>) -> Result<Vec<BarterExchange>, AppError> {
    let conn = state.0.lock()?;
    let sql = format!(
        "SELECT {} FROM local_supplier_exchanges e
         LEFT JOIN suppliers sp ON sp.id = e.local_supplier_id
         LEFT JOIN products p ON p.id = e.product_id
         LEFT JOIN inventory_items ii ON ii.id = e.received_item_id
         ORDER BY e.id DESC",
        BARTER_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BarterExchange {
                id: row.get(0)?,
                exchange_no: row.get(1)?,
                date: row.get(2)?,
                local_supplier_id: row.get(3)?,
                supplier_name: row.get(4)?,
                product_id: row.get(5)?,
                product_name: row.get(6)?,
                cartons_given: row.get(7)?,
                carton_value_milli: row.get(8)?,
                received_item_id: row.get(9)?,
                received_item_name: row.get(10)?,
                bags_received: row.get(11)?,
                bag_value_milli: row.get(12)?,
                net_value_milli: row.get(13)?,
                balance_milli: row.get(14)?,
                settlement_status: row.get(15)?,
                reference: row.get(16)?,
                notes: row.get(17)?,
                status: row.get(18)?,
                created_by: row.get(19)?,
                created_at: row.get(20)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_barter_exchange(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateBarterInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq = next_sequence(&conn, "BTY", &year)?;
    let exchange_no = format!("BTY-{}-{:04}", year, seq);

    let cartons = input.cartons_given.unwrap_or(0.0);
    let carton_val = input.carton_value_milli.unwrap_or(0);
    let bags = input.bags_received.unwrap_or(0.0);
    let bag_val = input.bag_value_milli.unwrap_or(0);
    let carton_total = (cartons * carton_val as f64) as i64;
    let bag_total = (bags * bag_val as f64) as i64;
    let net_value = carton_total - bag_total;

    conn.execute(
        "INSERT INTO local_supplier_exchanges(exchange_no, date, local_supplier_id, product_id, cartons_given, carton_value_milli, received_item_id, bags_received, bag_value_milli, net_value_milli, balance_milli, reference, notes, status, created_by, created_at) VALUES(?,date('now'),?,?,?,?,?,?,?,?,?,?,?,?, 'Draft', '', datetime('now'))",
        rusqlite::params![
            exchange_no,
            input.local_supplier_id,
            input.product_id,
            cartons,
            carton_val,
            input.received_item_id,
            bags,
            bag_val,
            net_value,
            net_value,
            input.reference,
            input.notes,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_barter_exchange", "local_supplier_exchanges", Some(id), None, Some(&exchange_no), None);
    Ok(id)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BarterBalance {
    pub local_supplier_id: i64,
    pub supplier_name: Option<String>,
    pub total_net_value_milli: i64,
    pub total_balance_milli: i64,
    pub open_exchanges: i64,
}

#[tauri::command]
pub fn get_barter_balance(
    state: State<'_, DbState>,
    local_supplier_id: i64,
) -> Result<BarterBalance, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT e.local_supplier_id,
                sp.name,
                COALESCE(SUM(e.net_value_milli), 0),
                COALESCE(SUM(e.balance_milli), 0),
                COUNT(*)
         FROM local_supplier_exchanges e
         LEFT JOIN suppliers sp ON sp.id = e.local_supplier_id
         WHERE e.local_supplier_id = ?1 AND e.settlement_status != 'settled'
         GROUP BY e.local_supplier_id",
        params![local_supplier_id],
        |row| {
            Ok(BarterBalance {
                local_supplier_id: row.get(0)?,
                supplier_name: row.get(1)?,
                total_net_value_milli: row.get(2)?,
                total_balance_milli: row.get(3)?,
                open_exchanges: row.get(4)?,
            })
        },
    )?)
}
