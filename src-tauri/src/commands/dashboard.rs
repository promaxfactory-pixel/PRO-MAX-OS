use crate::db::DbState;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_customers: i64,
    pub total_products: i64,
    pub total_employees: i64,
    pub total_invoices: i64,
    pub revenue_milli: i64,
    pub expenses_milli: i64,
    pub pending_invoices: i64,
    pub overdue_amount: i64,
    pub inventory_value: i64,
    pub low_stock_count: i64,
    pub production_today: i64,
    pub waste_today: i64,
    pub custody_total: i64,
    pub bank_balance: i64,
}

#[derive(Debug, Serialize)]
pub struct DailyBrief {
    pub unpaid_count: i64,
    pub unpaid_total: i64,
    pub overdue_total: i64,
    pub waste_yesterday: i64,
    pub last_backup_days: i64,
    pub backup_status: String,
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, DbState>) -> Result<DashboardStats, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    let total_customers: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
    let total_products: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE active=1", [], |r| r.get(0)).unwrap_or(0);
    let total_employees: i64 = conn.query_row("SELECT COUNT(*) FROM employees", [], |r| r.get(0)).unwrap_or(0);
    let total_invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices", [], |r| r.get(0)).unwrap_or(0);
    let revenue_milli: i64 = conn.query_row("SELECT COALESCE(SUM(total_milli),0) FROM sales_invoices WHERE status='Posted'", [], |r| r.get(0)).unwrap_or(0);
    let expenses_milli: i64 = conn.query_row("SELECT COALESCE(SUM(amount_milli),0) FROM expenses", [], |r| r.get(0)).unwrap_or(0);
    let pending_invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices WHERE status='Draft'", [], |r| r.get(0)).unwrap_or(0);
    let overdue_amount: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli - paid_milli),0) FROM sales_invoices WHERE status='Posted' AND total_milli > paid_milli AND date < date('now')",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let inventory_value: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CAST(qty_on_hand * avg_cost_milli AS INTEGER)),0) FROM inventory_items",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let low_stock_count: i64 = conn.query_row("SELECT COUNT(*) FROM inventory_items WHERE reorder_level > 0 AND qty_on_hand <= reorder_level", [], |r| r.get(0)).unwrap_or(0);
    let production_today: i64 = conn.query_row("SELECT COALESCE(SUM(cartons_good),0) FROM production_lines pl JOIN production_orders po ON pl.order_id=po.id WHERE po.date = date('now')", [], |r| r.get(0)).unwrap_or(0);
    let waste_today: i64 = conn.query_row("SELECT COALESCE(SUM(cartons_waste),0) FROM production_lines pl JOIN production_orders po ON pl.order_id=po.id WHERE po.date = date('now')", [], |r| r.get(0)).unwrap_or(0);
    let custody_total: i64 = conn.query_row("SELECT COALESCE(SUM(balance_milli),0) FROM cashbank_accounts WHERE atype='Custody'", [], |r| r.get(0)).unwrap_or(0);
    let bank_balance: i64 = conn.query_row("SELECT COALESCE(SUM(balance_milli),0) FROM cashbank_accounts WHERE atype='Bank'", [], |r| r.get(0)).unwrap_or(0);

    Ok(DashboardStats { total_customers, total_products, total_employees, total_invoices, revenue_milli, expenses_milli, pending_invoices, overdue_amount, inventory_value, low_stock_count, production_today, waste_today, custody_total, bank_balance })
}

#[tauri::command]
pub fn get_daily_brief(state: State<'_, DbState>) -> Result<DailyBrief, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    let unpaid_count: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices WHERE status IN ('Posted','Issued','Partially Paid') AND total_milli > paid_milli", [], |r| r.get(0)).unwrap_or(0);
    let unpaid_total: i64 = conn.query_row("SELECT COALESCE(SUM(total_milli - paid_milli),0) FROM sales_invoices WHERE status IN ('Posted','Issued','Partially Paid') AND total_milli > paid_milli", [], |r| r.get(0)).unwrap_or(0);
    let overdue_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli - paid_milli),0) FROM sales_invoices WHERE status='Posted' AND total_milli > paid_milli AND date < date('now')",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let waste_yesterday: i64 = conn.query_row("SELECT COALESCE(SUM(cartons_waste),0) FROM production_lines pl JOIN production_orders po ON pl.order_id=po.id WHERE po.date = date('now', '-1 day')", [], |r| r.get(0)).unwrap_or(0);

    Ok(DailyBrief { unpaid_count, unpaid_total, overdue_total, waste_yesterday, last_backup_days: 0, backup_status: "amber".to_string() })
}
