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
            let mut pin_bytes = [0u8; 4];
            OsRng.fill_bytes(&mut pin_bytes);
            let pin: String = pin_bytes.iter().map(|b| format!("{}", b % 10)).collect();
            hash_developer_pin(&pin)
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
            let mut pin_bytes = [0u8; 4];
            OsRng.fill_bytes(&mut pin_bytes);
            let auto_pin: String = pin_bytes.iter().map(|b| format!("{}", b % 10)).collect();
            let auto_secrets = AppSecrets {
                jwt_secret: generate_machine_secret(),
                licensing_secret: generate_machine_secret(),
                developer_pin_hash: hash_developer_pin(&auto_pin),
                encryption_key: generate_machine_secret(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&auto_secrets) {
                if let Err(e) = std::fs::write(&config_path, json) {
                    eprintln!("Warning: Failed to write secrets file: {}", e);
                }
            }
            if SECRETS.set(auto_secrets.clone()).is_err() {
                return SECRETS.get().cloned().unwrap_or_default();
            }
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
    // Use only machine_id + application salt for stable, non-rotating secrets
    let hash = Sha256::digest(format!("promax-os-v2-{}", machine_id).as_bytes());
    format!("{:x}", hash)
}

fn machine_uid() -> String {
    // Windows: use PowerShell to get machine UUID (wmic is deprecated)
    if cfg!(target_os = "windows") {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-CimInstance Win32_ComputerSystemProduct).UUID"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if !trimmed.is_empty() && trimmed.len() > 5 && trimmed != "UUID" {
                return trimmed.to_string();
            }
        }
    }
    // Fallback: use hostname + username
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".into());
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "user".into());
    format!("{}-{}", hostname, username)
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
    format!("promax_{}_{}_{}_{}_{}", user_id, ts, username, role, hash)
}

pub fn validate_tauri_token(token: &str) -> Result<(i64, String, String), String> {
    if !token.starts_with("promax_") {
        return Err("Invalid token format".to_string());
    }
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 6 {
        return Err("Invalid token structure".to_string());
    }
    let user_id: i64 = parts[1].parse().map_err(|_| "Invalid user ID in token".to_string())?;
    let ts: i64 = parts[2].parse().map_err(|_| "Invalid timestamp in token".to_string())?;
    let username = parts[3].to_string();
    let role = parts[4].to_string();
    let hash = parts[5..].join("_");

    let secrets = get_secrets();
    use sha2::{Digest, Sha256};
    let expected = format!("{:x}", Sha256::digest(
        format!("{}-{}-{}-{}-{}", user_id, username, role, ts, secrets.jwt_secret).as_bytes()
    ));

    if hash != expected {
        return Err("Token signature mismatch".to_string());
    }

    // Check expiry (7 days)
    let now = chrono::Utc::now().timestamp();
    if now - ts > 7 * 86400 {
        return Err("Token expired".to_string());
    }

    Ok((user_id, username, role))
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

use std::collections::HashSet;
use std::sync::Mutex;
static TOKEN_BLACKLIST: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub fn blacklist_token(jti: &str) {
    let list = TOKEN_BLACKLIST.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut guard) = list.lock() {
        guard.insert(jti.to_string());
        // Evict oldest entries when list exceeds 10K
        if guard.len() > 10000 {
            // HashSet doesn't preserve order; just clear half arbitrarily
            let to_remove: Vec<String> = guard.iter().take(5000).cloned().collect();
            for k in &to_remove {
                guard.remove(k);
            }
        }
    }
}

