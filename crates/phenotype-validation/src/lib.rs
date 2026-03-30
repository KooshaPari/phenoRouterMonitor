//! # Phenotype Validation
//!
//! Comprehensive validation framework for phenotype applications.
//!
//! Features:
//! - Unified error collection and reporting
//! - Common validators (required, email, URL, length, pattern, range)
//! - Custom validator trait for extensibility
//! - Composable validation chains
//! - Trait-based validation for domain types

use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;
use thiserror::Error;

// Pre-compiled regexes for performance
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://([a-zA-Z0-9-]+\.)*[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(:[0-9]{1,5})?(/.*)?$")
        .unwrap()
});

static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

static ALPHANUMERIC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

static SLUG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap());

/// Validation error for a single field
#[derive(Debug, Clone, Error)]
#[error("{field}: {message} (code: {code})")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl ValidationError {
    pub fn new(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: code.into(),
        }
    }
}

/// Collection of validation errors
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_if_err(&mut self, result: Result<(), ValidationError>) {
        if let Err(e) = result {
            self.errors.push(e);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn merge(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    pub fn into_result(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidationError> {
        self.errors.iter()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// Trait for types that can be validated
pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Custom validator trait for composable validation chains
pub trait FieldValidator {
    /// Validate a value and return a ValidationError if invalid
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError>;
}

/// Builder for composable validation chains
pub struct ValidationChain {
    validators: Vec<Box<dyn Fn(&str, &str) -> Result<(), ValidationError>>>,
}

impl ValidationChain {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a custom validator to the chain
    pub fn add<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str, &str) -> Result<(), ValidationError> + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Validate a value with all validators in the chain
    pub fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        for validator in &self.validators {
            validator(value, field)?;
        }
        Ok(())
    }
}

impl Default for ValidationChain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Basic String Validators
// ============================================================================

/// Validate that a field is not empty
pub fn required(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new(field, "is required", "required"))
    } else {
        Ok(())
    }
}

/// Validate minimum string length
pub fn min_length(value: &str, min: usize, field: &str) -> Result<(), ValidationError> {
    if value.len() < min {
        Err(ValidationError::new(
            field,
            format!("must be at least {} characters", min),
            "min_length",
        ))
    } else {
        Ok(())
    }
}

/// Validate maximum string length
pub fn max_length(value: &str, max: usize, field: &str) -> Result<(), ValidationError> {
    if value.len() > max {
        Err(ValidationError::new(
            field,
            format!("must be at most {} characters", max),
            "max_length",
        ))
    } else {
        Ok(())
    }
}

/// Validate string is within length range (inclusive)
pub fn length_range(
    value: &str,
    min: usize,
    max: usize,
    field: &str,
) -> Result<(), ValidationError> {
    let len = value.len();
    if len < min || len > max {
        Err(ValidationError::new(
            field,
            format!("must be between {} and {} characters", min, max),
            "length_range",
        ))
    } else {
        Ok(())
    }
}

/// Validate against a regex pattern
pub fn pattern(value: &str, regex: &Regex, field: &str) -> Result<(), ValidationError> {
    if !regex.is_match(value) {
        Err(ValidationError::new(
            field,
            "does not match required pattern",
            "pattern",
        ))
    } else {
        Ok(())
    }
}

/// Validate numeric value is within range (inclusive)
pub fn range<T: PartialOrd + fmt::Display>(
    value: T,
    min: T,
    max: T,
    field: &str,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        Err(ValidationError::new(
            field,
            format!("must be between {} and {}", min, max),
            "range",
        ))
    } else {
        Ok(())
    }
}

// ============================================================================
// Format Validators
// ============================================================================

/// Validate email format
pub fn email(value: &str, field: &str) -> Result<(), ValidationError> {
    if !EMAIL_REGEX.is_match(value) {
        Err(ValidationError::new(
            field,
            "is not a valid email address",
            "email",
        ))
    } else {
        Ok(())
    }
}

/// Validate URL format (http/https)
pub fn url(value: &str, field: &str) -> Result<(), ValidationError> {
    if !URL_REGEX.is_match(value) {
        Err(ValidationError::new(
            field,
            "is not a valid URL",
            "url",
        ))
    } else {
        Ok(())
    }
}

