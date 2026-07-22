use crate::db::DbState;
use serde::{Deserialize, Serialize};
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
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn generate_pint_om_xml(
    _inv_id: i64,
    inv_no: &str,
    net_milli: i64,
    vat_milli: i64,
    total_milli: i64,
    inv_date: &str,
    cust_name: &str,
    cust_vat: &str,
    company_name: &str,
    company_vat: &str,
    _company_addr: &str,
    company_cr: &str,
    _contact_person: &str,
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
        xml.push_str(&format!("        <ram:ID schemeID=\"9930\">OM</ram:ID>\n"));
    }
    xml.push_str("      </ram:SellerTradeParty>\n");
    xml.push_str("    </ram:ApplicableHeaderTradeAgreement>\n\n");
    xml.push_str("    <ram:ApplicableHeaderTradeDelivery>\n");
    xml.push_str(&format!("      <ram:ActualDeliverySupplyChainEvent><ram:OccurrenceDateTime><udt:DateTimeString format=\"102\">{}</udt:DateTimeString></ram:OccurrenceDateTime></ram:ActualDeliverySupplyChainEvent>\n", xml_escape(inv_date)));
    xml.push_str("    </ram:ApplicableHeaderTradeDelivery>\n\n");
    xml.push_str("    <ram:ApplicableHeaderTradeSettlement>\n");
    xml.push_str("      <ram:InvoiceCurrencyCode>OMR</ram:InvoiceCurrencyCode>\n");
    xml.push_str(&format!("      <ram:PayeeTradeParty><ram:Name>{}</ram:Name></ram:PayeeTradeParty>\n", xml_escape(company_name)));
    xml.push_str("      <ram:ApplicableTradeTax>\n");
    xml.push_str(&format!("        <ram:CalculatedAmount currencyID=\"OMR\">{:.3}</ram:CalculatedAmount>\n", vat_milli as f64 / 1000.0));
    xml.push_str("        <ram:TypeCode>VAT</ram:TypeCode>\n");
    xml.push_str(&format!("        <ram:BasisAmount currencyID=\"OMR\">{:.3}</ram:BasisAmount>\n", net_milli as f64 / 1000.0));
    xml.push_str("        <ram:RateApplicablePercent>5.00</ram:RateApplicablePercent>\n");
    xml.push_str("      </ram:ApplicableTradeTax>\n");
    xml.push_str(&format!("      <ram:SpecifiedTradeSettlementMonetarySummation>\n"));
    xml.push_str(&format!("        <ram:LineTotalAmount currencyID=\"OMR\">{:.3}</ram:LineTotalAmount>\n", net_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:TaxBasisTotalAmount currencyID=\"OMR\">{:.3}</ram:TaxBasisTotalAmount>\n", net_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:TaxTotalAmount currencyID=\"OMR\">{:.3}</ram:TaxTotalAmount>\n", vat_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:GrandTotalAmount currencyID=\"OMR\">{:.3}</ram:GrandTotalAmount>\n", total_milli as f64 / 1000.0));
    xml.push_str(&format!("        <ram:DuePayableAmount currencyID=\"OMR\">{:.3}</ram:DuePayableAmount>\n", total_milli as f64 / 1000.0));
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
        xml.push_str(&format!("        <ram:NetPriceAmount currencyID=\"OMR\">{:.3}</ram:NetPriceAmount>\n", price));
        xml.push_str("      </ram:SpecifiedLineTradeAgreement>\n");
        xml.push_str("      <ram:SpecifiedLineTradeDelivery>\n");
        xml.push_str(&format!("        <ram:BilledQuantity unitCode=\"C62\">{:.0}</ram:BilledQuantity>\n", qty));
        xml.push_str("      </ram:SpecifiedLineTradeDelivery>\n");
        xml.push_str("      <ram:SpecifiedLineTradeSettlement>\n");
        xml.push_str(&format!("        <ram:ApplicableTradeTax>\n"));
        xml.push_str("          <ram:TypeCode>VAT</ram:TypeCode>\n");
        xml.push_str("          <ram:RateApplicablePercent>5.00</ram:RateApplicablePercent>\n");
        xml.push_str("        </ram:ApplicableTradeTax>\n");
        xml.push_str(&format!("        <ram:SpecifiedTradeSettlementMonetarySummation>\n"));
        xml.push_str(&format!("          <ram:LineTotalAmount currencyID=\"OMR\">{:.3}</ram:LineTotalAmount>\n", total));
        xml.push_str("        </ram:SpecifiedTradeSettlementMonetarySummation>\n");
        xml.push_str("      </ram:SpecifiedLineTradeSettlement>\n");
        xml.push_str("    </ram:IncludedSupplyChainTradeLineItem>\n\n");
    }
    xml.push_str("  </rsm:SupplyChainTradeTransaction>\n");
    xml.push_str("</rsm:CrossIndustryInvoice>\n");
    xml
}

