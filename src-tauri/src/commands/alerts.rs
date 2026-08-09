use crate::db::DbState;
use crate::error::AppError;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
struct ExpiryAlert {
    product_name: String,
    expiry_date: String,
    batch: Option<String>,
}

#[derive(Debug, Serialize)]
struct OverdueOrderAlert {
    order_id: i64,
    product_name: String,
    due_date: String,
}

#[derive(Debug, Serialize)]
struct LowStockAlert {
    product_name: String,
    current_stock: f64,
    min_stock: f64,
}

#[derive(Debug, Serialize)]
struct OverdueInvoiceAlert {
    invoice_id: i64,
    invoice_number: String,
    due_date: String,
    amount: i64,
}

#[derive(Debug, Serialize)]
struct QualityPendingAlert {
    batch_id: i64,
    product_name: String,
    created_at: String,
}

#[derive(Debug, Serialize, Default)]
pub struct AlertsData {
    expiry: Vec<ExpiryAlert>,
    overdue_orders: Vec<OverdueOrderAlert>,
    low_stock: Vec<LowStockAlert>,
    overdue_invoices: Vec<OverdueInvoiceAlert>,
    quality_pending: Vec<QualityPendingAlert>,
}

fn check_expiry(conn: &rusqlite::Connection) -> Vec<ExpiryAlert> {
    let mut alerts = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT name, expiry_date, authority, COALESCE(alert_days, 30)
         FROM renewals WHERE status='active'
         AND expiry_date IS NOT NULL AND expiry_date != ''
         AND expiry_date <= date('now', printf('+%d days', COALESCE(alert_days, 30)))
         ORDER BY expiry_date ASC",
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            alerts.push(ExpiryAlert { product_name: r.0, expiry_date: r.1, batch: r.2 });
        }
    }
    alerts
}

fn check_overdue_orders(conn: &rusqlite::Connection) -> Vec<OverdueOrderAlert> {
    let mut alerts = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT po.id, po.date, COALESCE(p.name_ar, p.name_en, po.prod_no, '')
         FROM production_orders po
         LEFT JOIN production_lines pl ON pl.order_id = po.id
         LEFT JOIN products p ON p.id = pl.product_id
         WHERE (po.status='Draft' AND po.date <= date('now', '-7 days'))
            OR (po.status='Approved' AND po.downtime_minutes > 120 AND po.downtime_minutes > po.run_minutes)
         GROUP BY po.id
         ORDER BY po.date ASC",
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            alerts.push(OverdueOrderAlert { order_id: r.0, product_name: r.2, due_date: r.1 });
        }
    }
    alerts
}

fn check_low_stock(conn: &rusqlite::Connection) -> Vec<LowStockAlert> {
    let mut alerts = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT name_ar, name_en, qty_on_hand, reorder_level
         FROM inventory_items WHERE active=1 AND reorder_level > 0 AND qty_on_hand <= reorder_level
         ORDER BY (qty_on_hand / reorder_level) ASC",
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            let name = r.0.or(r.1).unwrap_or_else(|| "غير معروف".to_string());
            alerts.push(LowStockAlert { product_name: name, current_stock: r.2, min_stock: r.3 });
        }
    }
    alerts
}

fn check_overdue_invoices(conn: &rusqlite::Connection) -> Vec<OverdueInvoiceAlert> {
    let mut alerts = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT si.id, si.inv_no, si.date, si.total_milli - si.paid_milli
         FROM sales_invoices si JOIN customers c ON c.id = si.customer_id
         WHERE si.status IN ('Issued','Partially Paid','Posted')
           AND si.total_milli > si.paid_milli AND si.date <= date('now')
         ORDER BY si.date ASC",
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            if r.3 <= 0 {
                continue;
            }
            alerts.push(OverdueInvoiceAlert {
                invoice_id: r.0,
                invoice_number: r.1.unwrap_or_default(),
                due_date: r.2,
                amount: r.3,
            });
        }
    }
    alerts
}

fn check_quality_pending(conn: &rusqlite::Connection) -> Vec<QualityPendingAlert> {
    let mut alerts = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT qi.id, qi.date, COALESCE(p.name_ar, p.name_en, '')
         FROM quality_inspections qi
         LEFT JOIN production_lines pl ON pl.id = qi.production_line_id
         LEFT JOIN products p ON p.id = pl.product_id
         WHERE qi.status != 'Passed'
         ORDER BY qi.date DESC LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            alerts.push(QualityPendingAlert { batch_id: r.0, product_name: r.2, created_at: r.1 });
        }
    }
    alerts
}

#[tauri::command]
pub fn get_all_alerts(state: State<'_, DbState>) -> Result<AlertsData, AppError> {
    let conn = state.0.lock()?;
    let data = AlertsData {
        expiry: check_expiry(&conn),
        overdue_orders: check_overdue_orders(&conn),
        low_stock: check_low_stock(&conn),
        overdue_invoices: check_overdue_invoices(&conn),
        quality_pending: check_quality_pending(&conn),
    };
    Ok(data)
}
