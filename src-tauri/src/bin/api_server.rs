// PRO MAX OS - REST API Server v3
// Hardened, JWT-authenticated API for the mobile manager app and integrations.
// Usage: promax-api [--port 8080] [--db-path path] [--host 127.0.0.1] [--expose]

use actix_cors::Cors;
use actix_web::{
    web, App, HttpServer, HttpResponse, HttpRequest, middleware,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ─── App State ─────────────────────────────────────────────────────────

struct AppState {
    db: Mutex<Connection>,
    login_limiter: Mutex<RateLimiter>,
    api_limiter: Mutex<RateLimiter>,
    mobile_dir: String,
}

/// Sliding-window rate limiter keyed by arbitrary string.
struct RateLimiter {
    attempts: HashMap<String, Vec<Instant>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_attempts: usize, window: Duration) -> Self {
        Self {
            attempts: HashMap::new(),
            max_attempts,
            window,
        }
    }

    fn is_allowed(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let entries = self.attempts.entry(key.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < self.window);
        if entries.len() >= self.max_attempts {
            return false;
        }
        entries.push(now);
        true
    }

    fn prune(&mut self) {
        let now = Instant::now();
        let max_len = 100_000;
        if self.attempts.len() > max_len {
            self.attempts.retain(|_, v| {
                v.retain(|t| now.duration_since(*t) < self.window);
                !v.is_empty()
            });
        }
    }
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

fn peer_ip(req: &HttpRequest) -> String {
    req.peer_addr()
        .map(|s| s.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// ─── Auth Middleware ────────────────────────────────────────────────────

#[derive(Clone)]
struct AuthContext {
    username: String,
    role: String,
    user_id: i64,
    full_name: Option<String>,
    jti: String,
}

fn extract_jwt(req: &HttpRequest) -> Result<promax_os_lib::crypto::Claims, HttpResponse> {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing Authorization header"})))?;

    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid auth format, use Bearer <token>"})))?;

    let claims = promax_os_lib::crypto::verify_jwt(token)
        .map_err(|e| HttpResponse::Unauthorized().json(serde_json::json!({"error": e})))?;

    if promax_os_lib::crypto::is_token_blacklisted(&claims.jti) {
        return Err(HttpResponse::Unauthorized().json(serde_json::json!({"error": "Token has been revoked"})));
    }

    Ok(claims)
}

/// Authenticates a request: rate-limits by IP, verifies JWT, and confirms the
/// user still exists and is active (immediate revocation on user deactivation).
fn require_auth(state: &web::Data<AppState>, req: &HttpRequest) -> Result<AuthContext, HttpResponse> {
    let ip = peer_ip(req);

    {
        let mut limiter = state.api_limiter.lock().map_err(|_| err_500("Rate limiter error"))?;
        limiter.prune();
        if !limiter.is_allowed(&format!("ip:{}", ip)) {
            return Err(HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "Too many requests. Slow down and try again shortly."
            })));
        }
    }

    let claims = extract_jwt(req)?;

    let conn = state.db.lock().map_err(|_| err_500("Database lock error"))?;

    let user = conn.query_row(
        "SELECT id, active, COALESCE(full_name, username) FROM users WHERE username = ?1",
        [&claims.sub],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?)),
    );

    let (user_id, active, full_name) = match user {
        Ok(v) => v,
        Err(_) => {
            return Err(HttpResponse::Unauthorized().json(serde_json::json!({"error": "User no longer exists"})));
        }
    };

    if active == 0 {
        return Err(HttpResponse::Forbidden().json(serde_json::json!({"error": "User is deactivated"})));
    }

    Ok(AuthContext {
        username: claims.sub,
        role: claims.role,
        user_id,
        full_name: Some(full_name),
        jti: claims.jti,
    })
}

fn require_role(state: &web::Data<AppState>, req: &HttpRequest, role: &str) -> Result<AuthContext, HttpResponse> {
    let ctx = require_auth(state, req)?;
    if ctx.role != role && ctx.role != "admin" {
        return Err(HttpResponse::Forbidden().json(serde_json::json!({"error": "Insufficient permissions"})));
    }
    Ok(ctx)
}

fn err_500(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(serde_json::json!({"error": msg.to_string()}))
}

fn err_bad(msg: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(serde_json::json!({"error": msg.to_string()}))
}

fn err_not_found(msg: &str) -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({"error": msg.to_string()}))
}

fn err_forbidden(msg: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(serde_json::json!({"error": msg.to_string()}))
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
    full_name: Option<String>,
    user_id: i64,
    expires_in: i64,
    must_change_password: bool,
}

