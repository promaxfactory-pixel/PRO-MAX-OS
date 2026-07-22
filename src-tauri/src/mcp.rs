use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;

// ─── MCP Protocol Types ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// ─── MCP Tool Definition ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub inputSchema: Value,
}

// ─── MCP Resource Definition ──────────────────────────────────────────

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimeType: Option<String>,
}

// ─── Database Connection ──────────────────────────────────────────────

fn find_db_path() -> Option<String> {
    let paths = [
        std::env::var("PROMAX_DB_PATH").ok(),
        {
            let data_dir = dirs_next().or_else(|| {
                std::env::var("APPDATA").ok().map(|p| Path::new(&p).join("com.promaxos.app"))
            }).map(|p| p.join("promax.db").to_string_lossy().to_string());
            data_dir
        },
        Some("promax.db".into()),
    ];
    for p in paths.iter().flatten() {
        if Path::new(p).exists() {
            return Some(p.clone());
        }
    }
    None
}

fn dirs_next() -> Option<std::path::PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        Some(Path::new(&appdata).join("com.promaxos.app"))
    } else {
        None
    }
}

fn open_db() -> Result<Connection, String> {
    let path = find_db_path().ok_or("Database not found. Set PROMAX_DB_PATH env var.".to_string())?;
    Connection::open(&path).map_err(|e| format!("DB error: {}", e))
}

// ─── Tool Implementations ─────────────────────────────────────────────