#[tauri::command]
pub fn einvoice_generate(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<EInvoiceResult, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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

    let company = conn.query_row(
        "SELECT COALESCE(name, ''), COALESCE(vat_number, ''), COALESCE(address, ''), COALESCE(cr_number, '')
         FROM companies WHERE id = (SELECT company_id FROM sales_invoices WHERE id = ?1)
         UNION ALL
         SELECT COALESCE(name, ''), COALESCE(vat_number, ''), COALESCE(address, ''), ''
         FROM company_settings LIMIT 1",
        [invoice_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    ).unwrap_or_default();

    let (company_name, company_vat, company_addr, company_cr) = company;

    let contact_person = conn
        .query_row(
            "SELECT COALESCE(contact_person, '') FROM customers
             WHERE id = (SELECT customer_id FROM sales_invoices WHERE id = ?1)",
            [invoice_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();

    let mut lines_data: Vec<(String, f64, f64, f64)> = Vec::new();
    {
        let mut stmt_lines = conn
            .prepare(
                "SELECT COALESCE(p.name_ar, ''), sil.cartons, sil.unit_price_milli, sil.line_net_milli
                 FROM sales_invoice_lines sil
                 LEFT JOIN products p ON sil.product_id = p.id
                 WHERE sil.invoice_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt_lines
            .query_map([invoice_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)? as f64 / 1000.0,
                    row.get::<_, i64>(3)? as f64 / 1000.0,
                ))
            })
            .map_err(|e| e.to_string())?;
        for r in rows.filter_map(|r| r.ok()) {
            lines_data.push(r);
        }
    }

    let inv_date_short = if inv_date.len() >= 10 { &inv_date[..10] } else { &inv_date };
    let xml = generate_pint_om_xml(
        invoice_id, &inv_no, net_milli, vat_milli, total_milli,
        inv_date_short, &cust_name, &cust_vat,
        &company_name, &company_vat, &company_addr, &company_cr,
        &contact_person, &lines_data,
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
    .map_err(|e| e.to_string())?;

    Ok(EInvoiceResult {
        invoice_id,
        invoice_no: inv_no,
        xml_content: xml,
        hash: xml_hash,
        qr_code_data,
        generated_at: now,
    })
}

#[tauri::command]
pub fn einvoice_validate(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<EInvoiceValidation, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
) -> Result<Option<EInvoiceStatusInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
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
) -> Result<Vec<EInvoiceRecord>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    Ok(stmt)
}

#[tauri::command]
pub fn einvoice_mark_submitted(
    state: State<'_, DbState>,
    invoice_id: i64,
    submission_ref: Option<String>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = COALESCE(?2, zatca_uuid)
             WHERE invoice_id = ?3",
            rusqlite::params![now, submission_ref, invoice_id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("No e-invoice found for invoice_id {}", invoice_id));
    }
    Ok(format!("Invoice {} marked as submitted", invoice_id))
}

