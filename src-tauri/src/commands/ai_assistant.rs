use crate::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiSettings {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<i64>,
    pub temperature: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatRequest {
    pub message: String,
    pub context_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatResponse {
    pub system_prompt: String,
    pub user_message: String,
    pub context_summary: String,
    pub suggestions: Vec<String>,
    pub data_refs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAnalysis {
    pub analysis_type: String,
    pub title: String,
    pub summary: String,
    pub findings: Vec<AiFinding>,
    pub recommendations: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiFinding {
    pub category: String,
    pub description: String,
    pub severity: String,
    pub impact: String,
}

struct ErpContext {
    summary: String,
    data_refs: Vec<String>,
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn save_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO integrations_settings(key, value, updated_at) VALUES(?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        params![key, value, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn load_setting(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, String> {
    let result = conn.query_row(
        "SELECT value FROM integrations_settings WHERE key=?1",
        params![key],
        |row| row.get::<_, Option<String>>(0),
    );
    match result {
        Ok(v) => Ok(v),
        Err(_) => Ok(None),
    }
}

fn build_general_context(conn: &rusqlite::Connection) -> Result<ErpContext, String> {
    let company_name: String = conn
        .query_row(
            "SELECT COALESCE(name, 'Unknown Company') FROM company_settings LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "Unknown Company".to_string());

    let customer_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM customers WHERE active=1", [], |row| row.get(0))
        .unwrap_or(0);

    let product_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM products WHERE active=1", [], |row| row.get(0))
        .unwrap_or(0);

    let employee_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM employees WHERE active=1", [], |row| row.get(0))
        .unwrap_or(0);

    let total_items: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory_items WHERE active=1", [], |row| row.get(0))
        .unwrap_or(0);

    let total_invoices: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let active_orders: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM production_orders WHERE status IN ('Draft', 'Approved')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(ErpContext {
        summary: format!(
            "Company: {company_name}. Customers: {customer_count}. Products: {product_count}. \
             Employees: {employee_count}. Inventory items: {total_items}. \
             Total invoices: {total_invoices}. Total revenue: {total_revenue:.2} Rial. \
             Active production orders: {active_orders}.",
        ),
        data_refs: vec![
            "customers".into(),
            "products".into(),
            "employees".into(),
            "inventory_items".into(),
            "sales_invoices".into(),
            "production_orders".into(),
        ],
    })
}

fn build_financial_context(conn: &rusqlite::Connection) -> Result<ErpContext, String> {
    let total_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let total_paid: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(paid_milli), 0) / 1000.0 FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let overdue_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sales_invoices WHERE status IN ('Issued','Partially Paid','Posted') AND total_milli > paid_milli AND date < date('now')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let overdue_amount: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_milli - paid_milli), 0) / 1000.0 FROM sales_invoices WHERE status IN ('Issued','Partially Paid','Posted') AND total_milli > paid_milli AND date < date('now')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let total_expenses: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_milli + vat_milli), 0) / 1000.0 FROM expenses",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let recent_expenses: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_milli + vat_milli), 0) / 1000.0 FROM expenses WHERE date >= date('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let total_cogs: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(pl.cartons_good * p.default_cost_milli), 0) / 1000.0 FROM production_lines pl JOIN products p ON p.id = pl.product_id",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let gross_margin = if total_revenue > 0.0 {
        (total_revenue - total_cogs) / total_revenue * 100.0
    } else {
        0.0
    };

    let recent_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE status NOT IN ('Void','Draft') AND date >= date('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let mut top_customers = String::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT c.name, SUM(si.total_milli)/1000.0 as total
         FROM sales_invoices si JOIN customers c ON si.customer_id = c.id
         WHERE si.status NOT IN ('Void','Draft')
         GROUP BY c.name ORDER BY total DESC LIMIT 5",
    ) {
        let rows: Vec<(String, f64)> = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        for (name, total) in &rows {
            top_customers.push_str(&format!("- {name}: {total:.2} Rial\n"));
        }
    }

    let data_refs = vec![
        "sales_invoices".into(),
        "expenses".into(),
        "customer_payments".into(),
        "production_lines".into(),
    ];

    let summary = format!(
        "FINANCIAL SUMMARY:\n\
         - Total Revenue: {total_revenue:.2} Rial\n\
         - Total Paid: {total_paid:.2} Rial\n\
         - Outstanding Balance: {:.2} Rial\n\
         - Overdue Invoices: {overdue_count} totaling {overdue_amount:.2} Rial\n\
         - Total COGS: {total_cogs:.2} Rial\n\
         - Gross Margin: {gross_margin:.1}%\n\
         - Total Expenses: {total_expenses:.2} Rial\n\
         - Expenses (30 days): {recent_expenses:.2} Rial\n\
         - Revenue (30 days): {recent_revenue:.2} Rial\n\
         \nTOP CUSTOMERS:\n{top_customers}",
        total_revenue - total_paid,
    );

    Ok(ErpContext { summary, data_refs })
}

fn build_production_context(conn: &rusqlite::Connection) -> Result<ErpContext, String> {
    let total_orders: i64 = conn
        .query_row("SELECT COUNT(*) FROM production_orders", [], |row| row.get(0))
        .unwrap_or(0);

    let active_orders: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM production_orders WHERE status IN ('Draft','Approved')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_good: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(cartons_good), 0) FROM production_lines",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let total_waste: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(cartons_waste), 0) FROM production_lines",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let waste_pct = if total_good + total_waste > 0.0 {
        total_waste / (total_good + total_waste) * 100.0
    } else {
        0.0
    };

    let recent_good: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(pl.cartons_good), 0) FROM production_lines pl \
             JOIN production_orders po ON po.id = pl.order_id \
             WHERE po.date >= date('now', '-7 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let recent_waste: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(pl.cartons_waste), 0) FROM production_lines pl \
             JOIN production_orders po ON po.id = pl.order_id \
             WHERE po.date >= date('now', '-7 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let total_downtime: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(downtime_minutes), 0) FROM production_orders WHERE date >= date('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_run: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(run_minutes), 0) FROM production_orders WHERE date >= date('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut machine_stats = String::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT COALESCE(m.code, 'Unknown'),
                SUM(pl.cartons_good) as good,
                SUM(pl.cartons_waste) as waste,
                CASE WHEN SUM(pl.cartons_good) + SUM(pl.cartons_waste) > 0
                     THEN SUM(pl.cartons_waste) * 100.0 / (SUM(pl.cartons_good) + SUM(pl.cartons_waste))
                     ELSE 0 END as waste_pct
         FROM production_lines pl
         JOIN production_orders po ON po.id = pl.order_id
         LEFT JOIN machines m ON m.id = po.machine_id
         GROUP BY m.code
         ORDER BY waste_pct ASC",
    ) {
        let rows: Vec<(String, f64, f64, f64)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        for (machine, good, waste, pct) in &rows {
            machine_stats.push_str(&format!(
                "- {machine}: {good:.0} good, {waste:.0} waste, waste rate: {pct:.1}%\n"
            ));
        }
    }

    let data_refs = vec![
        "production_orders".into(),
        "production_lines".into(),
        "machines".into(),
        "quality_inspections".into(),
    ];

    let summary = format!(
        "PRODUCTION SUMMARY:\n\
         - Total Orders: {total_orders}\n\
         - Active Orders: {active_orders}\n\
         - Total Good Cartons: {total_good:.0}\n\
         - Total Waste Cartons: {total_waste:.0}\n\
         - Overall Waste Rate: {waste_pct:.1}%\n\
         - Last 7 Days Good: {recent_good:.0} cartons\n\
         - Last 7 Days Waste: {recent_waste:.0} cartons\n\
         - Last 30 Days Downtime: {total_downtime} minutes\n\
         - Last 30 Days Run Time: {total_run} minutes\n\
         \nMACHINE STATS:\n{machine_stats}",
    );

    Ok(ErpContext { summary, data_refs })
}

fn build_inventory_context(conn: &rusqlite::Connection) -> Result<ErpContext, String> {
    let total_items: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory_items WHERE active=1", [], |row| row.get(0))
        .unwrap_or(0);

    let low_stock_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0 AND active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_value: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(qty_on_hand * avg_cost_milli), 0) / 1000.0 FROM inventory_items WHERE active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let out_of_stock: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= 0 AND active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let dead_stock_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_items ii WHERE ii.qty_on_hand > 0 AND NOT EXISTS (SELECT 1 FROM inventory_movements im WHERE im.item_id = ii.id AND im.ts >= datetime('now', '-90 days'))",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut low_stock_list = String::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name_ar, qty_on_hand, reorder_level FROM inventory_items \
         WHERE qty_on_hand <= reorder_level AND reorder_level > 0 AND active=1 \
         ORDER BY (qty_on_hand / reorder_level) ASC LIMIT 10",
    ) {
        let rows: Vec<(String, f64, f64)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        for (name, qty, reorder) in &rows {
            low_stock_list.push_str(&format!("- {name}: {qty:.0} in stock (reorder at {reorder:.0})\n"));
        }
    }

    let data_refs = vec![
        "inventory_items".into(),
        "inventory_movements".into(),
        "bom".into(),
    ];

    let summary = format!(
        "INVENTORY SUMMARY:\n\
         - Total Active Items: {total_items}\n\
         - Low Stock Items: {low_stock_count}\n\
         - Out of Stock: {out_of_stock}\n\
         - Dead Stock (no movement 90+ days): {dead_stock_count}\n\
         - Total Inventory Value: {total_value:.2} Rial\n\
         \nLOW STOCK ITEMS:\n{low_stock_list}",
    );

    Ok(ErpContext { summary, data_refs })
}

