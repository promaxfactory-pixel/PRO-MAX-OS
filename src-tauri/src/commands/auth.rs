use crate::commands::rbac;
use crate::crypto;
use crate::db::DbState;
use crate::error::AppError;
use crate::validation::Validator;
use rusqlite::OptionalExtension;
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

#[derive(Debug, Serialize)]
pub struct InitialSetupStatus {
    pub required: bool,
    pub username: String,
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

fn hash_password_stored(password: &str) -> Result<String, AppError> {
    crypto::hash_password(password)
}

fn validate_new_password(password: &str, confirmation: &str) -> Result<(), AppError> {
    Validator::min_length("new_password", password, 12)?;
    Validator::max_length("new_password", password, 128)?;
    if password != confirmation {
        return Err(AppError::validation("كلمتا المرور غير متطابقتين"));
    }
    let has_upper = password.chars().any(char::is_uppercase);
    let has_lower = password.chars().any(char::is_lowercase);
    let has_digit = password.chars().any(|value| value.is_ascii_digit());
    let has_symbol = password.chars().any(|value| !value.is_alphanumeric());
    if !(has_upper && has_lower && has_digit && has_symbol) {
        return Err(AppError::validation(
            "كلمة المرور يجب أن تحتوي حرفًا كبيرًا وصغيرًا ورقمًا ورمزًا",
        ));
    }
    Ok(())
}

fn initial_setup_is_required(conn: &rusqlite::Connection) -> Result<bool, AppError> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key='initial_admin_setup_required'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value.as_deref() == Some("1"))
}

fn complete_initial_admin_setup_inner(
    conn: &rusqlite::Connection,
    new_password: &str,
    confirm_password: &str,
) -> Result<(), AppError> {
    validate_new_password(new_password, confirm_password)?;
    if !initial_setup_is_required(conn)? {
        return Err(AppError::permission("تهيئة المدير الأولية غير متاحة"));
    }

    let admin_id: i64 = conn
        .query_row(
            "SELECT id FROM users
             WHERE username='admin' AND role='admin' AND active=1 AND must_change_password=1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| AppError::auth("حساب المدير الأولي غير صالح"))?;
    let password_hash = hash_password_stored(new_password)?;

    let tx = conn.unchecked_transaction()?;
    let flag_changed = tx.execute(
        "UPDATE app_settings SET value='0'
         WHERE key='initial_admin_setup_required' AND value='1'",
        [],
    )?;
    if flag_changed != 1 {
        return Err(AppError::permission("تمت تهيئة المدير بالفعل"));
    }
    let admin_changed = tx.execute(
        "UPDATE users
         SET password_hash=?1, salt='', must_change_password=0
         WHERE id=?2 AND must_change_password=1",
        rusqlite::params![password_hash, admin_id],
    )?;
    if admin_changed != 1 {
        return Err(AppError::auth("تعذر تهيئة حساب المدير"));
    }
    rbac::log_audit(
        &tx,
        Some(admin_id),
        Some("admin"),
        "complete_initial_admin_setup",
        "users",
        Some(admin_id),
        Some("must_change_password=1"),
        Some("must_change_password=0"),
        Some("first_run"),
    )?;
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn get_initial_setup_status(
    state: State<'_, DbState>,
) -> Result<InitialSetupStatus, AppError> {
    let conn = state.0.lock()?;
    Ok(InitialSetupStatus {
        required: initial_setup_is_required(&conn)?,
        username: "admin".to_string(),
    })
}

#[tauri::command]
pub fn complete_initial_admin_setup(
    state: State<'_, DbState>,
    new_password: String,
    confirm_password: String,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    complete_initial_admin_setup_inner(&conn, &new_password, &confirm_password)?;
    Ok("تم إعداد حساب المدير بنجاح".to_string())
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

    validate_new_password(&new_password, &new_password)?;

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

    let new_hash = hash_password_stored(&new_password)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE app_settings(key TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE users(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                full_name TEXT,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                role TEXT NOT NULL,
                active INTEGER NOT NULL,
                must_change_password INTEGER NOT NULL,
                created_at TEXT
             );
             CREATE TABLE audit_logs(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts TEXT NOT NULL,
                user_id INTEGER,
                username TEXT,
                action TEXT,
                entity TEXT,
                entity_id INTEGER,
                old_value TEXT,
                new_value TEXT,
                reason TEXT
             );
             INSERT INTO users(username, full_name, password_hash, salt, role, active, must_change_password)
             VALUES('admin', 'Admin', 'unusable-bootstrap-hash', '', 'admin', 1, 1);
             INSERT INTO app_settings(key, value) VALUES('initial_admin_setup_required', '1');",
        )
        .unwrap();
        conn
    }

    fn strong_test_password() -> String {
        ["Factory", "#", "Secure", "2026"].concat()
    }

    #[test]
    fn initial_setup_sets_argon_password_and_is_one_time_only() {
        let conn = setup_connection();
        let password = strong_test_password();

        complete_initial_admin_setup_inner(&conn, &password, &password).unwrap();

        let (hash, must_change): (String, i64) = conn
            .query_row(
                "SELECT password_hash, must_change_password FROM users WHERE username='admin'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(hash.starts_with("$argon2"));
        assert!(verify_password_stored(&password, &hash, ""));
        assert_eq!(must_change, 0);
        assert!(!initial_setup_is_required(&conn).unwrap());
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM audit_logs WHERE action='complete_initial_admin_setup'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        assert!(complete_initial_admin_setup_inner(&conn, &password, &password).is_err());
    }

    #[test]
    fn initial_setup_rejects_weak_or_mismatched_passwords_without_mutation() {
        let conn = setup_connection();
        assert!(complete_initial_admin_setup_inner(&conn, "short", "short").is_err());
        let password = strong_test_password();
        assert!(complete_initial_admin_setup_inner(&conn, &password, "different").is_err());
        assert!(initial_setup_is_required(&conn).unwrap());
        let must_change: i64 = conn
            .query_row(
                "SELECT must_change_password FROM users WHERE username='admin'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(must_change, 1);
    }
}
