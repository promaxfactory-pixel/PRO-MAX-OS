use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Debug, Serialize)]
struct McpResource {
    uri: String,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    mime_type: Option<String>,
}

fn find_db_path() -> Option<String> {
    if let Ok(path) = std::env::var("PROMAX_DB_PATH") {
        return Some(path);
    }
    std::env::var("APPDATA")
        .ok()
        .map(|p| {
            std::path::Path::new(&p)
                .join("com.promaxos.app")
                .join("promax.db")
                .to_string_lossy()
                .to_string()
        })
}

fn open_db() -> Result<Connection, String> {
    let path = find_db_path().ok_or("Database not found. Set PROMAX_DB_PATH env var.".to_string())?;
    Connection::open(&path).map_err(|e| format!("DB error: {}", e))
}

fn query_rows(conn: &Connection, sql: &str, params: impl rusqlite::Params, limit: Option<usize>) -> Result<Vec<Value>, Value> {
    let mut stmt = conn.prepare(sql).map_err(|e| {
        json!({ "error": format!("SQL error: {}", e) })
    })?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("col").to_string())
        .collect();
    let iter = stmt.query_map(params, |row| {
        let mut map = serde_json::Map::new();
        for (i, name) in col_names.iter().enumerate() {
            if let Ok(v) = row.get::<_, String>(i) {
                map.insert(name.clone(), Value::String(v));
            } else if let Ok(v) = row.get::<_, i64>(i) {
                map.insert(name.clone(), Value::Number(v.into()));
            } else if let Ok(v) = row.get::<_, f64>(i) {
                if let Some(n) = serde_json::Number::from_f64(v) {
                    map.insert(name.clone(), Value::Number(n));
                } else {
                    map.insert(name.clone(), Value::Null);
                }
            } else if let Ok(v) = row.get::<_, bool>(i) {
                map.insert(name.clone(), Value::Bool(v));
            } else {
                map.insert(name.clone(), Value::Null);
            }
        }
        Ok(Value::Object(map))
    }).map_err(|e| {
        json!({ "error": format!("Query error: {}", e) })
    })?;
    let mut results = Vec::new();
    for (i, row) in iter.enumerate() {
        if let Some(max) = limit {
            if i >= max { break; }
        }
        if let Ok(row) = row {
            results.push(row);
        }
    }
    Ok(results)
}

// ─── Tool Implementations ─────────────────────────────────────────────

fn tool_list_customers(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let rows = match query_rows(conn,
        "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active
         FROM customers ORDER BY name LIMIT ?1",
        [limit], None) {
        Ok(r) => r,
        Err(e) => return e,
    };
    json!({ "customers": rows })
}

fn tool_get_customer(conn: &Connection, args: &Value) -> Value {
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let id = args.get("id").and_then(|v| v.as_i64());
    let result = if let Some(cid) = id {
        let rows = match query_rows(conn,
            "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active
             FROM customers WHERE id = ?1", [cid], Some(1)) {
            Ok(r) => r,
            Err(_) => return json!({ "customer": null }),
        };
        rows.into_iter().next()
    } else if !query.is_empty() {
        let param = format!("%{}%", query);
        let rows = match query_rows(conn,
            "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active
             FROM customers WHERE name LIKE ?1 LIMIT 1", [param.as_str()], None) {
            Ok(r) => r,
            Err(_) => return json!({ "customer": null }),
        };
        rows.into_iter().next()
    } else {
        None
    };
    json!({ "customer": result })
}

fn tool_list_invoices(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);
    let status = args.get("status").and_then(|v| v.as_str());
    let customer_id = args.get("customer_id").and_then(|v| v.as_i64());

    let rows = if let Some(s) = status {
        match query_rows(conn,
            "SELECT si.id, si.inv_no, si.date, si.total_milli, si.status, COALESCE(c.name,'')
             FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
             WHERE si.status = ?1 ORDER BY si.id DESC LIMIT ?2",
            rusqlite::params![s, limit], None) {
            Ok(r) => r,
            Err(e) => return e,
        }
    } else if let Some(cid) = customer_id {
        match query_rows(conn,
            "SELECT si.id, si.inv_no, si.date, si.total_milli, si.status, COALESCE(c.name,'')
             FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
             WHERE si.customer_id = ?1 ORDER BY si.id DESC LIMIT ?2",
            rusqlite::params![cid, limit], None) {
            Ok(r) => r,
            Err(e) => return e,
        }
    } else {
        match query_rows(conn,
            "SELECT si.id, si.inv_no, si.date, si.total_milli, si.status, COALESCE(c.name,'')
             FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
             ORDER BY si.id DESC LIMIT ?1",
            [limit], None) {
            Ok(r) => r,
            Err(e) => return e,
        }
    };
    json!({ "invoices": rows })
}