fn build_hr_context(conn: &rusqlite::Connection) -> Result<ErpContext, String> {
    let active_employees: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_employees: i64 = conn
        .query_row("SELECT COUNT(*) FROM employees", [], |row| row.get(0))
        .unwrap_or(0);

    let total_salary: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(salary_milli + allowances_milli), 0) / 1000.0 FROM employees WHERE active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let avg_salary: f64 = if active_employees > 0 {
        total_salary / active_employees as f64
    } else {
        0.0
    };

    let passport_expiring: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE active=1 AND passport_expiry IS NOT NULL AND passport_expiry <= date('now', '+30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let residence_expiring: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE active=1 AND residence_expiry IS NOT NULL AND residence_expiry <= date('now', '+30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let visa_expiring: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE active=1 AND visa_expiry IS NOT NULL AND visa_expiry <= date('now', '+30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let open_advances: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(remaining_milli), 0) / 1000.0 FROM employee_advances WHERE status='open'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let last_payroll_total: f64 = conn
        .query_row(
            "SELECT COALESCE(total_net_milli, 0) / 1000.0 FROM payroll_runs ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let mut nationality_breakdown = String::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT COALESCE(nationality, 'Unknown') as nat, COUNT(*) as cnt \
         FROM employees WHERE active=1 GROUP BY nat ORDER BY cnt DESC",
    ) {
        let rows: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        for (nat, cnt) in &rows {
            nationality_breakdown.push_str(&format!("- {nat}: {cnt} employees\n"));
        }
    }

    let data_refs = vec![
        "employees".into(),
        "payroll_runs".into(),
        "payroll_run_lines".into(),
        "employee_advances".into(),
        "overtime_records".into(),
    ];

    let summary = format!(
        "HR SUMMARY:\n\
         - Total Employees: {total_employees} (Active: {active_employees})\n\
         - Total Monthly Salary: {total_salary:.2} Rial\n\
         - Average Salary: {avg_salary:.2} Rial\n\
         - Last Payroll Net: {last_payroll_total:.2} Rial\n\
         - Open Advances: {open_advances:.2} Rial\n\
         - Passports Expiring (30 days): {passport_expiring}\n\
         - Residency Expiring (30 days): {residence_expiring}\n\
         - Visas Expiring (30 days): {visa_expiring}\n\
         \nNATIONALITY BREAKDOWN:\n{nationality_breakdown}",
    );

    Ok(ErpContext { summary, data_refs })
}

fn extract_suggestions(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let mut suggestions = Vec::new();

    if lower.contains("invoice") || lower.contains("factur") {
        suggestions.push("You can view invoices in Sales > Invoices".into());
        suggestions.push("Try asking: 'Show overdue invoices' or 'Top customers by revenue'".into());
    }
    if lower.contains("stock") || lower.contains("inventory") {
        suggestions.push("Check Inventory > Items for current stock levels".into());
        suggestions.push("Try asking: 'Which items are below reorder level?'".into());
    }
    if lower.contains("production") || lower.contains("manufactur") {
        suggestions.push("View Production > Orders for current production status".into());
        suggestions.push("Try asking: 'What is the current waste rate?'".into());
    }
    if lower.contains("employee") || lower.contains("hr") || lower.contains("payroll") {
        suggestions.push("Check HR > Employees for workforce details".into());
        suggestions.push("Try asking: 'Any passports expiring soon?'".into());
    }
    if lower.contains("expense") || lower.contains("cost") {
        suggestions.push("View Expenses module for expense tracking".into());
        suggestions.push("Try asking: 'What are the top expense categories?'".into());
    }
    if lower.contains("customer") || lower.contains("client") {
        suggestions.push("View Customers module for customer details".into());
        suggestions.push("Try asking: 'Which customers have overdue payments?'".into());
    }
    if lower.contains("report") || lower.contains("analysis") {
        suggestions.push("Use Reports module for detailed business reports".into());
        suggestions.push("Try asking: 'Generate a sales report for this month'".into());
    }
    if lower.contains("profit") || lower.contains("margin") || lower.contains("revenue") {
        suggestions.push("Check the financial summary for profit margins".into());
        suggestions.push("Try asking: 'What is our gross margin this month?'".into());
    }
    if lower.contains("waste") || lower.contains("quality") {
        suggestions.push("Review Quality module for inspection data".into());
        suggestions.push("Try asking: 'Which machine has the highest waste rate?'".into());
    }

    if suggestions.is_empty() {
        suggestions.push("Ask about sales, inventory, production, HR, or finances".into());
        suggestions.push("I can analyze any area of your ERP data".into());
    }

    suggestions
}

#[tauri::command]
pub fn save_ai_settings(state: State<'_, DbState>, input: AiSettings) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(ref key) = input.api_key {
        let encrypted = crate::crypto::encrypt_if_needed(key)
            .map_err(|e| format!("Failed to encrypt API key: {}", e))?;
        save_setting(&conn, "ai_api_key", &encrypted)?;
    }
    if let Some(ref model) = input.model {
        save_setting(&conn, "ai_model", model)?;
    }
    if let Some(max_tokens) = input.max_tokens {
        save_setting(&conn, "ai_max_tokens", &max_tokens.to_string())?;
    }
    if let Some(temp) = input.temperature {
        save_setting(&conn, "ai_temperature", &temp.to_string())?;
    }

    Ok("AI settings saved successfully".to_string())
}

