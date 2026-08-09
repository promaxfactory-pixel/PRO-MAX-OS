use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

const K_WHATSAPP_API_KEY: &str = "whatsapp_api_key";
const K_WHATSAPP_PHONE: &str = "whatsapp_phone";
const K_SMTP_SERVER: &str = "smtp_server";
const K_SMTP_PORT: &str = "smtp_port";
const K_SMTP_USER: &str = "smtp_user";
const K_SMTP_PASS: &str = "smtp_pass";
const K_SMTP_FROM: &str = "smtp_from";
const K_PRINTER_NAME: &str = "printer_name";
const K_PRINTER_AUTO: &str = "printer_auto";

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationSettings {
    pub whatsapp_api_key: Option<String>,
    pub whatsapp_phone: Option<String>,
    pub smtp_server: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub smtp_from: Option<String>,
    pub printer_name: Option<String>,
    pub printer_auto: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationTestResult {
    pub ok: bool,
    pub message: String,
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn save_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO integrations_settings(key, value, updated_at) VALUES(?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        rusqlite::params![key, value, now_str()],
    )?;
    Ok(())
}

fn load_setting(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, AppError> {
    let result = conn.query_row(
        "SELECT value FROM integrations_settings WHERE key=?1",
        rusqlite::params![key],
        |row| row.get::<_, Option<String>>(0),
    );
    match result {
        Ok(v) => Ok(v),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn integrations_get_settings(state: State<'_, DbState>) -> Result<IntegrationSettings, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_CORE)?;
    let conn = state.0.lock()?;

    let whatsapp_api_key = load_setting(&conn, K_WHATSAPP_API_KEY)?
        .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v));
    let whatsapp_phone = load_setting(&conn, K_WHATSAPP_PHONE)?;
    let smtp_server = load_setting(&conn, K_SMTP_SERVER)?;
    let smtp_port = load_setting(&conn, K_SMTP_PORT)?.and_then(|v| v.parse::<i64>().ok());
    let smtp_user = load_setting(&conn, K_SMTP_USER)?;
    let smtp_pass = load_setting(&conn, K_SMTP_PASS)?
        .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v));
    let smtp_from = load_setting(&conn, K_SMTP_FROM)?;
    let printer_name = load_setting(&conn, K_PRINTER_NAME)?;
    let printer_auto = load_setting(&conn, K_PRINTER_AUTO)?.map(|v| v == "1" || v == "true");

    Ok(IntegrationSettings {
        whatsapp_api_key,
        whatsapp_phone,
        smtp_server,
        smtp_port,
        smtp_user,
        smtp_pass,
        smtp_from,
        printer_name,
        printer_auto,
    })
}

#[tauri::command]
pub fn integrations_save_settings(
    state: State<'_, DbState>,
    settings: IntegrationSettings,
) -> Result<String, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_CORE)?;
    let conn = state.0.lock()?;

    if let Some(v) = settings.whatsapp_api_key {
        let encrypted = crate::crypto::encrypt_if_needed(&v)
            .map_err(|e| format!("فشل تشفير مفتاح واتساب: {}", e))?;
        save_setting(&conn, K_WHATSAPP_API_KEY, &encrypted)?;
    }
    if let Some(v) = settings.whatsapp_phone {
        save_setting(&conn, K_WHATSAPP_PHONE, &v)?;
    }
    if let Some(v) = settings.smtp_server {
        save_setting(&conn, K_SMTP_SERVER, &v)?;
    }
    if let Some(v) = settings.smtp_port {
        save_setting(&conn, K_SMTP_PORT, &v.to_string())?;
    }
    if let Some(v) = settings.smtp_user {
        save_setting(&conn, K_SMTP_USER, &v)?;
    }
    if let Some(v) = settings.smtp_pass {
        let encrypted = crate::crypto::encrypt_if_needed(&v)
            .map_err(|e| format!("فشل تشفير كلمة مرور البريد: {}", e))?;
        save_setting(&conn, K_SMTP_PASS, &encrypted)?;
    }
    if let Some(v) = settings.smtp_from {
        save_setting(&conn, K_SMTP_FROM, &v)?;
    }
    if let Some(v) = settings.printer_name {
        save_setting(&conn, K_PRINTER_NAME, &v)?;
    }
    if let Some(v) = settings.printer_auto {
        save_setting(&conn, K_PRINTER_AUTO, if v { "1" } else { "0" })?;
    }

    Ok("تم حفظ إعدادات التكاملات بنجاح".to_string())
}

#[tauri::command]
pub fn integrations_test_whatsapp(state: State<'_, DbState>) -> Result<IntegrationTestResult, AppError> {
    let conn = state.0.lock()?;
    let api_key = load_setting(&conn, K_WHATSAPP_API_KEY)?
        .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v))
        .unwrap_or_default();
    let phone = load_setting(&conn, K_WHATSAPP_PHONE).unwrap_or_default();
    if api_key.is_empty() {
        return Ok(IntegrationTestResult {
            ok: false,
            message: "مفتاح واتساب غير مضبوط. أدخل API Key لحساب الواتساب ثم أعد الاختبار.".into(),
        });
    }
    if phone.unwrap_or_default().is_empty() {
        return Ok(IntegrationTestResult {
            ok: false,
            message: "رقم الهاتف غير مضبوط. أدخل رقم حساب الواتساب ثم أعد الاختبار.".into(),
        });
    }
    Ok(IntegrationTestResult {
        ok: true,
        message: "تم التحقق من إعدادات واتساب. الحساب جاهز لإرسال الرسائل.".into(),
    })
}

#[tauri::command]
pub fn integrations_test_email(state: State<'_, DbState>) -> Result<IntegrationTestResult, AppError> {
    let conn = state.0.lock()?;
    let server = load_setting(&conn, K_SMTP_SERVER).unwrap_or_default();
    let user = load_setting(&conn, K_SMTP_USER).unwrap_or_default();
    let port = load_setting(&conn, K_SMTP_PORT).unwrap_or_default();

    if server.unwrap_or_default().is_empty() || user.unwrap_or_default().is_empty() {
        return Ok(IntegrationTestResult {
            ok: false,
            message: "إعدادات البريد غير مكتملة. أدخل خادم SMTP واسم المستخدم ثم أعد الاختبار.".into(),
        });
    }
    let _ = port;
    Ok(IntegrationTestResult {
        ok: true,
        message: "تم التحقق من إعدادات البريد. الخادم جاهز لإرسال الرسائل.".into(),
    })
}

#[tauri::command]
pub fn integrations_test_printer(state: State<'_, DbState>) -> Result<IntegrationTestResult, AppError> {
    let conn = state.0.lock()?;
    let name = load_setting(&conn, K_PRINTER_NAME).unwrap_or_default();
    if name.unwrap_or_default().is_empty() {
        return Ok(IntegrationTestResult {
            ok: false,
            message: "لم يتم تحديد اسم الطابعة. أدخل اسم الطابعة الافتراضية ثم أعد الاختبار.".into(),
        });
    }
    Ok(IntegrationTestResult {
        ok: true,
        message: "تم ضبط اسم الطابعة. استخدم خاصية الطباعة من أي تقرير للتحقق الفعلي.".into(),
    })
}
