use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceResult {
    pub invoice_id: i64,
    pub invoice_no: String,
    pub xml_content: String,
    pub hash: String,
    pub qr_code_data: String,
    pub generated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceValidation {
    pub is_valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub compliance_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub code: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceStatusInfo {
    pub invoice_id: i64,
    pub status: String,
    pub submitted_at: Option<String>,
    pub zatca_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceRecord {
    pub id: i64,
    pub invoice_id: i64,
    pub invoice_no: String,
    pub customer_name: String,
    pub total_milli: i64,
    pub status: String,
    pub compliance_score: f64,
    pub created_at: String,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceReport {
    pub from_date: String,
    pub to_date: String,
    pub total_invoices: i64,
    pub total_amount_milli: i64,
    pub total_vat_milli: i64,
    pub submitted: i64,
    pub accepted: i64,
    pub rejected: i64,
    pub pending: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceSettingsData {
    pub id: i64,
    pub company_id: i64,
    pub environment: String,
    pub auto_submit: bool,
    pub submit_on_post: bool,
    pub tax_authority_endpoint: Option<String>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceQueueItem {
    pub id: i64,
    pub invoice_id: i64,
    pub invoice_no: String,
    pub customer_name: String,
    pub total_milli: i64,
    pub action: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EInvoiceDashboard {
    pub total_generated: i64,
    pub total_submitted: i64,
    pub total_accepted: i64,
    pub total_rejected: i64,
    pub total_pending_submission: i64,
    pub total_amount_milli: i64,
    pub total_vat_milli: i64,
    pub queue_pending: i64,
    pub queue_failed: i64,
    pub settings_configured: bool,
}

fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_pint_om_xml(
    inv_no: &str,
    net_milli: i64,
    vat_milli: i64,
    total_milli: i64,
    inv_date: &str,
    cust_name: &str,
    cust_vat: &str,
    company_name: &str,
    company_vat: &str,
    company_cr: &str,
    currency: &str,
    vat_rate: f64,
    country: &str,
    lines: &[(String, f64, f64, f64)],
) -> String {
    let mut xml = String::with_capacity(4096);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rsm:CrossIndustryInvoice xmlns:rsm=\"urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100\"\n");
    xml.push_str("  xmlns:udt=\"urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100\"\n");
    xml.push_str("  xmlns:qdt=\"urn:un:unece:uncefact:data:standard:QualifiedDataType:100\"\n");
    xml.push_str("  xmlns:ram=\"urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100\">\n\n");
    xml.push_str("  <rsm:ExchangedDocument>\n");
    xml.push_str(&format!("    <ram:ID>{}</ram:ID>\n", xml_escape(inv_no)));
    xml.push_str("    <ram:TypeCode>380</ram:TypeCode>\n");
    xml.push_str(&format!("    <ram:IssueDateTime><udt:DateTimeString format=\"102\">{}</udt:DateTimeString></ram:IssueDateTime>\n", xml_escape(inv_date)));
    xml.push_str("  </rsm:ExchangedDocument>\n\n");
    xml.push_str("  <rsm:SupplyChainTradeTransaction>\n");
    xml.push_str("    <ram:ApplicableHeaderTradeAgreement>\n");
    xml.push_str("      <ram:BuyerTradeParty>\n");
    xml.push_str(&format!("        <ram:Name>{}</ram:Name>\n", xml_escape(cust_name)));
    if !cust_vat.is_empty() {
        xml.push_str(&format!("        <ram:ID schemeID=\"9921\">{}</ram:ID>\n", xml_escape(cust_vat)));
    }
    xml.push_str("      </ram:BuyerTradeParty>\n");
    xml.push_str("      <ram:SellerTradeParty>\n");
    xml.push_str(&format!("        <ram:Name>{}</ram:Name>\n", xml_escape(company_name)));
    if !company_vat.is_empty() {
        xml.push_str(&format!("        <ram:ID schemeID=\"9921\">{}</ram:ID>\n", xml_escape(company_vat)));
    }
    if !company_cr.is_empty() {
        xml.push_str(&format!("        <ram:ID schemeID=\"9930\">{}{}</ram:ID>\n", xml_escape(country), xml_escape(company_cr)));
    }
    xml.push_str("      </ram:SellerTradeParty>\n");
    xml.push_str("    </ram:ApplicableHeaderTradeAgreement>\n\n");
    xml.push_str("    <ram:ApplicableHeaderTradeDelivery>\n");
    xml.push_str(&format!("      <ram:ActualDeliverySupplyChainEvent><ram:OccurrenceDateTime><udt:DateTimeString format=\"102\">{}</udt:DateTimeString></ram:OccurrenceDateTime></ram:ActualDeliverySupplyChainEvent>\n", xml_escape(inv_date)));
    xml.push_str("    </ram:ApplicableHeaderTradeDelivery>\n\n");
    xml.push_str("    <ram:ApplicableHeaderTradeSettlement>\n");
    xml.push_str(&format!("      <ram:InvoiceCurrencyCode>{}</ram:InvoiceCurrencyCode>\n", xml_escape(currency)));
    xml.push_str(&format!("      <ram:PayeeTradeParty><ram:Name>{}</ram:Name></ram:PayeeTradeParty>\n", xml_escape(company_name)));
    xml.push_str("      <ram:ApplicableTradeTax>\n");
    xml.push_str(&format!("        <ram:CalculatedAmount currencyID=\"{}\">{:.3}</ram:CalculatedAmount>\n", xml_escape(currency), vat_milli as f64 / 1000.0));
    xml.push_str("        <ram:TypeCode>VAT</ram:TypeCode>\n");
    xml.push_str(&format!("        <ram:BasisAmount currencyID=\"{}\">{:.3}</ram:BasisAmount>\n", xml_escape(currency), net_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:RateApplicablePercent>{:.2}</ram:RateApplicablePercent>\n", vat_rate));
    xml.push_str("      </ram:ApplicableTradeTax>\n");
    xml.push_str("      <ram:SpecifiedTradeSettlementMonetarySummation>\n");
    xml.push_str(&format!("        <ram:LineTotalAmount currencyID=\"{}\">{:.3}</ram:LineTotalAmount>\n", xml_escape(currency), net_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:TaxBasisTotalAmount currencyID=\"{}\">{:.3}</ram:TaxBasisTotalAmount>\n", xml_escape(currency), net_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:TaxTotalAmount currencyID=\"{}\">{:.3}</ram:TaxTotalAmount>\n", xml_escape(currency), vat_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:GrandTotalAmount currencyID=\"{}\">{:.3}</ram:GrandTotalAmount>\n", xml_escape(currency), total_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:DuePayableAmount currencyID=\"{}\">{:.3}</ram:DuePayableAmount>\n", xml_escape(currency), total_milli as f64 / 1000.0));
    xml.push_str("      </ram:SpecifiedTradeSettlementMonetarySummation>\n");
    xml.push_str("    </ram:ApplicableHeaderTradeSettlement>\n\n");
    for (idx, (desc, qty, price, total)) in lines.iter().enumerate() {
        xml.push_str("    <ram:IncludedSupplyChainTradeLineItem>\n");
        xml.push_str("      <ram:AssociatedDocumentLineDocument>\n");
        xml.push_str(&format!("        <ram:LineID>{}</ram:LineID>\n", idx + 1));
        xml.push_str("      </ram:AssociatedDocumentLineDocument>\n");
        xml.push_str("      <ram:SpecifiedTradeProduct>\n");
        xml.push_str(&format!("        <ram:Name>{}</ram:Name>\n", xml_escape(desc)));
        xml.push_str("      </ram:SpecifiedTradeProduct>\n");
        xml.push_str("      <ram:SpecifiedLineTradeAgreement>\n");
        xml.push_str(&format!("        <ram:NetPriceAmount currencyID=\"{}\">{:.3}</ram:NetPriceAmount>\n", xml_escape(currency), price));
        xml.push_str("      </ram:SpecifiedLineTradeAgreement>\n");
        xml.push_str("      <ram:SpecifiedLineTradeDelivery>\n");
        xml.push_str(&format!("        <ram:BilledQuantity unitCode=\"C62\">{:.0}</ram:BilledQuantity>\n", qty));
        xml.push_str("      </ram:SpecifiedLineTradeDelivery>\n");
        xml.push_str("      <ram:SpecifiedLineTradeSettlement>\n");
        xml.push_str("        <ram:ApplicableTradeTax>\n");
        xml.push_str("          <ram:TypeCode>VAT</ram:TypeCode>\n");
        xml.push_str(&format!("          <ram:RateApplicablePercent>{:.2}</ram:RateApplicablePercent>\n", vat_rate));
        xml.push_str("        </ram:ApplicableTradeTax>\n");
        xml.push_str("        <ram:SpecifiedTradeSettlementMonetarySummation>\n");
        xml.push_str(&format!("          <ram:LineTotalAmount currencyID=\"{}\">{:.3}</ram:LineTotalAmount>\n", xml_escape(currency), total));
        xml.push_str("        </ram:SpecifiedTradeSettlementMonetarySummation>\n");
        xml.push_str("      </ram:SpecifiedLineTradeSettlement>\n");
        xml.push_str("    </ram:IncludedSupplyChainTradeLineItem>\n\n");
    }
    xml.push_str("  </rsm:SupplyChainTradeTransaction>\n");
    xml.push_str("</rsm:CrossIndustryInvoice>\n");
    xml
}

/// Prefer the active company record, falling back to company_settings.
pub(crate) fn load_company_for_einvoice(
    conn: &rusqlite::Connection,
) -> (String, String, String, String, String, f64) {
    conn.query_row(
        "SELECT COALESCE(name_ar, name_en, ''), COALESCE(vat_number, ''), COALESCE(address, ''),
                COALESCE(cr_number, ''), COALESCE(default_currency, 'KWD'), COALESCE(default_vat_pct, 5.0)
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
    .or_else(|_| {
        conn.query_row(
            "SELECT COALESCE(name, ''), COALESCE(vat_number, ''), COALESCE(address, ''),
                    '', 'KWD', COALESCE(default_vat_pct, 5.0)
             FROM company_settings LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    String::new(),
                    String::from("KWD"),
                    row.get::<_, f64>(5)?,
                ))
            },
        )
    })
    .unwrap_or_default()
}

fn country_code_for_currency(currency: &str) -> &'static str {
    match currency.to_uppercase().as_str() {
        "OMR" => "OM",
        "SAR" => "SA",
        "AED" => "AE",
        "BHD" => "BH",
        "QAR" => "QA",
        _ => "KW",
    }
}

/// Generate (or regenerate) the PINT-OM XML, hash, and QR payload for an invoice.
/// Does not enforce RBAC/licensing; callers are responsible for authorization.
pub(crate) fn generate_invoice_xml(
    conn: &rusqlite::Connection,
    invoice_id: i64,
) -> Result<EInvoiceResult, AppError> {
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
        .map_err(|e| format!("Invoice not found: {}", e))?;

    let (_inv_id, inv_no, net_milli, vat_milli, total_milli, inv_date, cust_name, cust_vat) = inv;

    let (company_name, company_vat, _company_addr, company_cr, currency, vat_rate) =
        load_company_for_einvoice(conn);

    let country = country_code_for_currency(&currency);

    let mut lines_data: Vec<(String, f64, f64, f64)> = Vec::new();
    {
        let mut stmt_lines = conn
            .prepare(
                "SELECT COALESCE(p.name_ar, ''), sil.cartons, sil.unit_price_milli, sil.line_net_milli
                 FROM sales_invoice_lines sil
                 LEFT JOIN products p ON sil.product_id = p.id
                 WHERE sil.invoice_id = ?1",
            )
            ?;
        let rows = stmt_lines
            .query_map([invoice_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)? as f64 / 1000.0,
                    row.get::<_, i64>(3)? as f64 / 1000.0,
                ))
            })
            ?;
        for r in rows.filter_map(|r| r.ok()) {
            lines_data.push(r);
        }
    }

    let inv_date_short = if inv_date.len() >= 10 { &inv_date[..10] } else { &inv_date };
    let xml = generate_pint_om_xml(
        &inv_no, net_milli, vat_milli, total_milli,
        inv_date_short, &cust_name, &cust_vat,
        &company_name, &company_vat, &company_cr,
        &currency, vat_rate, country, &lines_data,
    );

    let xml_hash = compute_hash(xml.as_bytes());

    let qr_summary = serde_json::json!({
        "invoice_no": inv_no,
        "date": inv_date_short,
        "total": total_milli as f64 / 1000.0,
        "vat": vat_milli as f64 / 1000.0,
        "seller": company_name,
        "buyer": cust_name,
    });
    let qr_code_data =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, qr_summary.to_string().as_bytes());

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO e_invoices (invoice_id, xml_content, qr_code, status, created_at)
         VALUES (?1, ?2, ?3, 'generated', ?4)",
        rusqlite::params![invoice_id, xml, qr_code_data, now],
    )
    ?;

    Ok(EInvoiceResult {
        invoice_id,
        invoice_no: inv_no,
        xml_content: xml,
        hash: xml_hash,
        qr_code_data,
        generated_at: now,
    })
}