#[tauri::command]
pub fn get_ai_settings(state: State<'_, DbState>) -> Result<AiSettings, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let api_key = load_setting(&conn, "ai_api_key")?
        .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v));
    let model = load_setting(&conn, "ai_model")?;
    let max_tokens = load_setting(&conn, "ai_max_tokens")?
        .and_then(|v| v.parse::<i64>().ok());
    let temperature = load_setting(&conn, "ai_temperature")?
        .and_then(|v| v.parse::<f64>().ok());

    Ok(AiSettings {
        api_key,
        model,
        max_tokens,
        temperature,
    })
}

#[tauri::command]
pub fn ai_chat(
    state: State<'_, DbState>,
    input: AiChatRequest,
) -> Result<AiChatResponse, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let context = match input.context_type.as_deref() {
        Some("financial") => build_financial_context(&conn)?,
        Some("production") => build_production_context(&conn)?,
        Some("inventory") => build_inventory_context(&conn)?,
        Some("hr") => build_hr_context(&conn)?,
        _ => build_general_context(&conn)?,
    };

    let _model = load_setting(&conn, "ai_model")?
        .unwrap_or_else(|| "gpt-4o".to_string());

    let context_summary = &context.summary;
    let system_prompt = format!(
        "You are an intelligent ERP assistant for a manufacturing company. \
         You have access to the following company data from the ERP database. \
         Analyze the data and provide actionable business insights.\n\n\
         {context_summary}\n\n\
         Respond in a clear, professional manner. Provide specific numbers and actionable recommendations. \
         If data is insufficient, acknowledge it and suggest what data would help. \
         Always consider the manufacturing/paper cup production industry context.",
    );

    let suggestions = extract_suggestions(&input.message);

    Ok(AiChatResponse {
        system_prompt,
        user_message: input.message,
        context_summary: context.summary,
        suggestions,
        data_refs: context.data_refs,
    })
}

