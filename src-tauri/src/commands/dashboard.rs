use crate::db::DbState;
use crate::error::AppError;
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
    pub sales_trend: Vec<TrendPoint>,
    pub production_trend: Vec<ProductionTrendPoint>,
    pub monthly_production: Vec<MonthlyProductionPoint>,
    pub top_customers: Vec<TopCustomerPoint>,
    pub expenses_by_category: Vec<CategoryAmountPoint>,
}

#[derive(Debug, Serialize)]
pub struct TrendPoint {
    pub date: String,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct ProductionTrendPoint {
    pub date: String,
    pub good: i64,
    pub waste: i64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyProductionPoint {
    pub month: String,
    pub cartons: i64,
    pub cups: i64,
}

#[derive(Debug, Serialize)]
pub struct TopCustomerPoint {
    pub name: String,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoryAmountPoint {
    pub category: String,
    pub amount: i64,
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
pub fn get_dashboard_stats(state: State<'_, DbState>) -> Result<DashboardStats, AppError> {
    let conn = state.0.lock()?;
    
    let total_customers: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
    let total_products: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE active=1", [], |r| r.get(0)).unwrap_or(0);
    let total_employees: i64 = conn.query_row("SELECT COUNT(*) FROM employees", [], |r| r.get(0)).unwrap_or(0);
    let total_invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices", [], |r| r.get(0)).unwrap_or(0);
    // Dashboard P&L metrics are VAT-exclusive and include only posted/approved documents.
    let revenue_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(net_milli),0) FROM sales_invoices WHERE LOWER(status) IN ('posted','issued')",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let expenses_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_milli),0) FROM expenses WHERE LOWER(COALESCE(approval_status,''))='approved'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let pending_invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices WHERE status='Draft'", [], |r| r.get(0)).unwrap_or(0);
    let overdue_amount: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
            MAX(0, si.total_milli - si.paid_milli
                - COALESCE((SELECT SUM(cn.total_milli) FROM credit_notes cn
                            WHERE cn.invoice_id=si.id AND LOWER(COALESCE(cn.status,''))!='void'),0))
         ),0)
         FROM sales_invoices si
         JOIN customers c ON c.id=si.customer_id
         WHERE LOWER(si.status) IN ('posted','issued')
           AND date(si.date, '+' || COALESCE(c.payment_terms_days,30) || ' days') < date('now','localtime')",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let inventory_value: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CAST(qty_on_hand * avg_cost_milli AS INTEGER)),0) FROM inventory_items",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let low_stock_count: i64 = conn.query_row("SELECT COUNT(*) FROM inventory_items WHERE reorder_level > 0 AND qty_on_hand <= reorder_level", [], |r| r.get(0)).unwrap_or(0);
    let production_today: i64 = conn.query_row("SELECT COALESCE(SUM(cartons_good),0) FROM production_lines pl JOIN production_orders po ON pl.order_id=po.id WHERE po.date = date('now')", [], |r| r.get(0)).unwrap_or(0);
    let waste_today: i64 = conn.query_row("SELECT COALESCE(SUM(cartons_waste),0) FROM production_lines pl JOIN production_orders po ON pl.order_id=po.id WHERE po.date = date('now')", [], |r| r.get(0)).unwrap_or(0);
    // Petty-cash subledger is the operational source of truth for custody.
    let custody_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_milli),0) FROM petty_cash_accounts WHERE active=1",
        [], |r| r.get(0)
    ).unwrap_or(0);
    // Bank dashboard balance is derived from posted GL movements so receipts/payments
    // cannot leave a stale manually-maintained cashbank balance.
    let bank_balance: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN jel.debit_milli>0 THEN jel.debit_milli ELSE -jel.credit_milli END),0)
         FROM journal_entry_lines jel
         WHERE jel.account_code='1101'
            OR jel.account_code IN (
                SELECT DISTINCT account_code FROM cashbank_accounts
                WHERE LOWER(COALESCE(atype,''))='bank' AND account_code IS NOT NULL AND trim(account_code)!=''
            )",
        [], |r| r.get(0)
    ).unwrap_or(0);

    // Sales trend (last 30 days)
    let sales_trend = {
        let mut stmt = conn.prepare(
            "SELECT date, COALESCE(SUM(net_milli),0) as total FROM sales_invoices WHERE LOWER(status) IN ('posted','issued') AND date >= date('now','localtime','-30 days') GROUP BY date ORDER BY date"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TrendPoint { date: row.get(0)?, amount: row.get(1)? })
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Production trend (last 30 days)
    let production_trend = {
        let mut stmt = conn.prepare(
            "SELECT po.date, COALESCE(SUM(pl.cartons_good),0) as good, COALESCE(SUM(pl.cartons_waste),0) as waste 
             FROM production_orders po LEFT JOIN production_lines pl ON pl.order_id=po.id 
             WHERE po.date >= date('now','-30 days') GROUP BY po.date ORDER BY po.date"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProductionTrendPoint { date: row.get(0)?, good: row.get(1)?, waste: row.get(2)? })
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Monthly production (last 6 months)
    let monthly_production = {
        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m', po.date) as month, COALESCE(SUM(pl.cartons_good),0) as cartons, COALESCE(SUM(pl.cups_good),0) as cups
             FROM production_orders po LEFT JOIN production_lines pl ON pl.order_id=po.id
             WHERE po.date >= date('now','-6 months') GROUP BY month ORDER BY month"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MonthlyProductionPoint { month: row.get(0)?, cartons: row.get(1)?, cups: row.get(2)? })
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Top 5 customers by revenue
    let top_customers = {
        let mut stmt = conn.prepare(
            "SELECT c.name, COALESCE(SUM(si.net_milli),0) as total FROM sales_invoices si JOIN customers c ON si.customer_id=c.id WHERE LOWER(si.status) IN ('posted','issued') GROUP BY si.customer_id ORDER BY total DESC LIMIT 5"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TopCustomerPoint { name: row.get(0)?, total: row.get(1)? })
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Expenses by category (current month)
    let expenses_by_category = {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(category,'أخرى') as cat, SUM(amount_milli) as total FROM expenses WHERE LOWER(COALESCE(approval_status,''))='approved' AND date >= date('now','localtime','start of month') GROUP BY cat ORDER BY total DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CategoryAmountPoint { category: row.get(0)?, amount: row.get(1)? })
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    Ok(DashboardStats { total_customers, total_products, total_employees, total_invoices, revenue_milli, expenses_milli, pending_invoices, overdue_amount, inventory_value, low_stock_count, production_today, waste_today, custody_total, bank_balance, sales_trend, production_trend, monthly_production, top_customers, expenses_by_category })
}

#[tauri::command]
pub fn get_daily_brief(state: State<'_, DbState>) -> Result<DailyBrief, AppError> {
    let conn = state.0.lock()?;
    let unpaid_count: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices WHERE status IN ('Posted','Issued','Partially Paid') AND total_milli > paid_milli", [], |r| r.get(0)).unwrap_or(0);
    let unpaid_total: i64 = conn.query_row("SELECT COALESCE(SUM(total_milli - paid_milli),0) FROM sales_invoices WHERE status IN ('Posted','Issued','Partially Paid') AND total_milli > paid_milli", [], |r| r.get(0)).unwrap_or(0);
    let overdue_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
            MAX(0, si.total_milli - si.paid_milli
                - COALESCE((SELECT SUM(cn.total_milli) FROM credit_notes cn
                            WHERE cn.invoice_id=si.id AND LOWER(COALESCE(cn.status,''))!='void'),0))
         ),0)
         FROM sales_invoices si
         JOIN customers c ON c.id=si.customer_id
         WHERE LOWER(si.status) IN ('posted','issued')
           AND date(si.date, '+' || COALESCE(c.payment_terms_days,30) || ' days') < date('now','localtime')",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let waste_yesterday: i64 = conn.query_row("SELECT COALESCE(SUM(cartons_waste),0) FROM production_lines pl JOIN production_orders po ON pl.order_id=po.id WHERE po.date = date('now', '-1 day')", [], |r| r.get(0)).unwrap_or(0);
    Ok(DailyBrief { unpaid_count, unpaid_total, overdue_total, waste_yesterday, last_backup_days: 0, backup_status: "amber".to_string() })
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ControlBalance {
    pub account_code: String,
    pub account_name: String,
    pub balance_milli: i64,
    pub balance_side: String,
    pub interpretation: String,
}

#[tauri::command]
pub fn get_control_balances(state: State<'_, DbState>) -> Result<Vec<ControlBalance>, AppError> {
    let conn = state.0.lock()?;
    let controls = [
        ("1110", "عهد الموظفين والصرف النثري", "asset", "رصيد أموال الشركة الموجودة بالعهد"),
        ("2250", "رواتب مستحقة", "liability", "صافي رواتب تم اعتمادها ولم تُسدّد بعد"),
        ("2260", "مستحقات رد مصروفات الموظفين", "liability", "مبالغ دفعتها العمالة/الموظفون من مالهم ولم تُسوَّ بعد"),
        ("2310", "جاري المالك - سيف محمد", "liability", "موجب: الشركة مدينة لسيف؛ سالب: سيف يحتفظ/يدين بأموال للشركة"),
        ("2320", "جاري المالك - محمد سيف (أبو سيف)", "liability", "موجب: الشركة مدينة لأبو سيف؛ سالب: أبو سيف يحتفظ/يدين بأموال للشركة"),
    ];
    let mut out = Vec::new();
    for (code, fallback_name, account_type, meaning) in controls {
        let name: String = conn.query_row(
            "SELECT COALESCE(NULLIF(name_ar,''), NULLIF(name_en,''), ?2) FROM accounts WHERE code=?1",
            params![code, fallback_name],
            |r| r.get(0),
        ).unwrap_or_else(|_| fallback_name.to_string());

        let (debit, credit): (i64, i64) = conn.query_row(
            "SELECT COALESCE(SUM(debit_milli),0), COALESCE(SUM(credit_milli),0)
             FROM journal_entry_lines WHERE account_code=?1",
            [code],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap_or((0,0));

        let balance = if account_type == "asset" { debit - credit } else { credit - debit };
        let side = if balance > 0 {
            if account_type == "asset" { "debit" } else { "credit" }
        } else if balance < 0 {
            if account_type == "asset" { "credit" } else { "debit" }
        } else {
            "zero"
        };

        out.push(ControlBalance {
            account_code: code.to_string(),
            account_name: name,
            balance_milli: balance,
            balance_side: side.to_string(),
            interpretation: meaning.to_string(),
        });
    }
    Ok(out)
}
