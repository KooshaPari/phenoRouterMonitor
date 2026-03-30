//! Command-level validator: orchestrates multiple field validators.
//!
//! A `CommandValidator` combines field validators and cross-field rules
//! to validate entire commands or data structures.

use super::field_validator::FieldValidator;
use super::error::ValidationError;
use std::collections::HashMap;

/// Command-level validation result.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Map of field name to errors (if any).
    pub field_errors: HashMap<String, Vec<ValidationError>>,
    /// Cross-field validation errors (if any).
    pub cross_field_errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Check if validation succeeded (no errors).
    pub fn is_ok(&self) -> bool {
        self.field_errors.is_empty() && self.cross_field_errors.is_empty()
    }

    /// Get total error count.
    pub fn error_count(&self) -> usize {
        let field_errors: usize = self.field_errors.values().map(|v| v.len()).sum();
        field_errors + self.cross_field_errors.len()
    }

    /// Get user-friendly error summary.
    pub fn summary(&self) -> String {
        if self.is_ok() {
            return "Validation succeeded".to_string();
        }

        let mut lines = vec![format!(
            "Validation failed with {} error(s):",
            self.error_count()
        )];

        for (field, errors) in &self.field_errors {
            for err in errors {
                lines.push(format!("  - {}: {}", field, err.display_message()));
            }
        }

        for err in &self.cross_field_errors {
            lines.push(format!("  - {}", err.display_message()));
        }

        lines.join("\n")
    }
}

/// Validates an entire command with multiple fields.
#[derive(Clone, Debug)]
pub struct CommandValidator {
    field_validators: HashMap<String, FieldValidator>,
}

impl CommandValidator {
    /// Create a new command validator.
    pub fn new() -> Self {
        Self {
            field_validators: HashMap::new(),
        }
    }

    /// Register a field validator.
    pub fn add_field(mut self, field_name: impl Into<String>, validator: FieldValidator) -> Self {
        self.field_validators.insert(field_name.into(), validator);
        self
    }

    /// Validate all fields in a command.
    ///
    /// # Arguments
    /// * `fields` - Map of field names to values
    ///
    /// # Returns
    /// A `ValidationResult` with all errors collected.
    pub fn validate(&self, fields: &HashMap<String, String>) -> ValidationResult {
        let mut result = ValidationResult {
            field_errors: HashMap::new(),
            cross_field_errors: vec![],
        };

        for (field_name, validator) in &self.field_validators {
            if let Some(value) = fields.get(field_name) {
                let errors = validator.validate_all(value);
                if !errors.is_empty() {
                    result.field_errors.insert(field_name.clone(), errors);
                }
            }
        }

        result
    }

    /// Get registered field count.
    pub fn field_count(&self) -> usize {
        self.field_validators.len()
    }

    /// Get registered field names.
    pub fn field_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.field_validators.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::rule::{RequiredRule, LengthRule, PatternRule};

    #[test]
    fn test_command_validator_single_field() {
        let validator = CommandValidator::new()
            .add_field(
                "name",
                FieldValidator::new().with_rule(RequiredRule::new()),
            );

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());

        let result = validator.validate(&fields);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_validator_multiple_fields() {
        let validator = CommandValidator::new()
            .add_field(
                "name",
                FieldValidator::new()
                    .with_rule(RequiredRule::new())
                    .with_rule(LengthRule::range(2, 50)),
            )
            .add_field(
                "email",
                FieldValidator::new()
                    .with_rule(RequiredRule::new())
                    .with_rule(PatternRule::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()),
            );

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John Doe".to_string());
        fields.insert("email".to_string(), "john@example.com".to_string());

        let result = validator.validate(&fields);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_validator_field_errors() {
        let validator = CommandValidator::new()
            .add_field(
                "name",
                FieldValidator::new().with_rule(RequiredRule::new()),
            )
            .add_field(
                "email",
                FieldValidator::new().with_rule(RequiredRule::new()),
            );

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "".to_string());
        fields.insert("email".to_string(), "".to_string());

        let result = validator.validate(&fields);
        assert!(!result.is_ok());
        assert_eq!(result.error_count(), 2);
        assert!(result.field_errors.contains_key("name"));
        assert!(result.field_errors.contains_key("email"));
    }

    #[test]
    fn test_validation_result_summary() {
        let mut result = ValidationResult {
            field_errors: HashMap::new(),
            cross_field_errors: vec![],
        };

        result.field_errors.insert(
            "name".to_string(),
            vec![ValidationError::new("required")],
        );

        let summary = result.summary();
        assert!(summary.contains("1 error"));
        assert!(summary.contains("name"));
    }

    #[test]
    fn test_command_validator_missing_field() {
        let validator = CommandValidator::new()
            .add_field(
                "name",
                FieldValidator::new().with_rule(RequiredRule::new()),
            );

        let fields = HashMap::new();
        let result = validator.validate(&fields);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_validator_field_names() {
        let validator = CommandValidator::new()
            .add_field("name", FieldValidator::new())
            .add_field("email", FieldValidator::new())
            .add_field("age", FieldValidator::new());

        let names = validator.field_names();
        assert_eq!(names, vec!["age", "email", "name"]);
    }
}
