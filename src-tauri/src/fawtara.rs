use crate::commands::einvoice::{generate_invoice_xml, submit_to_tax_authority};
use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct FawtaraQrTag {
    pub tag: u8,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FawtaraPayload {
    pub invoice_id: i64,
    pub invoice_no: String,
    pub xml_content: String,
    pub hash: String,
    pub qr_base64: String,
    pub qr_tags: Vec<FawtaraQrTag>,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FawtaraCheck {
    pub key: String,
    pub label: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FawtaraReadiness {
    pub ready: bool,
    pub score: f64,
    pub checks: Vec<FawtaraCheck>,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FawtaraConnectorInfo {
    pub active: String,
    pub available: Vec<String>,
    pub note: String,
}

pub fn tlv_encode(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + value.len());
    out.push(tag);
    if value.len() > 0xFF {
        out.push(0xFF);
        out.extend_from_slice(&(value.len() as u32).to_be_bytes());
    } else {
        out.push(value.len() as u8);
    }
    out.extend_from_slice(value);
    out
}

pub fn tlv_decode(data: &[u8]) -> Result<Vec<(u8, Vec<u8>)>, AppError> {
    let mut fields = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let tag = data[pos];
        pos += 1;
        if pos >= data.len() {
            return Err(AppError::validation("TLV payload truncated at length byte"));
        }
        let len_byte = data[pos];
        pos += 1;
        let len = if len_byte == 0xFF {
            if pos + 4 > data.len() {
                return Err(AppError::validation("TLV extended length truncated"));
            }
            let l = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
            pos += 4;
            l
        } else {
            len_byte as usize
        };
        if pos + len > data.len() {
            return Err(AppError::validation("TLV value exceeds payload length"));
        }
        fields.push((tag, data[pos..pos + len].to_vec()));
        pos += len;
    }
    Ok(fields)
}

pub fn build_fawtara_qr(
    seller: &str,
    tax_no: &str,
    timestamp: &str,
    total_baisa: i64,
    vat_baisa: i64,
) -> String {
    let to_omr = |b: i64| format!("{:.3}", b as f64 / 1000.0);
    let mut payload = Vec::new();
    payload.extend_from_slice(&tlv_encode(1, seller.as_bytes()));
    payload.extend_from_slice(&tlv_encode(2, tax_no.as_bytes()));
    payload.extend_from_slice(&tlv_encode(3, timestamp.as_bytes()));
    payload.extend_from_slice(&tlv_encode(4, to_omr(total_baisa).as_bytes()));
    payload.extend_from_slice(&tlv_encode(5, to_omr(vat_baisa).as_bytes()));
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, payload)
}

pub fn decode_fawtara_qr(base64_payload: &str) -> Result<Vec<FawtaraQrTag>, AppError> {
    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(base64_payload)
        .map_err(|e| AppError::validation(format!("QR payload is not valid base64: {}", e)))?;
    let names: [&str; 5] = ["seller", "tax_no", "timestamp", "total", "vat"];
    let fields = tlv_decode(&raw)?;
    Ok(fields
        .into_iter()
        .map(|(tag, val)| FawtaraQrTag {
            tag,
            name: names
                .get((tag as usize).checked_sub(1).unwrap_or(usize::MAX))
                .copied()
                .unwrap_or("unknown")
                .to_string(),
            value: String::from_utf8_lossy(&val).into_owned(),
        })
        .collect())
}

const FAWTARA_NOTE: &str = "أساس فني لتوافق فاوترة (القرار 189/2026) — لم يُعتمد بعد من هيئة الضرائب العمانية، ويتطلب ربط ASP/OTA لاحقاً";

pub trait FawtaraConnector: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn submit(&self, invoice_id: i64, xml: &str) -> Result<String, AppError>;
    fn cancel(&self, invoice_id: i64, _reason: &str) -> Result<(), AppError> {
        let _ = invoice_id;
        Ok(())
    }
    fn check_status(&self, invoice_id: i64) -> Result<String, AppError>;
}

pub struct DevConnector;

impl FawtaraConnector for DevConnector {
    fn id(&self) -> &'static str {
        "dev"
    }
    fn display_name(&self) -> &'static str {
        "بيئة تطوير محلية"
    }
    fn submit(&self, invoice_id: i64, _xml: &str) -> Result<String, AppError> {
        Ok(format!("FAWTARA-DEV-{}-{}", invoice_id, chrono::Utc::now().timestamp()))
    }
    fn cancel(&self, invoice_id: i64, reason: &str) -> Result<(), AppError> {
        let _ = (invoice_id, reason);
        Ok(())
    }
    fn check_status(&self, invoice_id: i64) -> Result<String, AppError> {
        Ok(format!("dev:invoice-{}", invoice_id))
    }
}

