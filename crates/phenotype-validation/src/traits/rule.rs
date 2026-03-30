//! Core `ValidationRule` trait and built-in implementations.
//!
//! A `ValidationRule` is a composable, reusable validation unit.
//! Each rule validates a single aspect (e.g., "required", "min_length", "email_format").

use super::error::{ValidationError, Severity, ValidationContext};

/// A single, composable validation rule.
///
/// Implementations should be stateless (or cheaply cloneable) for use in validators.
pub trait ValidationRule: Send + Sync {
    /// Validate a value against this rule.
    ///
    /// # Returns
    /// - `Ok(())` if validation passes
    /// - `Err(ValidationError)` if validation fails
    fn validate(&self, value: &str) -> Result<(), ValidationError>;

    /// Name of this rule (e.g., "required", "email_format", "min_length").
    fn name(&self) -> &'static str;

    /// Severity of failures from this rule (default: Error).
    fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Clone as a boxed trait object.
    fn boxed_clone(&self) -> Box<dyn ValidationRule>;
}

// === Built-in Rules ===

/// Validates that a value is non-empty and non-whitespace.
#[derive(Clone, Debug)]
pub struct RequiredRule;

impl ValidationRule for RequiredRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError::new("required")
                .with_suggestion("This field is required"))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }

    fn boxed_clone(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

impl RequiredRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequiredRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates length: minimum and/or maximum.
#[derive(Clone, Debug)]
pub struct LengthRule {
    min: Option<usize>,
    max: Option<usize>,
}

impl LengthRule {
    pub fn new() -> Self {
        Self { min: None, max: None }
    }

    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    pub fn range(min: usize, max: usize) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl Default for LengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationRule for LengthRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        let len = value.chars().count();

        if let Some(min) = self.min {
            if len < min {
                return Err(ValidationError::new("min_length")
                    .with_suggestion(format!("Minimum {} characters required", min)));
            }
        }

        if let Some(max) = self.max {
            if len > max {
                return Err(ValidationError::new("max_length")
                    .with_suggestion(format!("Maximum {} characters allowed", max)));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "length"
    }

    fn boxed_clone(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

/// Validates against a regex pattern.
#[derive(Clone, Debug)]
pub struct PatternRule {
    pattern: regex::Regex,
    pattern_str: String,
}

impl PatternRule {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let regex = regex::Regex::new(pattern)?;
        Ok(Self {
            pattern: regex,
            pattern_str: pattern.to_string(),
        })
    }
}

impl ValidationRule for PatternRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if self.pattern.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::new("pattern_mismatch")
                .with_suggestion(format!("Value must match pattern: {}", self.pattern_str)))
        }
    }

    fn name(&self) -> &'static str {
        "pattern"
    }

    fn boxed_clone(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

/// Validates numeric value is within range.
#[derive(Clone, Debug)]
pub struct NumericRangeRule {
    min: Option<f64>,
    max: Option<f64>,
}

impl NumericRangeRule {
    pub fn new() -> Self {
        Self { min: None, max: None }
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn range(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl Default for NumericRangeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationRule for NumericRangeRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        let num: f64 = value.parse().map_err(|_| {
            ValidationError::new("not_numeric")
                .with_suggestion("Value must be a valid number")
        })?;

        if let Some(min) = self.min {
            if num < min {
                return Err(ValidationError::new("below_minimum")
                    .with_suggestion(format!("Minimum value: {}", min)));
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return Err(ValidationError::new("above_maximum")
                    .with_suggestion(format!("Maximum value: {}", max)));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "numeric_range"
    }

    fn boxed_clone(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

/// Custom rule using a closure.
pub struct CustomRule {
    name: String,
    validator: Box<dyn Fn(&str) -> Result<(), ValidationError> + Send + Sync>,
}

impl CustomRule {
    pub fn new<F>(name: &str, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            validator: Box::new(validator),
        }
    }
}

impl ValidationRule for CustomRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        (self.validator)(value)
    }

    fn name(&self) -> &'static str {
        "custom"
    }

    fn boxed_clone(&self) -> Box<dyn ValidationRule> {
        panic!("CustomRule with closures cannot be cloned; use built-in rules or register in ValidatorRegistry")
    }
}

impl Clone for CustomRule {
    fn clone(&self) -> Self {
        panic!("CustomRule with closures cannot be cloned; use built-in rules or register in ValidatorRegistry")
    }
}

impl std::fmt::Debug for CustomRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomRule")
            .field("name", &self.name)
            .field("validator", &"<closure>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_rule_empty_fails() {
        let rule = RequiredRule::new();
        assert!(rule.validate("").is_err());
        assert!(rule.validate("   ").is_err());
    }

    #[test]
    fn test_required_rule_non_empty_passes() {
        let rule = RequiredRule::new();
        assert!(rule.validate("valid").is_ok());
    }

    #[test]
    fn test_length_rule_min() {
        let rule = LengthRule::new().min(5);
        assert!(rule.validate("hi").is_err());
        assert!(rule.validate("hello").is_ok());
    }

    #[test]
    fn test_length_rule_max() {
        let rule = LengthRule::new().max(5);
        assert!(rule.validate("hello world").is_err());
        assert!(rule.validate("hello").is_ok());
    }

    #[test]
    fn test_length_rule_range() {
        let rule = LengthRule::range(3, 10);
        assert!(rule.validate("ab").is_err());
        assert!(rule.validate("abc").is_ok());
        assert!(rule.validate("0123456789a").is_err());
    }

    #[test]
    fn test_pattern_rule_valid() {
        let rule = PatternRule::new(r"^[a-z]+$").unwrap();
        assert!(rule.validate("hello").is_ok());
        assert!(rule.validate("Hello").is_err());
    }

    #[test]
    fn test_pattern_rule_invalid_regex() {
        assert!(PatternRule::new("[invalid(regex").is_err());
    }

    #[test]
    fn test_numeric_range_rule_valid() {
        let rule = NumericRangeRule::range(0.0, 100.0);
        assert!(rule.validate("50").is_ok());
        assert!(rule.validate("0").is_ok());
        assert!(rule.validate("100").is_ok());
    }

    #[test]
    fn test_numeric_range_rule_out_of_range() {
        let rule = NumericRangeRule::range(0.0, 100.0);
        assert!(rule.validate("-1").is_err());
        assert!(rule.validate("101").is_err());
    }

    #[test]
    fn test_numeric_range_rule_non_numeric() {
        let rule = NumericRangeRule::range(0.0, 100.0);
        assert!(rule.validate("not a number").is_err());
    }

    #[test]
    fn test_custom_rule() {
        let rule = CustomRule::new("uppercase", |value| {
            if value.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
                Ok(())
            } else {
                Err(ValidationError::new("not_uppercase"))
            }
        });

        assert!(rule.validate("HELLO").is_ok());
        assert!(rule.validate("Hello").is_err());
    }
}