async fn api_login(state: web::Data<AppState>, req: HttpRequest, body: web::Json<LoginRequest>) -> HttpResponse {
    let ip = peer_ip(&req);
    let rate_key = format!("login:{}:{}", ip, body.username);
    {
        let mut limiter = match state.login_limiter.lock() {
            Ok(l) => l,
            Err(_) => return err_500("Internal rate limiter error"),
        };
        limiter.prune();
        if !limiter.is_allowed(&rate_key) {
            return HttpResponse::TooManyRequests()
                .json(serde_json::json!({"error": "Too many login attempts. Try again in 15 minutes."}));
        }
    }

    if body.username.trim().is_empty() || body.password.is_empty() {
        return err_bad("Username and password are required");
    }
    if body.password.len() > 512 || body.username.len() > 128 {
        return err_bad("Invalid input length");
    }

    let conn = match state.db.lock() {
        Ok(c) => c,
        Err(_) => return err_500("Database lock error"),
    };

    let row = conn.query_row(
        "SELECT id, username, role, password_hash, salt, active, COALESCE(full_name, username), must_change_password FROM users WHERE username = ?1",
        rusqlite::params![body.username],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
            ))
        },
    );

    let (user_id, _username, role, password_hash, _salt, active, full_name, must_change_password) = match row {
        Ok(r) => r,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"})),
    };

    if active == 0 {
        return err_forbidden("User is deactivated");
    }

    let valid = if password_hash.starts_with("$argon2") {
        promax_os_lib::crypto::verify_password(&body.password, &password_hash).unwrap_or(false)
    } else {
        use sha2::{Digest, Sha256};
        let mut current = format!("{}{}", body.password, _salt);
        for _ in 0..10000 {
            current = format!("{:x}", Sha256::digest(current.as_bytes()));
        }
        current == password_hash
    };

    if !valid {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"}));
    }

    // Upgrade legacy SHA-256 hashes to Argon2 on successful login.
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
            full_name: Some(full_name),
            user_id,
            expires_in: 86400,
            must_change_password: must_change_password != 0,
        }),
        Err(e) => err_500(&e),
    }
}

async fn api_logout(req: HttpRequest) -> HttpResponse {
    match extract_jwt(&req) {
        Ok(claims) => {
            promax_os_lib::crypto::blacklist_token(&claims.jti);
            HttpResponse::Ok().json(serde_json::json!({"ok": true}))
        }
        Err(resp) => resp,
    }
}

async fn api_me(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    HttpResponse::Ok().json(serde_json::json!({
        "user_id": ctx.user_id,
        "username": ctx.username,
        "role": ctx.role,
        "full_name": ctx.full_name,
    }))
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

async fn api_change_password(state: web::Data<AppState>, req: HttpRequest, body: web::Json<ChangePasswordRequest>) -> HttpResponse {
    let ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };

    if body.new_password.len() < 6 || body.new_password.len() > 128 {
        return err_bad("New password must be between 6 and 128 characters");
    }

    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let stored: Result<(String, String), _> = conn.query_row(
        "SELECT password_hash, salt FROM users WHERE id = ?1", [ctx.user_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    );

    let (password_hash, _salt) = match stored {
        Ok(v) => v,
        Err(_) => return err_not_found("User not found"),
    };

    let valid = if password_hash.starts_with("$argon2") {
        promax_os_lib::crypto::verify_password(&body.current_password, &password_hash).unwrap_or(false)
    } else {
        use sha2::{Digest, Sha256};
        let mut current = format!("{}{}", body.current_password, _salt);
        for _ in 0..10000 {
            current = format!("{:x}", Sha256::digest(current.as_bytes()));
        }
        current == password_hash
    };

    if !valid {
        return err_forbidden("Current password is incorrect");
    }

    match promax_os_lib::crypto::hash_password(&body.new_password) {
        Ok(new_hash) => {
            if let Err(e) = conn.execute(
                "UPDATE users SET password_hash = ?1, salt = '', must_change_password = 0 WHERE id = ?2",
                rusqlite::params![new_hash, ctx.user_id],
            ) {
                return err_500(&format!("Failed to update password: {e}"));
            }
            // Revoke the current token so the client re-authenticates.
            promax_os_lib::crypto::blacklist_token(&ctx.jti);
            HttpResponse::Ok().json(serde_json::json!({"ok": true, "message": "Password changed. Please sign in again."}))
        }
        Err(e) => err_500(&e),
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
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

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
        "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0 AND active = 1", [], |r| r.get(0)
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

// ─── Mobile KPIs ───────────────────────────────────────────────────────

async fn api_kpis(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let today_sales: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE date(date) = date('now') AND status NOT IN ('void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let month_sales: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM sales_invoices WHERE date(date) >= date('now','start of month') AND status NOT IN ('void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let month_purchases: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli), 0) FROM purchases WHERE date(date) >= date('now','start of month')", [], |r| r.get(0)
    ).unwrap_or(0);
    let month_expenses: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_milli), 0) FROM expenses WHERE date(date) >= date('now','start of month')", [], |r| r.get(0)
    ).unwrap_or(0);
    let receivables: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli - paid_milli), 0) FROM sales_invoices WHERE status NOT IN ('paid','void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let payables: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_milli - paid_milli), 0) FROM purchases WHERE status NOT IN ('void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let unpaid_invoices: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_invoices WHERE status NOT IN ('paid','void','cancelled')", [], |r| r.get(0)
    ).unwrap_or(0);
    let low_stock: i64 = conn.query_row(
        "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0 AND active = 1", [], |r| r.get(0)
    ).unwrap_or(0);
    let pending_approvals: i64 = conn.query_row(
        "SELECT COUNT(*) FROM approval_requests WHERE status = 'pending'", [], |r| r.get(0)
    ).unwrap_or(0);
    let pending_approval_amount: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_milli), 0) FROM approval_requests WHERE status = 'pending'", [], |r| r.get(0)
    ).unwrap_or(0);
    let expiring_renewals: i64 = conn.query_row(
        "SELECT COUNT(*) FROM renewals WHERE status = 'active' AND expiry_date IS NOT NULL AND expiry_date != '' AND expiry_date <= date('now', printf('+%d days', COALESCE(alert_days, 30)))", [], |r| r.get(0)
    ).unwrap_or(0);
    let unread_notifications: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notifications WHERE (user_id = ?1 OR user_id IS NULL) AND read_status = 'unread'", [ctx.user_id], |r| r.get(0)
    ).unwrap_or(0);
    let customer_count: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
    let product_count: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE active = 1", [], |r| r.get(0)).unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "today_sales_omr": today_sales as f64 / 1000.0,
        "month_sales_omr": month_sales as f64 / 1000.0,
        "month_purchases_omr": month_purchases as f64 / 1000.0,
        "month_expenses_omr": month_expenses as f64 / 1000.0,
        "receivables_omr": receivables as f64 / 1000.0,
        "payables_omr": payables as f64 / 1000.0,
        "unpaid_invoices": unpaid_invoices,
        "low_stock_items": low_stock,
        "pending_approvals": pending_approvals,
        "pending_approval_amount_omr": pending_approval_amount as f64 / 1000.0,
        "expiring_renewals": expiring_renewals,
        "unread_notifications": unread_notifications,
        "customers": customer_count,
        "products": product_count,
    }))
}

