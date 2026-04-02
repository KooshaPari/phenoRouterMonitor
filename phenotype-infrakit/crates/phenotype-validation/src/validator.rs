//! Core validator implementation

use crate::error::{Result, ValidationError};
use crate::ports::ValidatorPort;
use crate::types::ValidationResult;
use std::sync::Arc;

/// Core validator
#[derive(Debug, Clone)]
pub struct Validator {
    rules: Vec<Arc<dyn ValidatorPort>>,
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Add a validation rule
    pub fn add_rule(mut self, rule: impl ValidatorPort + 'static) -> Self {
        self.rules.push(Arc::new(rule));
        self
    }

    /// Validate input against all rules
    pub fn validate(&self, input: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        for rule in &self.rules {
            match rule.validate(input) {
                Ok(rule_result) => {
                    if !rule_result.valid {
                        for error in rule_result.errors {
                            result.add_error(error);
                        }
                    }
                    for warning in rule_result.warnings {
                        result.add_warning(warning);
                    }
                }
                Err(e) => {
                    result.add_error(e.to_string());
                }
            }
        }
        
        Ok(result)
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