/// When settings enable submit-on-post, generate the XML and enqueue it for submission.
/// Failures are non-fatal to the posting flow.
pub(crate) fn auto_enqueue_on_post(
    conn: &rusqlite::Connection,
    invoice_id: i64,
) -> Result<(), AppError> {
    let enabled: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM einvoice_settings WHERE active = 1 AND submit_on_post = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);
    if !enabled {
        return Ok(());
    }
    // Commercial (non-tax) invoices are not e-invoiced — skip them.
    let is_commercial: i64 = conn
        .query_row(
            "SELECT COALESCE(is_commercial, 0) FROM sales_invoices WHERE id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if is_commercial > 0 {
        return Ok(());
    }
    if let Ok(_result) = generate_invoice_xml(conn, invoice_id) {
        conn.execute(
            "INSERT OR IGNORE INTO einvoice_queue (invoice_id, action, priority) VALUES (?1, 'submit', 0)",
            [invoice_id],
        ).ok();
    }
    Ok(())
}

#[tauri::command]
pub fn einvoice_generate(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<EInvoiceResult, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_EINVOICE)?;
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let result = generate_invoice_xml(&conn, invoice_id)?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_generate", "e_invoices", Some(invoice_id), None, Some(&result.invoice_no), None);
    Ok(result)
}

