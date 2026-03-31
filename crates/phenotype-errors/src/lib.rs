//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("permission denied: {0}")]
    Permission(String),

    #[error("I/O error: {0}")]
    Io(String),
}

impl ErrorKind {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn permission(msg: impl Into<String>) -> Self {
        Self::Permission(msg.into())
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::Serialization(_) => "Serialization",
            Self::Validation(_) => "Validation",
            Self::Conflict(_) => "Conflict",
            Self::Internal(_) => "Internal",
            Self::Permission(_) => "Permission",
            Self::Io(_) => "Io",
        }
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

pub type Error = ErrorKind;

pub type Result<T> = std::result::Result<T, ErrorKind>;

pub use ErrorKind as CoreError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_not_found() {
        let err = ErrorKind::not_found("user/42");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let r: Result<i32> = Err(ErrorKind::not_found("missing"));
        assert!(r.is_err());
    }

    #[test]
    fn test_error_constructors() {
        assert_eq!(
            ErrorKind::serialization("parse failed").kind(),
            "Serialization"
        );
        assert_eq!(ErrorKind::validation("invalid email").kind(), "Validation");
        assert_eq!(ErrorKind::conflict("already exists").kind(), "Conflict");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = ErrorKind::from(io_err);
        assert_eq!(err.kind(), "Io");
    }

    #[test]
    fn test_error_alias() {
        let e: Error = ErrorKind::internal("test");
        assert_eq!(e.kind(), "Internal");
    }

    #[test]
    fn test_core_error_alias() {
        let err: CoreError = ErrorKind::permission("denied");
        assert_eq!(err.kind(), "Permission");
    }
}