fn tool_get_invoice(conn: &Connection, args: &Value) -> Value {
    let id = args.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let inv_rows = match query_rows(conn,
        "SELECT si.id, si.inv_no, si.date, si.net_milli, si.vat_milli, si.total_milli, si.status,
                si.notes, COALESCE(c.name,''), COALESCE(c.vat_number,'')
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
         WHERE si.id = ?1", [id], Some(1)) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let inv = inv_rows.into_iter().next();

    let lines = if inv.is_some() {
        query_rows(conn,
            "SELECT COALESCE(p.name_ar,''), sil.cartons, sil.unit_price_milli, sil.line_net_milli
             FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id = p.id
             WHERE sil.invoice_id = ?1", [id], None).unwrap_or_default()
    } else { vec![] };

    json!({ "invoice": inv, "lines": lines })
}

fn tool_list_products(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let rows = match query_rows(conn,
        "SELECT id, code, name_ar, name_en, default_price_milli, default_cost_milli, active
         FROM products ORDER BY name_ar LIMIT ?1",
        [limit], None) {
        Ok(r) => r,
        Err(e) => return e,
    };
    json!({ "products": rows })
}

fn tool_get_inventory(conn: &Connection, _args: &Value) -> Value {
    let rows = match query_rows(conn,
        "SELECT ii.id, COALESCE(ii.code, ''), COALESCE(ii.name_ar, ''), ii.qty_on_hand, ii.reorder_level, ii.avg_cost_milli
         FROM inventory_items ii ORDER BY ii.name_ar LIMIT 100",
        [], None) {
        Ok(r) => r,
        Err(e) => return e,
    };
    json!({ "inventory": rows })
}

fn tool_get_dashboard_stats(conn: &Connection, _args: &Value) -> Value {
    let customers: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
    let invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices", [], |r| r.get(0)).unwrap_or(0);
    let products: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap_or(0);
    let employees: i64 = conn.query_row("SELECT COUNT(*) FROM employees", [], |r| r.get(0)).unwrap_or(0);
    let revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
        [], |r| r.get::<_, i64>(0)).map(|v| v as f64 / 1000.0).unwrap_or(0.0);
    let unpaid: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_invoices WHERE status NOT IN ('Void','Draft','Paid') AND total_milli > paid_milli",
        [], |r| r.get(0)).unwrap_or(0);
    json!({
        "customers": customers, "invoices": invoices, "products": products,
        "employees": employees, "total_revenue_omr": revenue, "unpaid_invoices": unpaid,
    })
}

fn tool_list_suppliers(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let rows = match query_rows(conn,
        "SELECT id, name, phone, email, balance_milli, active FROM suppliers ORDER BY name LIMIT ?1",
        [limit], None) {
        Ok(r) => r,
        Err(e) => return e,
    };
    json!({ "suppliers": rows })
}

fn tool_get_company_info(conn: &Connection, _args: &Value) -> Value {
    let info = conn.query_row(
        "SELECT COALESCE(name,''), COALESCE(vat_number,''), COALESCE(address,''), COALESCE(phone,''), COALESCE(email,'')
         FROM company_settings LIMIT 1",
        [], |row| Ok(json!({
            "name": row.get::<_, String>(0)?, "vat_number": row.get::<_, String>(1)?,
            "address": row.get::<_, String>(2)?, "phone": row.get::<_, String>(3)?,
            "email": row.get::<_, String>(4)?,
        }))
    ).unwrap_or(json!({}));
    json!({ "company": info })
}

fn tool_search_employees(conn: &Connection, args: &Value) -> Value {
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let param = format!("%{}%", query);
    let rows = match query_rows(conn,
        "SELECT id, name, job, nationality, phone, passport_no, active
         FROM employees WHERE name LIKE ?1 OR passport_no LIKE ?1 LIMIT 20",
        [param.as_str()], None) {
        Ok(r) => r,
        Err(e) => return e,
    };
    json!({ "employees": rows })
}

