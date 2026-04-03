//! # phenotype-validation
//!
//! Composable validation rules and registry for the Phenotype ecosystem.
//!
//! This crate provides a trait-based validation system with plugin support.
//!
//! # Example
//!
//! ```rust
//! use phenotype_validation::{ValidationRule, RequiredRule, LengthRule};
//!
//! let rule = RequiredRule::new();
//! assert!(rule.validate("hello").is_ok());
//! assert!(rule.validate("").is_err());
//!
//! let rule = LengthRule::new(1, 100);
//! assert!(rule.validate("hello").is_ok());
//! assert!(rule.validate("").is_err());
//! ```

use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors that can occur during validation.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct ValidationError {
    #[from]
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            inner: msg.into().into(),
        }
    }

    /// Create an error for an invalid field.
    pub fn invalid_field(field: &str, reason: &str) -> Self {
        Self::new(format!("{}: {}", field, reason))
    }
}

/// Result type for validation operations.
pub type Result<T> = std::result::Result<T, ValidationError>;

// ---------------------------------------------------------------------------
// Core Trait
// ---------------------------------------------------------------------------

/// Core trait for validation rules.
pub trait ValidationRule: Send + Sync {
    /// Validate a string value.
    fn validate(&self, value: &str) -> Result<()>;

    /// Human-readable name of this rule.
    fn name(&self) -> &'static str;

    /// Optional description for error messages.
    fn description(&self) -> Option<&str> {
        None
    }
}

// ---------------------------------------------------------------------------
// Common Rules
// ---------------------------------------------------------------------------

/// Rule that requires a non-empty value.
#[derive(Clone, Debug, Default)]
pub struct RequiredRule {
    message: String,
}

impl RequiredRule {
    pub fn new() -> Self {
        Self {
            message: "field is required".to_string(),
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
}

impl ValidationRule for RequiredRule {
    fn validate(&self, value: &str) -> Result<()> {
        if value.trim().is_empty() {
            Err(ValidationError::new(&self.message))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }

    fn description(&self) -> Option<&str> {
        Some("value must not be empty")
    }
}

/// Rule that validates string length.
#[derive(Clone, Debug)]
pub struct LengthRule {
    min: usize,
    max: usize,
}

impl LengthRule {
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    pub fn min(mut self, min: usize) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }
}

impl ValidationRule for LengthRule {
    fn validate(&self, value: &str) -> Result<()> {
        let len = value.len();
        if len < self.min {
            return Err(ValidationError::new(format!(
                "length {} is less than minimum {}",
                len, self.min
            )));
        }
        if len > self.max {
            return Err(ValidationError::new(format!(
                "length {} exceeds maximum {}",
                len, self.max
            )));
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "length"
    }

    fn description(&self) -> Option<&str> {
        Some("value length must be within specified bounds")
    }
}

/// Rule that validates a regex pattern.
#[derive(Clone, Debug)]
pub struct PatternRule {
    pattern: regex::Regex,
    description: String,
}

impl PatternRule {
    pub fn new(pattern: &str) -> Result<Self> {
        Ok(Self {
            pattern: regex::Regex::new(pattern).map_err(|e| ValidationError::new(e.to_string()))?,
            description: format!("must match pattern: {}", pattern),
        })
    }

    pub fn email() -> Result<Self> {
        Self::new(r"^[\w.+-]+@[\w.-]+\.[a-zA-Z]{2,}$")
    }

    pub fn url() -> Result<Self> {
        Self::new(r"^https?://[^\s/$.?#].[^\s]*$")
    }
}

impl ValidationRule for PatternRule {
    fn validate(&self, value: &str) -> Result<()> {
        if self.pattern.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::new(&self.description))
        }
    }

    fn name(&self) -> &'static str {
        "pattern"
    }

    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }
}

/// Rule that validates numeric range.
#[derive(Clone, Debug)]
pub struct NumericRangeRule {
    min: Option<f64>,
    max: Option<f64>,
}

impl NumericRangeRule {
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }
}

impl Default for NumericRangeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationRule for NumericRangeRule {
    fn validate(&self, value: &str) -> Result<()> {
        let num: f64 = value
            .parse()
            .map_err(|_| ValidationError::new("value must be a number"))?;

        if let Some(min) = self.min {
            if num < min {
                return Err(ValidationError::new(format!(
                    "value {} is less than minimum {}",
                    num, min
                )));
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return Err(ValidationError::new(format!(
                    "value {} exceeds maximum {}",
                    num, max
                )));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "numeric_range"
    }

    fn description(&self) -> Option<&str> {
        Some("value must be within specified numeric range")
    }
}

// ---------------------------------------------------------------------------
// Field Validator (combines multiple rules)
// ---------------------------------------------------------------------------

/// Combines multiple validation rules for a single field.
#[derive(Default)]
pub struct FieldValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    field_name: String,
}

impl FieldValidator {
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            rules: Vec::new(),
            field_name: field_name.into(),
        }
    }

    pub fn with_rule<R: ValidationRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn with_required(self) -> Self {
        self.with_rule(RequiredRule::new())
    }

    pub fn with_length(self, min: usize, max: usize) -> Self {
        self.with_rule(LengthRule::new(min, max))
    }

    pub fn with_pattern(self, pattern: &str) -> Result<Self> {
        Ok(self.with_rule(PatternRule::new(pattern)?))
    }

    /// Validate a value against all rules.
    pub fn validate(&self, value: &str) -> Result<()> {
        for rule in &self.rules {
            rule.validate(value)
                .map_err(|e| ValidationError::invalid_field(&self.field_name, &e.to_string()))?;
        }
        Ok(())
    }

    /// Get the field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}