// ─── Customers ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct CustomerResponse {
    id: i64, name: String, phone: Option<String>, email: Option<String>,
    vat_number: Option<String>, credit_limit_omr: f64, balance_omr: f64,
    active: bool,
}

async fn api_list_customers(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let customers: Vec<CustomerResponse> = conn.prepare(
        "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active
         FROM customers ORDER BY name LIMIT 200"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(CustomerResponse {
                id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?,
                email: row.get(3)?, vat_number: row.get(4)?,
                credit_limit_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
                balance_omr: row.get::<_, i64>(6)? as f64 / 1000.0,
                active: row.get::<_, i64>(7)? != 0,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(customers)
}

async fn api_get_customer(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let id = path.into_inner();

    match conn.query_row(
        "SELECT id, name, phone, email, vat_number, credit_limit_milli, balance_milli, active
         FROM customers WHERE id = ?1", [id],
        |row| Ok(CustomerResponse {
            id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?,
            email: row.get(3)?, vat_number: row.get(4)?,
            credit_limit_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
            balance_omr: row.get::<_, i64>(6)? as f64 / 1000.0,
            active: row.get::<_, i64>(7)? != 0,
        })
    ) {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(_) => err_not_found("Customer not found"),
    }
}

// ─── Suppliers ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct SupplierResponse {
    id: i64, name: String, phone: Option<String>, email: Option<String>,
    vat_number: Option<String>, balance_omr: f64, active: bool,
}

async fn api_list_suppliers(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let suppliers: Vec<SupplierResponse> = conn.prepare(
        "SELECT id, name, phone, email, vat_number, balance_milli, active
         FROM suppliers ORDER BY name LIMIT 200"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(SupplierResponse {
                id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?,
                email: row.get(3)?, vat_number: row.get(4)?,
                balance_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
                active: row.get::<_, i64>(6)? != 0,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(suppliers)
}

// ─── Invoices ──────────────────────────────────────────────────────────

async fn api_list_invoices(state: web::Data<AppState>, req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50).min(500);
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
    net_omr: f64, vat_omr: f64, total_omr: f64, paid_omr: f64, status: String, notes: Option<String>,
    lines: Vec<InvoiceLineResponse>,
}

#[derive(Serialize)]
struct InvoiceLineResponse {
    product: String, qty: f64, unit_price_omr: f64, total_omr: f64,
}

