/// PRO MAX OS - Cryptographic & Security Module
/// Argon2 password hashing, JWT tokens, secrets management, AES-256-GCM encryption.

use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

// ─── Secrets Management ───────────────────────────────────────────────
// Secrets are stored in a JSON config file next to the database.
// They can also be overridden via environment variables.
// This avoids hardcoding any secrets in the binary.

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSecrets {
    pub jwt_secret: String,
    pub licensing_secret: String,
    pub developer_pin_hash: String,
    pub encryption_key: String,
}

impl Default for AppSecrets {
    fn default() -> Self {
        Self {
            jwt_secret: generate_machine_secret(),
            licensing_secret: generate_machine_secret(),
            developer_pin_hash: hash_developer_pin(&generate_machine_secret()[..6]),
            encryption_key: generate_machine_secret(),
        }
    }
}

static SECRETS: OnceLock<AppSecrets> = OnceLock::new();

pub fn init_secrets(db_path: &PathBuf) -> AppSecrets {
    if let Some(s) = SECRETS.get() {
        return s.clone();
    }

    // Try env vars first (override)
    let secrets = AppSecrets {
        jwt_secret: std::env::var("PROMAX_JWT_SECRET").unwrap_or_else(|_| {
            std::env::var("PROMAX_SECRET_KEY").unwrap_or_else(|_| {
                // Auto-generate a secret from machine-specific data
                generate_machine_secret()
            })
        }),
        licensing_secret: std::env::var("PROMAX_LICENSE_SECRET").unwrap_or_else(|_| {
            std::env::var("PROMAX_SECRET_KEY").unwrap_or_else(|_| {
                generate_machine_secret()
            })
        }),
        developer_pin_hash: std::env::var("PROMAX_DEV_PIN_HASH").unwrap_or_else(|_| {
            hash_developer_pin(&generate_machine_secret()[..6])
        }),
        encryption_key: std::env::var("PROMAX_ENC_KEY").unwrap_or_else(|_| {
            generate_machine_secret()
        }),
    };

    // Try to load from config file next to DB
    if let Some(parent) = db_path.parent() {
        let config_path = parent.join("promax.secrets.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(file_secrets) = serde_json::from_str::<AppSecrets>(&content) {
                    let merged = AppSecrets {
                        jwt_secret: if secrets.jwt_secret == AppSecrets::default().jwt_secret
                            || std::env::var("PROMAX_JWT_SECRET").is_ok() { secrets.jwt_secret } else { file_secrets.jwt_secret },
                        licensing_secret: if secrets.licensing_secret == AppSecrets::default().licensing_secret
                            || std::env::var("PROMAX_LICENSE_SECRET").is_ok() { secrets.licensing_secret } else { file_secrets.licensing_secret },
                        developer_pin_hash: if secrets.developer_pin_hash == AppSecrets::default().developer_pin_hash
                            || std::env::var("PROMAX_DEV_PIN_HASH").is_ok() { secrets.developer_pin_hash } else { file_secrets.developer_pin_hash },
                        encryption_key: if secrets.encryption_key == AppSecrets::default().encryption_key
                            || std::env::var("PROMAX_ENC_KEY").is_ok() { secrets.encryption_key } else { file_secrets.encryption_key },
                    };
                    SECRETS.set(merged.clone()).ok();
                    return merged;
                }
            }
        }

        // Auto-generate secrets file if it doesn't exist
        if !config_path.exists() {
            let auto_secrets = AppSecrets {
                jwt_secret: generate_machine_secret(),
                licensing_secret: generate_machine_secret(),
                developer_pin_hash: hash_developer_pin("1234"),
                encryption_key: generate_machine_secret(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&auto_secrets) {
                std::fs::write(&config_path, json).ok();
            }
            SECRETS.set(auto_secrets.clone()).ok();
            return auto_secrets;
        }
    }

    SECRETS.set(secrets.clone()).ok();
    secrets
}

pub fn get_secrets() -> AppSecrets {
    SECRETS.get().cloned().unwrap_or_default()
}

fn generate_machine_secret() -> String {
    use sha2::{Digest, Sha256};
    let machine_id = machine_uid();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let hash = Sha256::digest(format!("promax-{}-{}", machine_id, now / 86400).as_bytes());
    format!("{:x}", hash)
}

fn machine_uid() -> String {
    // Works on Windows via wmic
    if cfg!(target_os = "windows") {
        if let Ok(output) = std::process::Command::new("wmic")
            .args(["csproduct", "get", "uuid"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed != "UUID" && trimmed.len() > 5 {
                    return trimmed.to_string();
                }
            }
        }
    }
    "unknown-machine".into()
}

// ─── Secure Password Hashing (Argon2id) ──────────────────────────────

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Password hashing failed: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| format!("Invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// ─── Developer PIN (Argon2) ──────────────────────────────────────────

pub fn hash_developer_pin(pin: &str) -> String {
    hash_password(pin).unwrap_or_else(|_| "INVALID_HASH".into())
}

pub fn verify_developer_pin(pin: &str) -> bool {
    let secrets = get_secrets();
    verify_password(pin, &secrets.developer_pin_hash).unwrap_or(false)
}

// ─── JWT Tokens (for API Server) ─────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // username
    pub role: String,
    pub exp: usize,        // expiry timestamp
    pub iat: usize,        // issued at
    pub jti: String,       // unique token ID (for revocation)
}