#[tauri::command]
pub fn einvoice_cancel(
    state: State<'_, DbState>,
    invoice_id: i64,
    reason: String,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE e_invoices SET status = 'cancelled', cancel_reason = ?1, cancelled_at = ?2
             WHERE invoice_id = ?3 AND status IN ('generated', 'submitted', 'pending')",
            rusqlite::params![reason, now, invoice_id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("Cannot cancel invoice {}. Already processed or not found.", invoice_id));
    }
    Ok(format!("Invoice {} cancelled: {}", invoice_id, reason))
}

#[tauri::command]
pub fn einvoice_submit(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
        .map_err(|_| format!("E-invoice not generated for invoice_id {}", invoice_id))?;

    let (_einv_id, _inv_id, xml_content, status) = einv;

    if status == "submitted" || status == "accepted" {
        return Err(format!("Invoice already submitted (status: {})", status));
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

    let submission_result = submit_to_tax_authority(
        &invoice_id.to_string(),
        &env,
        endpoint.as_deref(),
        api_key.as_deref(),
        api_secret.as_deref(),
        &xml_content,
    );

    match submission_result {
        Ok(ref_no) => {
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = ?2 WHERE invoice_id = ?3",
                rusqlite::params![now, ref_no, invoice_id],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE einvoice_queue SET status = 'completed' WHERE invoice_id = ?1",
                [invoice_id],
            ).ok();
            Ok(format!("Invoice {} submitted successfully. Ref: {}", invoice_id, ref_no))
        }
        Err(e) => {
            conn.execute(
                "UPDATE e_invoices SET status = 'rejected', rejection_reason = ?1 WHERE invoice_id = ?2",
                rusqlite::params![e, invoice_id],
            ).ok();
            conn.execute(
                "UPDATE einvoice_queue SET status = 'failed', last_error = ?1, retry_count = retry_count + 1 WHERE invoice_id = ?2",
                rusqlite::params![e, invoice_id],
            ).ok();
            Err(format!("Submission failed: {}", e))
        }
    }
}

fn submit_to_tax_authority(
    invoice_id: &str,
    environment: &str,
    endpoint: Option<&str>,
    _api_key: Option<&str>,
    _api_secret: Option<&str>,
    _xml_content: &str,
) -> Result<String, String> {
    if environment == "sandbox" || endpoint.is_none() {
        return Ok(format!("SANDBOX-{}-{}", invoice_id, chrono::Utc::now().timestamp()));
    }
    let url = endpoint.unwrap_or("");
    if url.is_empty() {
        return Ok(format!("SANDBOX-SIMULATED-{}", invoice_id));
    }
    Err("Production endpoint not yet configured. Please set up e-invoicing settings.".into())
}

#[tauri::command]
pub fn einvoice_add_to_queue(
    state: State<'_, DbState>,
    invoice_id: i64,
    action: Option<String>,
    priority: Option<i32>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
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
    ).map_err(|e| e.to_string())?;

    Ok(format!("Invoice {} queued for {}", invoice_id, act))
}

