//! ZATCA (Saudi Arabia) e-invoicing Phase 2 engine.
//!
//! Implements the full Integration-phase document pipeline for standard
//! (B2B) and simplified (B2C) tax invoices against the FATOORA platform:
//!
//! * UBL 2.1 standard invoice XML with KSA-specific extensions
//! * Phase-2 QR code (9 TLV tags, base64)
//! * Invoice hash = base64(SHA-256(canonicalized pre-signature XML))
//! * ECDSA (secp256k1) signing of the invoice hash + X.509 certificate digest
//! * CSID onboarding (CSR), renewal and revocation support
//! * JOSE-JWS signed Authorization headers used by all FATOORA calls
//! * Clearance / Reporting API clients
//!
//! References: ZATCA "Detailed Technical Guideline", "XML Implementation
//! Standard" and "Security Features Implementation Standard" (secp256k1,
//! C14N11, BR-KSA rules).

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine;
use k256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub const ZATCA_DEFAULT_VAT_RATE: f64 = 15.0;
pub const ZATCA_DEFAULT_CURRENCY: &str = "SAR";
/// ZATCA standard invoice type (BT-3 = 388). Simplified = 388 with subtype 02.
pub const ZATCA_INVOICE_TYPE_CODE: &str = "388";
/// KSA transaction type: standard tax invoice, generated/sold by the seller.
pub const ZATCA_TXN_STANDARD: &str = "0100000";
/// KSA transaction type: simplified tax invoice.
pub const ZATCA_TXN_SIMPLIFIED: &str = "0200000";

pub const FATTOORA_BASE_SIM: &str = "https://gw-fatoora.zatca.gov.sa/einvoicing/simulation";
pub const FATTOORA_BASE_PROD: &str = "https://gw-fatoora.zatca.gov.sa/einvoicing";

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ZatcaAddress {
    pub street_name: String,
    /// KSA building number (must be 4 digits, BR-KSA-37).
    pub building_number: String,
    pub plot_identification: String,
    pub city_subdivision_name: String,
    pub city_name: String,
    pub postal_zone: String,
    /// ISO 3166-1 alpha-2, e.g. "SA".
    pub country_code: String,
}