async fn api_get_invoice(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let id = path.into_inner();

    let inv = conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.net_milli, si.vat_milli, si.total_milli, si.paid_milli, si.status, si.notes,
                COALESCE(c.name,''), COALESCE(c.vat_number,'')
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id WHERE si.id = ?1", [id],
        |row| Ok((
            row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?, row.get::<_, i64>(4)?, row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?, row.get::<_, String>(7)?, row.get::<_, Option<String>>(8)?,
            row.get::<_, String>(9)?, row.get::<_, String>(10)?,
        ))
    );

    let (inv_id, inv_no, date, net_milli, vat_milli, total_milli, paid_milli, status, notes, cname, cvat) = match inv {
        Ok(r) => r,
        Err(_) => return err_not_found("Invoice not found"),
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
        total_omr: total_milli as f64 / 1000.0, paid_omr: paid_milli as f64 / 1000.0,
        status, notes, lines,
    })
}

// ─── Purchases & Expenses ──────────────────────────────────────────────

#[derive(Serialize)]
struct PurchaseSummary {
    id: i64,
    pur_no: String,
    date: String,
    supplier_name: String,
    total_omr: f64,
    status: String,
}

async fn api_list_purchases(state: web::Data<AppState>, req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50).min(500);

    let rows: Vec<PurchaseSummary> = conn.prepare(
        "SELECT p.id, COALESCE(p.pur_no,''), p.date, COALESCE(s.name,''), p.total_milli, p.status
         FROM purchases p LEFT JOIN suppliers s ON p.supplier_id = s.id
         ORDER BY p.id DESC LIMIT ?"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([limit], |row| {
            Ok(PurchaseSummary {
                id: row.get(0)?, pur_no: row.get(1)?, date: row.get(2)?,
                supplier_name: row.get(3)?, total_omr: row.get::<_, i64>(4)? as f64 / 1000.0,
                status: row.get(5)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(rows)
}

#[derive(Serialize)]
struct ExpenseSummary {
    id: i64,
    exp_no: String,
    date: String,
    category: String,
    amount_omr: f64,
    status: String,
}

async fn api_list_expenses(state: web::Data<AppState>, req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50).min(500);

    let rows: Vec<ExpenseSummary> = conn.prepare(
        "SELECT id, COALESCE(exp_no,''), date, COALESCE(category,''), amount_milli, approval_status
         FROM expenses ORDER BY id DESC LIMIT ?"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([limit], |row| {
            Ok(ExpenseSummary {
                id: row.get(0)?, exp_no: row.get(1)?, date: row.get(2)?,
                category: row.get(3)?, amount_omr: row.get::<_, i64>(4)? as f64 / 1000.0,
                status: row.get(5)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(rows)
}

#[derive(Deserialize)]
struct CreateExpenseRequest {
    amount_omr: f64,
    date: Option<String>,
    category: Option<String>,
    method: Option<String>,
    vendor: Option<String>,
    reference: Option<String>,
    notes: Option<String>,
}

async fn api_create_expense(state: web::Data<AppState>, req: HttpRequest, body: web::Json<CreateExpenseRequest>) -> HttpResponse {
    let ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let amount = body.amount_omr;
    if !amount.is_finite() || amount <= 0.0 || amount > 1_000_000_000_000.0 {
        return err_bad("Amount must be a positive number");
    }
    let amount_milli = (amount * 1000.0).round() as i64;
    if amount_milli <= 0 {
        return err_bad("Amount is too small");
    }
    let date = body.date.clone().unwrap_or_default().trim().to_string();
    if !date.is_empty() && chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return err_bad("Date must be in YYYY-MM-DD format");
    }
    let date = if date.is_empty() { chrono::Local::now().format("%Y-%m-%d").to_string() } else { date };
    let category = body.category.clone().unwrap_or_default().trim().to_string();
    let method = body.method.clone().unwrap_or_default().trim().to_string();
    let method = if method.is_empty() { "cash".to_string() } else { method };
    let vendor = body.vendor.clone().unwrap_or_default().trim().to_string();
    let reference = body.reference.clone().unwrap_or_default().trim().to_string();
    let notes = body.notes.clone().unwrap_or_default().trim().to_string();

    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let result = conn.execute(
        "INSERT INTO expenses (exp_no, date, category, amount_milli, method, vendor, reference, notes, approval_status, created_by, created_at)
         VALUES ('', ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'posted', ?8, datetime('now'))",
        rusqlite::params![date, category, amount_milli, method, vendor, reference, notes, ctx.username],
    );
    let id = match result {
        Ok(_) => conn.last_insert_rowid(),
        Err(e) => return err_500(&format!("Failed to insert expense: {e}")),
    };
    let _ = conn.execute(
        "UPDATE expenses SET exp_no = 'EXP-' || id WHERE id = ?",
        [id],
    );

    let new_value = serde_json::json!({
        "amount_omr": amount, "date": date, "category": category,
        "method": method, "vendor": vendor, "reference": reference, "notes": notes
    });
    let _ = conn.execute(
        "INSERT INTO audit_logs (ts, user_id, username, action, entity, entity_id, new_value) VALUES (datetime('now'), ?1, ?2, 'create', 'expenses', ?3, ?4)",
        rusqlite::params![ctx.user_id, ctx.username, id, new_value.to_string()],
    );

    HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "id": id,
        "exp_no": format!("EXP-{id}"),
        "date": date,
        "category": category,
        "amount_omr": amount_milli as f64 / 1000.0,
        "status": "posted",
    }))
}