fn tool_list_customers(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let mut stmt = conn.prepare(
        "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active
         FROM customers ORDER BY name LIMIT ?1"
    ).unwrap();
    let rows: Vec<Value> = stmt.query_map([limit], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "phone": row.get::<_, Option<String>>(2)?,
            "email": row.get::<_, Option<String>>(3)?,
            "vat_number": row.get::<_, Option<String>>(4)?,
            "credit_limit": row.get::<_, i64>(5)? as f64 / 1000.0,
            "balance": row.get::<_, i64>(6)? as f64 / 1000.0,
            "active": row.get::<_, bool>(7)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    json!({ "customers": rows })
}

fn tool_get_customer(conn: &Connection, args: &Value) -> Value {
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let id = args.get("id").and_then(|v| v.as_i64());
    let result = if let Some(cid) = id {
        conn.query_row(
            "SELECT id, name, address, phone, email, vat_number, credit_limit_milli, balance_milli, active, created_at
             FROM customers WHERE id = ?1", [cid],
            |row| Ok(json!({
                "id": row.get::<_, i64>(0)?, "name": row.get::<_, String>(1)?,
                "address": row.get::<_, Option<String>>(2)?, "phone": row.get::<_, Option<String>>(3)?,
                "email": row.get::<_, Option<String>>(4)?, "vat_number": row.get::<_, Option<String>>(5)?,
                "credit_limit": row.get::<_, i64>(6)? as f64 / 1000.0,
                "balance": row.get::<_, i64>(7)? as f64 / 1000.0,
                "active": row.get::<_, bool>(8)?, "created_at": row.get::<_, String>(9)?,
            }))
        ).ok()
    } else if !query.is_empty() {
        conn.query_row(
            "SELECT id, name, address, phone, email, vat_number, credit_limit_milli, balance_milli, active, created_at
             FROM customers WHERE name LIKE ?1 LIMIT 1",
            [format!("%{}%", query)],
            |row| Ok(json!({
                "id": row.get::<_, i64>(0)?, "name": row.get::<_, String>(1)?,
                "address": row.get::<_, Option<String>>(2)?, "phone": row.get::<_, Option<String>>(3)?,
                "email": row.get::<_, Option<String>>(4)?, "vat_number": row.get::<_, Option<String>>(5)?,
                "credit_limit": row.get::<_, i64>(6)? as f64 / 1000.0,
                "balance": row.get::<_, i64>(7)? as f64 / 1000.0,
                "active": row.get::<_, bool>(8)?, "created_at": row.get::<_, String>(9)?,
            }))
        ).ok()
    } else {
        None
    };
    json!({ "customer": result })
}

fn tool_list_invoices(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);
    let status = args.get("status").and_then(|v| v.as_str());
    let customer_id = args.get("customer_id").and_then(|v| v.as_i64());

    let rows: Vec<Value> = if let Some(s) = status {
        let mut stmt = conn.prepare(
            "SELECT si.id, si.inv_no, si.date, si.total_milli, si.status, COALESCE(c.name,'')
             FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
             WHERE si.status = ?1 ORDER BY si.id DESC LIMIT ?2"
        ).unwrap();
        stmt.query_map(rusqlite::params![s, limit], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?, "invoice_no": row.get::<_, String>(1)?,
                "date": row.get::<_, String>(2)?, "total": row.get::<_, i64>(3)? as f64 / 1000.0,
                "status": row.get::<_, String>(4)?, "customer_name": row.get::<_, String>(5)?,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect()
    } else if let Some(cid) = customer_id {
        let mut stmt = conn.prepare(
            "SELECT si.id, si.inv_no, si.date, si.total_milli, si.status, COALESCE(c.name,'')
             FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
             WHERE si.customer_id = ?1 ORDER BY si.id DESC LIMIT ?2"
        ).unwrap();
        stmt.query_map(rusqlite::params![cid, limit], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?, "invoice_no": row.get::<_, String>(1)?,
                "date": row.get::<_, String>(2)?, "total": row.get::<_, i64>(3)? as f64 / 1000.0,
                "status": row.get::<_, String>(4)?, "customer_name": row.get::<_, String>(5)?,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT si.id, si.inv_no, si.date, si.total_milli, si.status, COALESCE(c.name,'')
             FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
             ORDER BY si.id DESC LIMIT ?1"
        ).unwrap();
        stmt.query_map([limit], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?, "invoice_no": row.get::<_, String>(1)?,
                "date": row.get::<_, String>(2)?, "total": row.get::<_, i64>(3)? as f64 / 1000.0,
                "status": row.get::<_, String>(4)?, "customer_name": row.get::<_, String>(5)?,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect()
    };

    json!({ "invoices": rows })
}

fn tool_get_invoice(conn: &Connection, args: &Value) -> Value {
    let id = args.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let inv = conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.net_milli, si.vat_milli, si.total_milli, si.status,
                si.notes, COALESCE(c.name,''), COALESCE(c.vat_number,'')
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
         WHERE si.id = ?1", [id],
        |row| Ok(json!({
            "id": row.get::<_, i64>(0)?, "invoice_no": row.get::<_, String>(1)?,
            "date": row.get::<_, String>(2)?, "net": row.get::<_, i64>(3)? as f64 / 1000.0,
            "vat": row.get::<_, i64>(4)? as f64 / 1000.0, "total": row.get::<_, i64>(5)? as f64 / 1000.0,
            "status": row.get::<_, String>(6)?, "notes": row.get::<_, Option<String>>(7)?,
            "customer_name": row.get::<_, String>(8)?, "customer_vat": row.get::<_, String>(9)?,
        }))
    ).ok();

    let lines = if inv.is_some() {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(p.name_ar,''), sil.cartons, sil.unit_price_milli, sil.line_net_milli
             FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id = p.id
             WHERE sil.invoice_id = ?1"
        ).unwrap();
        let rows: Vec<Value> = stmt.query_map([id], |row| {
            Ok(json!({
                "product": row.get::<_, String>(0)?, "qty": row.get::<_, f64>(1)?,
                "unit_price": row.get::<_, i64>(2)? as f64 / 1000.0,
                "total": row.get::<_, i64>(3)? as f64 / 1000.0,
            }))
        }).unwrap().filter_map(|r| r.ok()).collect();
        rows
    } else { vec![] };

    json!({ "invoice": inv, "lines": lines })
}

