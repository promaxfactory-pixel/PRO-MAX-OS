/// PRO MAX OS - REST API Server v2
/// Secure JWT-authenticated API for mobile app and third-party integration.
/// Usage: promax-api [--port 8080] [--db-path path] [--host 127.0.0.1]

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, middleware, HttpRequest};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

// ─── App State ─────────────────────────────────────────────────────────

struct AppState {
    db: Mutex<Connection>,
}

fn find_db_path() -> String {
    std::env::var("PROMAX_DB_PATH").or_else(|_| {
        std::env::var("APPDATA").map(|p| {
            Path::new(&p).join("com.promaxos.app").join("promax.db")
                .to_string_lossy().to_string()
        })
    }).unwrap_or_else(|_| "promax.db".into())
}

fn open_db(path: &str) -> Result<Connection, String> {
    Connection::open(path).map_err(|e| format!("Failed to open database: {}", e))
}

// ─── Auth Middleware ────────────────────────────────────────────────────

fn extract_jwt(req: &HttpRequest) -> Result<promax_os_lib::crypto::Claims, String> {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Missing Authorization header".to_string())?;

    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| "Invalid auth format, use Bearer <token>".to_string())?;

    let claims = promax_os_lib::crypto::verify_jwt(token)?;

    if promax_os_lib::crypto::is_token_blacklisted(&claims.jti) {
        return Err("Token has been revoked".to_string());
    }

    Ok(claims)
}

fn require_auth(req: &HttpRequest) -> Result<promax_os_lib::crypto::Claims, HttpResponse> {
    extract_jwt(req).map_err(|e| HttpResponse::Unauthorized().json(serde_json::json!({"error": e})))
}

#[allow(dead_code)]
fn require_role(req: &HttpRequest, role: &str) -> Result<promax_os_lib::crypto::Claims, HttpResponse> {
    let claims = require_auth(req)?;
    if claims.role != role && claims.role != "admin" {
        return Err(HttpResponse::Forbidden().json(serde_json::json!({"error": "Insufficient permissions"})));
    }
    Ok(claims)
}

// ─── Error Response ────────────────────────────────────────────────────

fn err(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(serde_json::json!({"error": msg.to_string()}))
}

// ─── Auth Endpoints ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    user: String,
    role: String,
}

async fn api_login(state: web::Data<AppState>, body: web::Json<LoginRequest>) -> HttpResponse {
    let conn = match state.db.lock() {
        Ok(c) => c,
        Err(e) => return err(&format!("Lock error: {}", e)),
    };

    // Get user with password hash
    let row = conn.query_row(
        "SELECT id, username, role, password_hash, salt, active FROM users WHERE username = ?1",
        rusqlite::params![body.username],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        },
    );

    let (user_id, _username, role, password_hash, _salt, active) = match row {
        Ok(r) => r,
        Err(_) => return HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Invalid credentials"})),
    };

    if active == 0 {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "User is deactivated"}));
    }

    // Verify password (supports both argon2 and legacy SHA-256)
    let valid = if password_hash.starts_with("$argon2") {
        promax_os_lib::crypto::verify_password(&body.password, &password_hash).unwrap_or(false)
    } else {
        // Legacy SHA-256
        use sha2::{Digest, Sha256};
        let mut current = format!("{}{}", body.password, _salt);
        for _ in 0..10000 {
            current = format!("{:x}", Sha256::digest(current.as_bytes()));
        }
        current == password_hash
    };

    if !valid {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"error": "Invalid credentials"}));
    }

    // Migrate legacy hash to argon2
    if !password_hash.starts_with("$argon2") {
        if let Ok(new_hash) = promax_os_lib::crypto::hash_password(&body.password) {
            let _ = conn.execute(
                "UPDATE users SET password_hash = ?1, salt = '' WHERE id = ?2",
                rusqlite::params![new_hash, user_id],
            );
        }
    }

    match promax_os_lib::crypto::create_jwt(&_username, &role) {
        Ok(token) => HttpResponse::Ok().json(LoginResponse {
            token,
            user: _username,
            role,
        }),
        Err(e) => err(&e),
    }
}

// ─── Dashboard ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct DashboardResponse {
    customers: i64,
    invoices: i64,
    products: i64,
    employees: i64,
    revenue_omr: f64,
    unpaid_invoices: i64,
    low_stock_items: i64,
    pending_shipments: i64,
    recent_invoices: Vec<InvoiceSummary>,
}

#[derive(Serialize)]
struct InvoiceSummary {
    id: i64,
    invoice_no: String,
    date: String,
    customer_name: String,
    total_omr: f64,
    status: String,
}

