use crate::errors::ValidationError;
use regex::Regex;
use std::fmt;

pub trait ValidationRule: fmt::Debug + Send + Sync {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct RequiredRule;

impl ValidationRule for RequiredRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError::new(field, "is required", "required"))
        } else {
            Ok(())
        }
    }
    fn name(&self) -> &str { "required" }
}

#[derive(Debug, Clone)]
pub struct MinLengthRule { pub min: usize }

impl MinLengthRule {
    pub fn new(min: usize) -> Self { Self { min } }
}

impl ValidationRule for MinLengthRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if value.len() < self.min {
            Err(ValidationError::new(field, format!("must be at least {} characters", self.min), "min_length"))
        } else { Ok(()) }
    }
    fn name(&self) -> &str { "min_length" }
}

#[derive(Debug, Clone)]
pub struct MaxLengthRule { pub max: usize }

impl MaxLengthRule {
    pub fn new(max: usize) -> Self { Self { max } }
}

impl ValidationRule for MaxLengthRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if value.len() > self.max {
            Err(ValidationError::new(field, format!("must be at most {} characters", self.max), "max_length"))
        } else { Ok(()) }
    }
    fn name(&self) -> &str { "max_length" }
}

#[derive(Debug, Clone)]
pub struct PatternRule {
    pub regex: Regex,
    pub pattern_str: String,
}

impl PatternRule {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        Ok(Self {
            regex,
            pattern_str: pattern.to_string(),
        })
    }
}

impl ValidationRule for PatternRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if !self.regex.is_match(value) {
            Err(ValidationError::new(field, format!("does not match required pattern: {}", self.pattern_str), "pattern"))
        } else { Ok(()) }
    }
    fn name(&self) -> &str { "pattern" }
}

#[derive(Debug, Clone)]
pub struct EmailRule;

impl EmailRule {
    pub fn new() -> Self { Self }
}

impl Default for EmailRule {
    fn default() -> Self { Self::new() }
}

impl ValidationRule for EmailRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_re.is_match(value) {
            Err(ValidationError::new(field, "is not a valid email address", "email"))
        } else { Ok(()) }
    }
    fn name(&self) -> &str { "email" }
}

#[derive(Debug, Clone)]
pub struct UrlRule;

impl UrlRule {
    pub fn new() -> Self { Self }
}

impl Default for UrlRule {
    fn default() -> Self { Self::new() }
}

impl ValidationRule for UrlRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        let url_re = Regex::new(r"^https?://[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(:[0-9]+)?(/[^\s]*)?$").unwrap();
        if !url_re.is_match(value) {
            Err(ValidationError::new(field, "is not a valid URL", "url"))
        } else { Ok(()) }
    }
    fn name(&self) -> &str { "url" }
}

#[derive(Debug, Clone)]
pub struct RangeRule { pub min: i64, pub max: i64 }

impl RangeRule {
    pub fn new(min: i64, max: i64) -> Self { Self { min, max } }
}

impl ValidationRule for RangeRule {
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        match value.parse::<i64>() {
            Ok(num) => {
                if num >= self.min && num <= self.max { Ok(()) }
                else { Err(ValidationError::new(field, format!("must be between {} and {}", self.min, self.max), "range")) }
            }
            Err(_) => Err(ValidationError::new(field, "must be a valid number", "not_a_number")),
        }
    }
    fn name(&self) -> &str { "range" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_rule_valid() {
        let rule = RequiredRule;
        assert!(rule.validate("hello", "name").is_ok());
    }

    #[test]
    fn test_required_rule_empty() {
        let rule = RequiredRule;
        assert!(rule.validate("", "name").is_err());
        assert!(rule.validate("   ", "name").is_err());
    }

    #[test]
    fn test_min_length_rule() {
        let rule = MinLengthRule::new(5);
        assert!(rule.validate("hello", "name").is_ok());
        assert!(rule.validate("hi", "name").is_err());
    }

    #[test]
    fn test_max_length_rule() {
        let rule = MaxLengthRule::new(5);
        assert!(rule.validate("hello", "name").is_ok());
        assert!(rule.validate("hello world", "name").is_err());
    }

    #[test]
    fn test_pattern_rule() {
        let rule = PatternRule::new(r"^\d{3}-\d{4}$").unwrap();
        assert!(rule.validate("123-4567", "phone").is_ok());
        assert!(rule.validate("abc", "phone").is_err());
    }

    #[test]
    fn test_email_rule_valid() {
        let rule = EmailRule::new();
        assert!(rule.validate("user@example.com", "email").is_ok());
        assert!(rule.validate("john.doe+tag@example.co.uk", "email").is_ok());
    }

    #[test]
    fn test_email_rule_invalid() {
        let rule = EmailRule::new();
        assert!(rule.validate("not-an-email", "email").is_err());
        assert!(rule.validate("@example.com", "email").is_err());
        assert!(rule.validate("user@", "email").is_err());
    }

    #[test]
    fn test_url_rule_valid() {
        let rule = UrlRule::new();
        assert!(rule.validate("https://example.com", "url").is_ok());
        assert!(rule.validate("http://example.com/path", "url").is_ok());
        assert!(rule.validate("https://example.com:8080/path", "url").is_ok());
    }

    #[test]
    fn test_url_rule_invalid() {
        let rule = UrlRule::new();
        assert!(rule.validate("not a url", "url").is_err());
        assert!(rule.validate("example.com", "url").is_err());
    }

    #[test]
    fn test_range_rule_valid() {
        let rule = RangeRule::new(1, 100);
        assert!(rule.validate("50", "age").is_ok());
        assert!(rule.validate("1", "age").is_ok());
        assert!(rule.validate("100", "age").is_ok());
    }

    #[test]
    fn test_range_rule_invalid() {
        let rule = RangeRule::new(1, 100);
        assert!(rule.validate("0", "age").is_err());
        assert!(rule.validate("101", "age").is_err());
        assert!(rule.validate("abc", "age").is_err());
    }
}