#[tauri::command]
pub fn ai_analyze_entity(
    state: State<'_, DbState>,
    entity_type: String,
    entity_id: i64,
) -> Result<AiAnalysis, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_str();

    let (analysis_type, title, summary, findings, recommendations) = match entity_type.as_str() {
        "customer" => {
            let name: String = conn
                .query_row("SELECT name FROM customers WHERE id=?1", params![entity_id], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            let total_invoices: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sales_invoices WHERE customer_id=?1 AND status NOT IN ('Void','Draft')",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let total_amount: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE customer_id=?1 AND status NOT IN ('Void','Draft')",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let _total_paid: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(paid_milli), 0) / 1000.0 FROM sales_invoices WHERE customer_id=?1 AND status NOT IN ('Void','Draft')",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let balance: f64 = conn
                .query_row(
                    "SELECT COALESCE(balance_milli, 0) / 1000.0 FROM customers WHERE id=?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let overdue_amount: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total_milli - paid_milli), 0) / 1000.0 FROM sales_invoices WHERE customer_id=?1 AND status IN ('Issued','Partially Paid','Posted') AND total_milli > paid_milli AND date < date('now')",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let mut findings = Vec::new();
            let mut recommendations = Vec::new();

            if balance > 0.0 && total_amount > 0.0 {
                let balance_ratio = balance / total_amount * 100.0;
                findings.push(AiFinding {
                    category: "Financial".into(),
                    description: format!("Outstanding balance of {balance:.2} Rial ({balance_ratio:.1}% of total purchases)",),
                    severity: if balance_ratio > 50.0 { "high" } else { "medium" }.into(),
                    impact: "Cash flow impact".into(),
                });
            }

            if overdue_amount > 0.0 {
                findings.push(AiFinding {
                    category: "Receivables".into(),
                    description: format!("Overdue amount: {overdue_amount:.2} Rial"),
                    severity: "high".into(),
                    impact: "Revenue recognition delay".into(),
                });
                recommendations.push("Follow up on overdue payments promptly".into());
            }

            if total_invoices > 0 && total_amount > 0.0 {
                let avg_order = total_amount / total_invoices as f64;
                findings.push(AiFinding {
                    category: "Sales".into(),
                    description: format!("Average order value: {avg_order:.2} Rial across {total_invoices} invoices"),
                    severity: "info".into(),
                    impact: "Revenue pattern".into(),
                });
            }

            if balance <= 0.0 && total_amount > 0.0 {
                recommendations.push("Excellent payment history. Consider offering loyalty discounts".into());
            }

            recommendations.push(format!("Continue monitoring {name}'s order patterns"));

            ("customer".into(), format!("Customer Analysis: {name}"), format!("Analysis of customer '{name}' with {total_invoices} invoices totaling {total_amount:.2} Rial"), findings, recommendations)
        }
        "product" => {
            let name: String = conn
                .query_row("SELECT COALESCE(name_en, name_ar, 'Unknown') FROM products WHERE id=?1", params![entity_id], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            let total_produced: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(cartons_good), 0) FROM production_lines WHERE product_id=?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let total_waste: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(cartons_waste), 0) FROM production_lines WHERE product_id=?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let total_sold: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(sil.cartons), 0) FROM sales_invoice_lines sil \
                     JOIN sales_invoices si ON si.id = sil.invoice_id \
                     WHERE sil.product_id=?1 AND si.status NOT IN ('Void','Draft')",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let waste_pct = if total_produced + total_waste > 0.0 {
                total_waste / (total_produced + total_waste) * 100.0
            } else {
                0.0
            };

            let mut findings = Vec::new();
            let mut recommendations = Vec::new();

            findings.push(AiFinding {
                category: "Production".into(),
                description: format!("Total produced: {total_produced:.0} cartons, Waste: {total_waste:.0} ({waste_pct:.1}%)"),
                severity: if waste_pct > 10.0 { "high" } else if waste_pct > 5.0 { "medium" } else { "low" }.into(),
                impact: "Manufacturing efficiency".into(),
            });

            findings.push(AiFinding {
                category: "Sales".into(),
                description: format!("Total sold: {total_sold:.0} cartons"),
                severity: "info".into(),
                impact: "Revenue generation".into(),
            });

            let unsold = total_produced - total_sold;
            if unsold > 0.0 {
                findings.push(AiFinding {
                    category: "Inventory".into(),
                    description: format!("Unsold stock: {unsold:.0} cartons"),
                    severity: if unsold > total_produced * 0.3 { "high" } else { "medium" }.into(),
                    impact: "Working capital tie-up".into(),
                });
            }

            if waste_pct > 5.0 {
                recommendations.push("Review production process for this product to reduce waste".into());
            }
            recommendations.push("Monitor sales velocity against production rate".into());

            ("product".into(), format!("Product Analysis: {name}"), format!("Production: {total_produced:.0} cartons, Sold: {total_sold:.0}, Waste: {waste_pct:.1}%"), findings, recommendations)
        }
        "supplier" => {
            let name: String = conn
                .query_row("SELECT name FROM suppliers WHERE id=?1", params![entity_id], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            let total_purchases: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM purchases WHERE supplier_id=?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let total_amount: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM purchases WHERE supplier_id=?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let balance: f64 = conn
                .query_row(
                    "SELECT COALESCE(balance_milli, 0) / 1000.0 FROM suppliers WHERE id=?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let mut findings = Vec::new();
            let mut recommendations = Vec::new();

            findings.push(AiFinding {
                category: "Purchasing".into(),
                description: format!("{total_purchases} purchase orders totaling {total_amount:.2} Rial"),
                severity: "info".into(),
                impact: "Supply chain".into(),
            });

            if balance > 0.0 {
                findings.push(AiFinding {
                    category: "Payables".into(),
                    description: format!("Outstanding payable: {balance:.2} Rial"),
                    severity: "info".into(),
                    impact: "Cash flow planning".into(),
                });
            }

            if total_purchases > 0 && total_amount > 0.0 {
                let avg_order = total_amount / total_purchases as f64;
                findings.push(AiFinding {
                    category: "Analysis".into(),
                    description: format!("Average order value: {avg_order:.2} Rial"),
                    severity: "info".into(),
                    impact: "Procurement pattern".into(),
                });
            }

            recommendations.push("Review supplier performance and pricing periodically".into());
            recommendations.push("Compare costs across suppliers for key materials".into());

            ("supplier".into(), format!("Supplier Analysis: {name}"), format!("Supplier '{name}' with {total_purchases} purchases totaling {total_amount:.2} Rial"), findings, recommendations)
        }
        _ => {
            return Err(format!("Unknown entity type: {entity_type}. Supported: customer, product, supplier"));
        }
    };

    Ok(AiAnalysis {
        analysis_type,
        title,
        summary,
        findings,
        recommendations,
        generated_at: now,
    })
}