async fn api_dashboard(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _claims = match require_auth(&req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(e) => return err(&e.to_string()) };

    let customers: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
    let invoices: i64 = conn.query_row("SELECT COUNT(*) FROM sales_invoices", [], |r| r.get(0)).unwrap_or(0);
    let products: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE active = 1", [], |r| r.get(0)).unwrap_or(0);
    let employees: i64 = conn.query_row("SELECT COUNT(*) FROM employees WHERE active = 1", [], |r| r.get(0)).unwrap_or(0);
    let revenue_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE status NOT IN ('void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let unpaid: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_invoices WHERE status NOT IN ('paid','void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let low_stock: i64 = conn.query_row(
        "SELECT COUNT(*) FROM inventory_items WHERE quantity <= min_stock AND min_stock > 0", [], |r| r.get(0)
    ).unwrap_or(0);

    let recent = conn.prepare(
        "SELECT si.id, si.inv_no, si.date, COALESCE(c.name,''), si.total_milli, si.status
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
         ORDER BY si.id DESC LIMIT 10"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(InvoiceSummary {
                id: row.get(0)?, invoice_no: row.get(1)?, date: row.get(2)?,
                customer_name: row.get(3)?, total_omr: row.get::<_, i64>(4)? as f64 / 1000.0,
                status: row.get(5)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(DashboardResponse {
        customers, invoices, products, employees,
        revenue_omr: revenue_milli as f64 / 1000.0, unpaid_invoices: unpaid,
        low_stock_items: low_stock, pending_shipments: 0, recent_invoices: recent,
    })
}

// ─── Customers ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct CustomerResponse {
    id: i64, name: String, phone: Option<String>, email: Option<String>,
    vat_number: Option<String>, credit_limit_omr: f64, balance_omr: f64,
    active: bool, created_at: String,
}

async fn api_list_customers(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _claims = match require_auth(&req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(e) => return err(&e.to_string()) };

    let customers: Vec<CustomerResponse> = conn.prepare(
        "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active, created_at
         FROM customers ORDER BY name LIMIT 200"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(CustomerResponse {
                id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?,
                email: row.get(3)?, vat_number: row.get(4)?,
                credit_limit_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
                balance_omr: row.get::<_, i64>(6)? as f64 / 1000.0,
                active: row.get(7)?, created_at: row.get(8)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(customers)
}

async fn api_get_customer(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> HttpResponse {
    let _claims = match require_auth(&req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(e) => return err(&e.to_string()) };
    let id = path.into_inner();

    match conn.query_row(
        "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active, created_at
         FROM customers WHERE id = ?1", [id],
        |row| Ok(CustomerResponse {
            id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?,
            email: row.get(3)?, vat_number: row.get(4)?,
            credit_limit_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
            balance_omr: row.get::<_, i64>(6)? as f64 / 1000.0,
            active: row.get(7)?, created_at: row.get(8)?,
        })
    ) {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Customer not found"})),
    }
}

// ─── Invoices ──────────────────────────────────────────────────────────

async fn api_list_invoices(state: web::Data<AppState>, req: HttpRequest, query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let _claims = match require_auth(&req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(e) => return err(&e.to_string()) };

    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let status = query.get("status").map(|s| s.as_str());
    let customer_id: Option<i64> = query.get("customer_id").and_then(|v| v.parse().ok());
    let search = query.get("search").map(|s| s.as_str());

    let mut sql = "SELECT si.id, si.inv_no, si.date, COALESCE(c.name,''), si.total_milli, si.status
                   FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
                   WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(s) = status {
        sql.push_str(" AND si.status = ?");
        params.push(Box::new(s.to_string()));
    }
    if let Some(cid) = customer_id {
        sql.push_str(" AND si.customer_id = ?");
        params.push(Box::new(cid));
    }
    if let Some(q) = search {
        sql.push_str(" AND (si.inv_no LIKE ? OR c.name LIKE ?)");
        params.push(Box::new(format!("%{}%", q)));
        params.push(Box::new(format!("%{}%", q)));
    }
    sql.push_str(" ORDER BY si.id DESC LIMIT ?");
    params.push(Box::new(limit));

    let invoices: Vec<InvoiceSummary> = conn.prepare(&sql).ok().and_then(|mut stmt| {
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        stmt.query_map(refs.as_slice(), |row| {
            Ok(InvoiceSummary {
                id: row.get(0)?, invoice_no: row.get(1)?, date: row.get(2)?,
                customer_name: row.get(3)?, total_omr: row.get::<_, i64>(4)? as f64 / 1000.0,
                status: row.get(5)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(invoices)
}

#[derive(Serialize)]
struct InvoiceDetailResponse {
    id: i64, invoice_no: String, date: String, customer_name: String, customer_vat: String,
    net_omr: f64, vat_omr: f64, total_omr: f64, status: String, notes: Option<String>,
    lines: Vec<InvoiceLineResponse>,
}

#[derive(Serialize)]
struct InvoiceLineResponse {
    product: String, qty: f64, unit_price_omr: f64, total_omr: f64,
}

async fn api_get_invoice(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> HttpResponse {
    let _claims = match require_auth(&req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(e) => return err(&e.to_string()) };
    let id = path.into_inner();

    let inv = conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.net_milli, si.vat_milli, si.total_milli, si.status, si.notes,
                COALESCE(c.name,''), COALESCE(c.vat_number,'')
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id WHERE si.id = ?1", [id],
        |row| Ok((
            row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?, row.get::<_, i64>(4)?, row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?, row.get::<_, Option<String>>(7)?,
            row.get::<_, String>(8)?, row.get::<_, String>(9)?,
        ))
    );

    let (inv_id, inv_no, date, net_milli, vat_milli, total_milli, status, notes, cname, cvat) = match inv {
        Ok(r) => r,
        Err(_) => return HttpResponse::NotFound().json(serde_json::json!({"error": "Invoice not found"})),
    };

    let lines: Vec<InvoiceLineResponse> = conn.prepare(
        "SELECT COALESCE(p.name_ar,''), sil.cartons, sil.unit_price_milli, sil.line_net_milli
         FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id = p.id
         WHERE sil.invoice_id = ?1"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([id], |row| {
            Ok(InvoiceLineResponse {
                product: row.get(0)?, qty: row.get(1)?,
                unit_price_omr: row.get::<_, i64>(2)? as f64 / 1000.0,
                total_omr: row.get::<_, i64>(3)? as f64 / 1000.0,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(InvoiceDetailResponse {
        id: inv_id, invoice_no: inv_no, date, customer_name: cname, customer_vat: cvat,
        net_omr: net_milli as f64 / 1000.0, vat_omr: vat_milli as f64 / 1000.0,
        total_omr: total_milli as f64 / 1000.0, status, notes, lines,
    })
}

// ─── Products ──────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ProductResponse {
    id: i64, code: Option<String>, name_ar: String, name_en: Option<String>,
    category: Option<String>, unit_price_omr: f64, active: bool,
}

async fn api_list_products(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _claims = match require_auth(&req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(e) => return err(&e.to_string()) };

    let products: Vec<ProductResponse> = conn.prepare(
        "SELECT id, code, name_ar, name_en, category, unit_price_milli, active FROM products ORDER BY name_ar LIMIT 200"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(ProductResponse {
                id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?,
                name_en: row.get(3)?, category: row.get(4)?,
                unit_price_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
                active: row.get(6)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(products)
}

// ─── Health ────────────────────────────────────────────────────────────

async fn api_health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": "2.0.0",
        "name": "PRO MAX OS API",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// ─── Server ────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 8080u16;
    let mut db_path = find_db_path();
    let mut host = String::from("127.0.0.1");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => { i += 1; if i < args.len() { port = args[i].parse().unwrap_or(8080); } }
            "--db-path" => { i += 1; if i < args.len() { db_path = args[i].clone(); std::env::set_var("PROMAX_DB_PATH", &db_path); } }
            "--host" | "-h" => { i += 1; if i < args.len() { host = args[i].clone(); } }
            "--expose" => { host = "0.0.0.0".to_string(); }
            "--help" => {
                println!("PRO MAX OS API Server v2.0.0");
                println!("Usage: promax-api [OPTIONS]");
                println!("  --port, -p PORT     Port (default: 8080)");
                println!("  --db-path PATH      Database path");
                println!("  --host ADDR         Bind address (default: 127.0.0.1)");
                println!("  --expose            Expose on 0.0.0.0 (dangerous without firewall)");
                println!("  --help              Show this help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    // Initialize secrets from DB path
    let db_path_buf = Path::new(&db_path).to_path_buf();
    let _ = promax_os_lib::crypto::init_secrets(&db_path_buf);

    let is_local = host == "127.0.0.1";
    let bind_addr = format!("{}:{}", &host, port);

    println!("🔐 PRO MAX OS API Server v2.0.0");
    println!("📁 Database: {}", db_path);
    println!("🌐 Listening on: http://{}", bind_addr);
    if is_local {
        println!("⚠️  Localhost only. Use --expose for network access.");
    }

    let db = match open_db(&db_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };
    let data = web::Data::new(AppState { db: Mutex::new(db) });

    HttpServer::new(move || {
        let cors = if is_local {
            Cors::default()
                .allowed_origin("http://localhost:8081")
                .allowed_origin("http://localhost:*")
                .allowed_origin_fn(|origin, _req_head| origin.as_bytes().starts_with(b"http://localhost"))
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        } else {
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        };

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(data.clone())
            .route("/api/health", web::get().to(api_health))
            .route("/api/auth/login", web::post().to(api_login))
            .route("/api/dashboard", web::get().to(api_dashboard))
            .route("/api/customers", web::get().to(api_list_customers))
            .route("/api/customers/{id}", web::get().to(api_get_customer))
            .route("/api/invoices", web::get().to(api_list_invoices))
            .route("/api/invoices/{id}", web::get().to(api_get_invoice))
            .route("/api/products", web::get().to(api_list_products))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