// ─── Products ──────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ProductResponse {
    id: i64, code: Option<String>, name_ar: String, name_en: Option<String>,
    price_omr: f64, cost_omr: f64, stock: f64, active: bool,
}

async fn api_list_products(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let products: Vec<ProductResponse> = conn.prepare(
        "SELECT p.id, p.code, p.name_ar, p.name_en, p.default_price_milli, p.default_cost_milli, p.active,
                COALESCE((SELECT SUM(qty_on_hand) FROM inventory_items WHERE product_id = p.id), 0)
         FROM products p
         WHERE p.active = 1
         ORDER BY p.name_ar LIMIT 200"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(ProductResponse {
                id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?,
                name_en: row.get(3)?,
                price_omr: row.get::<_, i64>(4)? as f64 / 1000.0,
                cost_omr: row.get::<_, i64>(5)? as f64 / 1000.0,
                active: row.get::<_, i64>(6)? != 0,
                stock: row.get::<_, f64>(7)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(products)
}

// ─── Approvals ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ApprovalResponse {
    id: i64,
    request_type: String,
    entity_type: String,
    entity_id: i64,
    entity_number: String,
    requested_by: String,
    requested_at: String,
    amount_omr: Option<f64>,
    description: Option<String>,
    status: String,
    priority: String,
}

