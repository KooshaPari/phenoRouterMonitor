//! Error types for port violations and port-related failures.

use std::fmt;

#[derive(Debug, Clone)]
pub enum PortError {
    ContractViolation { port: String, reason: String },
    AdapterUnavailable { adapter: String, reason: String },
    IncompatibleVersion { port: String, expected: String, actual: String },
    Timeout { name: String, duration_ms: u64 },
    NotFound { resource_type: String, resource_id: String },
    AdapterError { adapter: String, message: String },
    SerializationError { what: String, reason: String },
    PermissionDenied { operation: String, reason: String },
    InvalidConfiguration { name: String, issue: String },
    Internal(String),
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractViolation { port, reason } => {
                write!(f, "port contract violation [{}]: {}", port, reason)
            }
            Self::AdapterUnavailable { adapter, reason } => {
                write!(f, "adapter unavailable [{}]: {}", adapter, reason)
            }
            Self::IncompatibleVersion { port, expected, actual } => {
                write!(f, "incompatible version for port [{}]: expected {}, got {}", port, expected, actual)
            }
            Self::Timeout { name, duration_ms } => {
                write!(f, "timeout waiting for {} ({}ms)", name, duration_ms)
            }
            Self::NotFound { resource_type, resource_id } => {
                write!(f, "{} not found: {}", resource_type, resource_id)
            }
            Self::AdapterError { adapter, message } => {
                write!(f, "adapter error [{}]: {}", adapter, message)
            }
            Self::SerializationError { what, reason } => {
                write!(f, "serialization error [{}]: {}", what, reason)
            }
            Self::PermissionDenied { operation, reason } => {
                write!(f, "permission denied for {}: {}", operation, reason)
            }
            Self::InvalidConfiguration { name, issue } => {
                write!(f, "invalid configuration for {}: {}", name, issue)
            }
            Self::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for PortError {}

pub type Result<T> = std::result::Result<T, PortError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_violation_display() {
        let err = PortError::ContractViolation {
            port: "Repository".to_string(),
            reason: "find() returned invalid data".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "port contract violation [Repository]: find() returned invalid data"
        );
    }

    #[test]
    fn test_port_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PortError>();
    }
}
