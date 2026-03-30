//! # Phenotype Error Core
//!
//! Shared error types and utilities for the Phenotype ecosystem.
//!
//! This crate provides common error kinds and conversions used across
//! phenotype crates for consistent error handling.
//!
//! ## Features
//!
//! - Common error kinds (NotFound, Serialization, etc.)
//! - Conversions from standard library errors
//! - Chain error support for error tracing
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_error_core::ErrorKind;
//!
//! fn example() -> Result<(), ErrorKind> {
//!     Err(ErrorKind::not_found("resource"))
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::time::SystemTime;
use thiserror::Error;

/// Timestamp for when the error occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTimestamp {
    /// Seconds since Unix epoch.
    pub seconds: u64,
    /// Nanoseconds within the second.
    pub nanos: u32,
}

impl ErrorTimestamp {
    /// Create a new timestamp from the current system time.
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            seconds: now.as_secs(),
            nanos: now.subsec_nanos(),
        }
    }
}

impl fmt::Display for ErrorTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.seconds)
    }
}

/// Common error kinds used across phenotype crates.
///
/// This enum provides a standardized set of error variants that
/// can be used consistently across the ecosystem.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error(transparent)]
pub struct ErrorKind(pub Box<ErrorKindInner>);

impl ErrorKind {
    /// Create a new `NotFound` error.
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::NotFound(resource.into())))
    }

    /// Create a new `Serialization` error.
    pub fn serialization(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Serialization(message.into())))
    }

    /// Create a new `Validation` error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Validation(message.into())))
    }

    /// Create a new `Timeout` error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Timeout(message.into())))
    }

    /// Create a new `Internal` error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Internal(message.into())))
    }

    /// Create a new `Storage` error.
    pub fn storage(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Storage(message.into())))
    }

    /// Create a new `Connection` error.
    pub fn connection(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Connection(message.into())))
    }

    /// Create a new `Config` error.
    pub fn config(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Config(message.into())))
    }

    /// Create a new `PermissionDenied` error.
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::PermissionDenied(message.into())))
    }

    /// Create a new `Conflict` error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::Conflict(message.into())))
    }

    /// Create a new `AlreadyExists` error.
    pub fn already_exists(resource: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::AlreadyExists(resource.into())))
    }

    /// Create a new `ParseError` error.
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::ParseError(message.into())))
    }

    /// Create a new `NetworkError` error.
    pub fn network(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::NetworkError(message.into())))
    }

    /// Create a new `AuthError` error.
    pub fn auth(message: impl Into<String>) -> Self {
        Self(Box::new(ErrorKindInner::AuthError(message.into())))
    }

    /// Get the error kind name.
    pub fn kind(&self) -> &'static str {
        match self.0.as_ref() {
            ErrorKindInner::NotFound(_) => "NotFound",
            ErrorKindInner::Serialization(_) => "Serialization",
            ErrorKindInner::Validation(_) => "Validation",
            ErrorKindInner::Timeout(_) => "Timeout",
            ErrorKindInner::Internal(_) => "Internal",
            ErrorKindInner::Storage(_) => "Storage",
            ErrorKindInner::Connection(_) => "Connection",
            ErrorKindInner::Config(_) => "Config",
            ErrorKindInner::PermissionDenied(_) => "PermissionDenied",
            ErrorKindInner::Conflict(_) => "Conflict",
            ErrorKindInner::AlreadyExists(_) => "AlreadyExists",
            ErrorKindInner::ParseError(_) => "ParseError",
            ErrorKindInner::NetworkError(_) => "NetworkError",
            ErrorKindInner::AuthError(_) => "AuthError",
        }
    }
}

/// Inner error kinds with their associated messages.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ErrorKindInner {
    /// Resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Conflict with existing resource.
    #[error("conflict: {0}")]
    Conflict(String),

    /// Resource already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Parse error.
    #[error("parse error: {0}")]
    ParseError(String),

    /// Network error.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Authentication/authorization error.
    #[error("auth error: {0}")]
    AuthError(String),
}

impl From<IoError> for ErrorKind {
    fn from(err: IoError) -> Self {
        use std::io::ErrorKind as IoKind;
        match err.kind() {
            IoKind::NotFound => Self::not_found(err.to_string()),
            IoKind::PermissionDenied => Self::permission_denied(err.to_string()),
            IoKind::AlreadyExists => Self::already_exists(err.to_string()),
            _ => Self::internal(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization(err.to_string())
    }
}

impl From<regex::Error> for ErrorKind {
    fn from(err: regex::Error) -> Self {
        Self::parse_error(err.to_string())
    }
}

impl From<&str> for ErrorKind {
    fn from(s: &str) -> Self {
        Self::internal(s.to_string())
    }
}

impl From<String> for ErrorKind {
    fn from(s: String) -> Self {
        Self::internal(s)
    }
}

/// Extension trait for adding context to errors.
pub trait ErrorExt {
    /// Add a backtrace to the error.
    fn with_backtrace(self) -> ErrorContext
    where
        Self: Sized;

    /// Chain this error with a message.
    fn chain(self, message: impl Into<String>) -> ErrorContext
    where
        Self: Sized;
}

/// A structured error with optional backtrace and context.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The underlying error kind.
    pub kind: ErrorKind,
    /// Optional backtrace.
    pub backtrace: Option<String>,
    /// Optional chain message.
    pub chain: Option<String>,
    /// When the error occurred.
    pub timestamp: ErrorTimestamp,
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref chain) = self.chain {
            write!(f, "{}: {}: {}", self.timestamp, chain, self.kind)
        } else {
            write!(f, "{}: {}", self.timestamp, self.kind)
        }
    }
}

impl StdError for ErrorContext {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl From<ErrorKind> for ErrorContext {
    fn from(kind: ErrorKind) -> Self {
        Self {
            kind,
            backtrace: None,
            chain: None,
            timestamp: ErrorTimestamp::now(),
        }
    }
}

impl ErrorExt for ErrorKind {
    fn with_backtrace(self) -> ErrorContext {
        let backtrace = Backtrace::capture();
        ErrorContext {
            kind: self,
            backtrace: Some(backtrace.to_string()),
            chain: None,
            timestamp: ErrorTimestamp::now(),
        }
    }

    fn chain(self, message: impl Into<String>) -> ErrorContext {
        ErrorContext {
            kind: self,
            backtrace: None,
            chain: Some(message.into()),
            timestamp: ErrorTimestamp::now(),
        }
    }
}

/// Alias for call sites that import `phenotype_error_core::Error`.
pub type Error = ErrorKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_not_found() {
        let err = ErrorKind::not_found("user");
        assert_eq!(err.kind(), "NotFound");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_error_kind_from_io() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: ErrorKind = io_err.into();
        assert_eq!(err.kind(), "NotFound");
    }

    #[test]
    fn test_error_context_chain() {
        let err = ErrorKind::not_found("user");
        let ctx = err.chain("while fetching");
        assert!(ctx.to_string().contains("while fetching"));
    }
}