pub fn is_token_blacklisted(jti: &str) -> bool {
    if let Some(l) = TOKEN_BLACKLIST.get() {
        if let Ok(guard) = l.lock() {
            return guard.contains(jti);
        }
    }
    false
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure SECRETS is initialized once for all crypto tests
    fn init_test_secrets() {
        SECRETS.set(AppSecrets {
            jwt_secret: "test-jwt-secret-key-for-unit-tests".into(),
            licensing_secret: "test-licensing-secret".into(),
            developer_pin_hash: hash_developer_pin("1234"),
            encryption_key: "test-encryption-key-32-chars-long!?!".into(),
        }).ok();
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "SuperSecure!123";
        let hash = hash_password(password).expect("hash_password failed");
        assert!(verify_password(password, &hash).expect("verify_password failed"));
        assert!(!verify_password("wrong", &hash).expect("verify_password failed"));
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let h1 = hash_password("test").expect("hash1 failed");
        let h2 = hash_password("test").expect("hash2 failed");
        assert_ne!(h1, h2, "Argon2 should produce different hashes with different salts");
        assert!(verify_password("test", &h1).unwrap());
        assert!(verify_password("test", &h2).unwrap());
    }

    #[test]
    fn test_developer_pin_hash_and_verify() {
        let pin = "1234";
        let hash = hash_developer_pin(pin);
        assert!(verify_password(pin, &hash).unwrap());
        assert!(!verify_password("9999", &hash).unwrap());
    }

    #[test]
    fn test_jwt_create_and_verify() {
        init_test_secrets();
        let token = create_jwt("admin", "admin").expect("JWT creation failed");
        assert!(!token.is_empty());
        let claims = verify_jwt(&token).expect("JWT verification failed");
        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.role, "admin");
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn test_jwt_rejects_wrong_secret() {
        init_test_secrets();
        let token = create_jwt("user", "user").expect("JWT creation failed");
        // Tamper with token to simulate wrong secret
        let tampered = format!("{}.tampered_signature", &token[..token.rfind('.').unwrap()]);
        let result = verify_jwt(&tampered);
        assert!(result.is_err(), "Should reject tampered token");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        init_test_secrets();
        let plaintext = "Hello PRO MAX OS!";
        let encrypted = encrypt_if_needed(plaintext).expect("encrypt_if_needed failed");
        assert!(encrypted.starts_with("gcm:"));
        assert_ne!(encrypted, plaintext, "Encrypted should differ from plaintext");
        let decrypted = decrypt_if_needed(&encrypted).expect("decrypt_if_needed failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let result = encrypt_if_needed("").expect("encrypt empty failed");
        assert_eq!(result, "", "Empty string should pass through unchanged");
    }

    #[test]
    fn test_encrypt_if_needed_and_decrypt_if_needed() {
        init_test_secrets();
        let plaintext = "API_KEY_12345";
        let encrypted = encrypt_if_needed(plaintext).expect("encrypt_if_needed failed");
        assert!(encrypted.starts_with("gcm:"));
        let decrypted = decrypt_if_needed(&encrypted).expect("decrypt_if_needed failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_if_needed_passthrough() {
        let plain = "not_encrypted_value";
        let result = decrypt_if_needed(plain).expect("decrypt_if_needed failed");
        assert_eq!(result, plain, "Non-gcm: values should pass through");
    }

    #[test]
    fn test_token_blacklist() {
        let jti = uuid::Uuid::new_v4().to_string();
        assert!(!is_token_blacklisted(&jti));
        blacklist_token(&jti);
        assert!(is_token_blacklisted(&jti));
    }

    #[test]
    fn test_tauri_token_create_and_validate() {
        init_test_secrets();
        let token = create_tauri_token(42, "testuser", "manager");
        assert!(token.starts_with("promax_"));
        let (user_id, username, role) = validate_tauri_token(&token).expect("validate failed");
        assert_eq!(user_id, 42);
        assert_eq!(username, "testuser");
        assert_eq!(role, "manager");
    }

    #[test]
    fn test_tauri_token_rejects_invalid() {
        assert!(validate_tauri_token("bad_token").is_err());
        assert!(validate_tauri_token("promax_abc_def").is_err());
    }

    #[test]
    fn test_derive_encryption_key_deterministic() {
        let key1 = derive_encryption_key(b"secret123");
        let key2 = derive_encryption_key(b"secret123");
        assert_eq!(key1, key2, "Same input should produce same key");
    }
}
