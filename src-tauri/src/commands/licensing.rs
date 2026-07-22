use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseData {
    pub customer_name: String,
    pub license_type: String,
    pub expires_at: Option<String>,
    pub hardware_id: String,
    pub features: Vec<String>,
    pub max_users: i32,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub valid: bool,
    pub message: String,
    pub license: Option<LicenseInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub customer_name: String,
    pub license_type: String,
    pub expires_at: Option<String>,
    pub features: Vec<String>,
    pub max_users: i32,
    pub days_remaining: Option<i64>,
}

fn get_license_path() -> PathBuf {
    let data_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap_or(std::path::Path::new(".")).to_path_buf())
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    data_dir.join(".promax_os_license")
}

fn get_hardware_id() -> String {
    let mut parts = Vec::new();
    if let Ok(computer_name) = std::env::var("COMPUTERNAME") {
        parts.push(computer_name);
    }
    if let Ok(output) = std::process::Command::new("wmic")
        .args(["csproduct", "get", "uuid"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.len() == 36 && trimmed.chars().filter(|&c| c == '-').count() == 4 {
                parts.push(trimmed.to_string());
                break;
            }
        }
    }
    let combined = parts.join("|");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn secret_key() -> String {
    crate::crypto::get_secrets().licensing_secret
}

fn compute_signature(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.update(secret_key().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_license(
    customer_name: &str,
    license_type: &str,
    expires_at: Option<&str>,
    hardware_id: &str,
    features: Vec<String>,
    max_users: i32,
) -> String {
    let data = format!("{}|{}|{}|{}|{}|{}|{}",
        customer_name, license_type, expires_at.unwrap_or(""), hardware_id, features.join(","), max_users, secret_key());
    let signature = compute_signature(&data);
    let license = LicenseData {
        customer_name: customer_name.to_string(),
        license_type: license_type.to_string(),
        expires_at: expires_at.map(|s| s.to_string()),
        hardware_id: hardware_id.to_string(),
        features,
        max_users,
        signature,
    };
    let json = serde_json::to_string(&license).unwrap_or_default();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, json.as_bytes())
}

fn validate_license_data(license: &LicenseData, current_hardware_id: &str) -> Result<(), String> {
    let expected_sig = {
        let data = format!("{}|{}|{}|{}|{}|{}|{}",
            license.customer_name, license.license_type,
            license.expires_at.as_deref().unwrap_or(""),
            license.hardware_id, license.features.join(","), license.max_users, secret_key());
        compute_signature(&data)
    };
    if license.signature != expected_sig {
        return Err("توقيع الترخيص غير صالح".to_string());
    }
    if license.hardware_id != "ANY" && license.hardware_id != current_hardware_id {
        return Err("هذا الترخيص مربوط بجهاز آخر".to_string());
    }
    if let Some(expires) = &license.expires_at {
        if let Ok(exp_date) = chrono::NaiveDate::parse_from_str(expires, "%Y-%m-%d") {
            let today = chrono::Local::now().date_naive();
            if today > exp_date {
                return Err("انتهت صلاحية الترخيص".to_string());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn check_license() -> LicenseStatus {
    let path = get_license_path();
    if !path.exists() {
        return LicenseStatus {
            valid: false,
            message: "لم يتم تنشيط البرنامج. يرجى إدخال مفتاح التفعيل.".to_string(),
            license: None,
        };
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => return LicenseStatus {
            valid: false,
            message: format!("خطأ في قراءة ملف الترخيص: {}", e),
            license: None,
        },
    };
    let json = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, content.trim()) {
        Ok(b) => b,
        Err(_) => return LicenseStatus {
            valid: false,
            message: "ملف الترخيص تالف".to_string(),
            license: None,
        },
    };
    let license: LicenseData = match serde_json::from_slice(&json) {
        Ok(l) => l,
        Err(_) => return LicenseStatus {
            valid: false,
            message: "ملف الترخيص غير صالح".to_string(),
            license: None,
        },
    };
    let current_hw_id = get_hardware_id();
    match validate_license_data(&license, &current_hw_id) {
        Ok(()) => {
            let days_remaining = license.expires_at.as_ref().and_then(|exp| {
                chrono::NaiveDate::parse_from_str(exp, "%Y-%m-%d").ok().map(|d| {
                    (d - chrono::Local::now().date_naive()).num_days()
                })
            });
            LicenseStatus {
                valid: true,
                message: "الترخيص ساري المفعول".to_string(),
                license: Some(LicenseInfo {
                    customer_name: license.customer_name,
                    license_type: license.license_type,
                    expires_at: license.expires_at,
                    features: license.features,
                    max_users: license.max_users,
                    days_remaining,
                }),
            }
        }
        Err(msg) => LicenseStatus {
            valid: false,
            message: msg,
            license: None,
        },
    }
}

#[tauri::command]
pub fn activate_license(license_key: String) -> LicenseStatus {
    let json = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &license_key) {
        Ok(b) => b,
        Err(_) => return LicenseStatus {
            valid: false,
            message: "مفتاح التفعيل غير صالح (تنسيق خاطئ)".to_string(),
            license: None,
        },
    };
    let license: LicenseData = match serde_json::from_slice(&json) {
        Ok(l) => l,
        Err(_) => return LicenseStatus {
            valid: false,
            message: "مفتاح التفعيل غير صالح (بيانات غير صحيحة)".to_string(),
            license: None,
        },
    };
    let current_hw_id = get_hardware_id();
    if let Err(msg) = validate_license_data(&license, &current_hw_id) {
        return LicenseStatus {
            valid: false,
            message: msg,
            license: None,
        };
    }
    let path = get_license_path();
    if let Err(e) = std::fs::write(&path, license_key) {
        return LicenseStatus {
            valid: false,
            message: format!("خطأ في حفظ الترخيص: {}", e),
            license: None,
        };
    }
    let days_remaining = license.expires_at.as_ref().and_then(|exp| {
        chrono::NaiveDate::parse_from_str(exp, "%Y-%m-%d").ok().map(|d| {
            (d - chrono::Local::now().date_naive()).num_days()
        })
    });
    LicenseStatus {
        valid: true,
        message: "تم تنشيط البرنامج بنجاح".to_string(),
        license: Some(LicenseInfo {
            customer_name: license.customer_name,
            license_type: license.license_type,
            expires_at: license.expires_at,
            features: license.features,
            max_users: license.max_users,
            days_remaining,
        }),
    }
}

#[tauri::command]
pub fn get_license_info() -> LicenseStatus {
    check_license()
}

#[tauri::command]
pub fn deactivate_license() -> LicenseStatus {
    let path = get_license_path();
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }
    LicenseStatus {
        valid: false,
        message: "تم إلغاء تنشيط الترخيص".to_string(),
        license: None,
    }
}

#[allow(dead_code)]
pub fn require_license() -> bool {
    check_license().valid
}

#[tauri::command]
pub fn verify_developer_pin(pin: String) -> bool {
    crate::crypto::verify_developer_pin(&pin)
}

#[tauri::command]
pub fn generate_license_key(
    pin: String,
    customer_name: String,
    license_type: String,
    expires_days: Option<i64>,
    max_users: i32,
    features: Vec<String>,
) -> Result<String, String> {
    if !crate::crypto::verify_developer_pin(&pin) {
        return Err("رقم التعريف غير صحيح".to_string());
    }
    let expires_at = expires_days.map(|days| {
        let date = chrono::Local::now().date_naive() + chrono::Duration::days(days);
        date.format("%Y-%m-%d").to_string()
    });
    Ok(generate_license(
        &customer_name,
        &license_type,
        expires_at.as_deref(),
        "ANY",
        features,
        max_users,
    ))
}