#[tauri::command]
pub fn einvoice_process_queue(
    state: State<'_, DbState>,
) -> Result<String, String> {
    let items: Vec<i64> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT invoice_id FROM einvoice_queue
                 WHERE status = 'pending'
                 AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))
                 AND retry_count < max_retries
                 ORDER BY priority DESC, created_at ASC
                 LIMIT 10",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let mut processed = 0i64;
    let mut failed = 0i64;

    for inv_id in items {
        let conn = state.0.lock().map_err(|e| e.to_string())?;

        let einv = conn
            .query_row(
                "SELECT id, xml_content FROM e_invoices WHERE invoice_id = ?1",
                [inv_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .ok();

        if let Some((_eid, xml)) = einv {
            let result = submit_to_tax_authority(&inv_id.to_string(), "sandbox", None, None, None, &xml);
            match result {
                Ok(ref_no) => {
                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "UPDATE e_invoices SET status = 'submitted', submitted_at = ?1, zatca_uuid = ?2 WHERE invoice_id = ?3",
                        rusqlite::params![now, ref_no, inv_id],
                    ).ok();
                    conn.execute(
                        "UPDATE einvoice_queue SET status = 'completed' WHERE invoice_id = ?1",
                        [inv_id],
                    ).ok();
                    processed += 1;
                }
                Err(e) => {
                    conn.execute(
                        "UPDATE einvoice_queue SET status = 'failed', last_error = ?1, retry_count = retry_count + 1 WHERE invoice_id = ?2",
                        rusqlite::params![e, inv_id],
                    ).ok();
                    failed += 1;
                }
            }
        }
    }

    Ok(format!("Processed: {}, Failed: {}", processed, failed))
}

#[tauri::command]
pub fn einvoice_get_dashboard(
    state: State<'_, DbState>,
) -> Result<EInvoiceDashboard, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
) -> Result<Option<EInvoiceSettingsData>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn einvoice_save_settings(
    state: State<'_, DbState>,
    environment: String,
    auto_submit: bool,
    submit_on_post: bool,
    tax_authority_endpoint: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    portal_username: Option<String>,
    portal_password: Option<String>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let company_id: i64 = conn
        .query_row("SELECT id FROM companies LIMIT 1", [], |row| row.get(0))
        .unwrap_or(1);

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
                tax_authority_endpoint, api_key, api_secret, portal_username, portal_password, eid],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO einvoice_settings (company_id, environment, auto_submit, submit_on_post,
             tax_authority_endpoint, api_key, api_secret, portal_username, portal_password)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![company_id, environment, auto_submit as i32, submit_on_post as i32,
                tax_authority_endpoint, api_key, api_secret, portal_username, portal_password],
        ).map_err(|e| e.to_string())?;
    }

    Ok("Settings saved".into())
}

#[tauri::command]
pub fn einvoice_get_queue(
    state: State<'_, DbState>,
) -> Result<Vec<EInvoiceQueueItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn einvoice_retry_queue_item(
    state: State<'_, DbState>,
    queue_id: i64,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE einvoice_queue SET status = 'pending', last_error = NULL, next_retry_at = datetime('now') WHERE id = ?1",
        [queue_id],
    ).map_err(|e| e.to_string())?;
    Ok("Queue item reset for retry".into())
}

#[tauri::command]
pub fn einvoice_summary_report(
    state: State<'_, DbState>,
    from_date: String,
    to_date: String,
) -> Result<EInvoiceReport, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;

    Ok(report)
}

#[tauri::command]
pub fn einvoice_get_xml(
    state: State<'_, DbState>,
    invoice_id: i64,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let xml = conn
        .query_row(
            "SELECT xml_content FROM e_invoices WHERE invoice_id = ?1",
            [invoice_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "XML not found".to_string())?;
    Ok(xml)
}

#[tauri::command]
pub fn einvoice_bulk_generate(
    state: State<'_, DbState>,
    invoice_ids: Vec<i64>,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut count = 0i64;
    for inv_id in &invoice_ids {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM e_invoices WHERE invoice_id = ?1",
                [inv_id],
                |row| row.get::<_, i64>(0),
            ).map(|c| c > 0).unwrap_or(false);

        if !exists {
            let _ = conn.execute(
                "INSERT INTO e_invoices (invoice_id, xml_content, status, created_at)
                 VALUES (?1, '<pending/>', 'pending', datetime('now'))",
                [inv_id],
            );
        }
        conn.execute(
            "INSERT OR IGNORE INTO einvoice_queue (invoice_id, action, priority) VALUES (?1, 'submit', 0)",
            [inv_id],
        ).ok();
        count += 1;
    }
    Ok(count)
}