async fn api_list_approvals(state: web::Data<AppState>, req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let status = query.get("status").map(|s| s.as_str());
    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(100).min(500);

    let mut sql = "SELECT id, request_type, entity_type, entity_id, entity_number, requested_by, requested_at, amount_milli, description, status, priority
                   FROM approval_requests WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    if let Some(s) = status {
        sql.push_str(" AND status = ?");
        params.push(Box::new(s.to_string()));
    }
    sql.push_str(" ORDER BY CASE priority WHEN 'urgent' THEN 0 WHEN 'high' THEN 1 WHEN 'normal' THEN 2 ELSE 3 END, requested_at DESC LIMIT ?");
    params.push(Box::new(limit));

    let approvals: Vec<ApprovalResponse> = conn.prepare(&sql).ok().and_then(|mut stmt| {
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        stmt.query_map(refs.as_slice(), |row| {
            Ok(ApprovalResponse {
                id: row.get(0)?, request_type: row.get(1)?, entity_type: row.get(2)?,
                entity_id: row.get(3)?, entity_number: row.get(4)?,
                requested_by: row.get(5)?, requested_at: row.get(6)?,
                amount_omr: row.get::<_, Option<i64>>(7)?.map(|v| v as f64 / 1000.0),
                description: row.get(8)?, status: row.get(9)?, priority: row.get(10)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(approvals)
}

#[derive(Deserialize)]
struct DecideApprovalRequest {
    decision: String,
    reason: Option<String>,
}

async fn api_decide_approval(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>, body: web::Json<DecideApprovalRequest>) -> HttpResponse {
    let ctx = match require_role(&state, &req, "manager") { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let id = path.into_inner();

    let current_status: Result<String, _> = conn.query_row(
        "SELECT status FROM approval_requests WHERE id = ?", [id], |r| r.get(0)
    );
    let current_status = match current_status {
        Ok(s) => s,
        Err(_) => return err_not_found("Approval request not found"),
    };
    if current_status != "pending" {
        return err_bad(&format!("Cannot decide on request with status '{current_status}'"));
    }

    let decision = body.decision.trim().to_lowercase();
    match decision.as_str() {
        "approve" => {
            if let Err(e) = conn.execute(
                "UPDATE approval_requests SET status = 'approved', approved_by = ?1, approved_at = datetime('now') WHERE id = ?2",
                rusqlite::params![ctx.username, id],
            ) {
                return err_500(&format!("Failed to update: {e}"));
            }
        }
        "reject" => {
            if let Err(e) = conn.execute(
                "UPDATE approval_requests SET status = 'rejected', approved_by = ?1, approved_at = datetime('now'), rejection_reason = ?3 WHERE id = ?2",
                rusqlite::params![ctx.username, id, body.reason],
            ) {
                return err_500(&format!("Failed to update: {e}"));
            }
        }
        _ => return err_bad("Decision must be 'approve' or 'reject'"),
    }

    let _ = conn.execute(
        "INSERT INTO audit_logs (ts, user_id, username, action, entity, entity_id, new_value, reason) VALUES (datetime('now'), ?1, ?2, 'decide', 'approval_requests', ?3, ?4, ?5)",
        rusqlite::params![ctx.user_id, ctx.username, id, decision, body.reason],
    );

    HttpResponse::Ok().json(serde_json::json!({"ok": true, "status": decision}))
}

// ─── Notifications ─────────────────────────────────────────────────────

#[derive(Serialize)]
struct NotificationResponse {
    id: i64,
    notification_type: String,
    title: String,
    message: String,
    entity_type: Option<String>,
    entity_id: Option<i64>,
    severity: String,
    read_status: String,
    action_url: Option<String>,
    created_at: String,
}

async fn api_list_notifications(state: web::Data<AppState>, req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50).min(500);
    let unread_only = query.get("unread").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

    let mut sql = "SELECT id, notification_type, title, message, entity_type, entity_id, severity, read_status, action_url, created_at
                   FROM notifications WHERE (user_id = ?1 OR user_id IS NULL)".to_string();
    if unread_only {
        sql.push_str(" AND read_status = 'unread'");
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT ?");

    let rows: Vec<NotificationResponse> = conn.prepare(&sql).ok().and_then(|mut stmt| {
        stmt.query_map(rusqlite::params![ctx.user_id, limit], |row| {
            Ok(NotificationResponse {
                id: row.get(0)?, notification_type: row.get(1)?, title: row.get(2)?,
                message: row.get(3)?, entity_type: row.get(4)?, entity_id: row.get(5)?,
                severity: row.get(6)?, read_status: row.get(7)?,
                action_url: row.get(8)?, created_at: row.get(9)?,
            })
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(rows)
}

async fn api_mark_notification_read(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let id = path.into_inner();
    let affected = conn.execute(
        "UPDATE notifications SET read_status = 'read', read_at = datetime('now') WHERE id = ?1 AND read_status = 'unread'",
        [id],
    ).unwrap_or(0);
    if affected == 0 {
        return err_not_found("Notification not found or already read");
    }
    HttpResponse::Ok().json(serde_json::json!({"ok": true}))
}

// ─── Alerts ────────────────────────────────────────────────────────────

async fn api_alerts(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let low_stock: Vec<serde_json::Value> = conn.prepare(
        "SELECT i.id, i.name_ar, i.qty_on_hand, i.reorder_level
         FROM inventory_items i
         WHERE i.qty_on_hand <= i.reorder_level AND i.reorder_level > 0 AND i.active = 1
         ORDER BY i.qty_on_hand ASC LIMIT 50"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "item_id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "quantity": row.get::<_, f64>(2)?,
                "reorder_level": row.get::<_, f64>(3)?,
            }))
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    let expiring_renewals: Vec<serde_json::Value> = conn.prepare(
        "SELECT id, name, expiry_date FROM renewals
         WHERE status = 'active' AND expiry_date IS NOT NULL AND expiry_date != ''
         AND expiry_date <= date('now', printf('+%d days', COALESCE(alert_days, 30)))
         ORDER BY expiry_date ASC LIMIT 50"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "expiry_date": row.get::<_, String>(2)?,
            }))
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    let overdue_invoices: Vec<serde_json::Value> = conn.prepare(
        "SELECT si.id, si.inv_no, COALESCE(c.name,''), (si.total_milli - si.paid_milli), si.date
         FROM sales_invoices si LEFT JOIN customers c ON si.customer_id = c.id
         WHERE si.status NOT IN ('paid','void','cancelled') AND (si.total_milli - si.paid_milli) > 0
         ORDER BY si.date ASC LIMIT 50"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "invoice_no": row.get::<_, String>(1)?,
                "customer": row.get::<_, String>(2)?,
                "due_omr": row.get::<_, i64>(3)? as f64 / 1000.0,
                "date": row.get::<_, String>(4)?,
            }))
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "low_stock": low_stock,
        "expiring_renewals": expiring_renewals,
        "overdue_invoices": overdue_invoices,
    }))
}

// ─── Activity / Audit Log ──────────────────────────────────────────────

async fn api_activity(state: web::Data<AppState>, req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };
    let limit: i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50).min(200);

    let rows: Vec<serde_json::Value> = conn.prepare(
        "SELECT id, ts, COALESCE(username,''), action, entity, entity_id, reason FROM audit_logs ORDER BY id DESC LIMIT ?"
    ).ok().and_then(|mut stmt| {
        stmt.query_map([limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "ts": row.get::<_, String>(1)?,
                "user": row.get::<_, String>(2)?,
                "action": row.get::<_, String>(3)?,
                "entity": row.get::<_, String>(4)?,
                "entity_id": row.get::<_, Option<i64>>(5)?,
                "reason": row.get::<_, Option<String>>(6)?,
            }))
        }).ok().map(|iter| iter.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    HttpResponse::Ok().json(rows)
}

