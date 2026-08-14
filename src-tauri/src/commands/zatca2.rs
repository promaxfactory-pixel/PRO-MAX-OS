use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use crate::zatca2::{
    build_csr, build_phase2_qr, decode_phase2_qr, generate_signed_invoice, validate_invoice_data,
    FatooraClient, FatooraConfig, Phase2Qr, ZatcaAddress, ZatcaInvoiceData, ZatcaKeys, ZatcaLine,
    ZatcaParty, ZATCA_TXN_SIMPLIFIED, ZATCA_TXN_STANDARD,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use k256::ecdsa::SigningKey;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Zatca2SettingsData {
    pub id: i64,
    pub environment: String,
    pub vat_number: Option<String>,
    pub organization_name: Option<String>,
    pub csid_stage: String,
    pub icv_counter: i64,
    pub last_invoice_hash: Option<String>,
    pub onboarded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zatca2Generated {
    pub e_invoice_id: i64,
    pub invoice_no: String,
    pub invoice_hash: String,
    pub qr_payload: String,
    pub signature_value: String,
    pub icv: i64,
    pub pih: Option<String>,
    pub xml: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zatca2Validation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub compliance_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zatca2OnboardResult {
    pub stage: String,
    pub request_id: Option<String>,
    pub certificate_der: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zatca2SubmitResult {
    pub e_invoice_id: i64,
    pub invoice_no: String,
    pub status: String,
    pub zatca_uuid: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zatca2Record {
    pub id: i64,
    pub invoice_id: i64,
    pub invoice_no: String,
    pub status: String,
    pub zatca_stage: Option<String>,
    pub invoice_hash: Option<String>,
    pub icv: Option<i64>,
    pub submitted_at: Option<String>,
    pub created_at: String,
}

fn row_to_settings(row: &rusqlite::Row) -> rusqlite::Result<Zatca2SettingsData> {
    Ok(Zatca2SettingsData {
        id: row.get(0)?,
        environment: row.get(1)?,
        vat_number: row.get(2)?,
        organization_name: row.get(3)?,
        csid_stage: row.get(4)?,
        icv_counter: row.get(5)?,
        last_invoice_hash: row.get(6)?,
        onboarded: matches!(row.get::<_, String>(4)?.as_str(), "compliance" | "production"),
    })
}

fn load_settings(conn: &rusqlite::Connection, company_id: i64) -> Option<Zatca2SettingsData> {
    conn.query_row(
        "SELECT id, environment, vat_number, organization_name, csid_stage, icv_counter, last_invoice_hash
         FROM zatca_settings WHERE company_id = ?1 LIMIT 1",
        [company_id],
        row_to_settings,
    )
    .ok()
}

/// Load the EGS cryptographic keys (generated on first settings save).
fn load_keys(
    conn: &rusqlite::Connection,
    company_id: i64,
) -> Result<ZatcaKeys, AppError> {
    let enc_key: Option<String> = conn
        .query_row(
            "SELECT signing_key FROM zatca_settings WHERE company_id = ?1 LIMIT 1",
            [company_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    let Some(enc_key) = enc_key else {
        return Err(AppError::business("ZATCA: لم يتم تجهيز المفاتيح. احفظ الإعدادات أولاً."));
    };
    let raw_b64 = crate::crypto::decrypt_if_needed(&enc_key)
        .map_err(|e| AppError::business(format!("ZATCA: فشل فك تشفير المفتاح: {}", e)))?;
    let raw = BASE64
        .decode(raw_b64.as_bytes())
        .map_err(|e| AppError::business(format!("ZATCA: مفتاح تالف: {}", e)))?;
    let signing_key = SigningKey::from_slice(&raw)
        .map_err(|e| AppError::business(format!("ZATCA: مفتاح غير صالح: {}", e)))?;
    let certificate_der = {
        let cert: Option<String> = conn
            .query_row(
                "SELECT certificate_der FROM zatca_settings WHERE company_id = ?1 LIMIT 1",
                [company_id],
                |row| row.get(0),
            )
            .ok()
            .flatten();
        cert.and_then(|c| BASE64.decode(c.as_bytes()).ok()).unwrap_or_default()
    };
    Ok(ZatcaKeys { signing_key, certificate_der })
}

fn save_keys(conn: &rusqlite::Connection, company_id: i64, keys: &ZatcaKeys) -> Result<(), AppError> {
    let scalar = keys.signing_key.to_bytes();
    let enc = crate::crypto::encrypt_if_needed(&BASE64.encode(scalar))
        .map_err(|e| AppError::business(format!("ZATCA: فشل تشفير المفتاح: {}", e)))?;
    conn.execute(
        "UPDATE zatca_settings SET signing_key = ?1, updated_at = datetime('now') WHERE company_id = ?2",
        rusqlite::params![enc, company_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn zatca2_get_settings(
    state: State<'_, DbState>,
) -> Result<Option<Zatca2SettingsData>, AppError> {
    let conn = state.0.lock()?;
    Ok(load_settings(&conn, 1))
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn zatca2_save_settings(
    state: State<'_, DbState>,
    user_id: i64,
    environment: String,
    vat_number: Option<String>,
    organization_name: Option<String>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    zatca2_save_settings_impl(&conn, user_id, environment, vat_number, organization_name)
}

pub(crate) fn zatca2_save_settings_impl(
    conn: &Connection,
    user_id: i64,
    environment: String,
    vat_number: Option<String>,
    organization_name: Option<String>,
) -> Result<String, AppError> {
    rbac::require_role(conn, user_id, &["admin"])?;
    let company_id: i64 = conn
        .query_row("SELECT id FROM companies LIMIT 1", [], |row| row.get(0))
        .unwrap_or(1);
    let existing = load_settings(conn, company_id);
    let icv_counter = existing.as_ref().map(|s| s.icv_counter).unwrap_or(0);
    if let Some(eid) = existing.map(|s| s.id) {
        conn.execute(
            "UPDATE zatca_settings SET environment = ?1,
             vat_number = COALESCE(?2, vat_number),
             organization_name = COALESCE(?3, organization_name),
             updated_at = datetime('now') WHERE id = ?4",
            rusqlite::params![environment, vat_number, organization_name, eid],
        )?;
        let _ = rbac::log_audit(conn, Some(user_id), None, "zatca2_save_settings", "zatca_settings", Some(eid), None, None, None);
        Ok("ZATCA settings saved".into())
    } else {
        conn.execute(
            "INSERT INTO zatca_settings(company_id, environment, vat_number, organization_name, csid_stage, icv_counter)
             VALUES(?1, ?2, ?3, ?4, 'none', ?5)",
            rusqlite::params![company_id, environment, vat_number, organization_name, icv_counter],
        )?;
        // Generate and persist the EGS key pair.
        let keys = ZatcaKeys::random();
        save_keys(conn, company_id, &keys)?;
        let _ = rbac::log_audit(conn, Some(user_id), None, "zatca2_save_settings", "zatca_settings", None, None, None, None);
        Ok("ZATCA settings saved and keys generated".into())
    }
}

/// Return the base64 CSR to onboard a CSID (manual submission to Fatoora).
#[tauri::command]
pub fn zatca2_build_csr(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    zatca2_build_csr_impl(&conn, user_id)
}

pub(crate) fn zatca2_build_csr_impl(
    conn: &Connection,
    user_id: i64,
) -> Result<String, AppError> {
    rbac::require_role(conn, user_id, &["admin"])?;
    let settings = load_settings(conn, 1).ok_or_else(|| AppError::business("ZATCA: احفظ الإعدادات أولاً."))?;
    let vat = settings.vat_number.clone().unwrap_or_default();
    let org = settings.organization_name.clone().unwrap_or_else(|| "Organization".into());
    let keys = load_keys(conn, 1)?;
    let csr = build_csr(&keys, &vat, &org)?;
    Ok(BASE64.encode(&csr))
}

/// Attempt automatic CSID onboarding (compliance + production).
#[tauri::command]
pub fn zatca2_onboard(
    state: State<'_, DbState>,
    user_id: i64,
    sandbox: bool,
) -> Result<Zatca2OnboardResult, AppError> {
    let conn = state.0.lock()?;
    zatca2_onboard_impl(&conn, user_id, sandbox)
}

pub(crate) fn zatca2_onboard_impl(
    conn: &Connection,
    user_id: i64,
    sandbox: bool,
) -> Result<Zatca2OnboardResult, AppError> {
    rbac::require_role(conn, user_id, &["admin"])?;
    let settings = load_settings(conn, 1).ok_or_else(|| AppError::business("ZATCA: احفظ الإعدادات أولاً."))?;
    let vat = settings.vat_number.clone().unwrap_or_default();
    let org = settings.organization_name.clone().unwrap_or_else(|| "Organization".into());
    let keys = load_keys(conn, 1)?;
    let csr_b64 = BASE64.encode(build_csr(&keys, &vat, &org)?);
    let base_url = if sandbox {
        crate::zatca2::FATTOORA_BASE_SIM
    } else {
        crate::zatca2::FATTOORA_BASE_PROD
    };
    let client = FatooraClient::new(FatooraConfig {
        base_url: base_url.into(),
        cert_b64: String::new(),
        vat_number: vat.clone(),
    })?;
    match client.compliance_csid(&keys, &csr_b64) {
        Ok(resp) => {
            let comp_request_id = resp.get("requestID").and_then(|v| v.as_str()).map(String::from);
            let comp_cert = resp.get("binarySecurityToken").and_then(|v| v.as_str()).map(String::from);
            conn.execute(
                "UPDATE zatca_settings SET csid_stage = 'compliance', certificate_der = COALESCE(?1, certificate_der),
                 onboarding_request_id = COALESCE(?2, onboarding_request_id), updated_at = datetime('now') WHERE company_id = 1",
                rusqlite::params![comp_cert, comp_request_id],
            )?;
            let _ = rbac::log_audit(conn, Some(user_id), None, "zatca2_onboard", "zatca_settings", None, Some("compliance"), None, None);
            match comp_request_id
                .as_deref()
                .and_then(|rid| client.production_csid(&keys, rid).ok())
            {
                Some(prod) => {
                    let prod_cert = prod.get("binarySecurityToken").and_then(|v| v.as_str()).map(String::from);
                    conn.execute(
                        "UPDATE zatca_settings SET csid_stage = 'production', certificate_der = COALESCE(?1, certificate_der),
                         updated_at = datetime('now') WHERE company_id = 1",
                        rusqlite::params![prod_cert],
                    )?;
                    Ok(Zatca2OnboardResult {
                        stage: "production".into(),
                        request_id: prod.get("requestID").and_then(|v| v.as_str()).map(String::from),
                        certificate_der: prod_cert,
                        message: "Production CSID obtained".into(),
                    })
                }
                None => Ok(Zatca2OnboardResult {
                    stage: "compliance".into(),
                    request_id: comp_request_id,
                    certificate_der: comp_cert,
                    message: "Compliance CSID obtained (production step pending)".into(),
                }),
            }
        }
        Err(e) => Err(AppError::business(format!("ZATCA: فشل الاتصال بمنصة فاتورة: {}", e))),
    }
}

fn build_invoice_data(
    conn: &rusqlite::Connection,
    invoice_id: i64,
    settings: &Zatca2SettingsData,
    _keys: &ZatcaKeys,
) -> Result<ZatcaInvoiceData, AppError> {
    let inv = conn
        .query_row(
            "SELECT si.id, si.inv_no, si.net_milli, si.vat_milli, si.total_milli,
                    si.date, COALESCE(c.name, ''), COALESCE(c.vat_number, '')
             FROM sales_invoices si
             LEFT JOIN customers c ON si.customer_id = c.id
             WHERE si.id = ?1",
            [invoice_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .map_err(|e| AppError::business(format!("الفاتورة غير موجودة: {}", e)))?;
    let (_id, inv_no, net_milli, vat_milli, total_milli, inv_date, cust_name, cust_vat) = inv;

    let (company_name, company_vat, company_addr, company_cr, currency, vat_rate) = conn
        .query_row(
            "SELECT COALESCE(name_ar, name_en, ''), COALESCE(vat_number, ''), COALESCE(address, ''),
                    COALESCE(cr_number, ''), COALESCE(default_currency, 'SAR'), COALESCE(default_vat_pct, 15.0)
             FROM companies WHERE active = 1 ORDER BY id LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, f64>(5)?,
                ))
            },
        )
        .unwrap_or_default();

    let mut lines: Vec<ZatcaLine> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(p.name_ar, ''), sil.cartons, sil.unit_price_milli, sil.line_net_milli
                 FROM sales_invoice_lines sil
                 LEFT JOIN products p ON sil.product_id = p.id
                 WHERE sil.invoice_id = ?1",
            )
            ?;
        let rows = stmt.query_map([invoice_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        for (i, r) in rows.filter_map(|r| r.ok()).enumerate() {
            let (name, qty, unit_price, line_net) = r;
            lines.push(ZatcaLine {
                id: (i + 1) as u32,
                item_name: name,
                quantity: qty,
                unit_code: "PCE".into(),
                unit_price: unit_price as f64 / 1000.0,
                line_total: line_net as f64 / 1000.0,
                vat_rate,
                tax_category: if vat_rate <= 0.0 { "Z" } else { "S" }.into(),
            });
        }
    }

    let issue_date = if inv_date.len() >= 10 { inv_date[..10].to_string() } else { inv_date };
    let issue_time = format!("{}:{}:{}",
        chrono::Local::now().format("%H"), chrono::Local::now().format("%M"), chrono::Local::now().format("%S"));

    let transaction_type = if vat_milli <= 0 && settings.environment.eq_ignore_ascii_case("simplified") {
        ZATCA_TXN_SIMPLIFIED
    } else {
        ZATCA_TXN_STANDARD
    };

    let seller = ZatcaParty {
        crn: company_cr,
        vat_number: if !company_vat.is_empty() { company_vat } else { settings.vat_number.clone().unwrap_or_default() },
        name: company_name,
        address: ZatcaAddress {
            street_name: company_addr,
            country_code: "SA".into(),
            ..ZatcaAddress::default()
        },
    };

    let buyer = ZatcaParty {
        crn: String::new(),
        vat_number: cust_vat,
        name: cust_name,
        address: ZatcaAddress {
            country_code: "SA".into(),
            ..ZatcaAddress::default()
        },
    };

    let pih = settings.last_invoice_hash.clone();
    let icv = settings.icv_counter as u64 + 1;

    Ok(ZatcaInvoiceData {
        invoice_number: inv_no,
        uuid: uuid::Uuid::new_v4().to_string(),
        issue_date,
        issue_time: format!("{}+03:00", issue_time),
        transaction_type: transaction_type.into(),
        currency: if currency.is_empty() { "SAR".into() } else { currency },
        icv,
        pih,
        seller,
        buyer,
        lines,
        net_amount: net_milli as f64 / 1000.0,
        vat_amount: vat_milli as f64 / 1000.0,
        total_amount: total_milli as f64 / 1000.0,
        allowance_total: 0.0,
        notes: vec![],
    })
}

/// Generate a signed ZATCA Phase-2 invoice from a sales invoice and persist it.
#[tauri::command]
pub fn zatca2_generate(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<Zatca2Generated, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_EINVOICE)?;
    let conn = state.0.lock()?;
    zatca2_generate_impl(&conn, user_id, invoice_id)
}

pub(crate) fn zatca2_generate_impl(
    conn: &Connection,
    user_id: i64,
    invoice_id: i64,
) -> Result<Zatca2Generated, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager"])?;

    let settings = load_settings(conn, 1).ok_or_else(|| AppError::business("ZATCA: احفظ الإعدادات أولاً."))?;
    let keys = load_keys(conn, 1)?;
    let data = build_invoice_data(conn, invoice_id, &settings, &keys)?;
    let issues = validate_invoice_data(&data);
    if !issues.is_empty() {
        return Err(AppError::business(format!("ZATCA: بيانات الفاتورة غير متوافقة: {}", issues.join("; "))));
    }

    let (final_xml, hash_b64, sig_b64) = generate_signed_invoice(&data, &keys)
        .map_err(|e| AppError::business(format!("ZATCA: فشل التوقيع: {}", e)))?;

    let qr_payload = build_phase2_qr(&Phase2Qr {
        seller_name: data.seller.name.clone(),
        vat_number: data.seller.vat_number.clone(),
        timestamp: format!("{}T{}", data.issue_date, data.issue_time),
        total: data.total_amount,
        vat_amount: data.vat_amount,
        invoice_hash: hash_b64.clone(),
        ecdsa_signature: sig_b64.clone(),
        public_key: BASE64.encode(keys.public_key_raw()),
        ca_signature: String::new(),
    });

    // Upsert into e_invoices.
    let e_invoice_id: i64 = conn
        .query_row(
            "SELECT id FROM e_invoices WHERE invoice_id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .ok()
        .unwrap_or_else(|| {
            conn.execute(
                "INSERT INTO e_invoices(invoice_id, status, created_by) VALUES(?1, 'Generated', ?2)",
                rusqlite::params![invoice_id, user_id],
            )
            .ok();
            conn.query_row(
                "SELECT id FROM e_invoices WHERE invoice_id = ?1",
                [invoice_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
        });

    conn.execute(
        "UPDATE e_invoices SET xml_content = ?1, qr_code = ?2, status = 'Generated',
         invoice_hash = ?3, qr_content = ?4, signed_xml = ?5, signature_value = ?6,
         icv = ?7, pih = ?8, zatca_stage = ?9, zatca_environment = ?10,
         created_at = COALESCE(created_at, datetime('now'))
         WHERE id = ?11",
        rusqlite::params![
            data.issue_date.clone(),
            qr_payload,
            hash_b64,
            qr_payload,
            final_xml,
            sig_b64,
            data.icv as i64,
            data.pih.clone().unwrap_or_default(),
            if data.transaction_type == ZATCA_TXN_SIMPLIFIED { "simplified" } else { "standard" },
            settings.environment,
            e_invoice_id,
        ],
    )?;

    // Advance ICV chain.
    conn.execute(
        "UPDATE zatca_settings SET icv_counter = ?1, last_invoice_hash = ?2, updated_at = datetime('now') WHERE company_id = 1",
        rusqlite::params![data.icv as i64, hash_b64],
    )?;

    let _ = rbac::log_audit(conn, Some(user_id), None, "zatca2_generate", "e_invoices", Some(e_invoice_id), None, None, None);

    Ok(Zatca2Generated {
        e_invoice_id,
        invoice_no: data.invoice_number.clone(),
        invoice_hash: hash_b64,
        qr_payload,
        signature_value: sig_b64,
        icv: data.icv as i64,
        pih: data.pih,
        xml: final_xml,
        status: "Generated".into(),
    })
}

/// Validate a sales invoice against ZATCA Phase-2 business rules.
#[tauri::command]
pub fn zatca2_validate(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<Zatca2Validation, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_EINVOICE)?;
    let conn = state.0.lock()?;
    zatca2_validate_impl(&conn, user_id, invoice_id)
}

pub(crate) fn zatca2_validate_impl(
    conn: &Connection,
    user_id: i64,
    invoice_id: i64,
) -> Result<Zatca2Validation, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager"])?;
    let settings = load_settings(conn, 1).ok_or_else(|| AppError::business("ZATCA: احفظ الإعدادات أولاً."))?;
    let keys = load_keys(conn, 1)?;
    let data = build_invoice_data(conn, invoice_id, &settings, &keys)?;
    let mut errors = validate_invoice_data(&data);
    let mut warnings = Vec::new();
    if data.seller.crn.is_empty() {
        warnings.push("رقم السجل التجاري فارغ (BR-KSA-12)".into());
    }
    if data.buyer.vat_number.is_empty() {
        warnings.push("رقم ضريبة المشتري فارغ — غير مطلوب للفواتير المبسطة".into());
    }
    // QR decode round-trip check.
    if !errors.is_empty() {
        warnings.push("تعذر توليد QR بسبب أخطاء التوافق".into());
    } else {
        let qr_payload = build_phase2_qr(&Phase2Qr {
            seller_name: data.seller.name.clone(),
            vat_number: data.seller.vat_number.clone(),
            timestamp: format!("{}T{}", data.issue_date, data.issue_time),
            total: data.total_amount,
            vat_amount: data.vat_amount,
            invoice_hash: "x".into(),
            ecdsa_signature: "y".into(),
            public_key: "z".into(),
            ca_signature: String::new(),
        });
        if decode_phase2_qr(&qr_payload).is_err() {
            errors.push("QR encoding failed".into());
        }
    }
    let penalty = errors.len() as f64 * 15.0;
    let compliance_score = (100.0 - penalty).max(0.0);
    Ok(Zatca2Validation {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        compliance_score,
    })
}

/// Submit an invoice to Fatoora: clearance for standard, reporting for simplified.
#[tauri::command]
pub fn zatca2_submit(
    state: State<'_, DbState>,
    user_id: i64,
    e_invoice_id: i64,
    sandbox: bool,
) -> Result<Zatca2SubmitResult, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_EINVOICE)?;
    let conn = state.0.lock()?;
    zatca2_submit_impl(&conn, user_id, e_invoice_id, sandbox)
}

pub(crate) fn zatca2_submit_impl(
    conn: &Connection,
    user_id: i64,
    e_invoice_id: i64,
    sandbox: bool,
) -> Result<Zatca2SubmitResult, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager"])?;

    let row = conn
        .query_row(
            "SELECT ei.invoice_id, ei.invoice_hash, ei.signed_xml, ei.qr_code, ei.zatca_stage,
                    COALESCE(si.inv_no, '')
             FROM e_invoices ei LEFT JOIN sales_invoices si ON ei.invoice_id = si.id
             WHERE ei.id = ?1",
            [e_invoice_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|e| AppError::business(format!("سجل الفاتورة الإلكترونية غير موجود: {}", e)))?;
    let (_invoice_id, invoice_hash, signed_xml, qr_code, stage, inv_no) = row;

    let Some(hash) = invoice_hash else {
        return Err(AppError::business("ZATCA: لم يُنشأ سجل توقيع لهذه الفاتورة بعد."));
    };
    let Some(xml) = signed_xml else {
        return Err(AppError::business("ZATCA: XML الموقّع مفقود."));
    };

    let settings = load_settings(conn, 1).ok_or_else(|| AppError::business("ZATCA: احفظ الإعدادات أولاً."))?;
    let keys = load_keys(conn, 1)?;
    let cert_b64 = if keys.certificate_der.is_empty() {
        None
    } else {
        Some(BASE64.encode(&keys.certificate_der))
    };
    let Some(cert) = cert_b64 else {
        return Err(AppError::business("ZATCA: لا يوجد CSID. قم بالتسجيل أولاً."));
    };

    let base_url = if sandbox {
        crate::zatca2::FATTOORA_BASE_SIM
    } else {
        crate::zatca2::FATTOORA_BASE_PROD
    };
    let client = FatooraClient::new(FatooraConfig {
        base_url: base_url.into(),
        cert_b64: cert.clone(),
        vat_number: settings.vat_number.clone().unwrap_or_default(),
    })?;

    let invoice_b64 = BASE64.encode(xml.as_bytes());
    let is_simplified = stage.as_deref() == Some("simplified");
    let result = if is_simplified {
        client.reporting(&keys, &hash, &invoice_b64)
    } else {
        client.clearance(&keys, &hash, &invoice_b64)
    };

    match result {
        Ok(resp) => {
            let zatca_uuid = resp.get("uuid").and_then(|v| v.as_str()).map(String::from);
            let status = if is_simplified { "Reported" } else { "Cleared" };
            conn.execute(
                "UPDATE e_invoices SET status = ?1, zatca_uuid = COALESCE(?2, zatca_uuid),
                 zatca_submitted_at = datetime('now'), qr_code = COALESCE(?3, qr_code)
                 WHERE id = ?4",
                rusqlite::params![status, zatca_uuid, qr_code, e_invoice_id],
            )?;
            let _ = rbac::log_audit(conn, Some(user_id), None, "zatca2_submit", "e_invoices", Some(e_invoice_id), None, Some(status), None);
            Ok(Zatca2SubmitResult {
                e_invoice_id,
                invoice_no: inv_no,
                status: status.into(),
                zatca_uuid,
                message: format!("تم {} بنجاح", if is_simplified { "الإبلاغ" } else { "الإجازة" }),
            })
        }
        Err(e) => {
            let code = e.split("HTTP").nth(1).and_then(|s| s.split_whitespace().next()).unwrap_or("ERR");
            conn.execute(
                "UPDATE e_invoices SET status = 'Rejected', zatca_rejection_code = ?1, rejection_reason = ?2 WHERE id = ?3",
                rusqlite::params![code, e, e_invoice_id],
            )?;
            Err(AppError::business(format!("ZATCA: رفضت فاتورة التقديم: {}", e)))
        }
    }
}

#[tauri::command]
pub fn zatca2_list(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<Vec<Zatca2Record>, AppError> {
    let conn = state.0.lock()?;
    zatca2_list_impl(&conn, user_id)
}

pub(crate) fn zatca2_list_impl(
    conn: &Connection,
    user_id: i64,
) -> Result<Vec<Zatca2Record>, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let mut stmt = conn
        .prepare(
            "SELECT ei.id, ei.invoice_id, COALESCE(si.inv_no, ''), ei.status, ei.zatca_stage,
                    ei.invoice_hash, ei.icv, ei.zatca_submitted_at, COALESCE(ei.created_at, '')
             FROM e_invoices ei LEFT JOIN sales_invoices si ON ei.invoice_id = si.id
             ORDER BY ei.id DESC LIMIT 200",
        )
        ?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Zatca2Record {
                id: row.get(0)?,
                invoice_id: row.get(1)?,
                invoice_no: row.get(2)?,
                status: row.get(3)?,
                zatca_stage: row.get(4)?,
                invoice_hash: row.get(5)?,
                icv: row.get(6)?,
                submitted_at: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        ?;
    let mut out = Vec::new();
    for r in rows.filter_map(|r| r.ok()) {
        out.push(r);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let _ = crate::crypto::init_secrets(std::path::Path::new(":memory:"));
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn.execute(
            "INSERT INTO users(username, password_hash, salt, role) VALUES('admin', 'x', 'y', 'admin')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn settings_round_trip_and_key_generation() {
        let conn = test_db();
        zatca2_save_settings_impl(&conn, 1, "sandbox".into(), Some("300012345600003".into()), Some("شركة الأمثلة".into())).unwrap();
        let s = load_settings(&conn, 1).unwrap();
        assert_eq!(s.environment, "sandbox");
        assert_eq!(s.vat_number.as_deref(), Some("300012345600003"));
        assert!(!s.onboarded);
        let keys = load_keys(&conn, 1).unwrap();
        assert_eq!(keys.public_key_raw().len(), 64);
    }

    #[test]
    fn csr_build_returns_valid_der() {
        let conn = test_db();
        zatca2_save_settings_impl(&conn, 1, "sandbox".into(), Some("300012345600003".into()), Some("Org".into())).unwrap();
        let csr_b64 = zatca2_build_csr_impl(&conn, 1).unwrap();
        let der = BASE64.decode(csr_b64.as_bytes()).unwrap();
        assert_eq!(der[0], 0x30);
        assert!(der.len() > 100);
    }
}