// ---------------------------------------------------------------------------
// Preset Validators
// ---------------------------------------------------------------------------

/// Preset for email validation.
pub fn email_validator() -> FieldValidator {
    FieldValidator::new("email")
        .with_required()
        .with_length(3, 254)
        .with_pattern(r"^[\w.+-]+@[\w.-]+\.[a-zA-Z]{2,}$")
        .expect("valid email pattern")
}

/// Preset for URL validation.
pub fn url_validator() -> FieldValidator {
    FieldValidator::new("url")
        .with_required()
        .with_pattern(r"^https?://[^\s/$.?#].[^\s]*$")
        .expect("valid url pattern")
}

/// Preset for username validation (alphanumeric + underscore, 3-32 chars).
pub fn username_validator() -> FieldValidator {
    FieldValidator::new("username")
        .with_required()
        .with_length(3, 32)
        .with_pattern(r"^[a-zA-Z0-9_]+$")
        .expect("valid username pattern")
}

/// Preset for non-empty string validation.
pub fn required_validator(field: &str) -> FieldValidator {
    FieldValidator::new(field).with_required()
}

// ---------------------------------------------------------------------------
// Validator Registry
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::RwLock;

static VALIDATOR_REGISTRY: LazyLock<RwLock<ValidatorRegistry>> =
    LazyLock::new(|| RwLock::new(ValidatorRegistry::default()));

/// Global validator registry for plugin-style validation.
#[derive(Default)]
pub struct ValidatorRegistry {
    validators: HashMap<String, fn() -> FieldValidator>,
}

impl ValidatorRegistry {
    /// Register a named validator factory.
    pub fn register(name: &'static str, factory: fn() -> FieldValidator) {
        let _ = VALIDATOR_REGISTRY
            .write()
            .unwrap()
            .validators
            .insert(name.to_string(), factory);
    }

    /// Get a registered validator by name.
    pub fn get(name: &str) -> Option<FieldValidator> {
        VALIDATOR_REGISTRY
            .read()
            .unwrap()
            .validators
            .get(name)
            .map(|f| f())
    }

    /// Check if a validator is registered.
    pub fn contains(name: &str) -> bool {
        VALIDATOR_REGISTRY
            .read()
            .unwrap()
            .validators
            .contains_key(name)
    }
}

/// Register default validators.
pub fn register_defaults() {
    ValidatorRegistry::register("email", email_validator);
    ValidatorRegistry::register("url", url_validator);
    ValidatorRegistry::register("username", username_validator);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_rule() {
        let rule = RequiredRule::new();
        assert!(rule.validate("hello").is_ok());
        assert!(rule.validate("  ").is_err());
        assert!(rule.validate("").is_err());
    }

    #[test]
    fn test_length_rule() {
        let rule = LengthRule::new(2, 5);
        assert!(rule.validate("hi").is_ok());
        assert!(rule.validate("hello").is_ok());
        assert!(rule.validate("h").is_err());
        assert!(rule.validate("hello!").is_err());
    }

    #[test]
    fn test_pattern_rule_email() {
        let rule = PatternRule::email().unwrap();
        assert!(rule.validate("test@example.com").is_ok());
        assert!(rule.validate("invalid").is_err());
        assert!(rule.validate("test@").is_err());
    }

    #[test]
    fn test_numeric_range_rule() {
        let rule = NumericRangeRule::new().with_min(0.0).with_max(100.0);
        assert!(rule.validate("50").is_ok());
        assert!(rule.validate("0").is_ok());
        assert!(rule.validate("100").is_ok());
        assert!(rule.validate("-1").is_err());
        assert!(rule.validate("101").is_err());
    }

    #[test]
    fn test_field_validator() {
        let validator = FieldValidator::new("age")
            .with_required()
            .with_length(1, 3)
            .with_rule(NumericRangeRule::new().with_min(0.0));

        assert!(validator.validate("25").is_ok());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn test_email_validator_preset() {
        let validator = email_validator();
        assert!(validator.validate("test@example.com").is_ok());
        assert!(validator.validate("ab").is_err()); // too short
    }

    #[test]
    fn test_validator_registry() {
        register_defaults();
        assert!(ValidatorRegistry::contains("email"));
        assert!(ValidatorRegistry::contains("url"));
        assert!(!ValidatorRegistry::contains("nonexistent"));
    }
}