fn tool_list_products(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let mut stmt = conn.prepare(
        "SELECT id, code, name_ar, name_en, default_price_milli, default_cost_milli, active
         FROM products ORDER BY name_ar LIMIT ?1"
    ).unwrap();
    let rows: Vec<Value> = stmt.query_map([limit], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "code": row.get::<_, Option<String>>(1)?,
            "name_ar": row.get::<_, String>(2)?, "name_en": row.get::<_, Option<String>>(3)?,
            "price_milli": row.get::<_, i64>(4)?,
            "cost_milli": row.get::<_, i64>(5)?,
            "active": row.get::<_, bool>(6)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    json!({ "products": rows })
}

fn tool_get_inventory(conn: &Connection, _args: &Value) -> Value {
    let mut stmt = conn.prepare(
        "SELECT ii.id, COALESCE(ii.code, ''), COALESCE(ii.name_ar, ''), ii.qty_on_hand, ii.reorder_level, ii.avg_cost_milli
         FROM inventory_items ii
         ORDER BY ii.name_ar LIMIT 100"
    ).unwrap();
    let rows: Vec<Value> = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "code": row.get::<_, Option<String>>(1)?,
            "name": row.get::<_, Option<String>>(2)?,
            "qty_on_hand": row.get::<_, f64>(3)?, "reorder_level": row.get::<_, f64>(4)?,
            "avg_cost_milli": row.get::<_, f64>(5)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    json!({ "inventory": rows })
}

fn tool_get_dashboard_stats(conn: &Connection, _args: &Value) -> Value {
    let customers: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
    let invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices", [], |r| r.get(0)).unwrap_or(0);
    let products: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap_or(0);
    let employees: i64 = conn.query_row("SELECT COUNT(*) FROM employees", [], |r| r.get(0)).unwrap_or(0);
    let revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE status != 'void'",
        [], |r| r.get::<_, i64>(0)).map(|v| v as f64 / 1000.0).unwrap_or(0.0);
    let unpaid: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_invoices WHERE status NOT IN ('paid','void','cancelled')",
        [], |r| r.get(0)).unwrap_or(0);
    json!({
        "customers": customers, "invoices": invoices, "products": products,
        "employees": employees, "total_revenue_omr": revenue, "unpaid_invoices": unpaid,
    })
}

fn tool_list_suppliers(conn: &Connection, args: &Value) -> Value {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let mut stmt = conn.prepare(
        "SELECT id, name, phone, email, balance_milli, active FROM suppliers ORDER BY name LIMIT ?1"
    ).unwrap();
    let rows: Vec<Value> = stmt.query_map([limit], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "name": row.get::<_, String>(1)?,
            "phone": row.get::<_, Option<String>>(2)?, "email": row.get::<_, Option<String>>(3)?,
            "balance": row.get::<_, i64>(4)? as f64 / 1000.0,
            "active": row.get::<_, i64>(5)? != 0,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
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
    let mut stmt = conn.prepare(
        "SELECT id, name, job, nationality, phone, passport_no, active
         FROM employees WHERE name LIKE ?1 OR passport_no LIKE ?1 LIMIT 20"
    ).unwrap();
    let rows: Vec<Value> = stmt.query_map([format!("%{}%", query)], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "name": row.get::<_, String>(1)?,
            "job": row.get::<_, Option<String>>(2)?,
            "nationality": row.get::<_, Option<String>>(3)?,
            "phone": row.get::<_, Option<String>>(4)?,
            "passport_no": row.get::<_, Option<String>>(5)?,
            "active": row.get::<_, i64>(6)? != 0,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    json!({ "employees": rows })
}

fn tool_run_sql(conn: &Connection, args: &Value) -> Value {
    let sql = args.get("sql").and_then(|v| v.as_str()).unwrap_or("");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(100) as usize;

    if !sql.trim().to_lowercase().starts_with("select") {
        return json!({ "error": "Only SELECT queries are allowed via MCP" });
    }

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => return json!({ "error": format!("SQL error: {}", e) }),
    };

    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count).map(|i| stmt.column_name(i).unwrap_or("?").to_string()).collect();

    let rows: Vec<Value> = stmt.query_map([], |row| {
        let mut map = serde_json::Map::new();
        for i in 0..col_count {
            let name = &col_names[i];
            let val: rusqlite::Result<String> = row.get::<_, String>(i);
            match val {
                Ok(v) => { map.insert(name.clone(), Value::String(v)); }
                Err(_) => { map.insert(name.clone(), Value::Null); }
            }
        }
        Ok(Value::Object(map))
    }).unwrap().filter_map(|r| r.ok()).take(limit).collect();

    json!({ "columns": col_names, "rows": rows, "total": rows.len() })
}

// ─── Tool Definitions ──────────────────────────────────────────────────