/// Validate UUID format (v4)
pub fn uuid(value: &str, field: &str) -> Result<(), ValidationError> {
    if !UUID_REGEX.is_match(value) {
        Err(ValidationError::new(
            field,
            "is not a valid UUID",
            "uuid",
        ))
    } else {
        Ok(())
    }
}

/// Validate alphanumeric string (a-z, A-Z, 0-9, _, -)
pub fn alphanumeric(value: &str, field: &str) -> Result<(), ValidationError> {
    if !ALPHANUMERIC_REGEX.is_match(value) {
        Err(ValidationError::new(
            field,
            "must contain only alphanumeric characters, dashes, and underscores",
            "alphanumeric",
        ))
    } else {
        Ok(())
    }
}

/// Validate slug format (lowercase alphanumeric and dashes)
pub fn slug(value: &str, field: &str) -> Result<(), ValidationError> {
    if !SLUG_REGEX.is_match(value) {
        Err(ValidationError::new(
            field,
            "must be a valid slug (lowercase letters, numbers, and dashes)",
            "slug",
        ))
    } else {
        Ok(())
    }
}

/// Validate contains only numeric digits
pub fn numeric(value: &str, field: &str) -> Result<(), ValidationError> {
    if !value.chars().all(|c| c.is_ascii_digit()) {
        Err(ValidationError::new(
            field,
            "must contain only digits",
            "numeric",
        ))
    } else {
        Ok(())
    }
}

/// Validate contains only alphabetic characters
pub fn alpha(value: &str, field: &str) -> Result<(), ValidationError> {
    if !value.chars().all(|c| c.is_alphabetic()) {
        Err(ValidationError::new(
            field,
            "must contain only letters",
            "alpha",
        ))
    } else {
        Ok(())
    }
}

// ============================================================================
// Custom Validators
// ============================================================================

/// Validate that a value matches one of the allowed values
pub fn one_of(value: &str, allowed: &[&str], field: &str) -> Result<(), ValidationError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            format!(
                "must be one of: {}",
                allowed.join(", ")
            ),
            "one_of",
        ))
    }
}

/// Validate that a value does not match a pattern (negative match)
pub fn not_pattern(value: &str, regex: &Regex, field: &str) -> Result<(), ValidationError> {
    if regex.is_match(value) {
        Err(ValidationError::new(
            field,
            "contains invalid characters or pattern",
            "not_pattern",
        ))
    } else {
        Ok(())
    }
}

/// Validate that a value starts with a prefix
pub fn starts_with(value: &str, prefix: &str, field: &str) -> Result<(), ValidationError> {
    if !value.starts_with(prefix) {
        Err(ValidationError::new(
            field,
            format!("must start with '{}'", prefix),
            "starts_with",
        ))
    } else {
        Ok(())
    }
}

/// Validate that a value ends with a suffix
pub fn ends_with(value: &str, suffix: &str, field: &str) -> Result<(), ValidationError> {
    if !value.ends_with(suffix) {
        Err(ValidationError::new(
            field,
            format!("must end with '{}'", suffix),
            "ends_with",
        ))
    } else {
        Ok(())
    }
}

/// Validate that a value contains a substring
pub fn contains(value: &str, substring: &str, field: &str) -> Result<(), ValidationError> {
    if !value.contains(substring) {
        Err(ValidationError::new(
            field,
            format!("must contain '{}'", substring),
            "contains",
        ))
    } else {
        Ok(())
    }
}