// ─── Company Info ──────────────────────────────────────────────────────

async fn api_company(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let _ctx = match require_auth(&state, &req) { Ok(c) => c, Err(e) => return e };
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return err_500("Database lock error") };

    let company = conn.query_row(
        "SELECT name, factory_name, address, phone, email, vat_number, default_vat_pct FROM company_settings WHERE id = 1",
        [],
        |row| Ok(serde_json::json!({
            "name": row.get::<_, Option<String>>(0)?,
            "factory_name": row.get::<_, Option<String>>(1)?,
            "address": row.get::<_, Option<String>>(2)?,
            "phone": row.get::<_, Option<String>>(3)?,
            "email": row.get::<_, Option<String>>(4)?,
            "vat_number": row.get::<_, Option<String>>(5)?,
            "default_vat_pct": row.get::<_, f64>(6)?,
        })),
    );

    match company {
        Ok(c) => HttpResponse::Ok().json(c),
        Err(_) => HttpResponse::Ok().json(serde_json::json!({"name": null})),
    }
}

// ─── Mobile PWA (static files) ─────────────────────────────────────────
// Serves the mobile manager app from the configured mobile directory. The
// single page app is served at "/" with a fallback to index.html so client
// routing works. API paths are never served by this handler.

fn mime_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "webmanifest" => "application/manifest+json; charset=utf-8",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "map" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Resolves a request path under the mobile dir. Returns None on any path
/// traversal attempt (.., absolute paths, backslashes).
fn resolve_mobile_path(mobile_dir: &str, uri: &str) -> Option<PathBuf> {
    let rel = uri.split('?').next().unwrap_or("/");
    let mut clean = rel.to_string();
    while clean.starts_with('/') {
        clean = clean[1..].to_string();
    }
    // Reject encoded separators and dot-segments outright: %2f, %5c, %2e.
    let lower = clean.to_ascii_lowercase();
    if lower.contains("%2f") || lower.contains("%5c") || lower.contains("%2e") || lower.contains("%00") {
        return None;
    }
    if clean.contains('\\') || clean.is_empty() {
        clean = "index.html".to_string();
    }
    for part in clean.split('/') {
        if part == ".." || part == "." || part.contains(':') || part.contains('\0') {
            return None;
        }
    }
    Some(Path::new(mobile_dir).join(&clean))
}

async fn serve_static(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let uri = req.path().to_string();
    let root = PathBuf::from(&state.mobile_dir);
    let Some(rel) = resolve_mobile_path(&state.mobile_dir, &uri) else {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Bad path"}));
    };

    let index = root.join("index.html");
    let has_ext = rel.extension().is_some();

    // File IO runs on the blocking pool so it never stalls the actix workers.
    let read_result = web::block(move || -> std::io::Result<(PathBuf, Vec<u8>)> {
        // SPA fallback: extension-less navigation paths (or a missing file) serve index.html.
        if has_ext {
            if let Ok(bytes) = std::fs::read(&rel) {
                return Ok((rel, bytes));
            }
        }
        std::fs::read(&index).map(|bytes| (index, bytes))
    })
    .await;

    match read_result {
        Ok(Ok((served_path, bytes))) => {
            let is_sw = served_path.file_name().map(|n| n == "sw.js").unwrap_or(false);
            let cache_ctl = if is_sw {
                "no-cache"
            } else {
                "public, max-age=3600"
            };
            HttpResponse::Ok()
                .content_type(mime_for(&served_path))
                .insert_header(("Cache-Control", cache_ctl))
                .body(bytes)
        }
        Ok(Err(_)) => HttpResponse::NotFound().body("Not found"),
        Err(_) => HttpResponse::InternalServerError().body("I/O error"),
    }
}

