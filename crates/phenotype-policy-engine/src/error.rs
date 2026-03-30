//! Error types for the policy engine.

use phenotype_error_core::ErrorKind;
use thiserror::Error;

/// Errors that can occur during policy operations.
#[derive(Error, Debug)]
pub enum PolicyEngineError {
    /// Failed to compile a regex pattern.
    #[error("Failed to compile regex pattern '{pattern}': {source}")]
    RegexCompilationError {
        pattern: String,
        source: regex::Error,
    },

    /// Policy evaluation encountered an error.
    #[error("Policy evaluation error: {0}")]
    EvaluationError(String),

    /// Invalid policy configuration.
    #[error("Invalid policy configuration: {0}")]
    InvalidConfiguration(String),

    /// Policy not found by name.
    #[error("Policy '{name}' not found")]
    PolicyNotFound { name: String },

    /// Failed to serialize/deserialize policy data.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Failed to load policy from file.
    #[error("Failed to load policy from file: {0}")]
    LoadError(String),

    /// Generic error with message.
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for PolicyEngineError {
    fn from(err: serde_json::Error) -> Self {
        PolicyEngineError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for PolicyEngineError {
    fn from(err: toml::de::Error) -> Self {
        PolicyEngineError::SerializationError(err.to_string())
    }
}

impl From<regex::Error> for PolicyEngineError {
    fn from(err: regex::Error) -> Self {
        PolicyEngineError::RegexCompilationError {
            pattern: err.to_string(),
            source: err,
        }
    }
}

impl From<std::io::Error> for PolicyEngineError {
    fn from(err: std::io::Error) -> Self {
        PolicyEngineError::LoadError(err.to_string())
    }
}

impl From<PolicyEngineError> for ErrorKind {
    fn from(e: PolicyEngineError) -> Self {
        match e {
            PolicyEngineError::RegexCompilationError { pattern, source } => Self::validation(
                format!("regex compile failed for pattern '{pattern}': {source}"),
            ),
            PolicyEngineError::EvaluationError(m) => Self::internal(m),
            PolicyEngineError::InvalidConfiguration(m) => Self::validation(m),
            PolicyEngineError::PolicyNotFound { name } => {
                Self::not_found(format!("policy '{name}'"))
            }
            PolicyEngineError::SerializationError(m) => Self::serialization(m),
            PolicyEngineError::LoadError(m) => Self::storage(m),
            PolicyEngineError::Other(m) => Self::internal(m),
        }
    }
}

#[cfg(test)]
mod error_kind_tests {
    use super::*;

    // Traces to: FR-PHENO-001
    #[test]
    fn policy_not_found_maps_to_not_found_kind() {
        let e = PolicyEngineError::PolicyNotFound {
            name: "default".into(),
        };
        let k: ErrorKind = e.into();
        assert_eq!(k.kind(), "NotFound");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn serialization_error_maps_to_serialization_kind() {
        let e = PolicyEngineError::SerializationError("oops".into());
        let k: ErrorKind = e.into();
        assert_eq!(k.kind(), "Serialization");
    }
}
