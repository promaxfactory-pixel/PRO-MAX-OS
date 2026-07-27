use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(rusqlite::Error),
    Lock(String),
    Auth(String),
    Validation(String),
    NotFound(String),
    Permission(String),
    BusinessLogic(String),
    Config(String),
    Crypto(String),
    Io(std::io::Error),
    Json(String),
    Migration(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Lock(e) => write!(f, "Lock error: {}", e),
            AppError::Auth(e) => write!(f, "Authentication error: {}", e),
            AppError::Validation(e) => write!(f, "Validation error: {}", e),
            AppError::NotFound(e) => write!(f, "Not found: {}", e),
            AppError::Permission(e) => write!(f, "Permission denied: {}", e),
            AppError::BusinessLogic(e) => write!(f, "Business rule violation: {}", e),
            AppError::Config(e) => write!(f, "Configuration error: {}", e),
            AppError::Crypto(e) => write!(f, "Encryption error: {}", e),
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::Migration(e) => write!(f, "Migration error: {}", e),
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self { AppError::Database(e) }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e) }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self { AppError::Json(e.to_string()) }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, rusqlite::Connection>>> for AppError {
    fn from(e: std::sync::PoisonError<std::sync::MutexGuard<'_, rusqlite::Connection>>) -> Self {
        AppError::Lock(format!("Database lock poisoned: {}", e))
    }
}

impl From<String> for AppError {
    fn from(e: String) -> Self { AppError::BusinessLogic(e) }
}

impl From<&str> for AppError {
    fn from(e: &str) -> Self { AppError::BusinessLogic(e.to_string()) }
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self { AppError::Validation(msg.into()) }
    pub fn not_found(msg: impl Into<String>) -> Self { AppError::NotFound(msg.into()) }
    pub fn auth(msg: impl Into<String>) -> Self { AppError::Auth(msg.into()) }
    pub fn permission(msg: impl Into<String>) -> Self { AppError::Permission(msg.into()) }
    pub fn business(msg: impl Into<String>) -> Self { AppError::BusinessLogic(msg.into()) }
    pub fn config(msg: impl Into<String>) -> Self { AppError::Config(msg.into()) }
    pub fn lock(msg: impl Into<String>) -> Self { AppError::Lock(msg.into()) }
    pub fn crypto(msg: impl Into<String>) -> Self { AppError::Crypto(msg.into()) }
    pub fn migration(msg: impl Into<String>) -> Self { AppError::Migration(msg.into()) }
}
