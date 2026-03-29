//! # Phenotype Unified Error Hierarchy
//!
//! This crate provides a unified error type (`PhenotypeError`) for all Phenotype org crates.
//! It consolidates 35+ scattered error types from across the ecosystem into a single,
//! composable hierarchy with automatic conversions from standard library and common types.
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_errors::{PhenotypeError, Result};
//!
//! fn load_config(path: &str) -> Result<String> {
//!     std::fs::read_to_string(path).map_err(PhenotypeError::from)
//! }
//! ```

/// Core error enum for all Phenotype operations.
///
/// This enum provides variants for common error categories across the Phenotype ecosystem.
/// Each variant carries a descriptive message that can be converted to string.
#[derive(Debug, thiserror::Error)]
pub enum PhenotypeError {
    // System/IO errors
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(String),

    /// Configuration loading or validation failed
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization to bytes/string failed
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization from bytes/string failed
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    // Domain/validation errors
    /// State transition or invariant violated
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Contract or interface violation
    #[error("Invalid contract: {0}")]
    InvalidContract(String),

    /// Input validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    // Storage errors
    /// Resource not found in storage
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict: resource already exists or constraint violated
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Storage operation failed (DB, cache, filesystem)
    #[error("Storage failure: {0}")]
    StorageFailure(String),

    // Auth/Security errors
    /// User not authenticated
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// User authenticated but not authorized for action
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Token invalid or expired
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    // Policy/Rules errors
    /// Security or business policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Rule evaluation engine error
    #[error("Rule evaluation failed: {0}")]
    RuleEvaluation(String),

    // Generic
    /// Internal error (should rarely reach users)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Unknown error with no additional context
    #[error("Unknown error")]
    Unknown,
}

// Conversion implementations

impl From<std::io::Error> for PhenotypeError {
    fn from(err: std::io::Error) -> Self {
        PhenotypeError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PhenotypeError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_io() {
            PhenotypeError::Io(err.to_string())
        } else if err.is_syntax() {
            PhenotypeError::Deserialization(err.to_string())
        } else {
            PhenotypeError::Serialization(err.to_string())
        }
    }
}

impl From<regex::Error> for PhenotypeError {
    fn from(err: regex::Error) -> Self {
        PhenotypeError::ValidationFailed(err.to_string())
    }
}

impl From<chrono::ParseError> for PhenotypeError {
    fn from(err: chrono::ParseError) -> Self {
        PhenotypeError::ValidationFailed(err.to_string())
    }
}

impl From<String> for PhenotypeError {
    fn from(s: String) -> Self {
        PhenotypeError::Internal(s)
    }
}

impl From<&str> for PhenotypeError {
    fn from(s: &str) -> Self {
        PhenotypeError::Internal(s.to_string())
    }
}

// Optional Tokio support
#[cfg(feature = "tokio-support")]
impl From<tokio::task::JoinError> for PhenotypeError {
    fn from(err: tokio::task::JoinError) -> Self {
        PhenotypeError::Internal(err.to_string())
    }
}

/// Convenience result type using `PhenotypeError`
pub type Result<T> = std::result::Result<T, PhenotypeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let pheno_err = PhenotypeError::from(io_err);
        assert!(matches!(pheno_err, PhenotypeError::Io(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_str = r#"{"invalid json"#;
        let result: std::result::Result<serde_json::Value, _> =
            serde_json::from_str(json_str);
        let pheno_result: Result<serde_json::Value> =
            result.map_err(PhenotypeError::from);
        assert!(pheno_result.is_err());
    }

    #[test]
    fn test_from_string() {
        let msg = "test error".to_string();
        let err = PhenotypeError::from(msg);
        assert!(matches!(err, PhenotypeError::Internal(_)));
    }

    #[test]
    fn test_from_str() {
        let err = PhenotypeError::from("test error");
        assert!(matches!(err, PhenotypeError::Internal(_)));
    }

    #[test]
    fn test_error_display() {
        let err = PhenotypeError::NotFound("resource".to_string());
        assert_eq!(err.to_string(), "Not found: resource");
    }

    #[test]
    fn test_validation_failed() {
        let err = PhenotypeError::ValidationFailed("invalid input".to_string());
        assert!(matches!(err, PhenotypeError::ValidationFailed(_)));
    }

    #[test]
    fn test_policy_violation() {
        let err = PhenotypeError::PolicyViolation("quota exceeded".to_string());
        assert!(matches!(err, PhenotypeError::PolicyViolation(_)));
    }

    #[test]
    fn test_unauthorized() {
        let err = PhenotypeError::Unauthorized("credentials required".to_string());
        assert!(matches!(err, PhenotypeError::Unauthorized(_)));
    }

    #[test]
    fn test_forbidden() {
        let err = PhenotypeError::Forbidden("insufficient permissions".to_string());
        assert!(matches!(err, PhenotypeError::Forbidden(_)));
    }

    #[test]
    fn test_from_regex_error() {
        let result = regex::Regex::new("[invalid");
        assert!(result.is_err());
        let pheno_err = PhenotypeError::from(result.unwrap_err());
        assert!(matches!(pheno_err, PhenotypeError::ValidationFailed(_)));
    }

    #[test]
    fn test_from_chrono_error() {
        let result = chrono::DateTime::parse_from_rfc2822("invalid date");
        assert!(result.is_err());
        let pheno_err = PhenotypeError::from(result.unwrap_err());
        assert!(matches!(pheno_err, PhenotypeError::ValidationFailed(_)));
    }

    #[test]
    fn test_unknown_error() {
        let err = PhenotypeError::Unknown;
        assert_eq!(err.to_string(), "Unknown error");
    }

    #[test]
    fn test_conflict_error() {
        let err = PhenotypeError::Conflict("resource already exists".to_string());
        assert_eq!(
            err.to_string(),
            "Conflict: resource already exists"
        );
    }

    #[test]
    fn test_storage_failure_error() {
        let err = PhenotypeError::StorageFailure("database connection failed".to_string());
        assert!(matches!(err, PhenotypeError::StorageFailure(_)));
    }

    #[test]
    fn test_invalid_state_error() {
        let err = PhenotypeError::InvalidState("state machine in invalid state".to_string());
        assert!(matches!(err, PhenotypeError::InvalidState(_)));
    }

    #[test]
    fn test_invalid_contract_error() {
        let err = PhenotypeError::InvalidContract("contract precondition violated".to_string());
        assert!(matches!(err, PhenotypeError::InvalidContract(_)));
    }

    #[test]
    fn test_rule_evaluation_error() {
        let err = PhenotypeError::RuleEvaluation("rule engine crash".to_string());
        assert!(matches!(err, PhenotypeError::RuleEvaluation(_)));
    }

    #[test]
    fn test_serialization_error() {
        let err = PhenotypeError::Serialization("failed to serialize".to_string());
        assert!(matches!(err, PhenotypeError::Serialization(_)));
    }

    #[test]
    fn test_deserialization_error() {
        let err = PhenotypeError::Deserialization("failed to deserialize".to_string());
        assert!(matches!(err, PhenotypeError::Deserialization(_)));
    }

    #[test]
    fn test_config_error() {
        let err = PhenotypeError::Config("invalid config file".to_string());
        assert!(matches!(err, PhenotypeError::Config(_)));
    }

    #[test]
    fn test_invalid_token_error() {
        let err = PhenotypeError::InvalidToken("token expired".to_string());
        assert!(matches!(err, PhenotypeError::InvalidToken(_)));
    }
}
