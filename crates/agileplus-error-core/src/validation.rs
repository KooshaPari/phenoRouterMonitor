//! Validation error type for input validation and constraint checking.

use thiserror::Error;

/// Error type for input validation and constraint violations.
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("invalid type: expected {expected}, got {actual}")]
    InvalidType { expected: String, actual: String },

    #[error("constraint violated: {0}")]
    ConstraintViolated(String),

    #[error("out of range: {0}")]
    OutOfRange(String),

    #[error("invalid length: {0}")]
    InvalidLength(String),

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("duplicate entry: {0}")]
    Duplicate(String),
}

impl ValidationError {
    pub fn missing_field(field: impl Into<String>) -> Self { Self::MissingField(field.into()) }
    pub fn invalid_value(msg: impl Into<String>) -> Self { Self::InvalidValue(msg.into()) }
    pub fn invalid_type(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::InvalidType { expected: expected.into(), actual: actual.into() }
    }
    pub fn constraint_violated(msg: impl Into<String>) -> Self { Self::ConstraintViolated(msg.into()) }
    pub fn out_of_range(msg: impl Into<String>) -> Self { Self::OutOfRange(msg.into()) }
    pub fn invalid_length(msg: impl Into<String>) -> Self { Self::InvalidLength(msg.into()) }
    pub fn invalid_format(msg: impl Into<String>) -> Self { Self::InvalidFormat(msg.into()) }
    pub fn invalid_state(msg: impl Into<String>) -> Self { Self::InvalidState(msg.into()) }
    pub fn duplicate(entity: impl Into<String>) -> Self { Self::Duplicate(entity.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_field_error() {
        let err = ValidationError::missing_field("email");
        assert_eq!(err.to_string(), "missing required field: email");
    }

    #[test]
    fn test_invalid_value_error() {
        let err = ValidationError::invalid_value("email is not a valid format");
        assert_eq!(err.to_string(), "invalid value: email is not a valid format");
    }

    #[test]
    fn test_invalid_type_error() {
        let err = ValidationError::invalid_type("string", "number");
        assert!(err.to_string().contains("expected string, got number"));
    }

    #[test]
    fn test_constraint_violated_error() {
        let err = ValidationError::constraint_violated("status must be one of: pending, active, completed");
        assert_eq!(err.to_string(), "constraint violated: status must be one of: pending, active, completed");
    }

    #[test]
    fn test_out_of_range_error() {
        let err = ValidationError::out_of_range("page must be >= 1");
        assert_eq!(err.to_string(), "out of range: page must be >= 1");
    }

    #[test]
    fn test_invalid_length_error() {
        let err = ValidationError::invalid_length("name must be between 1 and 255 characters");
        assert_eq!(err.to_string(), "invalid length: name must be between 1 and 255 characters");
    }

    #[test]
    fn test_invalid_format_error() {
        let err = ValidationError::invalid_format("uuid");
        assert_eq!(err.to_string(), "invalid format: uuid");
    }

    #[test]
    fn test_invalid_state_error() {
        let err = ValidationError::invalid_state("cannot transition from completed to pending");
        assert_eq!(err.to_string(), "invalid state: cannot transition from completed to pending");
    }

    #[test]
    fn test_duplicate_error() {
        let err = ValidationError::duplicate("User(user@example.com)");
        assert_eq!(err.to_string(), "duplicate entry: User(user@example.com)");
    }
}