pub struct HttpsConnector {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

impl FawtaraConnector for HttpsConnector {
    fn id(&self) -> &'static str {
        "https"
    }
    fn display_name(&self) -> &'static str {
        "ربط ASP/OTA عبر HTTPS"
    }
    fn submit(&self, invoice_id: i64, xml: &str) -> Result<String, AppError> {
        submit_to_tax_authority(
            &invoice_id.to_string(),
            "production",
            Some(&self.endpoint),
            self.api_key.as_deref(),
            self.api_secret.as_deref(),
            xml,
        )
    }
    fn check_status(&self, invoice_id: i64) -> Result<String, AppError> {
        Ok(format!("https:invoice-{}", invoice_id))
    }
}

fn connector_from_settings(
    environment: &str,
    endpoint: Option<&str>,
    api_key: Option<&str>,
    api_secret: Option<&str>,
) -> Box<dyn FawtaraConnector> {
    if environment == "production" {
        if let Some(url) = endpoint.map(str::trim).filter(|u| !u.is_empty()) {
            return Box::new(HttpsConnector {
                endpoint: url.to_string(),
                api_key: api_key.map(|s| s.to_string()),
                api_secret: api_secret.map(|s| s.to_string()),
            });
        }
    }
    Box::new(DevConnector)
}

pub(crate) fn readiness_checks(conn: &rusqlite::Connection) -> FawtaraReadiness {
    let company = conn
        .query_row(
            "SELECT COALESCE(name_ar, name_en, ''), COALESCE(vat_number, ''), COALESCE(cr_number, ''),
                    COALESCE(default_currency, 'OMR'), COALESCE(default_vat_pct, 5.0)
             FROM companies WHERE active = 1 ORDER BY id LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, f64>(4)?,
                ))
            },
        )
        .unwrap_or_default();

    let connector_state = conn
        .query_row(
            "SELECT environment, COALESCE(tax_authority_endpoint, ''), COALESCE(api_key, ''), COALESCE(api_secret, '')
             FROM einvoice_settings WHERE active = 1 LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
        .unwrap_or(("sandbox".to_string(), String::new(), String::new(), String::new()));

    let name_ok = !company.0.is_empty();
    let vat_ok = !company.1.is_empty();
    let cr_ok = !company.2.is_empty();
    let omr_ok = company.3.eq_ignore_ascii_case("OMR");
    let vat_rate_ok = (company.4 - 5.0).abs() < f64::EPSILON;
    let connector_ok = connector_state.0 == "production" && !connector_state.1.is_empty();

    let checks = vec![
        FawtaraCheck {
            key: "company_name".into(),
            label: "اسم المنشأة".into(),
            ok: name_ok,
            detail: if name_ok { company.0 } else { "اسم المنشأة مفقود".into() },
        },
        FawtaraCheck {
            key: "company_vat".into(),
            label: "الرقم الضريبي (VAT)".into(),
            ok: vat_ok,
            detail: if vat_ok { company.1 } else { "الرقم الضريبي مفقود".into() },
        },
        FawtaraCheck {
            key: "company_cr".into(),
            label: "السجل التجاري (CR)".into(),
            ok: cr_ok,
            detail: if cr_ok { company.2 } else { "السجل التجاري مفقود — إلزامي في قرار فاوترة".into() },
        },
        FawtaraCheck {
            key: "currency_omr".into(),
            label: "العملة بالريال العماني".into(),
            ok: omr_ok,
            detail: if omr_ok { "OMR".into() } else { format!("العملة الحالية: {}", company.3) },
        },
        FawtaraCheck {
            key: "vat_rate_5".into(),
            label: "نسبة الضريبة 5%".into(),
            ok: vat_rate_ok,
            detail: if vat_rate_ok { "5%".into() } else { format!("النسبة الحالية: {:.2}%", company.4) },
        },
        FawtaraCheck {
            key: "asp_connection".into(),
            label: "ربط ASP/OTA".into(),
            ok: connector_ok,
            detail: if connector_ok {
                format!("مرتبط عبر HTTPS ({})", connector_state.1)
            } else {
                "غير مرتبط — وضع التطوير يعمل محلياً فقط".into()
            },
        },
    ];

    let passed = checks.iter().filter(|c| c.ok).count();
    let score = (passed as f64 / checks.len() as f64 * 100.0 * 100.0).round() / 100.0;
    FawtaraReadiness {
        ready: checks.iter().all(|c| c.ok),
        score,
        checks,
        note: FAWTARA_NOTE.to_string(),
    }
}