/// Validate that a value does not contain a substring
pub fn not_contains(value: &str, substring: &str, field: &str) -> Result<(), ValidationError> {
    if value.contains(substring) {
        Err(ValidationError::new(
            field,
            format!("must not contain '{}'", substring),
            "not_contains",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Required validation tests
    #[test]
    fn test_required_valid() {
        assert!(required("hello", "name").is_ok());
    }

    #[test]
    fn test_required_empty() {
        assert!(required("", "name").is_err());
        assert!(required("   ", "name").is_err());
    }

    // Length validators tests
    #[test]
    fn test_min_length_valid() {
        assert!(min_length("hello", 3, "name").is_ok());
    }

    #[test]
    fn test_min_length_too_short() {
        assert!(min_length("hi", 3, "name").is_err());
    }

    #[test]
    fn test_max_length_valid() {
        assert!(max_length("hi", 10, "name").is_ok());
    }

    #[test]
    fn test_max_length_too_long() {
        assert!(max_length("hello world", 5, "name").is_err());
    }

    #[test]
    fn test_length_range_valid() {
        assert!(length_range("hello", 3, 10, "name").is_ok());
    }

    #[test]
    fn test_length_range_too_short() {
        assert!(length_range("hi", 3, 10, "name").is_err());
    }

    #[test]
    fn test_length_range_too_long() {
        assert!(length_range("hello world", 3, 5, "name").is_err());
    }

    // Pattern tests
    #[test]
    fn test_pattern_valid() {
        let re = Regex::new(r"^\d{3}-\d{4}$").unwrap();
        assert!(pattern("123-4567", &re, "phone").is_ok());
    }

    #[test]
    fn test_pattern_invalid() {
        let re = Regex::new(r"^\d{3}-\d{4}$").unwrap();
        assert!(pattern("abc", &re, "phone").is_err());
    }

    // Range tests
    #[test]
    fn test_range_valid() {
        assert!(range(5, 1, 10, "age").is_ok());
    }

    #[test]
    fn test_range_out_of_bounds() {
        assert!(range(15, 1, 10, "age").is_err());
        assert!(range(0, 1, 10, "age").is_err());
    }

    // Email tests
    #[test]
    fn test_email_valid() {
        assert!(email("user@example.com", "email").is_ok());
        assert!(email("test.name+tag@example.co.uk", "email").is_ok());
    }

    #[test]
    fn test_email_invalid() {
        assert!(email("not-an-email", "email").is_err());
        assert!(email("user@", "email").is_err());
        assert!(email("@example.com", "email").is_err());
    }

    // URL tests
    #[test]
    fn test_url_valid() {
        assert!(url("https://example.com", "url").is_ok());
        assert!(url("http://test.example.co.uk:8080/path", "url").is_ok());
    }

    #[test]
    fn test_url_invalid() {
        assert!(url("not a url", "url").is_err());
        assert!(url("ftp://example.com", "url").is_err());
    }

    // UUID tests
    #[test]
    fn test_uuid_valid() {
        assert!(uuid("f47ac10b-58cc-4372-a567-0e02b2c3d479", "id").is_ok());
    }

    #[test]
    fn test_uuid_invalid() {
        assert!(uuid("not-a-uuid", "id").is_err());
        assert!(uuid("f47ac10b-58cc-4372-a567", "id").is_err());
    }

    // Alphanumeric tests
    #[test]
    fn test_alphanumeric_valid() {
        assert!(alphanumeric("hello_world-123", "name").is_ok());
        assert!(alphanumeric("test", "name").is_ok());
    }

    #[test]
    fn test_alphanumeric_invalid() {
        assert!(alphanumeric("hello world", "name").is_err());
        assert!(alphanumeric("hello@world", "name").is_err());
    }

    // Slug tests
    #[test]
    fn test_slug_valid() {
        assert!(slug("hello-world", "slug").is_ok());
        assert!(slug("my-slug-123", "slug").is_ok());
    }

    #[test]
    fn test_slug_invalid() {
        assert!(slug("Hello-World", "slug").is_err());
        assert!(slug("hello_world", "slug").is_err());
    }

    // Numeric tests
    #[test]
    fn test_numeric_valid() {
        assert!(numeric("12345", "age").is_ok());
    }

    #[test]
    fn test_numeric_invalid() {
        assert!(numeric("123a5", "age").is_err());
        assert!(numeric("", "age").is_err());
    }

    // Alpha tests
    #[test]
    fn test_alpha_valid() {
        assert!(alpha("HelloWorld", "name").is_ok());
    }

    #[test]
    fn test_alpha_invalid() {
        assert!(alpha("hello123", "name").is_err());
        assert!(alpha("hello-world", "name").is_err());
    }

    // One-of tests
    #[test]
    fn test_one_of_valid() {
        assert!(one_of("active", &["active", "inactive", "pending"], "status").is_ok());
    }

    #[test]
    fn test_one_of_invalid() {
        assert!(one_of("unknown", &["active", "inactive"], "status").is_err());
    }

    // Not pattern tests
    #[test]
    fn test_not_pattern_valid() {
        let forbidden = Regex::new(r"[<>]").unwrap();
        assert!(not_pattern("hello world", &forbidden, "text").is_ok());
    }

    #[test]
    fn test_not_pattern_invalid() {
        let forbidden = Regex::new(r"[<>]").unwrap();
        assert!(not_pattern("hello<world>", &forbidden, "text").is_err());
    }

    // String matching tests
    #[test]
    fn test_starts_with_valid() {
        assert!(starts_with("hello-world", "hello", "name").is_ok());
    }

    #[test]
    fn test_starts_with_invalid() {
        assert!(starts_with("hello-world", "world", "name").is_err());
    }

    #[test]
    fn test_ends_with_valid() {
        assert!(ends_with("hello-world", "world", "name").is_ok());
    }

    #[test]
    fn test_ends_with_invalid() {
        assert!(ends_with("hello-world", "hello", "name").is_err());
    }

    #[test]
    fn test_contains_valid() {
        assert!(contains("hello-world", "lo-", "name").is_ok());
    }

    #[test]
    fn test_contains_invalid() {
        assert!(contains("hello-world", "xyz", "name").is_err());
    }

    #[test]
    fn test_not_contains_valid() {
        assert!(not_contains("hello-world", "xyz", "name").is_ok());
    }

    #[test]
    fn test_not_contains_invalid() {
        assert!(not_contains("hello-world", "lo-", "name").is_err());
    }

    // ValidationErrors collection tests
    #[test]
    fn test_validation_errors_collection() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        errors.add_if_err(required("", "name"));
        errors.add_if_err(min_length("x", 5, "password"));
        assert_eq!(errors.len(), 2);
        assert!(errors.into_result().is_err());
    }

    #[test]
    fn test_validation_errors_merge() {
        let mut e1 = ValidationErrors::new();
        e1.add(ValidationError::new("a", "msg", "code"));
        let mut e2 = ValidationErrors::new();
        e2.add(ValidationError::new("b", "msg", "code"));
        e1.merge(e2);
        assert_eq!(e1.len(), 2);
    }

    #[test]
    fn test_validation_errors_iterator() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("field1", "error1", "code1"));
        errors.add(ValidationError::new("field2", "error2", "code2"));
        let count = errors.iter().count();
        assert_eq!(count, 2);
    }

    // Validatable trait tests
    #[test]
    fn test_validatable_trait() {
        struct User {
            name: String,
            email_addr: String,
        }
        impl Validatable for User {
            fn validate(&self) -> Result<(), ValidationErrors> {
                let mut errors = ValidationErrors::new();
                errors.add_if_err(required(&self.name, "name"));
                errors.add_if_err(email(&self.email_addr, "email"));
                errors.into_result()
            }
        }

        let valid = User {
            name: "Alice".into(),
            email_addr: "alice@test.com".into(),
        };
        assert!(valid.validate().is_ok());

        let invalid = User {
            name: "".into(),
            email_addr: "bad".into(),
        };
        assert!(invalid.validate().is_err());
    }

    // ValidationChain tests
    #[test]
    fn test_validation_chain_single() {
        let chain = ValidationChain::new().add(|v, f| required(v, f));
        assert!(chain.validate("hello", "name").is_ok());
        assert!(chain.validate("", "name").is_err());
    }

    #[test]
    fn test_validation_chain_multiple() {
        let chain = ValidationChain::new()
            .add(|v, f| required(v, f))
            .add(|v, f| min_length(v, 3, f))
            .add(|v, f| max_length(v, 10, f));

        assert!(chain.validate("hello", "name").is_ok());
        assert!(chain.validate("", "name").is_err());
        assert!(chain.validate("hi", "name").is_err());
        assert!(chain.validate("this is too long", "name").is_err());
    }

    #[test]
    fn test_validation_chain_complex() {
        let chain = ValidationChain::new()
            .add(|v, f| required(v, f))
            .add(|v, f| email(v, f));

        assert!(chain.validate("user@example.com", "email").is_ok());
        assert!(chain.validate("", "email").is_err());
        assert!(chain.validate("not-an-email", "email").is_err());
    }
}
