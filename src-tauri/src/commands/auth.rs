use crate::commands::rbac;
use crate::crypto;
use crate::db::DbState;
use crate::error::AppError;
use crate::validation::Validator;
use serde::{Deserialize, Serialize};
use tauri::State;

const MAX_LOGIN_ATTEMPTS: i64 = 5;
const LOCKOUT_MINUTES: i64 = 15;
const MAX_PASSWORD_CHANGE_ATTEMPTS: i64 = 3;
const PASSWORD_CHANGE_LOCKOUT_MINUTES: i64 = 30;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub full_name: Option<String>,
    pub role: String,
    pub active: i64,
    pub must_change_password: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResult {
    pub user: User,
    pub token: String,
}

fn verify_password_stored(password: &str, hash: &str, salt: &str) -> bool {
    if hash.starts_with("$argon2") {
        crypto::verify_password(password, hash).unwrap_or(false)
    } else {
        use sha2::{Digest, Sha256};
        let mut current = format!("{}{}", password, salt);
        for _ in 0..10000 {
            current = format!("{:x}", Sha256::digest(current.as_bytes()));
        }
        current == hash
    }
}

fn hash_password_stored(password: &str) -> (String, String) {
    let argon_hash = crypto::hash_password(password).unwrap_or_else(|_| {
        use sha2::{Digest, Sha256};
        let mut current = format!("{:x}", Sha256::digest(password.as_bytes()));
        for _ in 0..9999 {
            current = format!("{:x}", Sha256::digest(current.as_bytes()));
        }
        current
    });
    (argon_hash, String::new())
}

fn is_rate_limited(conn: &rusqlite::Connection, username: &str) -> Result<bool, AppError> {
    let cutoff = chrono::Utc::now().timestamp() as f64 - (LOCKOUT_MINUTES * 60) as f64;
    let recent_failures: i64 = conn.query_row(
        "SELECT COUNT(*) FROM login_attempts WHERE username=? AND ok=0 AND ts>=?",
        rusqlite::params![username, cutoff],
        |r| r.get(0),
    )?;
    Ok(recent_failures >= MAX_LOGIN_ATTEMPTS)
}

fn is_password_change_rate_limited(conn: &rusqlite::Connection, user_id: i64) -> Result<bool, AppError> {
    let cutoff = chrono::Utc::now().timestamp() as f64 - (PASSWORD_CHANGE_LOCKOUT_MINUTES * 60) as f64;
    let recent_failures: i64 = conn.query_row(
        "SELECT COUNT(*) FROM password_change_attempts WHERE user_id=? AND ok=0 AND ts>=?",
        rusqlite::params![user_id, cutoff],
        |r| r.get(0),
    )?;
    Ok(recent_failures >= MAX_PASSWORD_CHANGE_ATTEMPTS)
}

fn is_token_validate_rate_limited(conn: &rusqlite::Connection) -> Result<bool, AppError> {
    let cutoff = chrono::Utc::now().timestamp() as f64 - 60.0;
    let recent: i64 = conn.query_row(
        "SELECT COUNT(*) FROM login_attempts WHERE username='_validate_token_' AND ts>=?",
        rusqlite::params![cutoff],
        |r| r.get(0),
    )?;
    Ok(recent >= 60)
}

#[tauri::command]
pub fn login(
    state: State<'_, DbState>,
    username: String,
    password: String,
) -> Result<LoginResult, AppError> {
    Validator::required("username", &username)?;
    Validator::required("password", &password)?;
    Validator::max_length("username", &username, 50)?;

    let conn = state.0.lock()?;

    if is_rate_limited(&conn, &username)? {
        return Err(AppError::auth("تم حظر تسجيل الدخول مؤقتاً بسبب محاولات كثيرة. حاول مرة أخرى بعد 15 دقيقة"));
    }

    let row = conn
        .query_row(
            "SELECT id, username, full_name, role, active, must_change_password, created_at, password_hash, salt FROM users WHERE username = ? AND active = 1",
            [&username],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )
        .map_err(|_| AppError::auth("اسم المستخدم أو كلمة المرور غير صحيحة"))?;

    if !verify_password_stored(&password, &row.7, &row.8) {
        let _ = conn.execute(
            "INSERT INTO login_attempts(username, ts, ok) VALUES(?, ?, 0)",
            rusqlite::params![&username, chrono::Utc::now().timestamp() as f64],
        );
        return Err(AppError::auth("اسم المستخدم أو كلمة المرور غير صحيحة"));
    }

    if row.4 == 0 {
        return Err(AppError::auth("هذا المستخدم معطل"));
    }

    if !row.7.starts_with("$argon2") {
        if let Ok(new_hash) = crypto::hash_password(&password) {
            let _ = conn.execute(
                "UPDATE users SET password_hash = ?, salt = '' WHERE id = ?",
                rusqlite::params![new_hash, row.0],
            );
        }
    }

    let username_clone = row.1.clone();
    let user = User {
        id: row.0,
        username: row.1,
        full_name: row.2,
        role: row.3.clone(),
        active: row.4,
        must_change_password: row.5,
        created_at: row.6,
    };

    let token = crypto::create_tauri_token(row.0, &username_clone, &row.3);

    let _ = conn.execute(
        "INSERT INTO login_attempts(username, ts, ok) VALUES(?, ?, 1)",
        rusqlite::params![&username, chrono::Utc::now().timestamp() as f64],
    );

    Ok(LoginResult { user, token })
}