#[tauri::command]
pub fn ai_suggest_actions(
    state: State<'_, DbState>,
    context_type: String,
) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut suggestions = Vec::new();

    match context_type.as_str() {
        "financial" => {
            let overdue_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sales_invoices WHERE status IN ('Issued','Partially Paid','Posted') AND total_milli > paid_milli AND date < date('now')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            let overdue_amount: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total_milli - paid_milli), 0) / 1000.0 FROM sales_invoices WHERE status IN ('Issued','Partially Paid','Posted') AND total_milli > paid_milli AND date < date('now')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            if overdue_count > 0 {
                suggestions.push(format!("URGENT: {overdue_count} overdue invoices totaling {overdue_amount:.2} Rial need follow-up"));
            }

            let margin: f64 = conn
                .query_row(
                    "SELECT CASE WHEN SUM(si.total_milli) > 0 THEN (SUM(si.total_milli) - SUM(si.cogs_milli)) * 100.0 / SUM(si.total_milli) ELSE 0 END \
                     FROM sales_invoices si \
                     WHERE si.date >= date('now', '-30 days') AND si.status NOT IN ('Void','Draft')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            if margin > 0.0 && margin < 15.0 {
                suggestions.push(format!("WARNING: Gross margin at {margin:.1}% is below 15% target. Review pricing."));
            } else if margin >= 30.0 {
                suggestions.push(format!("GOOD: Gross margin at {margin:.1}% is healthy."));
            }

            let month_expenses: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(amount_milli + vat_milli), 0) / 1000.0 FROM expenses WHERE date >= date('now', '-30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);
            if month_expenses > 0.0 {
                suggestions.push(format!("Monthly expenses: {month_expenses:.2} Rial. Review top categories."));
            }
        }
        "production" => {
            let waste_pct: f64 = conn
                .query_row(
                    "SELECT CASE WHEN SUM(pl.cartons_good) + SUM(pl.cartons_waste) > 0 \
                     THEN SUM(pl.cartons_waste) * 100.0 / (SUM(pl.cartons_good) + SUM(pl.cartons_waste)) ELSE 0 END \
                     FROM production_lines pl JOIN production_orders po ON po.id = pl.order_id \
                     WHERE po.date >= date('now', '-30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            if waste_pct > 10.0 {
                suggestions.push(format!("CRITICAL: Waste rate at {waste_pct:.1}% exceeds 10%. Immediate action needed."));
            } else if waste_pct > 5.0 {
                suggestions.push(format!("WARNING: Waste rate at {waste_pct:.1}% exceeds 5% target."));
            } else if waste_pct > 0.0 {
                suggestions.push(format!("GOOD: Waste rate at {waste_pct:.1}% is within target."));
            }

            let stalled: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM production_orders WHERE status='Approved' AND date < date('now', '-3 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if stalled > 0 {
                suggestions.push(format!("ALERT: {stalled} approved orders older than 3 days not started."));
            }

            let high_downtime: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM production_orders WHERE downtime_minutes > run_minutes AND downtime_minutes > 120",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if high_downtime > 0 {
                suggestions.push(format!("MAINTENANCE: {high_downtime} orders with excessive downtime detected. Check machines."));
            }

            let quality_failures: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM quality_inspections WHERE status='Rejected' AND date >= date('now', '-30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if quality_failures > 0 {
                suggestions.push(format!("QUALITY: {quality_failures} rejected quality inspections this month. Review standards."));
            }
        }
        "inventory" => {
            let low_stock: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0 AND active=1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if low_stock > 0 {
                suggestions.push(format!("REORDER: {low_stock} items are at or below reorder level. Place purchase orders."));
            }

            let out_of_stock: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= 0 AND active=1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if out_of_stock > 0 {
                suggestions.push(format!("CRITICAL: {out_of_stock} items are out of stock. Production may be affected."));
            }

            let dead_stock: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM inventory_items ii WHERE ii.qty_on_hand > 0 AND NOT EXISTS (SELECT 1 FROM inventory_movements im WHERE im.item_id = ii.id AND im.ts >= datetime('now', '-90 days'))",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if dead_stock > 0 {
                suggestions.push(format!("INACTIVE: {dead_stock} items with no movement in 90+ days. Consider liquidation."));
            }

            let negative_stock: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand < 0",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if negative_stock > 0 {
                suggestions.push(format!("ERROR: {negative_stock} items have negative stock. Run stock adjustment."));
            }
        }
        "hr" => {
            let passports_expiring: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM employees WHERE active=1 AND passport_expiry IS NOT NULL AND passport_expiry <= date('now', '+30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if passports_expiring > 0 {
                suggestions.push(format!("URGENT: {passports_expiring} passports expiring within 30 days. Renew immediately."));
            }

            let residence_expiring: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM employees WHERE active=1 AND residence_expiry IS NOT NULL AND residence_expiry <= date('now', '+30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if residence_expiring > 0 {
                suggestions.push(format!("URGENT: {residence_expiring} residency permits expiring within 30 days."));
            }

            let visa_expiring: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM employees WHERE active=1 AND visa_expiry IS NOT NULL AND visa_expiry <= date('now', '+30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if visa_expiring > 0 {
                suggestions.push(format!("WARNING: {visa_expiring} visas expiring within 30 days."));
            }

            let open_advances: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM employee_advances WHERE status='open'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if open_advances > 3 {
                suggestions.push(format!("INFO: {open_advances} open employee advances. Review deduction schedule."));
            }

            let pending_overtime: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM overtime_records WHERE status='Pending'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if pending_overtime > 0 {
                suggestions.push(format!("ACTION: {pending_overtime} overtime records pending approval."));
            }
        }
        "general" | _ => {
            let overdue_inv: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sales_invoices WHERE status IN ('Issued','Partially Paid','Posted') AND total_milli > paid_milli AND date < date('now')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if overdue_inv > 0 {
                suggestions.push(format!("{overdue_inv} overdue sales invoices require attention"));
            }

            let low_stock: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0 AND active=1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if low_stock > 0 {
                suggestions.push(format!("{low_stock} inventory items below reorder level"));
            }

            let pending_orders: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM production_orders WHERE status='Approved' AND date <= date('now')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if pending_orders > 0 {
                suggestions.push(format!("{pending_orders} approved production orders ready to start"));
            }

            let expiring_renewals: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM renewals WHERE status='active' AND expiry_date IS NOT NULL AND expiry_date <= date('now', '+30 days')",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if expiring_renewals > 0 {
                suggestions.push(format!("{expiring_renewals} renewals expiring within 30 days"));
            }

            let open_cheques: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM cheques WHERE status='issued'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if open_cheques > 0 {
                suggestions.push(format!("{open_cheques} issued cheques pending clearance"));
            }

            suggestions.push("Select a specific context type (financial, production, inventory, hr) for detailed suggestions".into());
        }
    }

    if suggestions.is_empty() {
        suggestions.push("No immediate actions required. All systems are within normal parameters.".into());
    }

    Ok(suggestions)
}

