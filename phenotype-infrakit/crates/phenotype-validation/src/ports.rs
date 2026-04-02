//! Validator port traits

use crate::types::ValidationResult;
use crate::error::Result;

/// Port trait for validator implementations
pub trait ValidatorPort: Send + Sync {
    /// Validate the given input
    fn validate(&self, input: &str) -> Result<ValidationResult>;
}