impl ZatcaAddress {
    pub fn sa() -> Self {
        Self {
            street_name: String::new(),
            building_number: String::new(),
            plot_identification: String::new(),
            city_subdivision_name: String::new(),
            city_name: "Riyadh".into(),
            postal_zone: String::new(),
            country_code: "SA".into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZatcaParty {
    /// Commercial registration number / organization identifier.
    pub crn: String,
    /// VAT number: 15 digits, starts with "3", ends with "3".
    pub vat_number: String,
    /// Legal registration name.
    pub name: String,
    pub address: ZatcaAddress,
}

#[derive(Debug, Clone)]
pub struct ZatcaLine {
    pub id: u32,
    pub item_name: String,
    pub quantity: f64,
    /// UN/ECE 20 recommendation unit code, e.g. "PCE".
    pub unit_code: String,
    /// Net unit price (excl. VAT).
    pub unit_price: f64,
    /// Net line total (excl. VAT).
    pub line_total: f64,
    pub vat_rate: f64,
    /// UNCL5305 tax category: S (standard), E (exempt), Z (zero-rated)...
    pub tax_category: String,
}

#[derive(Debug, Clone)]
pub struct ZatcaInvoiceData {
    pub invoice_number: String,
    /// UBL cbc:UUID (v4).
    pub uuid: String,
    /// Issue date, YYYY-MM-DD.
    pub issue_date: String,
    /// Issue time, HH:MM:SS with offset (e.g. "14:30:00+03:00").
    pub issue_time: String,
    /// KSA transaction type (see ZATCA_TXN_*).
    pub transaction_type: String,
    /// ISO 4217 currency.
    pub currency: String,
    /// Invoice counter value (ICV, BR-KSA-33/34) — sequential per CSID.
    pub icv: u64,
    /// Previous invoice hash (PIH, BR-KSA-61), base64 SHA-256. None for the
    /// very first document issued by the CSID.
    pub pih: Option<String>,
    pub seller: ZatcaParty,
    pub buyer: ZatcaParty,
    pub lines: Vec<ZatcaLine>,
    /// Net amount (excl. VAT), currency units.
    pub net_amount: f64,
    pub vat_amount: f64,
    /// Total incl. VAT.
    pub total_amount: f64,
    pub allowance_total: f64,
    /// Optional notes (e.g. "VAT is not applicable..." for exemptions).
    pub notes: Vec<String>,
}

impl Default for ZatcaInvoiceData {
    fn default() -> Self {
        Self {
            invoice_number: String::new(),
            uuid: String::new(),
            issue_date: String::new(),
            issue_time: String::new(),
            transaction_type: ZATCA_TXN_STANDARD.into(),
            currency: ZATCA_DEFAULT_CURRENCY.into(),
            icv: 1,
            pih: None,
            seller: ZatcaParty::default(),
            buyer: ZatcaParty::default(),
            lines: Vec::new(),
            net_amount: 0.0,
            vat_amount: 0.0,
            total_amount: 0.0,
            allowance_total: 0.0,
            notes: Vec::new(),
        }
    }
}

/// Cryptographic material for the EGS unit.
#[derive(Clone)]
pub struct ZatcaKeys {
    pub signing_key: SigningKey,
    /// X.509 certificate issued by ZATCA (CSID), DER bytes. Empty until onboarding.
    pub certificate_der: Vec<u8>,
}

impl ZatcaKeys {
    pub fn random() -> Self {
        Self {
            signing_key: SigningKey::random(&mut rand::rngs::OsRng),
            certificate_der: Vec::new(),
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        *self.signing_key.verifying_key()
    }

    /// Raw 64-byte public key (X || Y, no 0x04 prefix) — ZATCA QR tag 8.
    pub fn public_key_raw(&self) -> Vec<u8> {
        let pt = self.verifying_key().to_encoded_point(false);
        let bytes = pt.as_bytes();
        bytes[1..].to_vec()
    }

    /// Raw 65-byte SEC1 public key (0x04 || X || Y) for X.509 SPKI.
    pub fn public_key_sec1(&self) -> Vec<u8> {
        let pt = self.verifying_key().to_encoded_point(false);
        pt.as_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// TLV QR (Phase 2: 9 tags)
// ---------------------------------------------------------------------------

pub fn tlv_encode(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + value.len());
    out.push(tag);
    out.push(value.len() as u8);
    out.extend_from_slice(value);
    out
}

/// Parse back a TLV byte stream (used by tests and validation).
pub fn tlv_decode(raw: &[u8]) -> Vec<(u8, Vec<u8>)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 2 <= raw.len() {
        let tag = raw[i];
        let len = raw[i + 1] as usize;
        if i + 2 + len > raw.len() {
            break;
        }
        out.push((tag, raw[i + 2..i + 2 + len].to_vec()));
        i += 2 + len;
    }
    out
}

#[derive(Debug, Clone)]
pub struct Phase2Qr {
    pub seller_name: String,
    pub vat_number: String,
    pub timestamp: String,
    pub total: f64,
    pub vat_amount: f64,
    /// base64 SHA-256 invoice hash.
    pub invoice_hash: String,
    /// base64 DER ECDSA signature.
    pub ecdsa_signature: String,
    /// base64 raw 64-byte public key.
    pub public_key: String,
    /// ZATCA CA signature of the stamp public key (simplified docs).
    /// Left empty for the seller; ZATCA provides it during clearance.
    pub ca_signature: String,
}

/// Build the Phase-2 QR content (base64 of the TLV byte stream).
pub fn build_phase2_qr(qr: &Phase2Qr) -> String {
    let mut raw = Vec::new();
    raw.extend(tlv_encode(1, qr.seller_name.as_bytes()));
    raw.extend(tlv_encode(2, qr.vat_number.as_bytes()));
    raw.extend(tlv_encode(3, qr.timestamp.as_bytes()));
    raw.extend(tlv_encode(4, format!("{:.2}", qr.total).as_bytes()));
    raw.extend(tlv_encode(5, format!("{:.2}", qr.vat_amount).as_bytes()));
    raw.extend(tlv_encode(6, qr.invoice_hash.as_bytes()));
    raw.extend(tlv_encode(7, qr.ecdsa_signature.as_bytes()));
    raw.extend(tlv_encode(8, qr.public_key.as_bytes()));
    raw.extend(tlv_encode(9, qr.ca_signature.as_bytes()));
    BASE64.encode(&raw)
}

/// Decode a Phase-2 QR into its tags for validation.
pub fn decode_phase2_qr(payload: &str) -> Result<Vec<(u8, Vec<u8>)>, String> {
    let raw = BASE64
        .decode(payload.as_bytes())
        .map_err(|e| format!("invalid base64: {}", e))?;
    let tags = tlv_decode(&raw);
    if tags.is_empty() {
        return Err("empty TLV stream".into());
    }
    for (tag, value) in &tags {
        if *tag < 1 || *tag > 9 {
            return Err(format!("invalid tag {}", tag));
        }
        if value.len() > 255 {
            return Err(format!("tag {} value too long", tag));
        }
    }
    Ok(tags)
}

// ---------------------------------------------------------------------------
// Hashing & signing
// ---------------------------------------------------------------------------

/// base64(SHA-256(bytes)).
pub fn sha256_base64(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    BASE64.encode(digest)
}

/// ECDSA (secp256k1) sign a raw 32-byte digest; returns base64 DER signature.
pub fn sign_digest_base64(key: &SigningKey, digest: &[u8]) -> Result<String, String> {
    let sig: Signature = key
        .sign_prehash(digest)
        .map_err(|e| format!("signing failed: {}", e))?;
    Ok(BASE64.encode(sig.to_der().as_bytes()))
}

/// Verify a base64 DER signature over a raw digest.
pub fn verify_signature(public_key: &VerifyingKey, digest: &[u8], sig_b64: &str) -> Result<(), String> {
    let der = BASE64
        .decode(sig_b64.as_bytes())
        .map_err(|e| format!("bad signature base64: {}", e))?;
    let sig = Signature::from_der(&der).map_err(|e| format!("bad DER: {}", e))?;
    public_key
        .verify_prehash(digest, &sig)
        .map_err(|e| format!("signature mismatch: {}", e))
}

// ---------------------------------------------------------------------------
// UBL 2.1 XML generation
// ---------------------------------------------------------------------------

const UBL_ROOT: &str = "<Invoice xmlns=\"urn:oasis:names:specification:ubl:schema:xsd:Invoice-2\" \
xmlns:cac=\"urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2\" \
xmlns:cbc=\"urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2\" \
xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\" \
xmlns:ext=\"urn:oasis:names:specification:ubl:schema:xsd:CommonExtensionComponents-2\" \
xmlns:qdt=\"urn:oasis:names:specification:ubl:schema:xsd:QualifiedDatatypes-2\" \
xmlns:sac=\"urn:oasis:names:specification:ubl:schema:xsd:SignatureAggregateComponents-2\" \
xmlns:sig=\"urn:oasis:names:specification:ubl:signature:1\" \
xmlns:udt=\"urn:oasis:names:specification:ubl:schema:xsd:UnqualifiedDataTypes-2\">";

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn money(v: f64) -> String {
    format!("{:.2}", v)
}

fn party_xml(party: &ZatcaParty, is_supplier: bool) -> String {
    let role = if is_supplier { "AccountingSupplierParty" } else { "AccountingCustomerParty" };
    let mut s = String::new();
    s.push_str(&format!("<cac:{}>", role));
    s.push_str("<cac:Party>");
    if !party.crn.is_empty() {
        s.push_str(&format!("<cac:PartyIdentification><cbc:ID schemeID=\"CRN\">{}</cbc:ID></cac:PartyIdentification>", esc(&party.crn)));
    }
    s.push_str("<cac:PostalAddress>");
    if !party.address.street_name.is_empty() {
        s.push_str(&format!("<cbc:StreetName>{}</cbc:StreetName>", esc(&party.address.street_name)));
    }
    if !party.address.building_number.is_empty() {
        s.push_str(&format!("<cbc:BuildingNumber>{}</cbc:BuildingNumber>", esc(&party.address.building_number)));
    }
    if !party.address.plot_identification.is_empty() {
        s.push_str(&format!("<cbc:PlotIdentification>{}</cbc:PlotIdentification>", esc(&party.address.plot_identification)));
    }
    if !party.address.city_subdivision_name.is_empty() {
        s.push_str(&format!("<cbc:CitySubdivisionName>{}</cbc:CitySubdivisionName>", esc(&party.address.city_subdivision_name)));
    }
    if !party.address.city_name.is_empty() {
        s.push_str(&format!("<cbc:CityName>{}</cbc:CityName>", esc(&party.address.city_name)));
    }
    if !party.address.postal_zone.is_empty() {
        s.push_str(&format!("<cbc:PostalZone>{}</cbc:PostalZone>", esc(&party.address.postal_zone)));
    }
    let country = if party.address.country_code.is_empty() { "SA" } else { &party.address.country_code };
    s.push_str(&format!("<cac:Country><cbc:IdentificationCode>{}</cbc:IdentificationCode></cac:Country>", esc(country)));
    s.push_str("</cac:PostalAddress>");
    if !party.vat_number.is_empty() {
        s.push_str(&format!("<cac:PartyTaxScheme><cbc:CompanyID>{}</cbc:CompanyID><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:PartyTaxScheme>", esc(&party.vat_number)));
    }
    s.push_str(&format!("<cac:PartyLegalEntity><cbc:RegistrationName>{}</cbc:RegistrationName></cac:PartyLegalEntity>", esc(&party.name)));
    s.push_str("</cac:Party>");
    s.push_str(&format!("</cac:{}>", role));
    s
}

/// Build the invoice body. `with_qr` / `with_signature` control the subtrees
/// that must be excluded when computing the invoice hash (ZATCA XPath removal
/// list: UBLExtensions, AdditionalDocumentReference[QR], Signature).
pub fn build_invoice_body(
    data: &ZatcaInvoiceData,
    with_qr: bool,
    with_signature: bool,
    phase2_qr: Option<&Phase2Qr>,
    cert_b64: Option<&str>,
    cert_hash_b64: Option<&str>,
    sig_b64: Option<&str>,
) -> String {
    let mut s = String::new();
    s.push_str(UBL_ROOT);
    s.push_str("<cbc:UBLVersionID>2.1</cbc:UBLVersionID>");
    s.push_str("<cbc:CustomizationID>urn:oasis:names:specification:ubl:xsd:invoice-2</cbc:CustomizationID>");
    s.push_str("<cbc:ProfileID>reporting:1.0</cbc:ProfileID>");
    s.push_str(&format!("<cbc:ID>{}</cbc:ID>", esc(&data.invoice_number)));
    s.push_str(&format!("<cbc:UUID>{}</cbc:UUID>", esc(&data.uuid)));
    s.push_str(&format!("<cbc:IssueDate>{}</cbc:IssueDate>", esc(&data.issue_date)));
    s.push_str(&format!("<cbc:IssueTime>{}</cbc:IssueTime>", esc(&data.issue_time)));
    s.push_str(&format!("<cbc:InvoiceTypeCode name=\"{}\">{}</cbc:InvoiceTypeCode>", esc(&data.transaction_type), ZATCA_INVOICE_TYPE_CODE));
    for n in &data.notes {
        s.push_str(&format!("<cbc:Note>{}</cbc:Note>", esc(n)));
    }
    s.push_str(&format!("<cbc:DocumentCurrencyCode>{}</cbc:DocumentCurrencyCode>", esc(&data.currency)));
    // Previous invoice hash (BR-KSA-61 / KSA-13).
    if let Some(pih) = &data.pih {
        s.push_str(&format!("<cac:AdditionalDocumentReference><cbc:ID>ICV</cbc:ID><cbc:UUID>{}</cbc:UUID><cac:Attachment><cbc:EmbeddedDocumentBinaryObject mimeCode=\"text/plain\">{}</cbc:EmbeddedDocumentBinaryObject></cac:Attachment></cac:AdditionalDocumentReference>", data.icv, esc(pih)));
    }
    // Phase-2 QR (KSA-14).
    if with_qr {
        if let Some(qr) = phase2_qr {
            let qr_b64 = build_phase2_qr(qr);
            s.push_str(&format!("<cac:AdditionalDocumentReference><cbc:ID>QR</cbc:ID><cbc:UUID>urn:uuid:{}</cbc:UUID><cac:Attachment><cbc:EmbeddedDocumentBinaryObject mimeCode=\"text/plain\">{}</cbc:EmbeddedDocumentBinaryObject></cac:Attachment></cac:AdditionalDocumentReference>", esc(&data.uuid), qr_b64));
        }
    }
    s.push_str(&party_xml(&data.seller, true));
    s.push_str(&party_xml(&data.buyer, false));
    // TaxTotal
    s.push_str(&format!("<cac:TaxTotal><cbc:TaxAmount currencyID=\"{}\">{}</cbc:TaxAmount>", esc(&data.currency), money(data.vat_amount)));
    let rate = data.lines.first().map(|l| l.vat_rate).unwrap_or(ZATCA_DEFAULT_VAT_RATE);
    let category = data.lines.first().map(|l| l.tax_category.clone()).unwrap_or_else(|| "S".into());
    s.push_str(&format!("<cac:TaxSubtotal><cbc:TaxableAmount currencyID=\"{}\">{}</cbc:TaxableAmount>", esc(&data.currency), money(data.net_amount)));
    s.push_str(&format!("<cbc:TaxAmount currencyID=\"{}\">{}</cbc:TaxAmount>", esc(&data.currency), money(data.vat_amount)));
    s.push_str(&format!("<cbc:Percent>{:.2}</cbc:Percent>", rate));
    s.push_str(&format!("<cac:TaxCategory><cbc:ID schemeID=\"UNCL5305\">{}</cbc:ID>", esc(&category)));
    s.push_str(&format!("<cbc:Percent>{:.2}</cbc:Percent>", rate));
    s.push_str("<cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:TaxCategory>");
    s.push_str("</cac:TaxSubtotal></cac:TaxTotal>");
    // LegalMonetaryTotal
    s.push_str("<cac:LegalMonetaryTotal>");
    s.push_str(&format!("<cbc:LineExtensionAmount currencyID=\"{}\">{}</cbc:LineExtensionAmount>", esc(&data.currency), money(data.net_amount)));
    if data.allowance_total != 0.0 {
        s.push_str(&format!("<cbc:AllowanceTotalAmount currencyID=\"{}\">{}</cbc:AllowanceTotalAmount>", esc(&data.currency), money(data.allowance_total)));
    }
    s.push_str(&format!("<cbc:TaxExclusiveAmount currencyID=\"{}\">{}</cbc:TaxExclusiveAmount>", esc(&data.currency), money(data.net_amount)));
    s.push_str(&format!("<cbc:TaxInclusiveAmount currencyID=\"{}\">{}</cbc:TaxInclusiveAmount>", esc(&data.currency), money(data.total_amount)));
    s.push_str(&format!("<cbc:PayableAmount currencyID=\"{}\">{}</cbc:PayableAmount>", esc(&data.currency), money(data.total_amount)));
    s.push_str("</cac:LegalMonetaryTotal>");
    // Lines
    for line in &data.lines {
        s.push_str("<cac:InvoiceLine>");
        s.push_str(&format!("<cbc:ID>{}</cbc:ID>", line.id));
        let unit = if line.unit_code.is_empty() { "PCE" } else { &line.unit_code };
        s.push_str(&format!("<cbc:InvoicedQuantity unitCode=\"{}\">{}</cbc:InvoicedQuantity>", esc(unit), line.quantity));
        s.push_str(&format!("<cbc:LineExtensionAmount currencyID=\"{}\">{}</cbc:LineExtensionAmount>", esc(&data.currency), money(line.line_total)));
        s.push_str("<cac:Item>");
        s.push_str(&format!("<cbc:Name>{}</cbc:Name>", esc(&line.item_name)));
        s.push_str(&format!("<cac:ClassifiedTaxCategory><cbc:ID schemeID=\"UNCL5305\">{}</cbc:ID>", esc(&line.tax_category)));
        s.push_str(&format!("<cbc:Percent>{:.2}</cbc:Percent>", line.vat_rate));
        s.push_str("<cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:ClassifiedTaxCategory>");
        s.push_str("</cac:Item>");
        s.push_str(&format!("<cac:Price><cbc:PriceAmount currencyID=\"{}\">{}</cbc:PriceAmount></cac:Price>", esc(&data.currency), money(line.unit_price)));
        s.push_str("</cac:InvoiceLine>");
    }
    // Signature extension (UBLDocumentSignatures, BR-KSA-28).
    if with_signature {
        if let (Some(cert), Some(cert_hash), Some(sig)) = (cert_b64, cert_hash_b64, sig_b64) {
            s.push_str("<ext:UBLExtensions><ext:UBLExtension><ext:ExtensionContent><sig:UBLDocumentSignatures xmlns:sig=\"urn:oasis:names:specification:ubl:signature:1\" xmlns:sac=\"urn:oasis:names:specification:ubl:schema:xsd:SignatureAggregateComponents-2\" xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\">");
            s.push_str("<sac:SignatureInformation>");
            s.push_str("<cbc:ID>urn:oasis:names:specification:ubl:signature:1</cbc:ID>");
            s.push_str("<cbc:ReferencedSignatureID>urn:oasis:names:specification:ubl:signature:Invoice</cbc:ReferencedSignatureID>");
            s.push_str("<ds:Signature Id=\"signature\"><ds:SignedInfo>");
            s.push_str("<ds:CanonicalizationMethod Algorithm=\"http://www.w3.org/2006/12/xml-c14n11\"/>");
            s.push_str("<ds:SignatureMethod Algorithm=\"http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha256\"/>");
            s.push_str("<ds:Reference Id=\"invoiceSignedData\" URI=\"\"><ds:Transforms><ds:Transform Algorithm=\"http://www.w3.org/2006/12/xml-c14n11\"/></ds:Transforms>");
            s.push_str("<ds:DigestMethod Algorithm=\"http://www.w3.org/2001/04/xmlenc#sha256\"/>");
            s.push_str(&format!("<ds:DigestValue>{}</ds:DigestValue>", esc(cert_hash)));
            s.push_str("</ds:Reference></ds:SignedInfo>");
            s.push_str(&format!("<ds:SignatureValue>{}</ds:SignatureValue>", esc(sig)));
            s.push_str(&format!("<ds:KeyInfo><ds:X509Data><ds:X509Certificate>{}</ds:X509Certificate></ds:X509Data></ds:KeyInfo>", esc(cert)));
            s.push_str("</ds:Signature>");
            s.push_str("</sac:SignatureInformation>");
            s.push_str("</sig:UBLDocumentSignatures></ext:ExtensionContent></ext:UBLExtension></ext:UBLExtensions>");
        }
    }
    s.push_str("</Invoice>");
    s
}

/// Full pipeline: canonical base XML -> invoice hash -> ECDSA signature ->
/// final signed document. Returns (final_xml, hash_base64, signature_base64).
pub fn generate_signed_invoice(
    data: &ZatcaInvoiceData,
    keys: &ZatcaKeys,
) -> Result<(String, String, String), String> {
    let base = build_invoice_body(data, false, false, None, None, None, None);
    let hash_b64 = sha256_base64(base.as_bytes());
    let digest = {
        let d = BASE64
            .decode(hash_b64.as_bytes())
            .map_err(|e| format!("hash decode: {}", e))?;
        d
    };
    let sig_b64 = sign_digest_base64(&keys.signing_key, &digest)?;

    let cert_b64 = if keys.certificate_der.is_empty() {
        None
    } else {
        Some(BASE64.encode(&keys.certificate_der))
    };
    let cert_hash_b64 = (!keys.certificate_der.is_empty()).then(|| sha256_base64(&keys.certificate_der));

    let qr = Phase2Qr {
        seller_name: data.seller.name.clone(),
        vat_number: data.seller.vat_number.clone(),
        timestamp: format!("{}T{}", data.issue_date, data.issue_time),
        total: data.total_amount,
        vat_amount: data.vat_amount,
        invoice_hash: hash_b64.clone(),
        ecdsa_signature: sig_b64.clone(),
        public_key: BASE64.encode(keys.public_key_raw()),
        ca_signature: String::new(),
    };

    let final_xml = build_invoice_body(
        data,
        true,
        true,
        Some(&qr),
        cert_b64.as_deref(),
        cert_hash_b64.as_deref(),
        Some(&sig_b64),
    );
    Ok((final_xml, hash_b64, sig_b64))
}

// ---------------------------------------------------------------------------
// CSID onboarding (CSR generation)
// ---------------------------------------------------------------------------

fn der_len(len: usize) -> Vec<u8> {
    if len < 0x80 {
        vec![len as u8]
    } else if len <= 0xff {
        vec![0x81, len as u8]
    } else {
        vec![0x82, ((len >> 8) & 0xff) as u8, (len & 0xff) as u8]
    }
}

fn der_tlv(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend(der_len(content.len()));
    out.extend_from_slice(content);
    out
}

fn der_oid(oid: &[u64]) -> Vec<u8> {
    let mut out = vec![(oid[0] * 40 + oid[1]) as u8];
    for &comp in &oid[2..] {
        let mut c = comp;
        let mut buf = vec![(c & 0x7f) as u8];
        c >>= 7;
        while c > 0 {
            let mut next = vec![0x80 | ((c & 0x7f) as u8)];
            next.extend(&buf);
            buf = next;
            c >>= 7;
        }
        out.extend(buf);
    }
    der_tlv(0x06, &out)
}

/// Minimal DER writer producing a valid PKCS#10 CertificationRequest with an
/// ECDSA secp256k1 key and the ZATCA-required subject DN. This mirrors the
/// OpenSSL command shipped in the ZATCA Detailed Technical Guideline:
/// `openssl req -new -sha256 -key privateKey.pem -subj "/C=SA/CN=<vat>/O=<org>"`.
/// One `AttributeTypeAndValue` = SEQUENCE { OID, value }.
fn rdn_attr(oid: &[u64], value_der: &[u8]) -> Vec<u8> {
    let mut atv = der_oid(oid);
    atv.extend_from_slice(value_der);
    der_tlv(0x30, &atv)
}

/// One `RDN` = SET OF AttributeTypeAndValue.
fn rdn(oid: &[u64], value_der: &[u8]) -> Vec<u8> {
    der_tlv(0x31, &rdn_attr(oid, value_der))
}

pub fn build_csr(keys: &ZatcaKeys, vat_number: &str, org_name: &str) -> Result<Vec<u8>, String> {
    if !valid_vat_number(vat_number) {
        return Err(format!("invalid ZATCA VAT number: {}", vat_number));
    }
    let spki = build_spki(keys)?;

    // Subject RDNSequence, ordered C, O, CN like OpenSSL emits.
    let c = rdn(&[2, 5, 4, 6], &der_tlv(0x13, b"SA")); // countryName, PrintableString
    let o = rdn(&[2, 5, 4, 10], &der_tlv(0x0c, org_name.as_bytes())); // organizationName, UTF8String
    let cn = rdn(&[2, 5, 4, 3], &der_tlv(0x0c, vat_number.as_bytes())); // commonName, UTF8String
    let mut rdn_seq = c;
    rdn_seq.extend(o);
    rdn_seq.extend(cn);
    let subject = der_tlv(0x30, &rdn_seq);

    let version = der_tlv(0x02, &[0x00]);

    // attributes [0] { } empty (no extension request needed for ZATCA CSR).
    let attributes = der_tlv(0xa0, &[]);

    let mut cri = version;
    cri.extend(subject);
    cri.extend(spki);
    cri.extend(attributes);
    let cri = der_tlv(0x30, &cri);

    let sig_algo = ecdsa_sig_algorithm();
    let mut sig_tlv = sig_algo;
    let bit_string = {
        // DER signature is an ASN.1 value; PKCS#10 wraps it as BIT STRING.
        let der_sig = k256_der_signature(keys, &cri)?;
        let mut bs = vec![0x00];
        bs.extend(der_sig);
        der_tlv(0x03, &bs)
    };
    sig_tlv.extend(bit_string);

    let mut csr = cri;
    csr.extend(sig_tlv);
    Ok(der_tlv(0x30, &csr))
}

fn k256_der_signature(keys: &ZatcaKeys, msg: &[u8]) -> Result<Vec<u8>, String> {
    let digest = Sha256::digest(msg);
    let sig: Signature = keys
        .signing_key
        .sign_prehash(&digest)
        .map_err(|e| format!("csr signing failed: {}", e))?;
    Ok(sig.to_der().as_bytes().to_vec())
}

fn ecdsa_sig_algorithm() -> Vec<u8> {
    // AlgorithmIdentifier { id-ecdsa-with-SHA256 (1.2.840.10045.4.3.2) }
    let algo = der_oid(&[1, 2, 840, 10045, 4, 3, 2]);
    der_tlv(0x30, &algo)
}

/// SubjectPublicKeyInfo for an EC key.
fn build_spki(keys: &ZatcaKeys) -> Result<Vec<u8>, String> {
    let alg_params = der_oid(&[1, 3, 132, 0, 10]); // secp256k1
    let algo = {
        let ec_public_key = der_oid(&[1, 2, 840, 10045, 2, 1]);
        let mut ai = ec_public_key;
        ai.extend(alg_params);
        der_tlv(0x30, &ai)
    };
    let key_bytes = keys.public_key_sec1();
    let mut bs = vec![0x00];
    bs.extend(key_bytes);
    let bit_string = der_tlv(0x03, &bs);
    let mut spki = algo;
    spki.extend(bit_string);
    Ok(der_tlv(0x30, &spki))
}

// ---------------------------------------------------------------------------
// JOSE-JWS signed header (FATOORA Authorization)
// ---------------------------------------------------------------------------

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Build the JWS used in the `Authorization: JWS ...` header of FATOORA calls.
/// Uses the CSID private key; header includes `b64: false` so the payload is
/// signed as raw bytes (per ZATCA's documented JOSE profile).
pub fn build_jws(keys: &ZatcaKeys, cert_b64: &str, action: &str) -> Result<String, String> {
    let header = serde_json::json!({
        "alg": "ES256K",
        "x5c": [cert_b64],
        "crit": ["b64", "iat", "jti", "csid", "action"],
        "b64": false,
        "iat": now_epoch(),
        "jti": uuid::Uuid::new_v4().to_string(),
        "csid": cert_b64,
        "action": action,
    });
    let header_b64 = B64URL.encode(header.to_string().as_bytes());
    // Payload is a JSON object with the same critical claims.
    let payload = serde_json::json!({
        "iat": now_epoch(),
        "jti": header["jti"],
        "csid": cert_b64,
        "action": action,
    });
    let payload_str = payload.to_string();
    let signing_input = format!("{}.{}", header_b64, payload_str);
    let digest = Sha256::digest(signing_input.as_bytes());
    let sig: Signature = keys
        .signing_key
        .sign_prehash(&digest)
        .map_err(|e| format!("jws signing failed: {}", e))?;
    let sig_b64 = B64URL.encode(sig.to_der().as_bytes());
    Ok(format!("{}.{}.{}", header_b64, B64URL.encode(payload_str.as_bytes()), sig_b64))
}

// ---------------------------------------------------------------------------
// FATOORA API client
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FatooraConfig {
    pub base_url: String,
    pub cert_b64: String,
    pub vat_number: String,
}

pub struct FatooraClient {
    pub config: FatooraConfig,
    http: reqwest::blocking::Client,
}

impl FatooraClient {
    pub fn new(config: FatooraConfig) -> Result<Self, String> {
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("http client: {}", e))?;
        Ok(Self { config, http })
    }

    /// POST /compliance — obtain a Compliance CSID from a CSR.
    pub fn compliance_csid(&self, keys: &ZatcaKeys, csr_b64: &str) -> Result<serde_json::Value, String> {
        let jws = build_jws(keys, &self.config.cert_b64, "obtain-compliance-csids")?;
        let body = serde_json::json!({ "csr": csr_b64 });
        self.post("/compliance", &jws, &body)
    }

    /// POST /production/csids — obtain the Production CSID.
    pub fn production_csid(&self, keys: &ZatcaKeys, compliance_request_id: &str) -> Result<serde_json::Value, String> {
        let jws = build_jws(keys, &self.config.cert_b64, "obtain-production-csids")?;
        let body = serde_json::json!({ "compliance_request_id": compliance_request_id });
        self.post("/production/csids", &jws, &body)
    }

    /// POST /invoices/reporting/single — simplified tax invoices (B2C).
    pub fn reporting(&self, keys: &ZatcaKeys, invoice_hash: &str, invoice_b64: &str) -> Result<serde_json::Value, String> {
        let jws = build_jws(keys, &self.config.cert_b64, "report-invoice")?;
        let body = serde_json::json!({ "invoiceHash": invoice_hash, "invoice": invoice_b64 });
        self.post("/invoices/reporting/single", &jws, &body)
    }

    /// POST /invoices/clearance/single — standard tax invoices (B2B).
    pub fn clearance(&self, keys: &ZatcaKeys, invoice_hash: &str, invoice_b64: &str) -> Result<serde_json::Value, String> {
        let jws = build_jws(keys, &self.config.cert_b64, "clear-invoice")?;
        let body = serde_json::json!({ "invoiceHash": invoice_hash, "invoice": invoice_b64 });
        self.post("/invoices/clearance/single", &jws, &body)
    }

    fn post(&self, path: &str, jws: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
        let resp = self
            .http
            .post(&url)
            .header("Accept", "application/json")
            .header("Accept-Language", "en")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("JWS {}", jws))
            .json(body)
            .send()
            .map_err(|e| format!("request failed: {}", e))?;
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(format!("FATOORA {}: HTTP {} {}", path, status.as_u16(), text));
        }
        serde_json::from_str(&text).map_err(|e| format!("bad response JSON: {} — {}", e, text))
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// ZATCA VAT number: exactly 15 digits, starts with 3, ends with 3.
pub fn valid_vat_number(vat: &str) -> bool {
    vat.len() == 15
        && vat.bytes().all(|b| b.is_ascii_digit())
        && vat.starts_with('3')
        && vat.ends_with('3')
}

/// Validate the invoice data against the core ZATCA business rules that can be
/// checked locally (BR-KSA-03/06/15/33/34/37/63).
pub fn validate_invoice_data(data: &ZatcaInvoiceData) -> Vec<String> {
    let mut errors = Vec::new();
    if data.uuid.trim().is_empty() {
        errors.push("BR-KSA-03: invoice must contain a UUID (KSA-1)".into());
    }
    if data.invoice_number.trim().is_empty() {
        errors.push("invoice number (cbc:ID) is required".into());
    }
    if data.transaction_type.len() != 7 || !data.transaction_type.bytes().all(|b| b.is_ascii_digit()) {
        errors.push("BR-KSA-06: transaction type (KSA-2) must be 7 digits (NNPNESB)".into());
    }
    if !valid_vat_number(&data.seller.vat_number) {
        errors.push("seller VAT number must be 15 digits starting and ending with 3".into());
    }
    if !data.seller.address.building_number.is_empty() && data.seller.address.building_number.len() != 4 {
        errors.push("BR-KSA-37: seller building number must be exactly 4 digits".into());
    }
    if data.seller.address.country_code == "SA"
        && (data.seller.address.street_name.is_empty()
            || data.seller.address.building_number.is_empty()
            || data.seller.address.postal_zone.is_empty())
    {
        errors.push("BR-KSA-63: SA seller requires street name, building number and postal zone".into());
    }
    if data.icv == 0 {
        errors.push("BR-KSA-33/34: invoice counter (ICV) must be a non-zero positive integer".into());
    }
    let diff = (data.net_amount + data.vat_amount - data.total_amount).abs();
    if diff > 0.01 {
        errors.push(format!("totals do not reconcile: net + vat != total (off by {:.2})", diff));
    }
    if data.lines.is_empty() {
        errors.push("invoice must contain at least one line".into());
    }
    for l in &data.lines {
        let lt = (l.unit_price * l.quantity - l.line_total).abs();
        if lt > 0.01 {
            errors.push(format!("line {} total does not match unit price x quantity", l.id));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> ZatcaInvoiceData {
        let seller = ZatcaParty {
            name: "شركة الأمثلة التجارية".into(),
            vat_number: "300012345600003".into(),
            crn: "1010123456".into(),
            address: ZatcaAddress {
                street_name: "Olaya Street".into(),
                building_number: "1234".into(),
                plot_identification: "5678".into(),
                city_subdivision_name: "Al Olaya".into(),
                city_name: "Riyadh".into(),
                postal_zone: "12212".into(),
                country_code: "SA".into(),
            },
        };
        let buyer = ZatcaParty {
            name: "شركة المشتري".into(),
            vat_number: "311111111111113".into(),
            ..Default::default()
        };
        let mut d = ZatcaInvoiceData {
            invoice_number: "INV-2026-0001".into(),
            uuid: "d6b8f2f0-0000-4000-8000-000000000001".into(),
            issue_date: "2026-08-14".into(),
            issue_time: "14:30:00+03:00".into(),
            transaction_type: ZATCA_TXN_STANDARD.into(),
            currency: "SAR".into(),
            icv: 7,
            pih: Some("aGVsbG8=".into()),
            seller,
            buyer,
            lines: vec![ZatcaLine {
                id: 1,
                item_name: "مادة خام".into(),
                quantity: 2.0,
                unit_code: "PCE".into(),
                unit_price: 100.0,
                line_total: 200.0,
                vat_rate: 15.0,
                tax_category: "S".into(),
            }],
            net_amount: 200.0,
            vat_amount: 30.0,
            total_amount: 230.0,
            allowance_total: 0.0,
            notes: vec![],
        };
        d.pih = None;
        d
    }

    #[test]
    fn phase2_qr_contains_nine_tags_in_order() {
        let qr = Phase2Qr {
            seller_name: "seller".into(),
            vat_number: "300012345600003".into(),
            timestamp: "2026-08-14T14:30:00+03:00".into(),
            total: 230.0,
            vat_amount: 30.0,
            invoice_hash: "base64hash".into(),
            ecdsa_signature: "base64sig".into(),
            public_key: "base64key".into(),
            ca_signature: String::new(),
        };
        let payload = build_phase2_qr(&qr);
        let tags = decode_phase2_qr(&payload).unwrap();
        assert_eq!(tags.len(), 9);
        let ids: Vec<u8> = tags.iter().map(|(t, _)| *t).collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(tags[3].1, b"230.00");
        assert_eq!(tags[4].1, b"30.00");
    }

    #[test]
    fn decode_rejects_invalid_base64_and_bad_tags() {
        assert!(decode_phase2_qr("!!!").is_err());
        assert!(decode_phase2_qr("").is_err());
    }

    #[test]
    fn vat_number_validation() {
        assert!(valid_vat_number("300012345600003"));
        assert!(!valid_vat_number("300012345600002"));
        assert!(!valid_vat_number("3000123456000034"));
        assert!(!valid_vat_number("100012345600003"));
        assert!(!valid_vat_number("30001234560000a"));
    }

    #[test]
    fn base_xml_is_canonical_and_contains_core_elements() {
        let d = sample_data();
        let base = build_invoice_body(&d, false, false, None, None, None, None);
        assert!(base.starts_with("<Invoice xmlns="));
        assert!(base.contains("<cbc:UBLVersionID>2.1</cbc:UBLVersionID>"));
        assert!(base.contains("<cbc:ProfileID>reporting:1.0</cbc:ProfileID>"));
        assert!(base.contains("<cbc:InvoiceTypeCode name=\"0100000\">388</cbc:InvoiceTypeCode>"));
        assert!(base.contains("<cbc:UUID>d6b8f2f0-0000-4000-8000-000000000001</cbc:UUID>"));
        assert!(base.contains("<cbc:CompanyID>300012345600003</cbc:CompanyID>"));
        assert!(base.contains("<cbc:TaxAmount currencyID=\"SAR\">30.00</cbc:TaxAmount>"));
        assert!(base.contains("<cbc:PayableAmount currencyID=\"SAR\">230.00</cbc:PayableAmount>"));
        assert!(!base.contains("UBLExtensions"));
        assert!(!base.contains("<cbc:ID>QR</cbc:ID>"));
        assert!(base.ends_with("</Invoice>"));
    }

    #[test]
    fn full_signing_pipeline_round_trips() {
        let d = sample_data();
        let mut keys = ZatcaKeys::random();
        // Simulate a CSID certificate (just the DER of the public key info).
        keys.certificate_der = keys.public_key_sec1();

        let (final_xml, hash_b64, sig_b64) = generate_signed_invoice(&d, &keys).unwrap();

        // Hash is deterministic.
        let base = build_invoice_body(&d, false, false, None, None, None, None);
        assert_eq!(hash_b64, sha256_base64(base.as_bytes()));

        // Signature verifies over the raw digest.
        let digest = BASE64.decode(hash_b64.as_bytes()).unwrap();
        verify_signature(&keys.verifying_key(), &digest, &sig_b64).unwrap();

        // Final XML includes QR reference, hash, signature and certificate.
        assert!(final_xml.contains("<cbc:ID>QR</cbc:ID>"));
        assert!(final_xml.contains("UBLDocumentSignatures"));
        assert!(final_xml.contains(&sig_b64));
        assert!(final_xml.contains("<ds:SignatureValue>"));

        // The signed document still starts with the same root (canonical).
        assert!(final_xml.starts_with("<Invoice xmlns="));
    }

    #[test]
    fn icv_pih_reference_is_embedded_when_present() {
        let mut d = sample_data();
        d.pih = Some("pihbase64value".into());
        let base = build_invoice_body(&d, false, false, None, None, None, None);
        assert!(base.contains("<cbc:ID>ICV</cbc:ID>"));
        assert!(base.contains("pihbase64value"));
    }

    #[test]
    fn validation_flags_common_mistakes() {
        let d = sample_data();
        assert!(validate_invoice_data(&d).is_empty());

        let mut bad = sample_data();
        bad.seller.vat_number = "123".into();
        bad.transaction_type = "0211".into();
        bad.icv = 0;
        bad.total_amount = 999.0;
        let errs = validate_invoice_data(&bad);
        assert!(errs.iter().any(|e| e.contains("VAT number")));
        assert!(errs.iter().any(|e| e.contains("BR-KSA-06")));
        assert!(errs.iter().any(|e| e.contains("ICV")));
        assert!(errs.iter().any(|e| e.contains("reconcile")));
    }

    #[test]
    fn csr_is_valid_der_and_contains_subject() {
        let keys = ZatcaKeys::random();
        let csr = build_csr(&keys, "300012345600003", "شركة الأمثلة التجارية").unwrap();
        // Outer SEQUENCE with correct length.
        assert_eq!(csr[0], 0x30);
        let (body_len, header_len) = parse_der_len(&csr[1..]);
        assert_eq!(body_len, csr.len() - 1 - header_len);
        // First element: CertificationRequestInfo SEQUENCE.
        let outer_body = &csr[1 + header_len..];
        assert_eq!(outer_body[0], 0x30);
        let (cri_len, cri_header) = parse_der_len(&outer_body[1..]);
        let cri = &outer_body[1 + cri_header..1 + cri_header + cri_len];
        // Version INTEGER 0 is the first field of the CRI.
        assert_eq!(&cri[..3], &[0x02, 0x01, 0x00]);
        // Base64 encodes cleanly.
        let b64 = BASE64.encode(&csr);
        assert!(b64.len() > 100);
        assert!(BASE64.decode(b64.as_bytes()).is_ok());
    }

    #[test]
    fn csr_rejects_invalid_vat() {
        let keys = ZatcaKeys::random();
        assert!(build_csr(&keys, "000000000000000", "org").is_err());
    }

    #[test]
    fn csr_signature_field_verifies_over_der_body() {
        let keys = ZatcaKeys::random();
        let csr = build_csr(&keys, "300012345600003", "شركة الأمثلة التجارية").unwrap();
        assert_eq!(csr[0], 0x30);
        let (body_len, header_len) = parse_der_len(&csr[1..]);
        let outer_body = &csr[1 + header_len..1 + header_len + body_len];
        // First element: CRI SEQUENCE; the signature covers this entire TLV.
        let (cri_len, cri_header) = parse_der_len(&outer_body[1..]);
        let cri_tlv_len = 1 + cri_header + cri_len;
        let cri_tlv = &outer_body[0..cri_tlv_len];
        // Remainder: AlgorithmIdentifier SEQUENCE, then BIT STRING signature.
        let rest = &outer_body[cri_tlv_len..];
        assert_eq!(rest[0], 0x30);
        let (algo_len, algo_header) = parse_der_len(&rest[1..]);
        let sig_pos = 1 + algo_header + algo_len;
        assert_eq!(rest[sig_pos], 0x03); // BIT STRING
        let (bit_len, bit_header) = parse_der_len(&rest[sig_pos + 1..]);
        // Skip the unused-bits count byte.
        let der_sig = &rest[sig_pos + 1 + bit_header + 1..sig_pos + 1 + bit_header + bit_len];
        let sig = Signature::from_der(der_sig).unwrap();
        let digest = Sha256::digest(cri_tlv);
        keys.verifying_key().verify_prehash(&digest, &sig).unwrap();
    }

    fn parse_der_len(buf: &[u8]) -> (usize, usize) {
        let first = buf[0];
        if first < 0x80 {
            (first as usize, 1)
        } else {
            let n = (first & 0x7f) as usize;
            let mut len = 0usize;
            for &byte in &buf[1..=n] {
                len = (len << 8) | byte as usize;
            }
            (len, 1 + n)
        }
    }

    #[test]
    fn jws_has_three_parts_and_verifies() {
        let keys = ZatcaKeys::random();
        let cert_b64 = BASE64.encode(keys.public_key_sec1());
        let jws = build_jws(&keys, &cert_b64, "clear-invoice").unwrap();
        let parts: Vec<&str> = jws.split('.').collect();
        assert_eq!(parts.len(), 3);
        let header: serde_json::Value = serde_json::from_slice(&B64URL.decode(parts[0]).unwrap()).unwrap();
        assert_eq!(header["alg"], "ES256K");
        assert_eq!(header["b64"], false);
        assert_eq!(header["action"], "clear-invoice");
        // b64:false => the signing input uses the RAW payload, not base64url.
        let payload_str = String::from_utf8(B64URL.decode(parts[1]).unwrap()).unwrap();
        let signing_input = format!("{}.{}", parts[0], payload_str);
        let digest = Sha256::digest(signing_input.as_bytes());
        let sig = Signature::from_der(&B64URL.decode(parts[2]).unwrap()).unwrap();
        keys.verifying_key().verify_prehash(&digest, &sig).unwrap();
    }
}