// ─── Real AI API Integration ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context_type: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub reply: String,
    pub model: String,
    pub provider: String,
    pub token_estimate: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiProviderStatus {
    pub configured: bool,
    pub provider: String,
    pub model: String,
    pub message: String,
}

#[tauri::command]
pub async fn chat_with_ai(
    state: State<'_, DbState>,
    input: ChatRequest,
) -> Result<ChatResponse, String> {
    let (api_key, model, max_tokens, temperature, context_summary) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let provider = input.provider.as_deref().unwrap_or("openai");
        let api_key = load_setting(&conn, &format!("ai_api_key_{provider}"))?
            .or_else(|| load_setting(&conn, "ai_api_key").ok().flatten())
            .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v))
            .ok_or_else(|| "API key not configured. Go to Settings > AI Integration to set up.".to_string())?;
        let model = load_setting(&conn, &format!("ai_model_{provider}"))?
            .or_else(|| load_setting(&conn, "ai_model").ok().flatten())
            .unwrap_or_else(|| {
                match provider {
                    "anthropic" => "claude-3-opus-20240229".into(),
                    "openai" => "gpt-4o".into(),
                    _ => "gpt-4o".into(),
                }
            });
        let max_tokens: i64 = load_setting(&conn, "ai_max_tokens")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(2048);
        let temperature: f64 = load_setting(&conn, "ai_temperature")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);
        let context = match input.context_type.as_deref() {
            Some("financial") => build_financial_context(&conn)?,
            Some("production") => build_production_context(&conn)?,
            Some("inventory") => build_inventory_context(&conn)?,
            Some("hr") => build_hr_context(&conn)?,
            _ => build_general_context(&conn)?,
        };
        (api_key, model, max_tokens, temperature, context.summary)
    };

    let system_prompt = format!(
        "You are PRO MAX AI, an intelligent ERP assistant for PRO MAX OS manufacturing ERP system. \
         You have access to the following company data:\n\n{}\n\n\
         Rules:\n\
         1. Answer in the same language as the user's question (Arabic, English, Hindi, or Urdu).\n\
         2. Provide specific numbers, actionable recommendations, and data-driven insights.\n\
         3. If data is insufficient, acknowledge it and suggest what data would help.\n\
         4. Always consider the manufacturing industry context (paper cups, packaging, production).\n\
         5. Be concise and professional. Use bullet points for clarity.\n\
         6. For financial questions, reference specific amounts in OMR (Omani Rial).\n\
         7. For production questions, consider machine efficiency, waste rates, and quality.",
        context_summary
    );

    let provider = input.provider.as_deref().unwrap_or("openai");
    let reply = match provider {
        "anthropic" => call_anthropic(&api_key, &model, &system_prompt, &input.message, max_tokens, temperature).await?,
        _ => call_openai(&api_key, &model, &system_prompt, &input.message, max_tokens, temperature).await?,
    };

    let token_estimate = (system_prompt.len() / 4 + input.message.len() / 4 + reply.len() / 4) as i64;

    Ok(ChatResponse {
        reply,
        model,
        provider: provider.to_string(),
        token_estimate,
    })
}