pub fn create_jwt(username: &str, role: &str) -> Result<String, String> {
    use jsonwebtoken::{encode, Header, EncodingKey};
    let secrets = get_secrets();
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        exp: now + 86400, // 24 hours
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secrets.jwt_secret.as_bytes()))
        .map_err(|e| format!("JWT creation failed: {}", e))
}

pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    let secrets = get_secrets();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60;
    validation.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(secrets.jwt_secret.as_bytes()), &validation)
        .map(|data| data.claims)
        .map_err(|e| format!("JWT verification failed: {}", e))
}

// ─── Tauri Token (for desktop app) ───────────────────────────────────

pub fn create_tauri_token(user_id: i64, username: &str, role: &str) -> String {
    use sha2::{Digest, Sha256};
    let secrets = get_secrets();
    let ts = chrono::Utc::now().timestamp();
    let payload = format!("{}-{}-{}-{}-{}", user_id, username, role, ts, secrets.jwt_secret);
    let hash = format!("{:x}", Sha256::digest(payload.as_bytes()));
    format!("promax_{}_{}_{}_{}", user_id, ts, role, hash)
}

pub fn validate_tauri_token(token: &str) -> Result<(i64, String, String), String> {
    if !token.starts_with("promax_") {
        return Err("Invalid token format".to_string());
    }
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 5 {
        return Err("Invalid token structure".to_string());
    }
    let user_id: i64 = parts[1].parse().map_err(|_| "Invalid user ID in token".to_string())?;
    let ts: i64 = parts[2].parse().map_err(|_| "Invalid timestamp in token".to_string())?;
    let role = parts[3].to_string();
    let hash = parts[4..].join("_");

    let secrets = get_secrets();
    use sha2::{Digest, Sha256};
    let expected = format!("{:x}", Sha256::digest(
        format!("{}-{}-{}-{}-{}", user_id, parts[3], role, ts, secrets.jwt_secret).as_bytes()
    ));

    if hash != expected {
        return Err("Token signature mismatch".to_string());
    }

    // Check expiry (7 days)
    let now = chrono::Utc::now().timestamp();
    if now - ts > 7 * 86400 {
        return Err("Token expired".to_string());
    }

    Ok((user_id, parts[3].to_string(), role))
}

// ─── AES-256-GCM Encryption at Rest ──────────────────────────────────

fn derive_encryption_key(secret: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(secret);
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

pub fn encrypt_value(plaintext: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(plaintext.to_string());
    }
    let secrets = get_secrets();
    let key_bytes = derive_encryption_key(secrets.encryption_key.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    let mut output = nonce_bytes.to_vec();
    output.extend_from_slice(&ciphertext);
    Ok(B64.encode(&output))
}

pub fn decrypt_value(encoded: &str) -> Result<String, String> {
    if encoded.is_empty() || !encoded.starts_with("gcm:") {
        return Ok(encoded.to_string());
    }
    let raw = encoded.strip_prefix("gcm:").unwrap_or(encoded);
    let data = B64.decode(raw).map_err(|e| format!("Invalid encrypted data: {}", e))?;
    if data.len() < 12 {
        return Err("Invalid encrypted data length".to_string());
    }
    let secrets = get_secrets();
    let key_bytes = derive_encryption_key(secrets.encryption_key.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher.decrypt(nonce, &data[12..])
        .map_err(|e| format!("Decryption failed (wrong key or corrupted data): {}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("Decrypted data is not valid UTF-8: {}", e))
}

pub fn encrypt_if_needed(plaintext: &str) -> Result<String, String> {
    if plaintext.starts_with("gcm:") || plaintext.is_empty() {
        return Ok(plaintext.to_string());
    }
    let encrypted = encrypt_value(plaintext)?;
    Ok(format!("gcm:{}", encrypted))
}

pub fn decrypt_if_needed(value: &str) -> Result<String, String> {
    if value.starts_with("gcm:") {
        decrypt_value(value)
    } else {
        Ok(value.to_string())
    }
}

// ─── Token Blacklist (for revocation) ─────────────────────────────────

use std::sync::Mutex;
static TOKEN_BLACKLIST: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn blacklist_token(jti: &str) {
    let list = TOKEN_BLACKLIST.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut guard) = list.lock() {
        guard.push(jti.to_string());
        // Evict oldest entries (FIFO) when list exceeds 10K, keep last 5K
        if guard.len() > 10000 {
            let drain_count = guard.len() - 5000;
            guard.drain(..drain_count);
        }
    }
}

pub fn is_token_blacklisted(jti: &str) -> bool {
    let list = TOKEN_BLACKLIST.get();
    if let Some(l) = list {
        if let Ok(guard) = l.lock() {
            return guard.contains(&jti.to_string());
        }
    }
    false
}