#[tauri::command]
pub fn einvoice_validate(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<EInvoiceValidation, AppError> {
    let conn = state.0.lock()?;

    let inv = conn
        .query_row(
            "SELECT si.inv_no, si.net_milli, si.vat_milli, si.total_milli,
                    COALESCE(c.name, ''), COALESCE(c.vat_number, '')
             FROM sales_invoices si
             LEFT JOIN customers c ON si.customer_id = c.id
             WHERE si.id = ?1",
            [invoice_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    let (inv_no, net_milli, vat_milli, total_milli, cust_name, cust_vat) = inv;

    let company = conn
        .query_row(
            "SELECT COALESCE(name, ''), COALESCE(vat_number, '') FROM company_settings LIMIT 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .unwrap_or_default();

    let line_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sales_invoice_lines WHERE invoice_id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut total_checks = 0.0f64;
    let mut passed = 0.0f64;

    macro_rules! check_field {
        ($field:expr, $code:expr, $msg:expr, $severity:expr, $condition:expr) => {
            total_checks += 1.0;
            if $condition {
                passed += 1.0;
            } else {
                let issue = ValidationIssue {
                    field: $field.into(),
                    code: $code.into(),
                    message: $msg.into(),
                    severity: $severity.into(),
                };
                if $severity == "error" {
                    errors.push(issue);
                } else {
                    warnings.push(issue);
                }
            }
        };
    }

    check_field!("Seller.Name", "SELLER_NAME_MISSING", "Seller name is required", "error", !company.0.is_empty());
    check_field!("Seller.VAT", "SELLER_VAT_MISSING", "Seller VAT is required", "error", !company.1.is_empty());
    check_field!("InvoiceNumber", "INV_NO_MISSING", "Invoice number required", "error", !inv_no.is_empty());
    check_field!("Buyer.Name", "BUYER_NAME_MISSING", "Buyer name recommended", "warning", !cust_name.is_empty());
    check_field!("Buyer.VAT", "BUYER_VAT_MISSING", "Buyer VAT recommended for B2B", "warning", !cust_vat.is_empty());
    check_field!("InvoiceLines", "NO_LINES", "Must have at least one line", "error", line_count > 0);
    check_field!("NetTotal", "NEGATIVE_TOTAL", "Net total cannot be negative", "error", net_milli >= 0);
    check_field!("GrandTotal", "NEGATIVE_GRAND", "Grand total cannot be negative", "error", total_milli >= 0);

    total_checks += 1.0;
    let expected_vat = net_milli * 5 / 100;
    if (vat_milli - expected_vat).abs() > 1 {
        warnings.push(ValidationIssue {
            field: "VATTotal".into(),
            code: "VAT_MISMATCH".into(),
            message: format!("VAT {} differs from expected {}", vat_milli, expected_vat),
            severity: "warning".into(),
        });
    } else {
        passed += 1.0;
    }

    total_checks += 1.0;
    if net_milli + vat_milli == total_milli {
        passed += 1.0;
    } else {
        warnings.push(ValidationIssue {
            field: "GrandTotal".into(),
            code: "TOTAL_MISMATCH".into(),
            message: "Grand total != net + VAT".into(),
            severity: "warning".into(),
        });
    }

    let compliance_score = if total_checks > 0.0 {
        (passed / total_checks * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    conn.execute(
        "UPDATE e_invoices SET compliance_score = ?1 WHERE invoice_id = ?2",
        rusqlite::params![compliance_score, invoice_id],
    ).ok();

    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_validate", "e_invoices", Some(invoice_id), None, Some(&format!("score={}", compliance_score)), None);

    Ok(EInvoiceValidation {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        compliance_score,
    })
}

#[tauri::command]
pub fn einvoice_get_status(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<Option<EInvoiceStatusInfo>, AppError> {
    let conn = state.0.lock()?;
    let result = conn
        .query_row(
            "SELECT invoice_id, status, submitted_at, zatca_uuid
             FROM e_invoices WHERE invoice_id = ?1",
            [invoice_id],
            |row| {
                Ok(EInvoiceStatusInfo {
                    invoice_id: row.get(0)?,
                    status: row.get(1)?,
                    submitted_at: row.get(2)?,
                    zatca_uuid: row.get(3)?,
                })
            },
        )
        .ok();
    Ok(result)
}

#[tauri::command]
pub fn einvoice_list(
    state: State<'_, DbState>,
    status: Option<String>,
) -> Result<Vec<EInvoiceRecord>, AppError> {
    let conn = state.0.lock()?;

    let stmt = if let Some(ref s) = status {
        let mut st = conn
            .prepare(
                "SELECT ei.id, ei.invoice_id, si.inv_no, COALESCE(c.name, ''),
                        si.total_milli, ei.status, COALESCE(ei.compliance_score, 0), ei.created_at, ei.submitted_at
                 FROM e_invoices ei
                 JOIN sales_invoices si ON ei.invoice_id = si.id
                 LEFT JOIN customers c ON si.customer_id = c.id
                 WHERE ei.status = ?1
                 ORDER BY ei.created_at DESC",
            )
            ?;
        let rows = st
            .query_map([s], |row| {
                Ok(EInvoiceRecord {
                    id: row.get(0)?,
                    invoice_id: row.get(1)?,
                    invoice_no: row.get(2)?,
                    customer_name: row.get(3)?,
                    total_milli: row.get(4)?,
                    status: row.get(5)?,
                    compliance_score: row.get(6)?,
                    created_at: row.get(7)?,
                    submitted_at: row.get(8)?,
                })
            })
            ?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    } else {
        let mut st = conn
            .prepare(
                "SELECT ei.id, ei.invoice_id, si.inv_no, COALESCE(c.name, ''),
                        si.total_milli, ei.status, COALESCE(ei.compliance_score, 0), ei.created_at, ei.submitted_at
                 FROM e_invoices ei
                 JOIN sales_invoices si ON ei.invoice_id = si.id
                 LEFT JOIN customers c ON si.customer_id = c.id
                 ORDER BY ei.created_at DESC",
            )
            ?;
        let rows = st
            .query_map([], |row| {
                Ok(EInvoiceRecord {
                    id: row.get(0)?,
                    invoice_id: row.get(1)?,
                    invoice_no: row.get(2)?,
                    customer_name: row.get(3)?,
                    total_milli: row.get(4)?,
                    status: row.get(5)?,
                    compliance_score: row.get(6)?,
                    created_at: row.get(7)?,
                    submitted_at: row.get(8)?,
                })
            })
            ?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    Ok(stmt)
}

#[tauri::command]
pub fn einvoice_mark_submitted(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
    submission_ref: Option<String>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = COALESCE(?2, zatca_uuid)
             WHERE invoice_id = ?3",
            rusqlite::params![now, submission_ref, invoice_id],
        )
        ?;
    if rows == 0 {
        return Err(AppError::not_found(format!("No e-invoice found for invoice_id {}", invoice_id)));
    }
    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_mark_submitted", "e_invoices", Some(invoice_id), None, Some("submitted"), None);
    Ok(format!("Invoice {} marked as submitted", invoice_id))
}

#[tauri::command]
pub fn einvoice_cancel(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
    reason: String,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE e_invoices SET status = 'cancelled', cancel_reason = ?1, cancelled_at = ?2
             WHERE invoice_id = ?3 AND status IN ('generated', 'submitted', 'pending')",
            rusqlite::params![reason, now, invoice_id],
        )
        ?;
    if rows == 0 {
        return Err(AppError::business(format!("Cannot cancel invoice {}. Already processed or not found.", invoice_id)));
    }
    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_cancel", "e_invoices", Some(invoice_id), None, Some("cancelled"), Some(&reason));
    Ok(format!("Invoice {} cancelled: {}", invoice_id, reason))
}

#[tauri::command]
pub fn einvoice_submit(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let einv = conn
        .query_row(
            "SELECT id, invoice_id, xml_content, status FROM e_invoices WHERE invoice_id = ?1",
            [invoice_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|_| AppError::not_found(format!("E-invoice not generated for invoice_id {}", invoice_id)))?;

    let (_einv_id, _inv_id, xml_content, status) = einv;

    if status == "submitted" || status == "accepted" {
        return Err(AppError::validation(format!("Invoice already submitted (status: {})", status)));
    }

    let settings = conn
        .query_row(
            "SELECT environment, tax_authority_endpoint, api_key, api_secret
             FROM einvoice_settings WHERE active = 1 LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .ok();

    let (env, endpoint, api_key, api_secret) = settings.unwrap_or_else(|| {
        ("sandbox".into(), None, None, None)
    });

    // Stored credentials are encrypted at rest; decrypt before transmission.
    let dec_key = api_key.as_deref().map(crate::crypto::decrypt_if_needed).transpose().map_err(AppError::crypto)?;
    let dec_secret = api_secret.as_deref().map(crate::crypto::decrypt_if_needed).transpose().map_err(AppError::crypto)?;

    let submission_result = submit_to_tax_authority(
        &invoice_id.to_string(),
        &env,
        endpoint.as_deref(),
        dec_key.as_deref(),
        dec_secret.as_deref(),
        &xml_content,
    );

    match submission_result {
        Ok(ref_no) => {
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = ?2 WHERE invoice_id = ?3",
                rusqlite::params![now, ref_no, invoice_id],
            )?;
            conn.execute(
                "UPDATE einvoice_queue SET status = 'completed' WHERE invoice_id = ?1",
                [invoice_id],
            ).ok();
            let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_submit", "e_invoices", None, None, Some(&format!("invoice={} ref={}", invoice_id, ref_no)), None);
            Ok(format!("Invoice {} submitted successfully. Ref: {}", invoice_id, ref_no))
        }
        Err(e) => {
            let err_str = e.to_string();
            conn.execute(
                "UPDATE e_invoices SET status = 'rejected', rejection_reason = ?1 WHERE invoice_id = ?2",
                rusqlite::params![err_str, invoice_id],
            ).ok();
            conn.execute(
                "UPDATE einvoice_queue SET status = 'failed', last_error = ?1, retry_count = retry_count + 1 WHERE invoice_id = ?2",
                rusqlite::params![err_str, invoice_id],
            ).ok();
            let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_submit_failed", "e_invoices", None, None, Some(&format!("invoice={}", invoice_id)), Some(&err_str));
            Err(AppError::business(format!("Submission failed: {}", e)))
        }
    }
}

pub(crate) fn submit_to_tax_authority(
    invoice_id: &str,
    environment: &str,
    endpoint: Option<&str>,
    api_key: Option<&str>,
    api_secret: Option<&str>,
    xml_content: &str,
) -> Result<String, AppError> {
    if environment == "sandbox" {
        return Ok(format!("SANDBOX-{}-{}", invoice_id, chrono::Utc::now().timestamp()));
    }

    let url = endpoint
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .ok_or_else(|| {
            AppError::config("Production tax authority endpoint is not configured. Set it in E-Invoice settings.")
        })?;

    let api_key = api_key.map(str::trim).filter(|k| !k.is_empty());
    let api_secret = api_secret.map(str::trim).filter(|s| !s.is_empty());

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::business(format!("Failed to build HTTP client: {e}")))?;

    let mut req = client
        .post(url)
        .header("Content-Type", "application/xml")
        .header("Accept", "application/json, application/xml")
        .header("X-Invoice-ID", invoice_id)
        .body(xml_content.to_string());

    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }
    if let Some(secret) = api_secret {
        req = req.header("X-API-Secret", secret);
    }

    let resp = req
        .send()
        .map_err(|e| AppError::business(format!("Submission HTTP request failed: {e}")))?;

    let status = resp.status();
    let body = resp.text().unwrap_or_default();

    if status.is_success() {
        // Extract the OTA/ASP reference (UUID) from the response when present.
        let reference = extract_submission_reference(&body)
            .unwrap_or_else(|| format!("OTA-{}-{}", invoice_id, chrono::Utc::now().timestamp()));
        Ok(reference)
    } else {
        let snippet: String = body.chars().take(500).collect();
        Err(AppError::business(format!(
            "Tax authority rejected submission (HTTP {}): {}",
            status.as_u16(),
            if snippet.is_empty() { "empty response body".into() } else { snippet }
        )))
    }
}

/// Extract an OTA/ASP submission reference (UUID) from a JSON or XML response body.
fn extract_submission_reference(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    // JSON responses commonly use fields like uuid / reference / id.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        for key in ["uuid", "reference", "submissionUuid", "trackingId", "id"] {
            if let Some(s) = value
                .get(key)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(s.to_string());
            }
            if let Some(s) = value
                .get("data")
                .and_then(|d| d.get(key))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                return Some(s.to_string());
            }
        }
    }
    // Fallback: first UUID-looking token anywhere in the body.
    for token in trimmed.split_whitespace() {
        let clean: String = token
            .chars()
            .filter(|c| c.is_ascii_hexdigit() || *c == '-')
            .collect();
        if is_uuid_like(&clean) {
            return Some(clean);
        }
    }
    None
}

fn is_uuid_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5 && parts.iter().all(|p| !p.is_empty() && p.len() <= 12)
}

