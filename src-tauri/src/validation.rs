/// PRO MAX OS - Input Validation Module
/// Shared validation functions for all command modules.

pub struct Validator;

impl Validator {
    pub fn required(name: &str, value: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            return Err(format!("{} is required", name));
        }
        Ok(())
    }

    pub fn min_length(name: &str, value: &str, min: usize) -> Result<(), String> {
        if value.len() < min {
            return Err(format!("{} must be at least {} characters", name, min));
        }
        Ok(())
    }

    pub fn max_length(name: &str, value: &str, max: usize) -> Result<(), String> {
        if value.len() > max {
            return Err(format!("{} must be at most {} characters", name, max));
        }
        Ok(())
    }

    pub fn positive(name: &str, value: i64) -> Result<(), String> {
        if value < 0 {
            return Err(format!("{} must be non-negative", name));
        }
        Ok(())
    }

    pub fn non_zero(name: &str, value: i64) -> Result<(), String> {
        if value == 0 {
            return Err(format!("{} cannot be zero", name));
        }
        Ok(())
    }

    pub fn range(name: &str, value: i64, min: i64, max: i64) -> Result<(), String> {
        if value < min || value > max {
            return Err(format!("{} must be between {} and {}", name, min, max));
        }
        Ok(())
    }

    pub fn one_of(name: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
        if !allowed.contains(&value) {
            return Err(format!("{} must be one of: {}", name, allowed.join(", ")));
        }
        Ok(())
    }

    pub fn email(value: &str) -> Result<(), String> {
        if !value.is_empty() && !value.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }

    pub fn phone(value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Ok(());
        }
        let cleaned: String = value.chars().filter(|c| c.is_ascii_digit() || *c == '+' || *c == '-' || *c == ' ').collect();
        if cleaned.len() < 7 {
            return Err("Phone number is too short".to_string());
        }
        Ok(())
    }

    pub fn date(value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Ok(());
        }
        if value.len() != 10 || !value.chars().nth(4).map_or(false, |c| c == '-')
            || !value.chars().nth(7).map_or(false, |c| c == '-') {
            return Err("Date must be in YYYY-MM-DD format".to_string());
        }
        Ok(())
    }

    pub fn doc_number(value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err("Document number is required".to_string());
        }
        if value.len() > 50 {
            return Err("Document number is too long".to_string());
        }
        Ok(())
    }

    pub fn valid_status(value: &str, valid: &[&str]) -> Result<(), String> {
        if !valid.contains(&value) {
            return Err(format!("Invalid status '{}'. Must be one of: {}", value, valid.join(", ")));
        }
        Ok(())
    }

    pub fn milli_amount(name: &str, value: i64) -> Result<(), String> {
        if value < 0 {
            return Err(format!("{} cannot be negative", name));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required() {
        assert!(Validator::required("name", "").is_err());
        assert!(Validator::required("name", "  ").is_err());
        assert!(Validator::required("name", "test").is_ok());
    }

    #[test]
    fn test_positive() {
        assert!(Validator::positive("amount", -1).is_err());
        assert!(Validator::positive("amount", 0).is_ok());
        assert!(Validator::positive("amount", 100).is_ok());
    }

    #[test]
    fn test_one_of() {
        assert!(Validator::one_of("status", "draft", &["draft", "posted"]).is_ok());
        assert!(Validator::one_of("status", "invalid", &["draft", "posted"]).is_err());
    }

    #[test]
    fn test_email() {
        assert!(Validator::email("test@example.com").is_ok());
        assert!(Validator::email("invalid").is_err());
        assert!(Validator::email("").is_ok());
    }

    #[test]
    fn test_date() {
        assert!(Validator::date("2026-01-15").is_ok());
        assert!(Validator::date("2026/01/15").is_err());
        assert!(Validator::date("bad").is_err());
        assert!(Validator::date("").is_ok());
    }

    #[test]
    fn test_range() {
        assert!(Validator::range("qty", 5, 1, 100).is_ok());
        assert!(Validator::range("qty", 0, 1, 100).is_err());
        assert!(Validator::range("qty", 101, 1, 100).is_err());
    }

    #[test]
    fn test_doc_number() {
        assert!(Validator::doc_number("INV-001").is_ok());
        assert!(Validator::doc_number("").is_err());
    }
}
