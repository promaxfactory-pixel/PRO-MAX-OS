use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub id: i64,
    pub prod_no: Option<String>,
    pub date: String,
    pub shift: Option<String>,
    pub machine_id: Option<i64>,
    pub operator: Option<String>,
    pub supervisor: Option<String>,
    pub run_minutes: i64,
    pub downtime_minutes: i64,
    pub status: String,
    pub notes: Option<String>,
    pub approved_by: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionLine {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub cups_per_carton: i64,
    pub cartons_good: f64,
    pub cups_good: f64,
    pub cartons_waste: f64,
    pub cups_waste: f64,
    pub unit_cost_milli: i64,
    pub worker: Option<String>,
    pub brand_type: String,
    pub customer_id: Option<i64>,
    pub batch_no: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderInput {
    pub date: String,
    pub shift: Option<String>,
    pub machine_id: Option<i64>,
    pub operator: Option<String>,
    pub supervisor: Option<String>,
    pub notes: Option<String>,
    pub lines: Option<Vec<CreateOrderLineInput>>,
    pub created_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderLineInput {
    pub product_id: i64,
    pub cups_per_carton: Option<i64>,
    pub cartons_good: f64,
    pub cartons_waste: f64,
    pub worker: Option<String>,
    pub brand_type: Option<String>,
    pub customer_id: Option<i64>,
    pub batch_no: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderInput {
    pub date: Option<String>,
    pub shift: Option<String>,
    pub machine_id: Option<i64>,
    pub operator: Option<String>,
    pub supervisor: Option<String>,
    pub run_minutes: Option<i64>,
    pub downtime_minutes: Option<i64>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddLineInput {
    pub order_id: i64,
    pub product_id: i64,
    pub cups_per_carton: Option<i64>,
    pub cartons_good: f64,
    pub cartons_waste: f64,
    pub worker: Option<String>,
    pub brand_type: Option<String>,
    pub customer_id: Option<i64>,
    pub batch_no: Option<String>,
}

#[tauri::command]
pub fn list_production_orders(
    state: State<'_, DbState>,
) -> Result<Vec<ProductionOrder>, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_PRODUCTION)?;
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT po.id, po.prod_no, po.date, po.shift, po.machine_id, po.operator, po.supervisor, po.run_minutes, po.downtime_minutes, po.status, po.notes, po.approved_by, po.created_by FROM production_orders po ORDER BY po.id DESC",
        )
        ?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ProductionOrder {
                id: row.get(0)?,
                prod_no: row.get(1)?,
                date: row.get(2)?,
                shift: row.get(3)?,
                machine_id: row.get(4)?,
                operator: row.get(5)?,
                supervisor: row.get(6)?,
                run_minutes: row.get(7)?,
                downtime_minutes: row.get(8)?,
                status: row.get(9)?,
                notes: row.get(10)?,
                approved_by: row.get(11)?,
                created_by: row.get(12)?,
            })
        })
        ?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn get_production_order(
    state: State<'_, DbState>,
    id: i64,
) -> Result<ProductionOrder, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT po.id, po.prod_no, po.date, po.shift, po.machine_id, po.operator, po.supervisor, po.run_minutes, po.downtime_minutes, po.status, po.notes, po.approved_by, po.created_by FROM production_orders po WHERE po.id=?",
        [id],
        |row| {
            Ok(ProductionOrder {
                id: row.get(0)?,
                prod_no: row.get(1)?,
                date: row.get(2)?,
                shift: row.get(3)?,
                machine_id: row.get(4)?,
                operator: row.get(5)?,
                supervisor: row.get(6)?,
                run_minutes: row.get(7)?,
                downtime_minutes: row.get(8)?,
                status: row.get(9)?,
                notes: row.get(10)?,
                approved_by: row.get(11)?,
                created_by: row.get(12)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn create_production_order(
    state: State<'_, DbState>,
    input: CreateOrderInput,
) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq: i64 = tx
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='PROD' AND year=?",
            [&year],
            |r| r.get(0),
        )
        .unwrap_or(1);
    tx.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('PROD',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    )
    .map_err(|e| format!("Failed to increment production order sequence: {}", e))?;
    let prod_no = format!("PROD-{}-{:04}", year, seq);

    tx.execute(
        "INSERT INTO production_orders(prod_no, date, shift, machine_id, operator, supervisor, status, notes, created_by) VALUES(?,?,?,?,?,?,?,'Draft',?)",
        rusqlite::params![
            prod_no,
            input.date,
            input.shift,
            input.machine_id,
            input.operator,
            input.supervisor,
            input.notes,
            input.created_by,
        ],
    )
    ?;
    let order_id = tx.last_insert_rowid();

    if let Some(lines) = &input.lines {
        for line in lines {
            tx.execute(
                "INSERT INTO production_lines(order_id, product_id, cups_per_carton, cartons_good, cartons_waste, worker, brand_type, customer_id, batch_no) VALUES(?,?,?,?,?,?,?,?,?)",
                rusqlite::params![
                    order_id,
                    line.product_id,
                    line.cups_per_carton.unwrap_or(1000),
                    line.cartons_good,
                    line.cartons_waste,
                    line.worker,
                    line.brand_type.clone().unwrap_or_else(|| "factory".into()),
                    line.customer_id,
                    line.batch_no,
                ],
            )?;
        }
    }

    let _ = rbac::log_audit(&*tx, None, None, "create_production_order", "production_orders", Some(order_id), None, Some(&prod_no), None);
    tx.commit()?;
    Ok(order_id)
}

#[tauri::command]
pub fn update_production_order(
    state: State<'_, DbState>,
    id: i64,
    input: UpdateOrderInput,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.date {
        sets.push("date=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.shift {
        sets.push("shift=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.machine_id {
        sets.push("machine_id=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.operator {
        sets.push("operator=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.supervisor {
        sets.push("supervisor=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.run_minutes {
        sets.push("run_minutes=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.downtime_minutes {
        sets.push("downtime_minutes=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.status {
        sets.push("status=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.notes {
        sets.push("notes=?");
        params.push(Box::new(v.clone()));
    }

    if sets.is_empty() {
        return Err(AppError::validation("No changes provided"));
    }

    params.push(Box::new(id));
    let sql = format!(
        "UPDATE production_orders SET {} WHERE id=?",
        sets.join(", ")
    );
    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
        ?;
    let _ = rbac::log_audit(&conn, None, None, "update_production_order", "production_orders", Some(id), None, None, None);
    Ok("Updated successfully".to_string())
}

#[tauri::command]
pub fn approve_production_order(
    state: State<'_, DbState>,
    id: i64,
    approved_by: String,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    conn.execute(
        "UPDATE production_orders SET status='Approved', approved_by=?, approved_at=? WHERE id=? AND status='Draft'",
        rusqlite::params![approved_by, now, id],
    )
    ?;
    let _ = rbac::log_audit(&conn, None, None, "approve_production_order", "production_orders", Some(id), None, None, None);
    Ok("Approved".to_string())
}

#[tauri::command]
pub fn get_production_lines(
    state: State<'_, DbState>,
    order_id: i64,
) -> Result<Vec<ProductionLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT pl.id, pl.order_id, pl.product_id, p.name_ar, pl.cups_per_carton, pl.cartons_good, pl.cups_good, pl.cartons_waste, pl.cups_waste, pl.unit_cost_milli, pl.worker, pl.brand_type, pl.customer_id, pl.batch_no FROM production_lines pl LEFT JOIN products p ON pl.product_id=p.id WHERE pl.order_id=? ORDER BY pl.id",
        )
        ?;
    let rows = stmt
        .query_map([order_id], |row| {
            Ok(ProductionLine {
                id: row.get(0)?,
                order_id: row.get(1)?,
                product_id: row.get(2)?,
                product_name: row.get(3)?,
                cups_per_carton: row.get(4)?,
                cartons_good: row.get(5)?,
                cups_good: row.get(6)?,
                cartons_waste: row.get(7)?,
                cups_waste: row.get(8)?,
                unit_cost_milli: row.get(9)?,
                worker: row.get(10)?,
                brand_type: row.get(11)?,
                customer_id: row.get(12)?,
                batch_no: row.get(13)?,
            })
        })
        ?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn add_production_line(
    state: State<'_, DbState>,
    input: AddLineInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    conn.execute(
        "INSERT INTO production_lines(order_id, product_id, cups_per_carton, cartons_good, cartons_waste, worker, brand_type, customer_id, batch_no) VALUES(?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            input.order_id,
            input.product_id,
            input.cups_per_carton.unwrap_or(1000),
            input.cartons_good,
            input.cartons_waste,
            input.worker,
            input.brand_type.unwrap_or_else(|| "factory".into()),
            input.customer_id,
            input.batch_no,
        ],
    )
    ?;
    let line_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "add_production_line", "production_lines", Some(line_id), None, None, None);
    Ok(line_id)
}