#[tauri::command]
pub fn fawtara_build_payload(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<FawtaraPayload, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let result = generate_invoice_xml(&conn, invoice_id)?;

    let seller = load_seller_name(&conn);
    let tax_no = load_seller_tax_no(&conn);
    let timestamp = chrono::Utc::now().to_rfc3339();

    let totals: (i64, i64) = conn
        .query_row(
            "SELECT total_milli, vat_milli FROM sales_invoices WHERE id=?1",
            [invoice_id],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )
        .unwrap_or((0, 0));

    let qr_base64 = build_fawtara_qr(&seller, &tax_no, &timestamp, totals.0, totals.1);
    let qr_tags = decode_fawtara_qr(&qr_base64)?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "fawtara_build_payload", "e_invoices", Some(invoice_id), None, Some(&result.invoice_no), None);

    Ok(FawtaraPayload {
        invoice_id,
        invoice_no: result.invoice_no,
        xml_content: result.xml_content,
        hash: result.hash,
        qr_base64,
        qr_tags,
        note: FAWTARA_NOTE.to_string(),
    })
}

#[tauri::command]
pub fn fawtara_readiness(state: State<'_, DbState>) -> Result<FawtaraReadiness, AppError> {
    let conn = state.0.lock()?;
    Ok(readiness_checks(&conn))
}

