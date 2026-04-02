//! Validation rules

use crate::error::{Result, ValidationError};
use crate::ports::ValidatorPort;
use crate::types::ValidationResult;
use std::sync::Arc;

/// Non-empty string rule
#[derive(Debug, Clone)]
pub struct NonEmptyRule {
    field_name: String,
}

impl NonEmptyRule {
    /// Create a new non-empty rule
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
        }
    }
}

impl ValidatorPort for NonEmptyRule {
    fn validate(&self, input: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        if input.trim().is_empty() {
            result.add_error(format!("{} cannot be empty", self.field_name));
        }
        
        Ok(result)
    }
}

/// Length rule
#[derive(Debug, Clone)]
pub struct LengthRule {
    field_name: String,
    min: Option<usize>,
    max: Option<usize>,
}

impl LengthRule {
    /// Create a new length rule
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
            min: None,
            max: None,
        }
    }

    /// Set minimum length
    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum length
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }
}

impl ValidatorPort for LengthRule {
    fn validate(&self, input: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let len = input.len();
        
        if let Some(min) = self.min {
            if len < min {
                result.add_error(format!("{} must be at least {} characters", self.field_name, min));
            }
        }
        
        if let Some(max) = self.max {
            if len > max {
                result.add_error(format!("{} must be at most {} characters", self.field_name, max));
            }
        }
        
        Ok(result)
    }
}