fn tool_run_sql(conn: &Connection, args: &Value) -> Value {
    let sql = args.get("sql").and_then(|v| v.as_str()).unwrap_or("");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(100);

    // Strip leading/trailing whitespace and comments
    let normalized = sql.trim();

    // Reject empty queries
    if normalized.is_empty() {
        return json!({ "error": "Empty SQL query" });
    }

    // Reject multi-statement queries (semicolons)
    if normalized.contains(';') {
        return json!({ "error": "Multi-statement queries are not allowed" });
    }

    // Reject block comments
    if normalized.contains("/*") || normalized.contains("*/") {
        return json!({ "error": "Block comments are not allowed" });
    }

    // Reject line comments
    if normalized.starts_with("--") {
        return json!({ "error": "Line comments are not allowed" });
    }

    // Must start with SELECT (case-insensitive)
    let lower = normalized.to_lowercase();
    if !lower.starts_with("select") {
        return json!({ "error": "Only SELECT queries are allowed via MCP" });
    }

    // Block dangerous keywords anywhere in the query
    let blocked_keywords = [
        "insert", "update", "delete", "drop", "alter", "create",
        "attach", "detach", "pragma", "vacuum", "reindex", "replace",
        "begin", "commit", "rollback", "savepoint", "release",
        "into", "values", "set", "from", "where", "having", "group by", "order by",
    ];
    for kw in &blocked_keywords {
        // Check if keyword appears as a whole word (not substring of a column name)
        if let Some(pos) = lower.find(kw) {
            let before_ok = pos == 0 || !lower.as_bytes()[pos - 1].is_ascii_alphanumeric();
            let after_pos = pos + kw.len();
            let after_ok = after_pos >= lower.len() || !lower.as_bytes()[after_pos].is_ascii_alphanumeric();
            if before_ok && after_ok {
                // Allow "from" and "where" and "order by" and "group by" and "having" in SELECT context
                if !["from", "where", "having", "group by", "order by", "limit", "offset", "join", "on", "as", "and", "or", "not", "in", "like", "between", "is", "null", "case", "when", "then", "else", "end", "select", "distinct", "union", "all", "exists", "having"].contains(kw) {
                    return json!({ "error": format!("Keyword '{}' is not allowed in SQL queries via MCP", kw) });
                }
            }
        }
    }

    match query_rows(conn, normalized, [], Some(limit as usize)) {
        Ok(rows) => json!({ "rows": rows, "total": rows.len() }),
        Err(e) => e,
    }
}

// ─── Tool Definitions ──────────────────────────────────────────────────

fn get_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "list_customers".into(),
            description: "List all customers with optional limit".into(),
            input_schema: json!({ "type": "object", "properties": { "limit": { "type": "integer", "default": 50 } } }),
        },
        McpTool {
            name: "get_customer".into(),
            description: "Get customer details by ID or name search".into(),
            input_schema: json!({ "type": "object", "properties": {
                "id": { "type": "integer" }, "query": { "type": "string" }
            } }),
        },
        McpTool {
            name: "list_invoices".into(),
            description: "List invoices with optional status or customer filter".into(),
            input_schema: json!({ "type": "object", "properties": {
                "limit": { "type": "integer", "default": 20 },
                "status": { "type": "string" }, "customer_id": { "type": "integer" }
            } }),
        },
        McpTool {
            name: "get_invoice".into(),
            description: "Get full invoice details with line items".into(),
            input_schema: json!({ "type": "object", "properties": { "id": { "type": "integer" } }, "required": ["id"] }),
        },
        McpTool {
            name: "list_products".into(),
            description: "List all products with pricing".into(),
            input_schema: json!({ "type": "object", "properties": { "limit": { "type": "integer", "default": 50 } } }),
        },
        McpTool {
            name: "get_inventory".into(),
            description: "Get current inventory levels and stock alerts".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "get_dashboard_stats".into(),
            description: "Get ERP dashboard overview".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "list_suppliers".into(),
            description: "List all suppliers".into(),
            input_schema: json!({ "type": "object", "properties": { "limit": { "type": "integer", "default": 50 } } }),
        },
        McpTool {
            name: "get_company_info".into(),
            description: "Get current company settings and information".into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "search_employees".into(),
            description: "Search employees by name or passport number".into(),
            input_schema: json!({ "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] }),
        },
        McpTool {
            name: "run_sql".into(),
            description: "Run a read-only SELECT query on the ERP database (expert only)".into(),
            input_schema: json!({ "type": "object", "properties": {
                "sql": { "type": "string" }, "limit": { "type": "integer", "default": 100 }
            }, "required": ["sql"] }),
        },
    ]
}

// ─── Resource Definitions ──────────────────────────────────────────────