#[tauri::command]
pub fn fawtara_connector_status(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<FawtaraConnectorInfo, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let (env, endpoint): (String, String) = conn
        .query_row(
            "SELECT COALESCE(environment, 'sandbox'), COALESCE(tax_authority_endpoint, '')
             FROM einvoice_settings WHERE active = 1 LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or(("sandbox".to_string(), String::new()));

    let active = if env == "production" && !endpoint.is_empty() {
        "https".to_string()
    } else {
        "dev".to_string()
    };

    Ok(FawtaraConnectorInfo {
        active,
        available: vec!["dev".to_string(), "https".to_string()],
        note: FAWTARA_NOTE.to_string(),
    })
}

#[tauri::command]
pub fn fawtara_submit(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let xml = conn
        .query_row(
            "SELECT xml_content FROM e_invoices WHERE invoice_id=?1",
            [invoice_id],
            |r| r.get::<_, String>(0),
        )
        .map_err(|_| AppError::not_found("قم بتوليد XML الفاتورة أولاً (fawtara_build_payload)"))?;

    let (env, endpoint, api_key, api_secret): (String, Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT environment, tax_authority_endpoint, api_key, api_secret
             FROM einvoice_settings WHERE active = 1 LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .unwrap_or(("sandbox".into(), None, None, None));

    let dec_key = api_key
        .as_deref()
        .map(crate::crypto::decrypt_if_needed)
        .transpose()
        .map_err(AppError::crypto)?;
    let dec_secret = api_secret
        .as_deref()
        .map(crate::crypto::decrypt_if_needed)
        .transpose()
        .map_err(AppError::crypto)?;

    let connector = connector_from_settings(&env, endpoint.as_deref(), dec_key.as_deref(), dec_secret.as_deref());

    let reference = connector.submit(invoice_id, &xml)?;

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = ?2 WHERE invoice_id = ?3",
        rusqlite::params![now, reference, invoice_id],
    )?;
    conn.execute(
        "UPDATE einvoice_queue SET status = 'completed' WHERE invoice_id = ?1",
        [invoice_id],
    ).ok();

    let _ = rbac::log_audit(&conn, Some(user_id), None, "fawtara_submit", "e_invoices", Some(invoice_id), None, Some(&format!("connector={} ref={}", connector.id(), reference)), None);

    Ok(format!("تم الإرسال عبر ({}) — مرجع: {}", connector.display_name(), reference))
}

fn load_seller_name(conn: &rusqlite::Connection) -> String {
    conn.query_row(
        "SELECT COALESCE(name_ar, name_en, '') FROM companies WHERE active = 1 ORDER BY id LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    )
    .unwrap_or_default()
}

fn load_seller_tax_no(conn: &rusqlite::Connection) -> String {
    conn.query_row(
        "SELECT COALESCE(vat_number, '') FROM companies WHERE active = 1 ORDER BY id LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    )
    .or_else(|_| {
        conn.query_row(
            "SELECT COALESCE(vat_number, '') FROM company_settings LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn tlv_round_trips_multiple_fields() {
        let mut data = Vec::new();
        data.extend_from_slice(&tlv_encode(1, b"Al Fanar Factory"));
        data.extend_from_slice(&tlv_encode(2, b"OM1234567890"));
        data.extend_from_slice(&tlv_encode(4, b"250.000"));
        let decoded = tlv_decode(&data).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0], (1u8, b"Al Fanar Factory".to_vec()));
        assert_eq!(decoded[1], (2u8, b"OM1234567890".to_vec()));
        assert_eq!(decoded[2], (4u8, b"250.000".to_vec()));
    }

    #[test]
    fn tlv_handles_large_values_with_extended_length() {
        let big = "x".repeat(400);
        let encoded = tlv_encode(5, big.as_bytes());
        let decoded = tlv_decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].0, 5u8);
        assert_eq!(String::from_utf8(decoded[0].1.clone()).unwrap(), big);
    }

    #[test]
    fn tlv_rejects_truncated_payload() {
        assert!(tlv_decode(&[1u8, 10, 97]).is_err());
        assert!(tlv_decode(&[1u8, 0xFF, 0, 0, 0]).is_err());
    }

    #[test]
    fn fawtara_qr_contains_expected_tags_and_values() {
        let qr = build_fawtara_qr("Seller Co", "OM1000000", "2026-08-15T10:00:00Z", 250000, 12500);
        let tags = decode_fawtara_qr(&qr).unwrap();
        assert_eq!(tags.len(), 5);
        let by_tag: std::collections::HashMap<u8, String> =
            tags.into_iter().map(|t| (t.tag, t.value)).collect();
        assert_eq!(by_tag[&1], "Seller Co");
        assert_eq!(by_tag[&2], "OM1000000");
        assert_eq!(by_tag[&4], "250.000");
        assert_eq!(by_tag[&5], "12.500");
    }

    #[test]
    fn fawtara_qr_rejects_bad_base64() {
        assert!(decode_fawtara_qr("!!!not-base64!!!").is_err());
    }

    #[test]
    fn readiness_detects_missing_cr_and_non_omr() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE companies (id INTEGER PRIMARY KEY AUTOINCREMENT, name_ar TEXT, name_en TEXT,
                vat_number TEXT, cr_number TEXT, address TEXT, default_currency TEXT,
                default_vat_pct REAL, active INTEGER DEFAULT 1);
             CREATE TABLE company_settings (name TEXT, vat_number TEXT, address TEXT, default_currency TEXT, default_vat_pct REAL);
             CREATE TABLE einvoice_settings (id INTEGER PRIMARY KEY AUTOINCREMENT, company_id INTEGER,
                environment TEXT, auto_submit INTEGER, submit_on_post INTEGER, tax_authority_endpoint TEXT,
                api_key TEXT, api_secret TEXT, active INTEGER);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO companies (name_ar, vat_number, cr_number, default_currency, default_vat_pct) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["مصنع العافية", "OM123456", "", "KWD", 5.0],
        )
        .unwrap();

        let r = readiness_checks(&conn);
        assert!(!r.ready);
        let cr = r.checks.iter().find(|c| c.key == "company_cr").unwrap();
        assert!(!cr.ok);
        let currency = r.checks.iter().find(|c| c.key == "currency_omr").unwrap();
        assert!(!currency.ok);
        assert!(r.score > 0.0 && r.score < 100.0);
    }

    #[test]
    fn readiness_fully_configured_is_ready() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE companies (id INTEGER PRIMARY KEY AUTOINCREMENT, name_ar TEXT, name_en TEXT,
                vat_number TEXT, cr_number TEXT, address TEXT, default_currency TEXT,
                default_vat_pct REAL, active INTEGER DEFAULT 1);
             CREATE TABLE einvoice_settings (id INTEGER PRIMARY KEY AUTOINCREMENT, company_id INTEGER,
                environment TEXT, auto_submit INTEGER, submit_on_post INTEGER, tax_authority_endpoint TEXT,
                api_key TEXT, api_secret TEXT, active INTEGER);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO companies (name_ar, vat_number, cr_number, default_currency, default_vat_pct) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["مصنع العافية", "OM123456", "CR-2026-01", "OMR", 5.0],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO einvoice_settings (company_id, environment, tax_authority_endpoint, active) VALUES (1, 'production', 'https://asp.example/ota', 1)",
            [],
        )
        .unwrap();

        let r = readiness_checks(&conn);
        assert!(r.ready);
        assert_eq!(r.score, 100.0);
    }

    #[test]
    fn dev_connector_returns_synthetic_reference() {
        let c = DevConnector;
        let ref_no = c.submit(7, "<Invoice/>").unwrap();
        assert!(ref_no.starts_with("FAWTARA-DEV-7-"));
        assert!(c.check_status(7).unwrap().contains("dev"));
    }

    #[test]
    fn sandbox_environment_resolves_to_dev_connector() {
        let c = connector_from_settings("sandbox", None, None, None);
        assert_eq!(c.id(), "dev");
        assert_eq!(c.display_name(), "بيئة تطوير محلية");
    }

    #[test]
    fn production_without_endpoint_falls_back_to_dev() {
        let c = connector_from_settings("production", None, None, None);
        assert_eq!(c.id(), "dev");
    }
}
