//! Field-level validator: composition of multiple rules.
//!
//! A `FieldValidator` combines one or more `ValidationRule`s to validate
//! a single field against multiple criteria.

use super::rule::ValidationRule;
use super::error::ValidationError;

/// A multi-rule validator for a single field.
///
/// Accumulates validation rules and validates a value against all of them.
#[derive(Clone, Debug)]
pub struct FieldValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl FieldValidator {
    /// Create a new, empty field validator.
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    /// Add a validation rule.
    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Add a rule by convenience generic.
    pub fn with_rule<R: ValidationRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Validate a value against all rules.
    ///
    /// Returns the first error encountered, if any.
    pub fn validate(&self, value: &str) -> Result<(), ValidationError> {
        for rule in &self.rules {
            rule.validate(value)?;
        }
        Ok(())
    }

    /// Validate and collect all errors (not just the first).
    pub fn validate_all(&self, value: &str) -> Vec<ValidationError> {
        let mut errors = vec![];
        for rule in &self.rules {
            if let Err(err) = rule.validate(value) {
                errors.push(err);
            }
        }
        errors
    }

    /// Get the count of registered rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get rule names in order.
    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|r| r.name()).collect()
    }
}

impl Default for FieldValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::rule::{RequiredRule, LengthRule, PatternRule};

    #[test]
    fn test_field_validator_empty() {
        let validator = FieldValidator::new();
        assert!(validator.validate("anything").is_ok());
    }

    #[test]
    fn test_field_validator_single_rule() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new());

        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn test_field_validator_multiple_rules() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(LengthRule::range(3, 10));

        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("").is_err());
        assert!(validator.validate("ab").is_err());
        assert!(validator.validate("abcdefghijk").is_err());
    }

    #[test]
    fn test_field_validator_stops_on_first_error() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(LengthRule::min(5));

        let result = validator.validate("");
        assert!(result.is_err());
    }

    #[test]
    fn test_field_validator_collect_all_errors() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(LengthRule::min(5));

        let errors = validator.validate_all("ab");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "min_length");
    }

    #[test]
    fn test_field_validator_rule_names() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(LengthRule::range(3, 10));

        let names = validator.rule_names();
        assert_eq!(names, vec!["required", "length"]);
    }

    #[test]
    fn test_field_validator_with_pattern() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(PatternRule::new(r"^[a-z]+$").unwrap());

        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("Hello").is_err());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn test_field_validator_cloneable() {
        let validator1 = FieldValidator::new()
            .with_rule(RequiredRule::new());

        let validator2 = validator1.clone();

        assert_eq!(validator1.rule_count(), validator2.rule_count());
        assert!(validator2.validate("test").is_ok());
    }
}
