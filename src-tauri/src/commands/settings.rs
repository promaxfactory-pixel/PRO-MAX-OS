use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanySettings {
    pub id: i64,
    pub name: Option<String>,
    pub factory_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub vat_number: Option<String>,
    pub cr_number: Option<String>,
    pub logo_path: Option<String>,
    pub stamp_path: Option<String>,
    pub signature_path: Option<String>,
    pub footer_notes: Option<String>,
    pub bank_details: Option<String>,
    pub default_vat_pct: f64,
    pub currency: String,
    pub fiscal_year_start: String,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub bank_iban: Option<String>,
    pub bank_swift: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsInput {
    pub name: Option<String>,
    pub factory_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub vat_number: Option<String>,
    pub cr_number: Option<String>,
    pub logo_path: Option<String>,
    pub stamp_path: Option<String>,
    pub signature_path: Option<String>,
    pub footer_notes: Option<String>,
    pub bank_details: Option<String>,
    pub default_vat_pct: Option<f64>,
    pub currency: Option<String>,
    pub fiscal_year_start: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub bank_iban: Option<String>,
    pub bank_swift: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsUser {
    pub id: i64,
    pub username: String,
    pub full_name: Option<String>,
    pub role: String,
    pub active: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    pub full_name: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserInput {
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub active: Option<i64>,
    pub must_change_password: Option<i64>,
}

fn row_to_settings(row: &rusqlite::Row) -> rusqlite::Result<CompanySettings> {
    Ok(CompanySettings {
        id: row.get(0)?,
        name: row.get(1)?,
        factory_name: row.get(2)?,
        address: row.get(3)?,
        phone: row.get(4)?,
        email: row.get(5)?,
        vat_number: row.get(6)?,
        cr_number: row.get(7)?,
        logo_path: row.get(8)?,
        stamp_path: row.get(9)?,
        signature_path: row.get(10)?,
        footer_notes: row.get(11)?,
        bank_details: row.get(12)?,
        default_vat_pct: row.get(13)?,
        currency: row.get(14)?,
        fiscal_year_start: row.get(15)?,
        bank_name: row.get(16)?,
        bank_account_no: row.get(17)?,
        bank_iban: row.get(18)?,
        bank_swift: row.get(19)?,
    })
}

fn settings_select_sql() -> &'static str {
    "SELECT id, name, factory_name, address, phone, email, vat_number, cr_number, logo_path, stamp_path, signature_path, footer_notes, bank_details, default_vat_pct, currency, fiscal_year_start, bank_name, bank_account_no, bank_iban, bank_swift FROM company_settings"
}

fn settings_insert_sql() -> &'static str {
    "INSERT INTO company_settings(id, name, factory_name, address, phone, email, vat_number, cr_number, logo_path, stamp_path, signature_path, footer_notes, bank_details, default_vat_pct, currency, fiscal_year_start, bank_name, bank_account_no, bank_iban, bank_swift) VALUES(1, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5.0, 'OMR', '01-01', NULL, NULL, NULL, NULL)"
}

fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<SettingsUser> {
    Ok(SettingsUser {
        id: row.get(0)?,
        username: row.get(1)?,
        full_name: row.get(2)?,
        role: row.get(3)?,
        active: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[tauri::command]
pub fn get_company_settings(state: State<'_, DbState>) -> Result<CompanySettings, AppError> {
    let conn = state.0.lock()?;

    let result = conn.query_row(
        settings_select_sql(),
        [],
        row_to_settings,
    );

    match result {
        Ok(s) => Ok(s),
        Err(_) => {
            conn.execute(settings_insert_sql(), [])?;

            conn.query_row(
                settings_select_sql(),
                [],
                row_to_settings,
            ).map_err(|e| AppError::not_found(format!("Settings not found after init: {}", e)))
        }
    }
}

#[tauri::command]
pub fn update_company_settings(state: State<'_, DbState>, user_id: i64, input: UpdateSettingsInput) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager"])?;

    let existing = conn.query_row(
        "SELECT id FROM company_settings LIMIT 1",
        [],
        |row| row.get::<_, i64>(0),
    );

    match existing {
        Ok(_id) => {
            let mut sets = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(v) = &input.name { sets.push("name=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.factory_name { sets.push("factory_name=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.address { sets.push("address=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.phone { sets.push("phone=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.email { sets.push("email=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.vat_number { sets.push("vat_number=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.cr_number { sets.push("cr_number=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.logo_path { sets.push("logo_path=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.stamp_path { sets.push("stamp_path=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.signature_path { sets.push("signature_path=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.footer_notes { sets.push("footer_notes=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.bank_details { sets.push("bank_details=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = input.default_vat_pct { sets.push("default_vat_pct=?"); params.push(Box::new(v)); }
            if let Some(v) = &input.currency { sets.push("currency=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.fiscal_year_start { sets.push("fiscal_year_start=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.bank_name { sets.push("bank_name=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.bank_account_no { sets.push("bank_account_no=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.bank_iban { sets.push("bank_iban=?"); params.push(Box::new(v.clone())); }
            if let Some(v) = &input.bank_swift { sets.push("bank_swift=?"); params.push(Box::new(v.clone())); }

            if sets.is_empty() { return Err(AppError::validation("لا توجد تعديلات")); }

            let sql = format!("UPDATE company_settings SET {} WHERE id=1", sets.join(", "));
            conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
        }
        Err(_) => {
            conn.execute(
                "INSERT INTO company_settings(id, name, factory_name, address, phone, email, vat_number, cr_number, logo_path, stamp_path, signature_path, footer_notes, bank_details, default_vat_pct, currency, fiscal_year_start, bank_name, bank_account_no, bank_iban, bank_swift) VALUES(1,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                rusqlite::params![
                    input.name, input.factory_name, input.address, input.phone, input.email,
                    input.vat_number, input.cr_number, input.logo_path, input.stamp_path, input.signature_path,
                    input.footer_notes, input.bank_details,
                    input.default_vat_pct.unwrap_or(5.0),
                    input.currency.unwrap_or_else(|| "OMR".into()),
                    input.fiscal_year_start.unwrap_or_else(|| "01-01".into()),
                    input.bank_name, input.bank_account_no, input.bank_iban, input.bank_swift,
                ],
            )?;
        }
    }

    let _ = rbac::log_audit(&conn, Some(user_id), None, "update_company_settings", "company_settings", Some(1), None, None, None);
    Ok("تم حفظ الإعدادات بنجاح".to_string())
}

#[tauri::command]
pub fn list_users(state: State<'_, DbState>) -> Result<Vec<SettingsUser>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, username, full_name, role, active, created_at FROM users ORDER BY username"
    )?;
    let rows = stmt.query_map([], row_to_user)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::Database)
}

#[tauri::command]
pub fn create_user(state: State<'_, DbState>, caller_id: i64, input: CreateUserInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;

    rbac::require_role(&conn, caller_id, &["admin"])?;

    if input.username.is_empty() {
        return Err(AppError::validation("اسم المستخدم مطلوب"));
    }
    if input.password.len() < 8 {
        return Err(AppError::validation("كلمة المرور يجب أن تكون 8 أحرف على الأقل"));
    }

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username=?",
        [&input.username],
        |r| r.get::<_, i64>(0),
    )? > 0;

    if exists {
        return Err(AppError::validation("اسم المستخدم موجود بالفعل"));
    }

    let hash = crate::crypto::hash_password(&input.password)
        .map_err(|e| AppError::crypto(format!("فشل تشفير كلمة المرور: {}", e)))?;

    conn.execute(
        "INSERT INTO users(username, password_hash, salt, full_name, role) VALUES(?,?,'',?,?)",
        rusqlite::params![input.username, hash, input.full_name, input.role],
    )?;
    let uid = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_user", "users", Some(uid), None, Some(&input.username), None);
    Ok(uid)
}

#[tauri::command]
pub fn update_user(state: State<'_, DbState>, caller_id: i64, id: i64, input: UpdateUserInput) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    rbac::require_role(&conn, caller_id, &["admin"])?;

    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.full_name { sets.push("full_name=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.role { sets.push("role=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = input.active { sets.push("active=?"); params.push(Box::new(v)); }
    if let Some(v) = input.must_change_password { sets.push("must_change_password=?"); params.push(Box::new(v)); }

    if sets.is_empty() { return Err(AppError::validation("لا توجد تعديلات")); }

    params.push(Box::new(id));
    let sql = format!("UPDATE users SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
    let _ = rbac::log_audit(&conn, None, None, "update_user", "users", Some(id), None, Some(&sets.join(", ")), None);
    Ok("تم تحديث المستخدم بنجاح".to_string())
}

#[tauri::command]
pub fn reset_user_password(state: State<'_, DbState>, caller_id: i64, id: i64, new_password: String) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    rbac::require_role(&conn, caller_id, &["admin"])?;

    if new_password.len() < 8 {
        return Err(AppError::validation("كلمة المرور يجب أن تكون 8 أحرف على الأقل"));
    }

    let hash = crate::crypto::hash_password(&new_password)
        .map_err(|e| AppError::crypto(format!("فشل تشفير كلمة المرور: {}", e)))?;

    conn.execute(
        "UPDATE users SET password_hash=?, salt='', must_change_password=1 WHERE id=?",
        rusqlite::params![hash, id],
    )?;
    let _ = rbac::log_audit(&conn, None, None, "reset_password", "users", Some(id), None, None, None);
    Ok("تم إعادة تعيين كلمة المرور بنجاح".to_string())
}

#[tauri::command]
pub fn delete_user(state: State<'_, DbState>, caller_id: i64, id: i64) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    rbac::require_role(&conn, caller_id, &["admin"])?;

    let username: String = conn.query_row(
        "SELECT username FROM users WHERE id=?",
        [id],
        |r| r.get(0),
    ).map_err(|_| AppError::not_found("المستخدم غير موجود"))?;

    if username == "admin" {
        return Err(AppError::business("لا يمكن حذف المستخدم الرئيسي"));
    }

    conn.execute("DELETE FROM users WHERE id=?", [id])?;

    let _ = rbac::log_audit(&conn, Some(0), None, "delete_user", "users", Some(id), None, None, None);

    Ok("تم حذف المستخدم بنجاح".to_string())
}
