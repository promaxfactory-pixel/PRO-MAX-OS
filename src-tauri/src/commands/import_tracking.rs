use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportShipment {
    pub id: i64,
    pub shipment_no: Option<String>,
    pub supplier_id: Option<i64>,
    pub supplier_name: Option<String>,
    pub currency: Option<String>,
    pub exchange_rate: f64,
    pub status: Option<String>,
    pub shipping_company: Option<String>,
    pub container_no: Option<String>,
    pub bl_no: Option<String>,
    pub vessel_flight: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub estimated_arrival: Option<String>,
    pub actual_arrival: Option<String>,
    pub customs_declaration_no: Option<String>,
    pub customs_clearance_date: Option<String>,
    pub duty_amount_milli: i64,
    pub vat_on_import_milli: i64,
    pub freight_cost_milli: i64,
    pub insurance_cost_milli: i64,
    pub handling_cost_milli: i64,
    pub commercial_invoice_no: Option<String>,
    pub packing_list_no: Option<String>,
    pub origin_country: Option<String>,
    pub gross_weight_kg: f64,
    pub cbm: f64,
    pub clearance_agent: Option<String>,
    pub total_landed_cost_milli: i64,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShipmentInput {
    pub supplier_id: Option<i64>,
    pub currency: Option<String>,
    pub exchange_rate: Option<f64>,
    pub shipping_company: Option<String>,
    pub container_no: Option<String>,
    pub bl_no: Option<String>,
    pub vessel_flight: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub estimated_arrival: Option<String>,
    pub commercial_invoice_no: Option<String>,
    pub packing_list_no: Option<String>,
    pub origin_country: Option<String>,
    pub gross_weight_kg: Option<f64>,
    pub cbm: Option<f64>,
    pub clearance_agent: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShipmentInput {
    pub supplier_id: Option<i64>,
    pub currency: Option<String>,
    pub exchange_rate: Option<f64>,
    pub shipping_company: Option<String>,
    pub container_no: Option<String>,
    pub bl_no: Option<String>,
    pub vessel_flight: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub estimated_arrival: Option<String>,
    pub commercial_invoice_no: Option<String>,
    pub packing_list_no: Option<String>,
    pub origin_country: Option<String>,
    pub gross_weight_kg: Option<f64>,
    pub cbm: Option<f64>,
    pub clearance_agent: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShipmentStatusInput {
    pub status: String,
    pub customs_declaration_no: Option<String>,
    pub actual_arrival: Option<String>,
    pub customs_clearance_date: Option<String>,
    pub duty_amount_milli: Option<i64>,
    pub vat_on_import_milli: Option<i64>,
    pub total_landed_cost_milli: Option<i64>,
}

const SHIPMENT_COLUMNS: &str = "s.id, s.shipment_no, s.supplier_id, sp.name AS supplier_name, s.currency, s.exchange_rate, s.status, s.shipping_company, s.container_no, s.bl_no, s.vessel_flight, s.port_of_loading, s.port_of_discharge, s.estimated_arrival, s.actual_arrival, s.customs_declaration_no, s.customs_clearance_date, s.duty_amount_milli, s.vat_on_import_milli, s.freight_cost_milli, s.insurance_cost_milli, s.handling_cost_milli, s.commercial_invoice_no, s.packing_list_no, s.origin_country, s.gross_weight_kg, s.cbm, s.clearance_agent, s.total_landed_cost_milli, s.notes, s.created_by, s.created_at";

#[tauri::command]
pub fn list_shipments(state: State<'_, DbState>) -> Result<Vec<ImportShipment>, AppError> {
    let conn = state.0.lock()?;
    let sql = format!(
        "SELECT {} FROM import_shipments s LEFT JOIN suppliers sp ON sp.id = s.supplier_id ORDER BY s.id DESC",
        SHIPMENT_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ImportShipment {
                id: row.get(0)?,
                shipment_no: row.get(1)?,
                supplier_id: row.get(2)?,
                supplier_name: row.get(3)?,
                currency: row.get(4)?,
                exchange_rate: row.get(5)?,
                status: row.get(6)?,
                shipping_company: row.get(7)?,
                container_no: row.get(8)?,
                bl_no: row.get(9)?,
                vessel_flight: row.get(10)?,
                port_of_loading: row.get(11)?,
                port_of_discharge: row.get(12)?,
                estimated_arrival: row.get(13)?,
                actual_arrival: row.get(14)?,
                customs_declaration_no: row.get(15)?,
                customs_clearance_date: row.get(16)?,
                duty_amount_milli: row.get(17)?,
                vat_on_import_milli: row.get(18)?,
                freight_cost_milli: row.get(19)?,
                insurance_cost_milli: row.get(20)?,
                handling_cost_milli: row.get(21)?,
                commercial_invoice_no: row.get(22)?,
                packing_list_no: row.get(23)?,
                origin_country: row.get(24)?,
                gross_weight_kg: row.get(25)?,
                cbm: row.get(26)?,
                clearance_agent: row.get(27)?,
                total_landed_cost_milli: row.get(28)?,
                notes: row.get(29)?,
                created_by: row.get(30)?,
                created_at: row.get(31)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn get_shipment(state: State<'_, DbState>, id: i64) -> Result<ImportShipment, AppError> {
    let conn = state.0.lock()?;
    let sql = format!(
        "SELECT {} FROM import_shipments s LEFT JOIN suppliers sp ON sp.id = s.supplier_id WHERE s.id=?",
        SHIPMENT_COLUMNS
    );
    Ok(conn.query_row(&sql, [id], |row| {
        Ok(ImportShipment {
            id: row.get(0)?,
            shipment_no: row.get(1)?,
            supplier_id: row.get(2)?,
            supplier_name: row.get(3)?,
            currency: row.get(4)?,
            exchange_rate: row.get(5)?,
            status: row.get(6)?,
            shipping_company: row.get(7)?,
            container_no: row.get(8)?,
            bl_no: row.get(9)?,
            vessel_flight: row.get(10)?,
            port_of_loading: row.get(11)?,
            port_of_discharge: row.get(12)?,
            estimated_arrival: row.get(13)?,
            actual_arrival: row.get(14)?,
            customs_declaration_no: row.get(15)?,
            customs_clearance_date: row.get(16)?,
            duty_amount_milli: row.get(17)?,
            vat_on_import_milli: row.get(18)?,
            freight_cost_milli: row.get(19)?,
            insurance_cost_milli: row.get(20)?,
            handling_cost_milli: row.get(21)?,
            commercial_invoice_no: row.get(22)?,
            packing_list_no: row.get(23)?,
            origin_country: row.get(24)?,
            gross_weight_kg: row.get(25)?,
            cbm: row.get(26)?,
            clearance_agent: row.get(27)?,
            total_landed_cost_milli: row.get(28)?,
            notes: row.get(29)?,
            created_by: row.get(30)?,
            created_at: row.get(31)?,
        })
    })?)
}

#[tauri::command]
pub fn create_shipment(
    state: State<'_, DbState>,
    input: CreateShipmentInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='IMP' AND year=?",
            [&year],
            |r| r.get(0),
        )
        .unwrap_or(1);
    conn.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('IMP',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    ).map_err(|e| format!("Failed to increment import shipment sequence: {}", e))?;
    let shipment_no = format!("IMP-{}-{:04}", year, seq);

    conn.execute(
        "INSERT INTO import_shipments(shipment_no, supplier_id, currency, exchange_rate, status, shipping_company, container_no, bl_no, vessel_flight, port_of_loading, port_of_discharge, estimated_arrival, commercial_invoice_no, packing_list_no, origin_country, gross_weight_kg, cbm, clearance_agent, notes, created_by, created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,datetime('now'))",
        rusqlite::params![
            shipment_no,
            input.supplier_id,
            input.currency.unwrap_or_else(|| "USD".to_string()),
            input.exchange_rate.unwrap_or(1.0),
            "Ordered",
            input.shipping_company,
            input.container_no,
            input.bl_no,
            input.vessel_flight,
            input.port_of_loading,
            input.port_of_discharge,
            input.estimated_arrival,
            input.commercial_invoice_no,
            input.packing_list_no,
            input.origin_country.unwrap_or_else(|| "China".to_string()),
            input.gross_weight_kg.unwrap_or(0.0),
            input.cbm.unwrap_or(0.0),
            input.clearance_agent,
            input.notes,
            input.supplier_id.map(|_| "".to_string()),
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_shipment", "import_shipments", Some(id), None, Some(&shipment_no), None);
    Ok(id)
}

#[tauri::command]
pub fn update_shipment(
    state: State<'_, DbState>,
    id: i64,
    input: UpdateShipmentInput,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let mut sets = Vec::new();
    let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = input.supplier_id {
        sets.push("supplier_id=?");
        p.push(Box::new(v));
    }
    if let Some(v) = &input.currency {
        sets.push("currency=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = input.exchange_rate {
        sets.push("exchange_rate=?");
        p.push(Box::new(v));
    }
    if let Some(v) = &input.shipping_company {
        sets.push("shipping_company=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.container_no {
        sets.push("container_no=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.bl_no {
        sets.push("bl_no=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.vessel_flight {
        sets.push("vessel_flight=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.port_of_loading {
        sets.push("port_of_loading=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.port_of_discharge {
        sets.push("port_of_discharge=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.estimated_arrival {
        sets.push("estimated_arrival=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.commercial_invoice_no {
        sets.push("commercial_invoice_no=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.packing_list_no {
        sets.push("packing_list_no=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.origin_country {
        sets.push("origin_country=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = input.gross_weight_kg {
        sets.push("gross_weight_kg=?");
        p.push(Box::new(v));
    }
    if let Some(v) = input.cbm {
        sets.push("cbm=?");
        p.push(Box::new(v));
    }
    if let Some(v) = &input.clearance_agent {
        sets.push("clearance_agent=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.notes {
        sets.push("notes=?");
        p.push(Box::new(v.clone()));
    }

    if sets.is_empty() {
        return Err(AppError::validation("No changes provided"));
    }

    p.push(Box::new(id));
    let sql = format!("UPDATE import_shipments SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(p.iter()))?;
    let _ = rbac::log_audit(&conn, None, None, "update_shipment", "import_shipments", Some(id), None, None, None);
    Ok("Updated successfully".to_string())
}

#[tauri::command]
pub fn update_shipment_status(
    state: State<'_, DbState>,
    id: i64,
    input: UpdateShipmentStatusInput,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let mut sets = vec!["status=?"];
    let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    p.push(Box::new(input.status));

    if let Some(v) = &input.customs_declaration_no {
        sets.push("customs_declaration_no=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.actual_arrival {
        sets.push("actual_arrival=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.customs_clearance_date {
        sets.push("customs_clearance_date=?");
        p.push(Box::new(v.clone()));
    }
    if let Some(v) = input.duty_amount_milli {
        sets.push("duty_amount_milli=?");
        p.push(Box::new(v));
    }
    if let Some(v) = input.vat_on_import_milli {
        sets.push("vat_on_import_milli=?");
        p.push(Box::new(v));
    }
    if let Some(v) = input.total_landed_cost_milli {
        sets.push("total_landed_cost_milli=?");
        p.push(Box::new(v));
    }

    p.push(Box::new(id));
    let sql = format!("UPDATE import_shipments SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(p.iter()))?;
    let _ = rbac::log_audit(&conn, None, None, "update_shipment_status", "import_shipments", Some(id), None, None, None);
    Ok("Status updated successfully".to_string())
}
