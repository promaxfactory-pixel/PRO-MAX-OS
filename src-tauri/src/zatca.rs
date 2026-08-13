use crate::error::AppError;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use qrcode::QrCode;
use std::io::Cursor;

/// ZATCA (Simplified Tax Invoice) QR fields.
///
/// Tag 1: Seller's name
/// Tag 2: VAT registration number
/// Tag 3: Invoice time stamp (ISO 8601)
/// Tag 4: Invoice total (with VAT)
/// Tag 5: VAT amount
pub struct ZatcaQrFields<'a> {
    pub seller_name: &'a str,
    pub vat_number: &'a str,
    pub timestamp: &'a str,
    pub total_units: f64,
    pub vat_units: f64,
}

/// Encode a single TLV record (tag, length, value). Length is single-byte
/// per the ZATCA QR spec (all fields fit well under 255 bytes).
pub fn tlv_encode(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + value.len());
    out.push(tag);
    out.push(value.len() as u8);
    out.extend_from_slice(value);
    out
}

/// Build the base64 QR content per ZATCA Simplified Tax Invoice spec.
pub fn build_zatca_payload(f: &ZatcaQrFields) -> String {
    let mut raw = Vec::new();
    raw.extend(tlv_encode(1, f.seller_name.as_bytes()));
    raw.extend(tlv_encode(2, f.vat_number.as_bytes()));
    raw.extend(tlv_encode(3, f.timestamp.as_bytes()));
    raw.extend(tlv_encode(4, format!("{:.2}", f.total_units).as_bytes()));
    raw.extend(tlv_encode(5, format!("{:.2}", f.vat_units).as_bytes()));
    BASE64.encode(&raw)
}

/// Render the payload as a PNG QR code and return a `data:image/png;base64,...` URL.
pub fn qr_png_data_url(payload: &str) -> Result<String, AppError> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|e| AppError::business(format!("QR generation failed: {}", e)))?;
    let dims = code.width();
    let scale = 8usize;
    let mut img = image::RgbImage::from_pixel(
        (dims * scale) as u32,
        (dims * scale) as u32,
        image::Rgb([255, 255, 255]),
    );
    for (y, row) in code.to_colors().chunks(dims).enumerate() {
        for (x, color) in row.iter().enumerate() {
            if *color == qrcode::Color::Dark {
                for dy in 0..scale {
                    for dx in 0..scale {
                        img.put_pixel((x * scale + dx) as u32, (y * scale + dy) as u32, image::Rgb([0, 0, 0]));
                    }
                }
            }
        }
    }
    let mut png: Vec<u8> = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png);
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| AppError::business(format!("QR PNG encode failed: {}", e)))?;
    }
    Ok(format!("data:image/png;base64,{}", BASE64.encode(&png)))
}

/// Convert a stored date/created_at into an ISO 8601 timestamp suitable for ZATCA.
pub fn to_iso8601(date: &str, created_at: &Option<String>) -> String {
    if let Some(ca) = created_at {
        let norm = ca.trim();
        if !norm.is_empty() {
            if norm.contains('T') {
                return norm.to_string();
            }
            return norm.replace(' ', "T");
        }
    }
    format!("{}T12:00:00", date.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tlv_encode_produces_tag_length_value() {
        let enc = tlv_encode(1, b"Test Seller");
        assert_eq!(enc[0], 1);
        assert_eq!(enc[1], 11);
        assert_eq!(&enc[2..], b"Test Seller");
    }

    #[test]
    fn payload_contains_all_five_zatca_tags_in_order() {
        let f = ZatcaQrFields {
            seller_name: "Test Seller",
            vat_number: "123456789012345",
            timestamp: "2022-04-25T14:30:00Z",
            total_units: 200.0,
            vat_units: 33.33,
        };
        let payload = build_zatca_payload(&f);
        let raw = BASE64.decode(payload.as_bytes()).expect("valid base64");

        let mut tags = Vec::new();
        let mut i = 0usize;
        while i + 2 <= raw.len() {
            let tag = raw[i];
            let len = raw[i + 1] as usize;
            let value = &raw[i + 2..i + 2 + len];
            tags.push((tag, value.to_vec()));
            i += 2 + len;
        }
        assert_eq!(tags.len(), 5);
        assert_eq!(tags[0].0, 1);
        assert_eq!(tags[0].1, b"Test Seller");
        assert_eq!(tags[1].0, 2);
        assert_eq!(tags[1].1, b"123456789012345");
        assert_eq!(tags[2].0, 3);
        assert_eq!(tags[2].1, b"2022-04-25T14:30:00Z");
        assert_eq!(tags[3].0, 4);
        assert_eq!(tags[3].1, b"200.00");
        assert_eq!(tags[4].0, 5);
        assert_eq!(tags[4].1, b"33.33");
    }

    #[test]
    fn qr_png_data_url_is_valid_png() {
        let payload = build_zatca_payload(&ZatcaQrFields {
            seller_name: "شركة التجربة",
            vat_number: "300012345600003",
            timestamp: "2026-08-13T12:00:00",
            total_units: 125.5,
            vat_units: 5.98,
        });
        let url = qr_png_data_url(&payload).expect("qr renders");
        assert!(url.starts_with("data:image/png;base64,"));
        let b64 = &url["data:image/png;base64,".len()..];
        let png = BASE64.decode(b64).expect("png base64 decodes");
        assert_eq!(&png[0..4], b"\x89PNG");
    }

    #[test]
    fn to_iso8601_handles_common_formats() {
        assert_eq!(&to_iso8601("2026-08-13", &None), "2026-08-13T12:00:00");
        assert_eq!(
            &to_iso8601("2026-08-13", &Some("2026-08-13 09:45:30".into())),
            "2026-08-13T09:45:30"
        );
        assert_eq!(
            &to_iso8601("2026-08-13", &Some("2026-08-13T09:45:30".into())),
            "2026-08-13T09:45:30"
        );
    }
}