async fn call_openai(api_key: &str, model: &str, system: &str, user: &str, max_tokens: i64, temperature: f64) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "max_tokens": max_tokens,
        "temperature": temperature,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("OpenAI API request failed: {e}"))?;

    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| format!("Failed to parse OpenAI response: {e}"))?;

    if !status.is_success() {
        let err_msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
        return Err(format!("OpenAI API error ({}): {}", status.as_u16(), err_msg));
    }

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No response content from OpenAI".to_string())
}

async fn call_anthropic(api_key: &str, model: &str, system: &str, user: &str, max_tokens: i64, temperature: f64) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let body = json!({
        "model": model,
        "system": system,
        "messages": [{"role": "user", "content": user}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anthropic API request failed: {e}"))?;

    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| format!("Failed to parse Anthropic response: {e}"))?;

    if !status.is_success() {
        let err_msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
        return Err(format!("Anthropic API error ({}): {}", status.as_u16(), err_msg));
    }

    json["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No response content from Anthropic".to_string())
}

#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, DbState>,
    provider: Option<String>,
) -> Result<AiProviderStatus, String> {
    let prov = provider.as_deref().unwrap_or("openai");
    let (api_key, model) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let api_key = load_setting(&conn, &format!("ai_api_key_{prov}"))?
            .or_else(|| load_setting(&conn, "ai_api_key").ok().flatten())
            .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v));
        let model = load_setting(&conn, &format!("ai_model_{prov}"))?
            .or_else(|| load_setting(&conn, "ai_model").ok().flatten())
            .unwrap_or_else(|| match prov {
                "anthropic" => "claude-3-opus-20240229".into(),
                _ => "gpt-4o".into(),
            });
        (api_key, model)
    };

    match api_key {
        Some(key) if !key.is_empty() => {
            let test_msg = "Reply with exactly: OK";
            let result = match prov {
                "anthropic" => call_anthropic(&key, &model, "Reply with exactly: OK", test_msg, 10, 0.0).await,
                _ => call_openai(&key, &model, "Reply with exactly: OK", test_msg, 10, 0.0).await,
            };
            match result {
                Ok(_) => Ok(AiProviderStatus {
                    configured: true,
                    provider: prov.to_string(),
                    model,
                    message: "Connection successful! API key is valid.".to_string(),
                }),
                Err(e) => Ok(AiProviderStatus {
                    configured: true,
                    provider: prov.to_string(),
                    model,
                    message: format!("Connection failed: {e}"),
                }),
            }
        }
        _ => Ok(AiProviderStatus {
            configured: false,
            provider: prov.to_string(),
            model,
            message: "API key not configured. Please add your API key in Settings.".to_string(),
        }),
    }
}

#[tauri::command]
pub fn save_ai_provider_settings(
    state: State<'_, DbState>,
    provider: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let encrypted = crate::crypto::encrypt_if_needed(&api_key)
        .map_err(|e| format!("Failed to encrypt API key: {}", e))?;
    save_setting(&conn, &format!("ai_api_key_{provider}"), &encrypted)?;
    save_setting(&conn, &format!("ai_model_{provider}"), &model)?;
    Ok(format!("{provider} settings saved successfully"))
}
