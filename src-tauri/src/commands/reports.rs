use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;
use rusqlite::params;

// ============================================================
// Existing report functions (referenced by lib.rs)
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct LowStockItem {
    pub id: i64,
    pub name: Option<String>,
    pub qty_on_hand: f64,
    pub reorder_level: f64,
    pub shortage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerAgingItem {
    pub customer_id: i64,
    pub customer_name: String,
    pub total_due: i64,
    pub overdue_days: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesReportItem {
    pub product_name: String,
    pub qty_sold: f64,
    pub revenue_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VatReturnData {
    pub period: String,
    pub sales_vat_milli: i64,
    pub purchase_vat_milli: i64,
    pub net_vat_milli: i64,
}

#[tauri::command]
pub fn low_stock_report(state: State<'_, DbState>) -> Result<Vec<LowStockItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, COALESCE(name_ar, name_en, ''), qty_on_hand, reorder_level, reorder_level - qty_on_hand
         FROM inventory_items WHERE reorder_level > 0 AND qty_on_hand <= reorder_level
         ORDER BY (qty_on_hand - reorder_level) ASC"
    ).map_err(|e| e.to_string())?;
    let items = stmt.query_map([], |r| {
        Ok(LowStockItem {
            id: r.get(0)?, name: r.get(1)?,
            qty_on_hand: r.get(2)?, reorder_level: r.get(3)?,
            shortage: r.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(items)
}

#[tauri::command]
pub fn customers_aging(state: State<'_, DbState>) -> Result<Vec<CustomerAgingItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, COALESCE(SUM(si.total_milli - si.paid_milli), 0),
                CAST(julianday('now') - julianday(MAX(si.date)) AS INTEGER)
         FROM customers c
         LEFT JOIN sales_invoices si ON si.customer_id = c.id AND si.status IN ('Posted', 'Issued') AND si.total_milli > si.paid_milli
         WHERE c.active = 1
         GROUP BY c.id
         HAVING COALESCE(SUM(si.total_milli - si.paid_milli), 0) > 0
         ORDER BY SUM(si.total_milli - si.paid_milli) DESC"
    ).map_err(|e| e.to_string())?;
    let items = stmt.query_map([], |r| {
        Ok(CustomerAgingItem {
            customer_id: r.get(0)?, customer_name: r.get(1)?,
            total_due: r.get(2)?, overdue_days: r.get::<_, i64>(3).unwrap_or(0),
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(items)
}

#[tauri::command]
pub fn sales_report(state: State<'_, DbState>, date_from: String, date_to: String) -> Result<Vec<SalesReportItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(p.name_ar, p.name_en, ''), SUM(sil.cartons), SUM(sil.line_net_milli)
         FROM sales_invoice_lines sil
         JOIN sales_invoices si ON si.id = sil.invoice_id
         LEFT JOIN products p ON p.id = sil.product_id
         WHERE si.date >= ?1 AND si.date <= ?2 AND si.status IN ('Posted', 'Issued')
         GROUP BY sil.product_id
         ORDER BY SUM(sil.line_net_milli) DESC"
    ).map_err(|e| e.to_string())?;
    let items = stmt.query_map(params![date_from, date_to], |r| {
        Ok(SalesReportItem {
            product_name: r.get(0)?, qty_sold: r.get(1)?,
            revenue_milli: r.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(items)
}

#[tauri::command]
pub fn production_report(state: State<'_, DbState>, date_from: String, date_to: String) -> Result<Vec<ProductReportLine>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(p.name_ar, p.name_en, ''), psl.customer_brand,
                SUM(psl.cartons_produced), SUM(psl.cartons_produced * psl.cups_per_carton), SUM(psl.waste_cartons)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE ods.date >= ?1 AND ods.date <= ?2
         GROUP BY psl.product_id, psl.customer_brand
         ORDER BY SUM(psl.cartons_produced) DESC"
    ).map_err(|e| e.to_string())?;
    let items = stmt.query_map(params![date_from, date_to], |r| {
        Ok(ProductReportLine {
            product_name: r.get(0)?, customer_brand: r.get(1)?,
            total_cartons: r.get(2)?, total_cups: r.get(3)?,
            waste_cartons: r.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(items)
}

#[tauri::command]
pub fn vat_return(state: State<'_, DbState>, year: i32, month: i32) -> Result<VatReturnData, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let prefix = format!("{}-{:02}", year, month);
    let sales_vat: i64 = conn.query_row(
        "SELECT COALESCE(SUM(vat_milli), 0) FROM sales_invoices WHERE date LIKE ?1 AND status IN ('Posted', 'Issued')",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0);
    let purchase_vat: i64 = conn.query_row(
        "SELECT COALESCE(SUM(vat_milli), 0) FROM purchases WHERE date LIKE ?1 AND status = 'Posted'",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0);
    Ok(VatReturnData {
        period: format!("{}-{:02}", year, month),
        sales_vat_milli: sales_vat,
        purchase_vat_milli: purchase_vat,
        net_vat_milli: sales_vat - purchase_vat,
    })
}

#[tauri::command]
pub fn daily_factory_closing(state: State<'_, DbState>, date: String) -> Result<ComprehensiveDailyReport, String> {
    get_comprehensive_daily_report(state, date)
}

#[tauri::command]
pub fn owner_summary(state: State<'_, DbState>, date_from: String, date_to: String) -> Result<serde_json::Value, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let revenue: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE date >= ?1 AND date <= ?2 AND status IN ('Posted', 'Issued')",
        params![date_from, date_to], |r| r.get(0),
    ).unwrap_or(0);
    let expenses: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_milli), 0) FROM expenses WHERE date >= ?1 AND date <= ?2",
        params![date_from, date_to], |r| r.get(0),
    ).unwrap_or(0);
    let production: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date >= ?1 AND ods.date <= ?2",
        params![date_from, date_to], |r| r.get(0),
    ).unwrap_or(0.0);
    Ok(serde_json::json!({
        "date_from": date_from, "date_to": date_to,
        "revenue_milli": revenue, "expenses_milli": expenses,
        "net_profit_milli": revenue - expenses,
        "production_cartons": production,
    }))
}

#[tauri::command]
pub fn inventory_margin_report(state: State<'_, DbState>) -> Result<serde_json::Value, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let total_value: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CAST(qty_on_hand * default_cost_milli AS INTEGER)), 0) FROM inventory_items",
        [], |r| r.get(0),
    ).unwrap_or(0);
    let total_sales_value: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CAST(qty_on_hand * default_price_milli AS INTEGER)), 0) FROM inventory_items",
        [], |r| r.get(0),
    ).unwrap_or(0);
    Ok(serde_json::json!({
        "inventory_cost_milli": total_value,
        "inventory_sales_value_milli": total_sales_value,
        "potential_margin_milli": total_sales_value - total_value,
    }))
}

#[tauri::command]
pub fn sales_by_customer_report(state: State<'_, DbState>, date_from: String, date_to: String) -> Result<serde_json::Value, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT c.name, COUNT(si.id), COALESCE(SUM(si.total_milli), 0)
         FROM sales_invoices si
         JOIN customers c ON c.id = si.customer_id
         WHERE si.date >= ?1 AND si.date <= ?2 AND si.status IN ('Posted', 'Issued')
         GROUP BY si.customer_id ORDER BY SUM(si.total_milli) DESC"
    ).map_err(|e| e.to_string())?;
    let data: Vec<serde_json::Value> = stmt.query_map(params![date_from, date_to], |r| {
        Ok(serde_json::json!({
            "customer_name": r.get::<_, String>(0)?,
            "invoice_count": r.get::<_, i64>(1)?,
            "total_milli": r.get::<_, i64>(2)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(serde_json::json!({ "date_from": date_from, "date_to": date_to, "customers": data }))
}

#[tauri::command]
pub fn unpaid_invoices_report(state: State<'_, DbState>) -> Result<serde_json::Value, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT si.id, si.inv_no, c.name, si.date, si.total_milli, si.paid_milli, si.total_milli - si.paid_milli
         FROM sales_invoices si
         JOIN customers c ON c.id = si.customer_id
         WHERE si.total_milli > si.paid_milli AND si.status IN ('Posted', 'Issued')
         ORDER BY si.date ASC"
    ).map_err(|e| e.to_string())?;
    let data: Vec<serde_json::Value> = stmt.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?,
            "inv_no": r.get::<_, Option<String>>(1)?,
            "customer_name": r.get::<_, String>(2)?,
            "date": r.get::<_, String>(3)?,
            "total_milli": r.get::<_, i64>(4)?,
            "due_milli": r.get::<_, i64>(6)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(serde_json::json!({ "invoices": data, "total_due": data.iter().map(|d| d["due_milli"].as_i64().unwrap_or(0)).sum::<i64>() }))
}

// ============================================================
// New Production Reports
// ============================================================

#[derive(Debug, Serialize)]
pub struct ShiftReport {
    pub date: String,
    pub shift: String,
    pub sheet_id: i64,
    pub sheet_no: Option<String>,
    pub total_cartons: f64,
    pub total_cups: f64,
    pub waste_cartons: f64,
    pub status: String,
    pub completed_by: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DailyProductionReport {
    pub date: String,
    pub total_cartons: f64,
    pub total_cups: f64,
    pub waste_cartons: f64,
    pub morning_shift: Option<ShiftReport>,
    pub evening_shift: Option<ShiftReport>,
    pub by_product: Vec<ProductReportLine>,
}

#[derive(Debug, Serialize)]
pub struct ProductReportLine {
    pub product_name: String,
    pub customer_brand: Option<String>,
    pub total_cartons: f64,
    pub total_cups: f64,
    pub waste_cartons: f64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyProductionReport {
    pub year: i32,
    pub month: i32,
    pub total_days: i64,
    pub total_cartons: f64,
    pub total_cups: f64,
    pub waste_cartons: f64,
    pub morning_cartons: f64,
    pub evening_cartons: f64,
    pub daily_breakdown: Vec<DailyBriefReport>,
}

#[derive(Debug, Serialize)]
pub struct DailyBriefReport {
    pub date: String,
    pub cartons: f64,
    pub cups: f64,
}

#[derive(Debug, Serialize)]
pub struct ComprehensiveDailyReport {
    pub date: String,
    // Production
    pub production_cartons: f64,
    pub production_cups: f64,
    pub production_waste: f64,
    pub morning_cartons: f64,
    pub evening_cartons: f64,
    pub production_lines: Vec<ShiftLineReport>,
    // Sales
    pub sales_count: i64,
    pub sales_total_milli: i64,
    pub sales_vat_milli: i64,
    pub top_products: Vec<SalesProductReport>,
    // Inventory
    pub low_stock_items: Vec<LowStockReport>,
    pub inventory_value_milli: i64,
    // Financial summary
    pub net_profit_milli: i64,
}

#[derive(Debug, Serialize)]
pub struct ShiftLineReport {
    pub product_name: String,
    pub shift: String,
    pub cartons: f64,
    pub cups: f64,
    pub customer_brand: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SalesProductReport {
    pub product_name: String,
    pub qty_cartons: f64,
    pub revenue_milli: i64,
}

#[derive(Debug, Serialize)]
pub struct LowStockReport {
    pub name: Option<String>,
    pub qty_on_hand: f64,
    pub reorder_level: f64,
}

#[tauri::command]
pub fn get_daily_production_report(state: State<'_, DbState>, date: String) -> Result<DailyProductionReport, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let total_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let total_cups: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced * psl.cups_per_carton), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let waste_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.waste_cartons), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let get_shift = |shift: &str| -> Option<ShiftReport> {
        conn.query_row(
            "SELECT ods.id, ods.sheet_no, ods.date, ods.shift,
                    COALESCE(SUM(psl.cartons_produced), 0),
                    COALESCE(SUM(psl.cartons_produced * psl.cups_per_carton), 0),
                    COALESCE(SUM(psl.waste_cartons), 0),
                    ods.status, ods.completed_by, ods.completed_at, ods.created_at
             FROM operations_daily_sheets ods
             LEFT JOIN production_shift_lines psl ON psl.sheet_id = ods.id
             WHERE ods.date = ?1 AND ods.shift = ?2
             GROUP BY ods.id",
            params![date, shift],
            |r| Ok(ShiftReport {
                date: r.get(2)?, shift: r.get(3)?,
                sheet_id: r.get(0)?, sheet_no: r.get(1)?,
                total_cartons: r.get(4)?, total_cups: r.get(5)?,
                waste_cartons: r.get(6)?, status: r.get(7)?,
                completed_by: r.get(8)?, completed_at: r.get(9)?,
                created_at: r.get(10)?,
            }),
        ).ok()
    };

    let morning_shift = get_shift("صباحي");
    let evening_shift = get_shift("مسائي");

    let mut stmt = conn.prepare(
        "SELECT COALESCE(p.name_ar, p.name_en, '') as name, psl.customer_brand,
                SUM(psl.cartons_produced), SUM(psl.cartons_produced * psl.cups_per_carton),
                SUM(psl.waste_cartons)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE ods.date = ?1
         GROUP BY psl.product_id, psl.customer_brand
         ORDER BY SUM(psl.cartons_produced) DESC"
    ).map_err(|e| e.to_string())?;

    let by_product = stmt.query_map(params![date], |r| {
        Ok(ProductReportLine {
            product_name: r.get(0)?, customer_brand: r.get(1)?,
            total_cartons: r.get(2)?, total_cups: r.get(3)?,
            waste_cartons: r.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(DailyProductionReport {
        date, total_cartons, total_cups, waste_cartons,
        morning_shift, evening_shift, by_product,
    })
}

#[tauri::command]
pub fn get_monthly_production_report(state: State<'_, DbState>, year: i32, month: i32) -> Result<MonthlyProductionReport, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let month_str = format!("{:02}", month);
    let prefix = format!("{}-{}", year, month_str);

    let total_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date LIKE ?1",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0.0);

    let total_cups: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced * psl.cups_per_carton), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id WHERE ods.date LIKE ?1",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0.0);

    let waste_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.waste_cartons), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id WHERE ods.date LIKE ?1",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0.0);

    let morning_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date LIKE ?1 AND ods.shift = 'صباحي'",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0.0);

    let evening_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date LIKE ?1 AND ods.shift = 'مسائي'",
        params![format!("{}%", prefix)], |r| r.get(0),
    ).unwrap_or(0.0);

    let mut stmt = conn.prepare(
        "SELECT ods.date, COALESCE(SUM(psl.cartons_produced), 0), COALESCE(SUM(psl.cartons_produced * psl.cups_per_carton), 0)
         FROM operations_daily_sheets ods
         LEFT JOIN production_shift_lines psl ON psl.sheet_id = ods.id
         WHERE ods.date LIKE ?1
         GROUP BY ods.date ORDER BY ods.date"
    ).map_err(|e| e.to_string())?;

    let daily_breakdown: Vec<DailyBriefReport> = stmt.query_map(params![format!("{}%", prefix)], |r| {
        Ok(DailyBriefReport {
            date: r.get(0)?, cartons: r.get(1)?, cups: r.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let total_days = daily_breakdown.len() as i64;

    Ok(MonthlyProductionReport {
        year, month, total_days, total_cartons, total_cups, waste_cartons,
        morning_cartons, evening_cartons, daily_breakdown,
    })
}

#[tauri::command]
pub fn get_comprehensive_daily_report(state: State<'_, DbState>, date: String) -> Result<ComprehensiveDailyReport, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Production totals
    let production_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id WHERE ods.date = ?1",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let production_cups: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced * psl.cups_per_carton), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id WHERE ods.date = ?1",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let production_waste: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.waste_cartons), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id WHERE ods.date = ?1",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let morning_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1 AND ods.shift = 'صباحي'",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    let evening_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0) FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1 AND ods.shift = 'مسائي'",
        params![date], |r| r.get(0),
    ).unwrap_or(0.0);

    // Production lines detail
    let mut pl_stmt = conn.prepare(
        "SELECT COALESCE(p.name_ar, p.name_en, ''), ods.shift,
                psl.cartons_produced, psl.cartons_produced * psl.cups_per_carton, psl.customer_brand
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE ods.date = ?1
         ORDER BY ods.shift, psl.ts"
    ).map_err(|e| e.to_string())?;

    let production_lines = pl_stmt.query_map(params![date], |r| {
        Ok(ShiftLineReport {
            product_name: r.get(0)?, shift: r.get(1)?,
            cartons: r.get(2)?, cups: r.get(3)?,
            customer_brand: r.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Sales
    let sales_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_invoices WHERE date = ?1 AND status IN ('Posted', 'Issued')",
        params![date], |r| r.get(0),
    ).unwrap_or(0);

    let sales_total_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE date = ?1 AND status IN ('Posted', 'Issued')",
        params![date], |r| r.get(0),
    ).unwrap_or(0);

    let sales_vat_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(vat_milli), 0) FROM sales_invoices WHERE date = ?1 AND status IN ('Posted', 'Issued')",
        params![date], |r| r.get(0),
    ).unwrap_or(0);

    let mut top_stmt = conn.prepare(
        "SELECT COALESCE(p.name_ar, p.name_en, ''), SUM(sil.cartons), SUM(sil.line_net_milli)
         FROM sales_invoice_lines sil
         JOIN sales_invoices si ON si.id = sil.invoice_id
         LEFT JOIN products p ON p.id = sil.product_id
         WHERE si.date = ?1 AND si.status IN ('Posted', 'Issued')
         GROUP BY sil.product_id
         ORDER BY SUM(sil.line_net_milli) DESC LIMIT 5"
    ).map_err(|e| e.to_string())?;

    let top_products = top_stmt.query_map(params![date], |r| {
        Ok(SalesProductReport {
            product_name: r.get(0)?, qty_cartons: r.get(1)?,
            revenue_milli: r.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Low stock
    let mut ls_stmt = conn.prepare(
        "SELECT COALESCE(name_ar, name_en, ''), qty_on_hand, reorder_level
         FROM inventory_items WHERE reorder_level > 0 AND qty_on_hand <= reorder_level
         ORDER BY (qty_on_hand - reorder_level) ASC LIMIT 10"
    ).map_err(|e| e.to_string())?;

    let low_stock_items = ls_stmt.query_map([], |r| {
        Ok(LowStockReport {
            name: r.get(0)?, qty_on_hand: r.get(1)?,
            reorder_level: r.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let inventory_value_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CAST(qty_on_hand * default_cost_milli AS INTEGER)), 0) FROM inventory_items",
        [], |r| r.get(0),
    ).unwrap_or(0);

    let net_profit_milli = sales_total_milli - sales_vat_milli;

    Ok(ComprehensiveDailyReport {
        date, production_cartons, production_cups, production_waste,
        morning_cartons, evening_cartons, production_lines,
        sales_count, sales_total_milli, sales_vat_milli, top_products,
        low_stock_items, inventory_value_milli, net_profit_milli,
    })
}
