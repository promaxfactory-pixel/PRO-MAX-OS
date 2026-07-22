use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Purchase {
    pub id: i64,
    pub pur_no: Option<String>,
    pub date: String,
    pub supplier_id: i64,
    pub supplier_name: Option<String>,
    pub supplier_invoice_no: Option<String>,
    pub vat_enabled: i64,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub total_milli: i64,
    pub paid_milli: i64,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseLine {
    pub id: i64,
    pub purchase_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub qty: f64,
    pub unit_cost_milli: i64,
    pub line_net_milli: i64,
    pub vat_pct: f64,
    pub vat_milli: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseInput {
    pub supplier_id: i64,
    pub date: String,
    pub supplier_invoice_no: Option<String>,
    pub vat_enabled: Option<bool>,
    pub notes: Option<String>,
    pub lines: Vec<CreatePurchaseLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseLineInput {
    pub item_id: i64,
    pub qty: f64,
    pub unit_cost_milli: i64,
    pub vat_pct: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSupplierPaymentInput {
    pub supplier_id: i64,
    pub date: String,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_purchases(state: State<'_, DbState>) -> Result<Vec<Purchase>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.pur_no, p.date, p.supplier_id, s.name AS supplier_name,
                    p.supplier_invoice_no, p.vat_enabled, p.net_milli, p.vat_milli,
                    p.total_milli, p.paid_milli, p.status, p.notes, p.created_by, p.created_at
             FROM purchases p
             LEFT JOIN suppliers s ON s.id = p.supplier_id
             ORDER BY p.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(Purchase {
            id: row.get(0)?,
            pur_no: row.get(1)?,
            date: row.get(2)?,
            supplier_id: row.get(3)?,
            supplier_name: row.get(4)?,
            supplier_invoice_no: row.get(5)?,
            vat_enabled: row.get(6)?,
            net_milli: row.get(7)?,
            vat_milli: row.get(8)?,
            total_milli: row.get(9)?,
            paid_milli: row.get(10)?,
            status: row.get(11)?,
            notes: row.get(12)?,
            created_by: row.get(13)?,
            created_at: row.get(14)?,
        })
    })
    .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_purchase(id: i64, state: State<'_, DbState>) -> Result<Purchase, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.pur_no, p.date, p.supplier_id, s.name AS supplier_name,
                    p.supplier_invoice_no, p.vat_enabled, p.net_milli, p.vat_milli,
                    p.total_milli, p.paid_milli, p.status, p.notes, p.created_by, p.created_at
             FROM purchases p
             LEFT JOIN suppliers s ON s.id = p.supplier_id
             WHERE p.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let purchase = stmt
        .query_row([id], |row| {
            Ok(Purchase {
                id: row.get(0)?,
                pur_no: row.get(1)?,
                date: row.get(2)?,
                supplier_id: row.get(3)?,
                supplier_name: row.get(4)?,
                supplier_invoice_no: row.get(5)?,
                vat_enabled: row.get(6)?,
                net_milli: row.get(7)?,
                vat_milli: row.get(8)?,
                total_milli: row.get(9)?,
                paid_milli: row.get(10)?,
                status: row.get(11)?,
                notes: row.get(12)?,
                created_by: row.get(13)?,
                created_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(purchase)
}

#[tauri::command]
pub fn get_purchase_lines(purchase_id: i64, state: State<'_, DbState>) -> Result<Vec<PurchaseLine>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT pl.id, pl.purchase_id, pl.item_id, i.name AS item_name,
                    pl.qty, pl.unit_cost_milli, pl.line_net_milli, pl.vat_pct, pl.vat_milli
             FROM purchase_lines pl
             LEFT JOIN inventory_items i ON i.id = pl.item_id
             WHERE pl.purchase_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([purchase_id], |row| {
        Ok(PurchaseLine {
            id: row.get(0)?,
            purchase_id: row.get(1)?,
            item_id: row.get(2)?,
            item_name: row.get(3)?,
            qty: row.get(4)?,
            unit_cost_milli: row.get(5)?,
            line_net_milli: row.get(6)?,
            vat_pct: row.get(7)?,
            vat_milli: row.get(8)?,
        })
    })
    .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_purchase(input: CreatePurchaseInput, state: State<'_, DbState>) -> Result<i64, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let year: String = tx
        .query_row("SELECT substr(?1, 1, 4)", [&input.date], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let next_num: i64 = tx
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type=? AND year=?",
            ["PUR", &year],
            |row| row.get(0),
        )
        .unwrap_or(1);

    tx.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES(?,?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        ["PUR", &year, &next_num.to_string()],
    )
    .map_err(|e| e.to_string())?;

    let pur_no = format!("PUR-{}-{:04}", year, next_num);
    let vat_enabled: i64 = if input.vat_enabled.unwrap_or(false) { 1 } else { 0 };

    let mut net_milli: i64 = 0;
    let mut vat_milli: i64 = 0;
    for line in &input.lines {
        let line_vat = ((line.unit_cost_milli as f64) * (line.qty) * (line.vat_pct.unwrap_or(0.0)) / 100.0) as i64;
        let line_net = ((line.unit_cost_milli as f64) * (line.qty)) as i64;
        net_milli += line_net;
        vat_milli += line_vat;
    }
    let total_milli = net_milli + vat_milli;

    tx.execute(
        "INSERT INTO purchases(pur_no, date, supplier_id, supplier_invoice_no, vat_enabled, net_milli, vat_milli, total_milli, paid_milli, status, notes)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 'draft', ?9)",
        rusqlite::params![
            pur_no,
            input.date,
            input.supplier_id,
            input.supplier_invoice_no,
            vat_enabled,
            net_milli,
            vat_milli,
            total_milli,
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;

    let purchase_id: i64 = tx.last_insert_rowid();

    for line in &input.lines {
        let line_vat = ((line.unit_cost_milli as f64) * line.qty * (line.vat_pct.unwrap_or(0.0)) / 100.0) as i64;
        let line_net = ((line.unit_cost_milli as f64) * line.qty) as i64;

        tx.execute(
            "INSERT INTO purchase_lines(purchase_id, item_id, qty, unit_cost_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                purchase_id,
                line.item_id,
                line.qty,
                line.unit_cost_milli,
                line_net,
                line.vat_pct.unwrap_or(0.0),
                line_vat,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    let _ = rbac::log_audit(&*tx, None, None, "create_purchase", "purchases", Some(purchase_id), None, Some(&pur_no), None);
    tx.commit().map_err(|e| e.to_string())?;
    Ok(purchase_id)
}

#[tauri::command]
pub fn list_suppliers_for_select(state: State<'_, DbState>) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name FROM suppliers ORDER BY name")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
        }))
    })
    .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_supplier_payment(input: CreateSupplierPaymentInput, state: State<'_, DbState>) -> Result<i64, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO supplier_payments(supplier_id, date, amount_milli, method, reference, notes)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            input.supplier_id,
            input.date,
            input.amount_milli,
            input.method,
            input.reference,
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;

    let payment_id: i64 = tx.last_insert_rowid();

    tx.execute(
        "UPDATE purchases SET paid_milli = paid_milli + ?1 WHERE supplier_id = ?2 AND status != 'Void'",
        [input.amount_milli, input.supplier_id],
    )
    .map_err(|e| e.to_string())?;

    let _ = rbac::log_audit(&*tx, None, None, "create_supplier_payment", "supplier_payments", Some(payment_id), None, None, None);
    tx.commit().map_err(|e| e.to_string())?;
    Ok(payment_id)
}
