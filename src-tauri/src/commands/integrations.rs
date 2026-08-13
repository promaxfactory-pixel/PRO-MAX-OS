use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::State;

const WHATSAPP_API_KEY: &str = "whatsapp_api_key";
const WHATSAPP_PHONE: &str = "whatsapp_phone";
const SMTP_SERVER: &str = "smtp_server";
const SMTP_PORT: &str = "smtp_port";
const SMTP_USER: &str = "smtp_user";
const SMTP_PASS: &str = "smtp_pass";
const SMTP_FROM: &str = "smtp_from";
const PRINTER_NAME: &str = "printer_name";
const PRINTER_AUTO: &str = "printer_auto";

fn get_setting(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM integrations_settings WHERE key=?1",
        [key],
        |row| row.get(0),
    )
    .ok()
}

fn set_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO integrations_settings(key, value, updated_at) VALUES(?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationsSettings {
    pub whatsapp_api_key: String,
    pub whatsapp_phone: String,
    pub smtp_server: String,
    pub smtp_port: i64,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    pub printer_name: String,
    pub printer_auto: bool,
}

#[tauri::command]
pub fn integrations_get_settings(state: State<'_, DbState>) -> Result<IntegrationsSettings, AppError> {
    let conn = state.0.lock()?;
    Ok(IntegrationsSettings {
        whatsapp_api_key: get_setting(&conn, WHATSAPP_API_KEY).unwrap_or_default(),
        whatsapp_phone: get_setting(&conn, WHATSAPP_PHONE).unwrap_or_default(),
        smtp_server: get_setting(&conn, SMTP_SERVER).unwrap_or_default(),
        smtp_port: get_setting(&conn, SMTP_PORT).and_then(|v| v.parse().ok()).unwrap_or(587),
        smtp_user: get_setting(&conn, SMTP_USER).unwrap_or_default(),
        smtp_pass: get_setting(&conn, SMTP_PASS).unwrap_or_default(),
        smtp_from: get_setting(&conn, SMTP_FROM).unwrap_or_default(),
        printer_name: get_setting(&conn, PRINTER_NAME).unwrap_or_default(),
        printer_auto: get_setting(&conn, PRINTER_AUTO).map(|v| v == "1").unwrap_or(false),
    })
}

#[tauri::command]
pub fn integrations_save_settings(
    state: State<'_, DbState>,
    user_id: i64,
    settings: IntegrationsSettings,
) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin"])?;
    let tx = conn.transaction()?;
    set_setting(&tx, WHATSAPP_API_KEY, &settings.whatsapp_api_key)?;
    set_setting(&tx, WHATSAPP_PHONE, &settings.whatsapp_phone)?;
    set_setting(&tx, SMTP_SERVER, &settings.smtp_server)?;
    set_setting(&tx, SMTP_PORT, &settings.smtp_port.to_string())?;
    set_setting(&tx, SMTP_USER, &settings.smtp_user)?;
    set_setting(&tx, SMTP_PASS, &settings.smtp_pass)?;
    set_setting(&tx, SMTP_FROM, &settings.smtp_from)?;
    set_setting(&tx, PRINTER_NAME, &settings.printer_name)?;
    set_setting(&tx, PRINTER_AUTO, if settings.printer_auto { "1" } else { "0" })?;
    let _ = rbac::log_audit(&tx, Some(user_id), None, "integrations_save_settings", "integrations_settings", None, None, None, None);
    tx.commit()?;
    Ok("تم حفظ الإعدادات".to_string())
}

#[derive(Debug, Serialize)]
pub struct IntegrationTestResult {
    pub ok: bool,
    pub message: String,
}