// ─── Health ────────────────────────────────────────────────────────────
async fn api_health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": "3.0.0",
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
    let mut mobile_dir = std::env::var("PROMAX_MOBILE_DIR").unwrap_or_else(|_| "mobile".to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => { i += 1; if i < args.len() { port = args[i].parse().unwrap_or(8080); } }
            "--db-path" => { i += 1; if i < args.len() { db_path = args[i].clone(); std::env::set_var("PROMAX_DB_PATH", &db_path); } }
            "--host" | "-h" => { i += 1; if i < args.len() { host = args[i].clone(); } }
            "--expose" => { host = "0.0.0.0".to_string(); }
            "--mobile-dir" => { i += 1; if i < args.len() { mobile_dir = args[i].clone(); } }
            "--help" => {
                println!("PRO MAX OS API Server v3.0.0");
                println!("Usage: promax-api [OPTIONS]");
                println!("  --port, -p PORT     Port (default: 8080)");
                println!("  --db-path PATH      Database path");
                println!("  --host ADDR         Bind address (default: 127.0.0.1)");
                println!("  --expose            Expose on 0.0.0.0 (use with firewall / HTTPS reverse proxy)");
                println!("  --mobile-dir DIR    Serve the mobile PWA from DIR (default: mobile)");
                println!("  --help              Show this help");
                println!();
                println!("SECURITY NOTES:");
                println!("  * Authenticated endpoints are rate-limited per IP and verify the user is active.");
                println!("  * Use HTTPS in production: put nginx/caddy in front with TLS.");
                println!("  * Prefer --host 127.0.0.1 for local-only use.");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    let db_path_buf = Path::new(&db_path).to_path_buf();
    let _ = promax_os_lib::crypto::init_secrets(&db_path_buf);

    let is_local = host == "127.0.0.1";
    let bind_addr = format!("{}:{}", host, port);

    println!("PRO MAX OS API Server v3.0.0");
    println!("Database: {}", db_path);
    println!("Listening on: http://{}", bind_addr);
    println!("Mobile PWA: {}", Path::new(&mobile_dir).canonicalize().unwrap_or_else(|_| PathBuf::from(&mobile_dir)).display());
    if is_local {
        println!("Localhost only. Use --expose for network access.");
    } else {
        println!("WARNING: Network exposure enabled. Ensure a firewall is configured and use an HTTPS reverse proxy.");
    }

    let db = match open_db(&db_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    let data = web::Data::new(AppState {
        db: Mutex::new(db),
        login_limiter: Mutex::new(RateLimiter::new(10, Duration::from_secs(900))),
        api_limiter: Mutex::new(RateLimiter::new(600, Duration::from_secs(300))),
        mobile_dir: mobile_dir.clone(),
    });

    HttpServer::new(move || {
        let cors = if is_local {
            Cors::default()
                .allowed_origin("http://localhost:8081")
                .allowed_origin("http://localhost:5173")
                .allowed_origin("http://localhost:1420")
                .allowed_origin_fn(|origin, _req_head| origin.as_bytes().starts_with(b"http://localhost"))
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        } else {
            // Non-local: native mobile apps send no Origin header, so the
            // origin allowlist only matters for browsers. Restrict methods/headers.
            Cors::default()
                .allowed_methods(["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(["Authorization", "Content-Type"])
                .max_age(3600)
        };

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("X-Frame-Options", "DENY"))
                .add(("X-XSS-Protection", "1; mode=block"))
                .add(("Referrer-Policy", "no-referrer"))
                .add(("Permissions-Policy", "camera=(), microphone=(), geolocation=()"))
                .add(("Cache-Control", "no-store"))
            )
            .app_data(web::JsonConfig::default().limit(1_048_576)) // 1MB max request body
            .app_data(data.clone())
            .route("/api/health", web::get().to(api_health))
            .route("/api/auth/login", web::post().to(api_login))
            .route("/api/auth/logout", web::post().to(api_logout))
            .route("/api/auth/me", web::get().to(api_me))
            .route("/api/auth/change-password", web::post().to(api_change_password))
            .route("/api/dashboard", web::get().to(api_dashboard))
            .route("/api/kpis", web::get().to(api_kpis))
            .route("/api/customers", web::get().to(api_list_customers))
            .route("/api/customers/{id}", web::get().to(api_get_customer))
            .route("/api/suppliers", web::get().to(api_list_suppliers))
            .route("/api/invoices", web::get().to(api_list_invoices))
            .route("/api/invoices/{id}", web::get().to(api_get_invoice))
            .route("/api/purchases", web::get().to(api_list_purchases))
            .route("/api/expenses", web::get().to(api_list_expenses))
            .route("/api/expenses", web::post().to(api_create_expense))
            .route("/api/products", web::get().to(api_list_products))
            .route("/api/approvals", web::get().to(api_list_approvals))
            .route("/api/approvals/{id}/decide", web::post().to(api_decide_approval))
            .route("/api/notifications", web::get().to(api_list_notifications))
            .route("/api/notifications/{id}/read", web::post().to(api_mark_notification_read))
            .route("/api/alerts", web::get().to(api_alerts))
            .route("/api/activity", web::get().to(api_activity))
            .route("/api/company", web::get().to(api_company))
            .route("/{path:.*}", web::get().to(serve_static))
    })
    .workers(2)
    .max_connection_rate(300)
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_secs(30))
    .bind(&bind_addr)?
    .run()
    .await
}
