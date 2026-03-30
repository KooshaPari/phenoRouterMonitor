//! Error types for router core
//!
//! Provides comprehensive error handling for routing failures, configuration issues,
//! and backend communication errors.

use thiserror::Error;

/// Result type alias for router operations
pub type Result<T> = std::result::Result<T, RouterError>;

/// Comprehensive error type for router operations
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Route not found for path: {path}")]
    RouteNotFound { path: String },

    #[error("No healthy backends available for service: {service}")]
    NoHealthyBackends { service: String },

    #[error("Backend communication error: {0}")]
    BackendError(#[from] reqwest::Error),

    #[error("Invalid path pattern: {pattern}: {reason}")]
    InvalidPattern { pattern: String, reason: String },

    #[error("Timeout forwarding request to {backend}")]
    RequestTimeout { backend: String },

    #[error("Invalid route configuration: {0}")]
    InvalidRoute(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-002 (Error handling)
    #[test]
    fn test_error_display() {
        let err = RouterError::RouteNotFound {
            path: "/test".to_string(),
        };
        assert!(err.to_string().contains("/test"));
    }

    // Traces to: FR-ROUTER-002
    #[test]
    fn test_no_healthy_backends_error() {
        let err = RouterError::NoHealthyBackends {
            service: "api".to_string(),
        };
        assert!(err.to_string().contains("api"));
    }
}