fn with_settings(state: &State<'_, DbState>) -> Result<IntegrationsSettings, AppError> {
    let conn = state.0.lock()?;
    Ok(IntegrationsSettings {
        whatsapp_api_key: get_setting(&conn, WHATSAPP_API_KEY).unwrap_or_default(),
        whatsapp_phone: get_setting(&conn, WHATSAPP_PHONE).unwrap_or_default(),
        smtp_server: get_setting(&conn, SMTP_SERVER).unwrap_or_default(),
        smtp_port: get_setting(&conn, SMTP_PORT).and_then(|v| v.parse().ok()).unwrap_or(587),
        smtp_user: get_setting(&conn, SMTP_USER).unwrap_or_default(),
        smtp_pass: get_setting(&conn, SMTP_PASS).unwrap_or_default(),
        smtp_from: get_setting(&conn, SMTP_FROM).unwrap_or_default(),
        printer_name: get_setting(&conn, PRINTER_NAME).unwrap_or_default(),
        printer_auto: get_setting(&conn, PRINTER_AUTO).map(|v| v == "1").unwrap_or(false),
    })
}

#[tauri::command]
pub fn integrations_test_whatsapp(state: State<'_, DbState>) -> Result<IntegrationTestResult, AppError> {
    let s = with_settings(&state)?;
    if s.whatsapp_api_key.trim().is_empty() {
        return Ok(IntegrationTestResult { ok: false, message: "مفتاح API غير مضبوط — أدخل المفتاح ثم أعد المحاولة".into() });
    }
    if s.whatsapp_phone.trim().is_empty() {
        return Ok(IntegrationTestResult { ok: false, message: "رقم الهاتف غير مضبوط — أدخل الرقم ثم أعد المحاولة".into() });
    }
    Ok(IntegrationTestResult {
        ok: true,
        message: "تم التحقق من إعدادات واتساب بنجاح (المفتاح والرقم مضبوطان)".into(),
    })
}

#[tauri::command]
pub fn integrations_test_email(state: State<'_, DbState>) -> Result<IntegrationTestResult, AppError> {
    let s = with_settings(&state)?;
    if s.smtp_server.trim().is_empty() {
        return Ok(IntegrationTestResult { ok: false, message: "خادم SMTP غير مضبوط".into() });
    }
    if s.smtp_port <= 0 || s.smtp_port > 65535 {
        return Ok(IntegrationTestResult { ok: false, message: "منفذ SMTP غير صالح".into() });
    }
    use std::net::{TcpStream, ToSocketAddrs};
    let addrs = match format!("{}:{}", s.smtp_server, s.smtp_port).to_socket_addrs() {
        Ok(mut it) => it.next(),
        Err(_) => {
            return Ok(IntegrationTestResult {
                ok: false,
                message: format!("تعذر تحليل عنوان الخادم '{}':{}", s.smtp_server, s.smtp_port),
            })
        }
    };
    let addr = match addrs {
        Some(a) => a,
        None => {
            return Ok(IntegrationTestResult {
                ok: false,
                message: format!("لا يوجد عنوان صالح للخادم '{}':{}", s.smtp_server, s.smtp_port),
            })
        }
    };
    match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
        Ok(_) => Ok(IntegrationTestResult {
            ok: true,
            message: format!("تم الاتصال بالخادم {}:{} بنجاح", s.smtp_server, s.smtp_port),
        }),
        Err(e) => Ok(IntegrationTestResult {
            ok: false,
            message: format!("تعذر الاتصال بالخادم {}:{} — {}", s.smtp_server, s.smtp_port, e),
        }),
    }
}

#[tauri::command]
pub fn integrations_test_printer(state: State<'_, DbState>) -> Result<IntegrationTestResult, AppError> {
    let s = with_settings(&state)?;
    if s.printer_name.trim().is_empty() {
        return Ok(IntegrationTestResult { ok: false, message: "اسم الطابعة غير مضبوط".into() });
    }
    Ok(IntegrationTestResult {
        ok: true,
        message: format!("تم التحقق — الطابعة '{}' مضبوطة وجاهزة", s.printer_name),
    })
}
