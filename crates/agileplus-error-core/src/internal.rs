//! Internal error type for system and unrecoverable errors.

use thiserror::Error;

/// Error type for system-level and unrecoverable errors.
#[derive(Debug, Clone, Error)]
pub enum InternalError {
    #[error("system error: {0}")]
    System(String),

    #[error("panic: {0}")]
    Panic(String),

    #[error("unrecoverable error: {0}")]
    Unrecoverable(String),

    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("deadlock detected: {0}")]
    Deadlock(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("invariant violation: {0}")]
    InvariantViolation(String),
}

impl InternalError {
    pub fn system(msg: impl Into<String>) -> Self { Self::System(msg.into()) }
    pub fn panic(msg: impl Into<String>) -> Self { Self::Panic(msg.into()) }
    pub fn unrecoverable(msg: impl Into<String>) -> Self { Self::Unrecoverable(msg.into()) }
    pub fn resource_exhausted(msg: impl Into<String>) -> Self { Self::ResourceExhausted(msg.into()) }
    pub fn deadlock(msg: impl Into<String>) -> Self { Self::Deadlock(msg.into()) }
    pub fn timeout(msg: impl Into<String>) -> Self { Self::Timeout(msg.into()) }
    pub fn network(msg: impl Into<String>) -> Self { Self::Network(msg.into()) }
    pub fn permission_denied(msg: impl Into<String>) -> Self { Self::PermissionDenied(msg.into()) }
    pub fn not_implemented(msg: impl Into<String>) -> Self { Self::NotImplemented(msg.into()) }
    pub fn invariant_violation(msg: impl Into<String>) -> Self { Self::InvariantViolation(msg.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_error() { let err = InternalError::system("out of memory"); assert_eq!(err.to_string(), "system error: out of memory"); }

    #[test]
    fn test_panic_error() { let err = InternalError::panic("unexpected state"); assert_eq!(err.to_string(), "panic: unexpected state"); }

    #[test]
    fn test_unrecoverable_error() { let err = InternalError::unrecoverable("database corruption detected"); assert_eq!(err.to_string(), "unrecoverable error: database corruption detected"); }

    #[test]
    fn test_resource_exhausted_error() { let err = InternalError::resource_exhausted("file descriptor limit reached"); assert_eq!(err.to_string(), "resource exhausted: file descriptor limit reached"); }

    #[test]
    fn test_deadlock_error() { let err = InternalError::deadlock("mutex deadlock"); assert_eq!(err.to_string(), "deadlock detected: mutex deadlock"); }

    #[test]
    fn test_timeout_error() { let err = InternalError::timeout("operation exceeded 30s"); assert_eq!(err.to_string(), "timeout: operation exceeded 30s"); }

    #[test]
    fn test_network_error() { let err = InternalError::network("connection refused"); assert_eq!(err.to_string(), "network error: connection refused"); }

    #[test]
    fn test_permission_denied_error() { let err = InternalError::permission_denied("access denied to /etc/shadow"); assert_eq!(err.to_string(), "permission denied: access denied to /etc/shadow"); }

    #[test]
    fn test_not_implemented_error() { let err = InternalError::not_implemented("feature X not yet implemented"); assert_eq!(err.to_string(), "not implemented: feature X not yet implemented"); }

    #[test]
    fn test_invariant_violation_error() { let err = InternalError::invariant_violation("expected count > 0"); assert_eq!(err.to_string(), "invariant violation: expected count > 0"); }
}