fn get_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "list_customers".into(),
            description: "List all customers with optional limit".into(),
            inputSchema: json!({ "type": "object", "properties": { "limit": { "type": "integer", "default": 50 } } }),
        },
        McpTool {
            name: "get_customer".into(),
            description: "Get customer details by ID or name search".into(),
            inputSchema: json!({ "type": "object", "properties": {
                "id": { "type": "integer" }, "query": { "type": "string" }
            } }),
        },
        McpTool {
            name: "list_invoices".into(),
            description: "List invoices with optional status or customer filter".into(),
            inputSchema: json!({ "type": "object", "properties": {
                "limit": { "type": "integer", "default": 20 },
                "status": { "type": "string" }, "customer_id": { "type": "integer" }
            } }),
        },
        McpTool {
            name: "get_invoice".into(),
            description: "Get full invoice details with line items".into(),
            inputSchema: json!({ "type": "object", "properties": { "id": { "type": "integer" } }, "required": ["id"] }),
        },
        McpTool {
            name: "list_products".into(),
            description: "List all products with pricing".into(),
            inputSchema: json!({ "type": "object", "properties": { "limit": { "type": "integer", "default": 50 } } }),
        },
        McpTool {
            name: "get_inventory".into(),
            description: "Get current inventory levels and stock alerts".into(),
            inputSchema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "get_dashboard_stats".into(),
            description: "Get ERP dashboard overview (customer/invoice/product counts, revenue)".into(),
            inputSchema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "list_suppliers".into(),
            description: "List all suppliers".into(),
            inputSchema: json!({ "type": "object", "properties": { "limit": { "type": "integer", "default": 50 } } }),
        },
        McpTool {
            name: "get_company_info".into(),
            description: "Get current company settings and information".into(),
            inputSchema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "search_employees".into(),
            description: "Search employees by name, passport, or iqama number".into(),
            inputSchema: json!({ "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] }),
        },
        McpTool {
            name: "run_sql".into(),
            description: "Run a read-only SELECT query on the ERP database (expert only)".into(),
            inputSchema: json!({ "type": "object", "properties": {
                "sql": { "type": "string" }, "limit": { "type": "integer", "default": 100 }
            }, "required": ["sql"] }),
        },
    ]
}

// ─── Resource Definitions ──────────────────────────────────────────────

fn get_resources() -> Vec<McpResource> {
    vec![
        McpResource { uri: "promax://dashboard".into(), name: "ERP Dashboard".into(), description: "Overview statistics of the entire ERP system".into(), mimeType: Some("application/json".into()) },
        McpResource { uri: "promax://customers".into(), name: "Customers".into(), description: "All active customers".into(), mimeType: Some("application/json".into()) },
        McpResource { uri: "promax://invoices/recent".into(), name: "Recent Invoices".into(), description: "Last 20 invoices".into(), mimeType: Some("application/json".into()) },
        McpResource { uri: "promax://inventory".into(), name: "Inventory".into(), description: "Current inventory stock levels".into(), mimeType: Some("application/json".into()) },
        McpResource { uri: "promax://company".into(), name: "Company Info".into(), description: "Current company settings".into(), mimeType: Some("application/json".into()) },
        McpResource { uri: "promax://products".into(), name: "Products".into(), description: "All products and services".into(), mimeType: Some("application/json".into()) },
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
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "promax-mcp",
                    "version": "2.0.0"
                }
            })
        }
        "tools/list" => {
            json!({ "tools": get_tools() })
        }
        "tools/call" => {
            let empty = json!({});
            let params = req.params.as_ref().unwrap_or(&empty);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").unwrap_or(&empty);

            let result = match tool_name {
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
            };

            json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                }]
            })
        }
        "resources/list" => {
            json!({ "resources": get_resources() })
        }
        "resources/read" => {
            let empty = json!({});
            let params = req.params.as_ref().unwrap_or(&empty);
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            match read_resource(conn, uri) {
                Ok(data) => json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&data).unwrap_or_default()
                    }]
                }),
                Err(e) => return JsonRpcResponse { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcError { code: -32602, message: e, data: None }) },
            }
        }
        "prompts/list" => {
            json!({ "prompts": [] })
        }
        "ping" => {
            json!({})
        }
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
                let resp_str = serde_json::to_string(&err_resp).unwrap_or_default();
                writeln!(stdout_lock, "{}", resp_str).ok();
                continue;
            }
        };

        let resp = handle_request(&conn, &req);
        let resp_str = serde_json::to_string(&resp).unwrap_or_default();
        writeln!(stdout_lock, "{}", resp_str).ok();
        stdout_lock.flush().ok();
    }

    Ok(())
}