#[tauri::command]
pub fn einvoice_add_to_queue(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_id: i64,
    action: Option<String>,
    priority: Option<i32>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let act = action.unwrap_or_else(|| "submit".into());
    let pri = priority.unwrap_or(0);

    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM einvoice_queue WHERE invoice_id = ?1 AND status IN ('pending', 'failed')",
            [invoice_id],
            |row| row.get::<_, i64>(0),
        ).map(|c| c > 0).unwrap_or(false);

    if exists {
        return Err("Invoice already queued".into());
    }

    conn.execute(
        "INSERT INTO einvoice_queue (invoice_id, action, priority) VALUES (?1, ?2, ?3)",
        rusqlite::params![invoice_id, act, pri],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_add_to_queue", "einvoice_queue", None, None, Some(&format!("invoice={} action={}", invoice_id, act)), None);

    Ok(format!("Invoice {} queued for {}", invoice_id, act))
}

#[tauri::command]
pub fn einvoice_process_queue(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let (env, endpoint, api_key, api_secret): (String, Option<String>, Option<String>, Option<String>) = {
        let conn = state.0.lock()?;
        conn.query_row(
            "SELECT environment, tax_authority_endpoint, api_key, api_secret
             FROM einvoice_settings WHERE active = 1 LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .unwrap_or_else(|_| ("sandbox".into(), None, None, None))
    };

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

    let items: Vec<i64> = {
        let conn = state.0.lock()?;
        let mut stmt = conn
            .prepare(
                "SELECT invoice_id FROM einvoice_queue
                 WHERE status = 'pending'
                 AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))
                 AND retry_count < max_retries
                 ORDER BY priority DESC, created_at ASC
                 LIMIT 10",
            )
            ?;
        let rows = stmt
            .query_map([], |row| row.get::<_, i64>(0))
            ?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let mut processed = 0i64;
    let mut failed = 0i64;

    for inv_id in items {
        let conn = state.0.lock()?;

        let einv = conn
            .query_row(
                "SELECT id, xml_content FROM e_invoices WHERE invoice_id = ?1",
                [inv_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .ok();

        if let Some((_eid, xml)) = einv {
            let result = submit_to_tax_authority(
                &inv_id.to_string(),
                &env,
                endpoint.as_deref(),
                dec_key.as_deref(),
                dec_secret.as_deref(),
                &xml,
            );
            match result {
                Ok(ref_no) => {
                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = ?2 WHERE invoice_id = ?3",
                        rusqlite::params![now, ref_no, inv_id],
                    ).ok();
                    conn.execute(
                        "UPDATE einvoice_queue SET status = 'completed', last_error = NULL WHERE invoice_id = ?1",
                        [inv_id],
                    ).ok();
                    processed += 1;
                }
                Err(e) => {
                    let err_str = e.to_string();
                    conn.execute(
                        "UPDATE einvoice_queue
                         SET status = CASE WHEN retry_count + 1 >= max_retries THEN 'failed' ELSE 'pending' END,
                             last_error = ?1,
                             retry_count = retry_count + 1,
                             next_retry_at = datetime('now', '+' || ((retry_count + 1) * 60) || ' seconds')
                         WHERE invoice_id = ?2",
                        rusqlite::params![err_str, inv_id],
                    ).ok();
                    failed += 1;
                }
            }
        }
    }

    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_process_queue", "einvoice_queue", None, None, Some(&format!("processed={} failed={}", processed, failed)), None);

    Ok(format!("Processed: {}, Failed: {}", processed, failed))
}

#[tauri::command]
pub fn einvoice_get_dashboard(
    state: State<'_, DbState>,
) -> Result<EInvoiceDashboard, AppError> {
    let conn = state.0.lock()?;

    let stats = conn
        .query_row(
            "SELECT
                COUNT(*) as total_generated,
                SUM(CASE WHEN status = 'submitted' THEN 1 ELSE 0 END) as submitted,
                SUM(CASE WHEN status = 'accepted' THEN 1 ELSE 0 END) as accepted,
                SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) as rejected,
                SUM(CASE WHEN status IN ('generated', 'pending') THEN 1 ELSE 0 END) as pending_submit,
                COALESCE(SUM(si.total_milli), 0) as total_amt,
                COALESCE(SUM(si.vat_milli), 0) as total_vat
             FROM e_invoices ei
             JOIN sales_invoices si ON ei.invoice_id = si.id",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .unwrap_or((0, 0, 0, 0, 0, 0, 0));

    let queue_pending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM einvoice_queue WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let queue_failed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM einvoice_queue WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let settings_configured: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM einvoice_settings WHERE active = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    Ok(EInvoiceDashboard {
        total_generated: stats.0,
        total_submitted: stats.1,
        total_accepted: stats.2,
        total_rejected: stats.3,
        total_pending_submission: stats.4,
        total_amount_milli: stats.5,
        total_vat_milli: stats.6,
        queue_pending,
        queue_failed,
        settings_configured,
    })
}

#[tauri::command]
pub fn einvoice_get_settings(
    state: State<'_, DbState>,
) -> Result<Option<EInvoiceSettingsData>, AppError> {
    let conn = state.0.lock()?;
    let result = conn
        .query_row(
            "SELECT id, company_id, environment, auto_submit, submit_on_post,
                    tax_authority_endpoint, active
             FROM einvoice_settings WHERE active = 1 LIMIT 1",
            [],
            |row| {
                Ok(EInvoiceSettingsData {
                    id: row.get(0)?,
                    company_id: row.get(1)?,
                    environment: row.get(2)?,
                    auto_submit: row.get::<_, i32>(3)? != 0,
                    submit_on_post: row.get::<_, i32>(4)? != 0,
                    tax_authority_endpoint: row.get(5)?,
                    active: row.get::<_, i32>(6)? != 0,
                })
            },
        )
        .ok();
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn einvoice_save_settings(
    state: State<'_, DbState>,
    user_id: i64,
    environment: String,
    auto_submit: bool,
    submit_on_post: bool,
    tax_authority_endpoint: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    portal_username: Option<String>,
    portal_password: Option<String>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let company_id: i64 = conn
        .query_row("SELECT id FROM companies LIMIT 1", [], |row| row.get(0))
        .unwrap_or(1);

    let enc_key = api_key.as_ref().map(|k| crate::crypto::encrypt_if_needed(k)).transpose()?;
    let enc_secret = api_secret.as_ref().map(|s| crate::crypto::encrypt_if_needed(s)).transpose()?;
    let enc_portal_user = portal_username.as_ref().map(|u| crate::crypto::encrypt_if_needed(u)).transpose()?;
    let enc_portal_pass = portal_password.as_ref().map(|p| crate::crypto::encrypt_if_needed(p)).transpose()?;

    let existing = conn
        .query_row(
            "SELECT id FROM einvoice_settings WHERE company_id = ?1",
            [company_id],
            |row| row.get::<_, i64>(0),
        )
        .ok();

    if let Some(eid) = existing {
        conn.execute(
            "UPDATE einvoice_settings SET environment = ?1, auto_submit = ?2, submit_on_post = ?3,
             tax_authority_endpoint = ?4, api_key = COALESCE(?5, api_key),
             api_secret = COALESCE(?6, api_secret),
             portal_username = COALESCE(?7, portal_username),
             portal_password = COALESCE(?8, portal_password),
             updated_at = datetime('now')
             WHERE id = ?9",
            rusqlite::params![environment, auto_submit as i32, submit_on_post as i32,
                tax_authority_endpoint, enc_key, enc_secret, enc_portal_user, enc_portal_pass, eid],
        )?;
    } else {
        conn.execute(
            "INSERT INTO einvoice_settings (company_id, environment, auto_submit, submit_on_post,
             tax_authority_endpoint, api_key, api_secret, portal_username, portal_password)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![company_id, environment, auto_submit as i32, submit_on_post as i32,
                tax_authority_endpoint, enc_key, enc_secret, enc_portal_user, enc_portal_pass],
        )?;
    }

    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_save_settings", "einvoice_settings", None, None, Some(&environment), None);
    Ok("Settings saved".into())
}

#[tauri::command]
pub fn einvoice_get_queue(
    state: State<'_, DbState>,
) -> Result<Vec<EInvoiceQueueItem>, AppError> {
    let conn = state.0.lock()?;

    let mut stmt = conn
        .prepare(
            "SELECT eq.id, eq.invoice_id, COALESCE(si.inv_no, ''), COALESCE(c.name, ''),
                    COALESCE(si.total_milli, 0), eq.action, eq.retry_count, eq.max_retries,
                    eq.last_error, eq.next_retry_at, eq.status, eq.created_at
             FROM einvoice_queue eq
             LEFT JOIN sales_invoices si ON eq.invoice_id = si.id
             LEFT JOIN customers c ON si.customer_id = c.id
             ORDER BY eq.priority DESC, eq.created_at DESC",
        )
        ?;

    let rows = stmt
        .query_map([], |row| {
            Ok(EInvoiceQueueItem {
                id: row.get(0)?,
                invoice_id: row.get(1)?,
                invoice_no: row.get(2)?,
                customer_name: row.get(3)?,
                total_milli: row.get(4)?,
                action: row.get(5)?,
                retry_count: row.get(6)?,
                max_retries: row.get(7)?,
                last_error: row.get(8)?,
                next_retry_at: row.get(9)?,
                status: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        ?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn einvoice_retry_queue_item(
    state: State<'_, DbState>,
    user_id: i64,
    queue_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    conn.execute(
        "UPDATE einvoice_queue SET status = 'pending', last_error = NULL, next_retry_at = datetime('now') WHERE id = ?1",
        [queue_id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_retry_queue_item", "einvoice_queue", Some(queue_id), None, Some("pending"), None);
    Ok("Queue item reset for retry".into())
}

#[tauri::command]
pub fn einvoice_summary_report(
    state: State<'_, DbState>,
    from_date: String,
    to_date: String,
) -> Result<EInvoiceReport, AppError> {
    let conn = state.0.lock()?;

    let report = conn
        .query_row(
            "SELECT
                COUNT(*) as total,
                COALESCE(SUM(si.total_milli), 0) as total_amount,
                COALESCE(SUM(si.vat_milli), 0) as total_vat,
                SUM(CASE WHEN ei.status = 'submitted' THEN 1 ELSE 0 END),
                SUM(CASE WHEN ei.status = 'accepted' THEN 1 ELSE 0 END),
                SUM(CASE WHEN ei.status = 'rejected' THEN 1 ELSE 0 END),
                SUM(CASE WHEN ei.status IN ('generated', 'pending') THEN 1 ELSE 0 END)
             FROM e_invoices ei
             JOIN sales_invoices si ON ei.invoice_id = si.id
             WHERE ei.created_at >= ?1 AND ei.created_at <= ?2",
            rusqlite::params![from_date, to_date],
            |row| {
                Ok(EInvoiceReport {
                    from_date: from_date.clone(),
                    to_date: to_date.clone(),
                    total_invoices: row.get(0)?,
                    total_amount_milli: row.get(1)?,
                    total_vat_milli: row.get(2)?,
                    submitted: row.get(3)?,
                    accepted: row.get(4)?,
                    rejected: row.get(5)?,
                    pending: row.get(6)?,
                })
            },
        )
        ?;

    Ok(report)
}

#[tauri::command]
pub fn einvoice_get_xml(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    let xml = conn
        .query_row(
            "SELECT xml_content FROM e_invoices WHERE invoice_id = ?1",
            [invoice_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| AppError::not_found("ملف XML غير موجود"))?;
    Ok(xml)
}

#[tauri::command]
pub fn einvoice_bulk_generate(
    state: State<'_, DbState>,
    user_id: i64,
    invoice_ids: Vec<i64>,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let mut count = 0i64;
    for inv_id in &invoice_ids {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM e_invoices WHERE invoice_id = ?1",
                [inv_id],
                |row| row.get::<_, i64>(0),
            ).map(|c| c > 0).unwrap_or(false);

        if !exists {
            conn.execute(
                "INSERT INTO e_invoices (invoice_id, xml_content, status, created_at)
                 VALUES (?1, '<pending/>', 'pending', datetime('now'))",
                [inv_id],
            ).map_err(|e| format!("Failed to create e-invoice record for invoice {}: {}", inv_id, e))?;
        }
    conn.execute(
        "INSERT OR IGNORE INTO einvoice_queue (invoice_id, action, priority) VALUES (?1, 'submit', 0)",
        [inv_id],
    ).ok();
        count += 1;
    }
    let _ = rbac::log_audit(&conn, Some(user_id), None, "einvoice_bulk_generate", "e_invoices", None, None, Some(&format!("count={}", count)), None);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn sha256_hash_is_deterministic_and_64_hex() {
        let a = compute_hash(b"hello");
        let b = compute_hash(b"hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn sha256_differs_for_adjacent_xml() {
        let a = compute_hash(b"<Invoice/>");
        let b = compute_hash(b"<Invoice/ >");
        assert_ne!(a, b);
    }

    #[test]
    fn extract_submission_reference_from_json_top_level() {
        let body = r#"{"uuid":"7c9e6679-7425-40de-944b-e07fc1f90ae7","status":"submitted"}"#;
        assert_eq!(
            extract_submission_reference(body).as_deref(),
            Some("7c9e6679-7425-40de-944b-e07fc1f90ae7")
        );
    }

    #[test]
    fn extract_submission_reference_from_json_nested_data() {
        let body = r#"{"data":{"trackingId":"11111111-2222-3333-4444-555555555555"}}"#;
        assert_eq!(
            extract_submission_reference(body).as_deref(),
            Some("11111111-2222-3333-4444-555555555555")
        );
    }

    #[test]
    fn extract_submission_reference_falls_back_to_uuid_scan() {
        let body = "Submission accepted. Reference: 6ba7b810-9dad-11d1-80b4-00c04fd430c8";
        assert_eq!(
            extract_submission_reference(body).as_deref(),
            Some("6ba7b810-9dad-11d1-80b4-00c04fd430c8")
        );
    }

    #[test]
    fn extract_submission_reference_none_for_garbage() {
        assert_eq!(extract_submission_reference(""), None);
        assert_eq!(extract_submission_reference("{\"error\":\"nope\"}"), None);
    }

    #[test]
    fn is_uuid_like_accepts_real_uuid_only() {
        assert!(is_uuid_like("7c9e6679-7425-40de-944b-e07fc1f90ae7"));
        assert!(!is_uuid_like("7c9e6679-7425-40de-944b"));
        assert!(!is_uuid_like("not-a-uuid"));
        assert!(!is_uuid_like(""));
    }

    #[test]
    fn sandbox_submission_returns_sandbox_reference() {
        let ref_no = submit_to_tax_authority("42", "sandbox", None, None, None, "<Invoice/>").unwrap();
        assert!(ref_no.starts_with("SANDBOX-42-"));
    }

    #[test]
    fn production_submission_requires_endpoint() {
        let err = submit_to_tax_authority("42", "production", None, None, None, "<Invoice/>").unwrap_err();
        assert!(err.to_string().contains("endpoint"));
    }

    #[test]
    fn sha256_matches_known_vector() {
        // "hello" is a well-known SHA-256 test vector.
        assert_eq!(
            compute_hash(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn xml_escape_handles_all_five_entities() {
        assert_eq!(
            xml_escape("A & B <tag> \"quote\" 'apos'"),
            "A &amp; B &lt;tag&gt; &quot;quote&quot; &apos;apos&apos;"
        );
    }

    #[test]
    fn xml_escape_leaves_safe_input_unchanged() {
        assert_eq!(xml_escape("plain invoice 123"), "plain invoice 123");
        assert_eq!(xml_escape(""), "");
    }

    #[test]
    fn pint_om_xml_contains_core_identifiers() {
        let xml = generate_pint_om_xml(
            "INV-0001", 100_000, 5_000, 105_000, "2026-08-10",
            "ACME LLC", "OM123456", "PRO MAX FACTORY", "OM654321",
            "CR12345", "OMR", 5.0, "OM", &[
                ("Widget".to_string(), 10.0, 10.000, 100.000),
            ],
        );
        assert!(xml.contains("<ram:ID>INV-0001</ram:ID>"));
        assert!(xml.contains("<ram:TypeCode>380</ram:TypeCode>"));
        assert!(xml.contains("urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"));
        assert!(xml.contains("<ram:Name>ACME LLC</ram:Name>"));
        assert!(xml.contains("<ram:Name>PRO MAX FACTORY</ram:Name>"));
        assert!(xml.contains("schemeID=\"9921\""));
        assert!(xml.contains("schemeID=\"9930\""));
        assert!(xml.contains("<ram:ID schemeID=\"9930\">OMCR12345</ram:ID>"));
    }

    #[test]
    fn pint_om_xml_uses_omr_amounts_and_vat_rate() {
        let xml = generate_pint_om_xml(
            "INV-0002", 250_000, 12_500, 262_500, "2026-08-10",
            "Buyer", "", "Seller", "", "", "OMR", 5.0, "OM", &[],
        );
        assert!(xml.contains("<ram:InvoiceCurrencyCode>OMR</ram:InvoiceCurrencyCode>"));
        assert!(xml.contains("<ram:LineTotalAmount currencyID=\"OMR\">250.000</ram:LineTotalAmount>"));
        assert!(xml.contains("<ram:TaxBasisTotalAmount currencyID=\"OMR\">250.000</ram:TaxBasisTotalAmount>"));
        assert!(xml.contains("<ram:TaxTotalAmount currencyID=\"OMR\">12.500</ram:TaxTotalAmount>"));
        assert!(xml.contains("<ram:GrandTotalAmount currencyID=\"OMR\">262.500</ram:GrandTotalAmount>"));
        assert!(xml.contains("<ram:DuePayableAmount currencyID=\"OMR\">262.500</ram:DuePayableAmount>"));
        assert!(xml.contains("<ram:RateApplicablePercent>5.00</ram:RateApplicablePercent>"));
    }

    #[test]
    fn pint_om_xml_uses_configured_currency_vat_rate_and_cr() {
        let xml = generate_pint_om_xml(
            "INV-0005", 100_000, 15_000, 115_000, "2026-08-10",
            "Buyer", "", "KWT Factory", "888888888888888", "CR-7788",
            "KWD", 15.0, "KW", &[],
        );
        assert!(xml.contains("<ram:InvoiceCurrencyCode>KWD</ram:InvoiceCurrencyCode>"));
        assert!(xml.contains("<ram:RateApplicablePercent>15.00</ram:RateApplicablePercent>"));
        assert!(xml.contains("<ram:ID schemeID=\"9930\">KWCR-7788</ram:ID>"));
        assert!(xml.contains("<ram:GrandTotalAmount currencyID=\"KWD\">115.000</ram:GrandTotalAmount>"));
        assert!(!xml.contains("OMR"));
    }

    #[test]
    fn pint_om_xml_escapes_special_chars_in_names() {
        let xml = generate_pint_om_xml(
            "INV-0003", 100_000, 5_000, 105_000, "2026-08-10",
            "ACME & Sons <Ltd>", "", "A&B Co", "", "", "OMR", 5.0, "OM", &[],
        );
        assert!(xml.contains("<ram:Name>ACME &amp; Sons &lt;Ltd&gt;</ram:Name>"));
        assert!(xml.contains("<ram:Name>A&amp;B Co</ram:Name>"));
        assert!(!xml.contains("& Sons <Ltd>"));
    }

    #[test]
    fn pint_om_xml_includes_line_items_with_indexes() {
        let xml = generate_pint_om_xml(
            "INV-0004", 30_000, 1_500, 31_500, "2026-08-10",
            "Buyer", "", "Seller", "", "", "OMR", 5.0, "OM", &[
                ("First".to_string(), 2.0, 10.000, 20.000),
                ("Second".to_string(), 1.0, 10.000, 10.000),
            ],
        );
        assert!(xml.contains("<ram:LineID>1</ram:LineID>"));
        assert!(xml.contains("<ram:LineID>2</ram:LineID>"));
        assert!(xml.contains("<ram:Name>First</ram:Name>"));
        assert!(xml.contains("<ram:Name>Second</ram:Name>"));
        assert!(xml.contains("<ram:BilledQuantity unitCode=\"C62\">2</ram:BilledQuantity>"));
        assert!(xml.contains("<ram:NetPriceAmount currencyID=\"OMR\">10.000</ram:NetPriceAmount>"));
        assert!(xml.ends_with("</rsm:CrossIndustryInvoice>\n"));
    }

    #[test]
    fn country_code_for_currency_maps_known_currencies() {
        assert_eq!(country_code_for_currency("KWD"), "KW");
        assert_eq!(country_code_for_currency("OMR"), "OM");
        assert_eq!(country_code_for_currency("SAR"), "SA");
        assert_eq!(country_code_for_currency("AED"), "AE");
        assert_eq!(country_code_for_currency("BHD"), "BH");
        assert_eq!(country_code_for_currency("QAR"), "QA");
        assert_eq!(country_code_for_currency("USD"), "KW");
    }

    #[test]
    fn load_company_prefers_active_company_record() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn.execute_batch(
            "CREATE TABLE companies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name_ar TEXT NOT NULL,
                name_en TEXT,
                cr_number TEXT,
                vat_number TEXT,
                address TEXT,
                phone TEXT,
                email TEXT,
                website TEXT,
                logo_path TEXT,
                default_currency TEXT NOT NULL DEFAULT 'OMR',
                default_vat_pct REAL NOT NULL DEFAULT 5.0,
                fiscal_year_start TEXT NOT NULL DEFAULT '01-01',
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ).unwrap();
        conn.execute(
            "INSERT INTO companies(code, name_ar, name_en, cr_number, vat_number, address, default_currency, default_vat_pct)
             VALUES('MAIN', 'الشركة الرئيسية', 'Main Co', 'CR-7788', '777777777777777', 'Kuwait City', 'KWD', 15.0)",
            [],
        ).unwrap();
        let (name, vat, addr, cr, currency, vat_pct) = load_company_for_einvoice(&conn);
        assert_eq!(name, "الشركة الرئيسية");
        assert_eq!(vat, "777777777777777");
        assert_eq!(addr, "Kuwait City");
        assert_eq!(cr, "CR-7788");
        assert_eq!(currency, "KWD");
        assert_eq!(vat_pct, 15.0);
    }

    #[test]
    fn load_company_falls_back_to_company_settings() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn.execute(
            "INSERT INTO company_settings(id, name, vat_number, address, default_vat_pct)
             VALUES(1, 'Legacy Co', '123456789', 'Old Street', 10.0)",
            [],
        ).unwrap();
        let (name, vat, addr, cr, currency, vat_pct) = load_company_for_einvoice(&conn);
        assert_eq!(name, "Legacy Co");
        assert_eq!(vat, "123456789");
        assert_eq!(addr, "Old Street");
        assert_eq!(cr, "");
        assert_eq!(currency, "KWD");
        assert_eq!(vat_pct, 10.0);
    }
}