#[tauri::command]
pub fn quick_login(state: State<'_, DbState>) -> Result<LoginResult, AppError> {
    let conn = state.0.lock()?;

    let row = conn
        .query_row(
            "SELECT id, username, full_name, role, active, must_change_password, created_at, password_hash FROM users WHERE role='admin' AND active=1 ORDER BY id LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .map_err(|_| AppError::auth("لا يوجد حساب مدير نشط للدخول السريع"))?;

    let user = User {
        id: row.0,
        username: row.1.clone(),
        full_name: row.2,
        role: row.3.clone(),
        active: row.4,
        must_change_password: row.5,
        created_at: row.6,
    };

    let token = crypto::create_tauri_token(row.0, &row.1, &row.3);

    Ok(LoginResult { user, token })
}

#[tauri::command]
pub fn get_current_user(state: State<'_, DbState>, user_id: i64) -> Result<User, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, username, full_name, role, active, must_change_password, created_at FROM users WHERE id = ?",
        [user_id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                full_name: row.get(2)?,
                role: row.get(3)?,
                active: row.get(4)?,
                must_change_password: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
    .map_err(|_| AppError::not_found("المستخدم غير موجود"))
}

#[tauri::command]
pub fn change_password(
    state: State<'_, DbState>,
    user_id: i64,
    old_password: String,
    new_password: String,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    rbac::require_role(&conn, user_id, &["admin", "manager", "user"])?;

    if is_password_change_rate_limited(&conn, user_id)? {
        return Err(AppError::auth("تم حظر تغيير كلمة المرور مؤقتاً بسبب محاولات كثيرة. حاول مرة أخرى بعد 30 دقيقة"));
    }

    Validator::min_length("new_password", &new_password, 8)?;
    Validator::max_length("new_password", &new_password, 128)?;

    let current: (String, String) = conn
        .query_row(
            "SELECT password_hash, salt FROM users WHERE id = ?",
            [user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::not_found("المستخدم غير موجود"))?;

    if !verify_password_stored(&old_password, &current.0, &current.1) {
        let _ = conn.execute(
            "INSERT INTO password_change_attempts(user_id, ts, ok) VALUES(?, ?, 0)",
            rusqlite::params![user_id, chrono::Utc::now().timestamp() as f64],
        );
        return Err(AppError::auth("كلمة المرور القديمة غير صحيحة"));
    }

    let (new_hash, _) = hash_password_stored(&new_password);
    conn.execute(
        "UPDATE users SET password_hash = ?, salt = '', must_change_password = 0 WHERE id = ?",
        rusqlite::params![new_hash, user_id],
    )?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "change_password", "users", Some(user_id), None, None, None);

    Ok("تم تغيير كلمة المرور بنجاح".to_string())
}

#[tauri::command]
pub fn validate_token(state: State<'_, DbState>, token: String) -> Result<User, AppError> {
    let conn = state.0.lock()?;

    if is_token_validate_rate_limited(&conn)? {
        return Err(AppError::auth("طلبات التحقق من التوكن كثيرة جداً. حاول مرة أخرى بعد دقيقة"));
    }

    let _ = conn.execute(
        "INSERT INTO login_attempts(username, ts, ok) VALUES('_validate_token_', ?, 1)",
        rusqlite::params![chrono::Utc::now().timestamp() as f64],
    );

    let (user_id, _username, _role) = crypto::validate_tauri_token(&token)?;

    conn.query_row(
        "SELECT id, username, full_name, role, active, must_change_password, created_at FROM users WHERE id = ? AND active = 1",
        [user_id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                full_name: row.get(2)?,
                role: row.get(3)?,
                active: row.get(4)?,
                must_change_password: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
    .map_err(|_| AppError::auth("Token invalid: user not found or inactive"))
}