fn get_resources() -> Vec<McpResource> {
    vec![
        McpResource { uri: "promax://dashboard".into(), name: "ERP Dashboard".into(), description: "Overview statistics of the entire ERP system".into(), mime_type: Some("application/json".into()) },
        McpResource { uri: "promax://customers".into(), name: "Customers".into(), description: "All active customers".into(), mime_type: Some("application/json".into()) },
        McpResource { uri: "promax://invoices/recent".into(), name: "Recent Invoices".into(), description: "Last 20 invoices".into(), mime_type: Some("application/json".into()) },
        McpResource { uri: "promax://inventory".into(), name: "Inventory".into(), description: "Current inventory stock levels".into(), mime_type: Some("application/json".into()) },
        McpResource { uri: "promax://company".into(), name: "Company Info".into(), description: "Current company settings".into(), mime_type: Some("application/json".into()) },
        McpResource { uri: "promax://products".into(), name: "Products".into(), description: "All products and services".into(), mime_type: Some("application/json".into()) },
    ]
}

fn read_resource(conn: &Connection, uri: &str) -> Result<Value, String> {
    match uri {
        "promax://dashboard" => Ok(tool_get_dashboard_stats(conn, &json!({}))),
        "promax://customers" => Ok(tool_list_customers(conn, &json!({"limit": 100}))),
        "promax://invoices/recent" => Ok(tool_list_invoices(conn, &json!({"limit": 20}))),
        "promax://inventory" => Ok(tool_get_inventory(conn, &json!({}))),
        "promax://company" => Ok(tool_get_company_info(conn, &json!({}))),
        "promax://products" => Ok(tool_list_products(conn, &json!({"limit": 100}))),
        _ => {
            if let Some(id_str) = uri.strip_prefix("promax://customers/") {
                if let Ok(id) = id_str.parse::<i64>() {
                    return Ok(tool_get_customer(conn, &json!({"id": id})));
                }
            }
            Err(format!("Unknown resource: {}", uri))
        }
    }
}

// ─── Request Handler ──────────────────────────────────────────────────

fn handle_request(conn: &Connection, req: &JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();

    let result = match req.method.as_str() {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
                "serverInfo": { "name": "promax-mcp", "version": "2.0.0" }
            })
        }
        "tools/list" => json!({ "tools": get_tools() }),
        "tools/call" => {
            let empty = json!({});
            let params = req.params.as_ref().unwrap_or(&empty);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").unwrap_or(&empty);

            match tool_name {
                "list_customers" => tool_list_customers(conn, arguments),
                "get_customer" => tool_get_customer(conn, arguments),
                "list_invoices" => tool_list_invoices(conn, arguments),
                "get_invoice" => tool_get_invoice(conn, arguments),
                "list_products" => tool_list_products(conn, arguments),
                "get_inventory" => tool_get_inventory(conn, arguments),
                "get_dashboard_stats" => tool_get_dashboard_stats(conn, arguments),
                "list_suppliers" => tool_list_suppliers(conn, arguments),
                "get_company_info" => tool_get_company_info(conn, arguments),
                "search_employees" => tool_search_employees(conn, arguments),
                "run_sql" => tool_run_sql(conn, arguments),
                _ => json!({ "error": format!("Unknown tool: {}", tool_name) }),
            }
        }
        "resources/list" => json!({ "resources": get_resources() }),
        "resources/read" => {
            let empty = json!({});
            let params = req.params.as_ref().unwrap_or(&empty);
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            match read_resource(conn, uri) {
                Ok(data) => json!({
                    "contents": [{ "uri": uri, "mimeType": "application/json", "text": serde_json::to_string_pretty(&data).unwrap_or_default() }]
                }),
                Err(e) => return JsonRpcResponse { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcError { code: -32602, message: e, data: None }) },
            }
        }
        "prompts/list" => json!({ "prompts": [] }),
        "ping" => json!({}),
        _ => {
            if req.method.starts_with("notifications/") {
                return JsonRpcResponse { jsonrpc: "2.0".into(), id, result: Some(json!({})), error: None };
            }
            return JsonRpcResponse { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcError { code: -32601, message: format!("Method not found: {}", req.method), data: None }) };
        }
    };

    JsonRpcResponse { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
}

// ─── Main Server Loop ─────────────────────────────────────────────────

pub fn run_server() -> Result<(), String> {
    let conn = open_db()?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("Stdin error: {}", e))?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700, message: format!("Parse error: {}", e), data: None,
                    }),
                };
                if let Ok(resp_str) = serde_json::to_string(&err_resp) {
                    writeln!(stdout_lock, "{}", resp_str).ok();
                }
                continue;
            }
        };

        let resp = handle_request(&conn, &req);
        if let Ok(resp_str) = serde_json::to_string(&resp) {
            writeln!(stdout_lock, "{}", resp_str).ok();
            stdout_lock.flush().ok();
        }
    }

    Ok(())
}
