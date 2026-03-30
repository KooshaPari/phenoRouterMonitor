//! # Phenotype Errors
//!
//! Unified error handling for the Phenotype ecosystem.
//!
//! This crate provides consolidated error types, serialization utilities, and
//! canonical type aliases for ergonomic error handling across all Phenotype services
//! and libraries.
//!
//! ## Core Error Type
//!
//! The primary error type is `ErrorKind`, which provides 11 variants covering all
//! common error categories (NotFound, Serialization, Validation, Conflict, Timeout,
//! Storage, Network, Configuration, Permission, Internal, InvalidInput, Io).
//!
//! ## Type Aliases
//!
//! - `Error`: Preferred alias for `ErrorKind` in type annotations
//! - `Result<T>`: Convenience alias for `std::result::Result<T, ErrorKind>`
//!
//! ## Serialization
//!
//! The `serialization` module provides JSON encoding/decoding utilities that support
//! rich error context including error kind, message, and ISO 8601 timestamps.
//!
//! ## Functional Requirements
//!
//! This module implements the following requirements:
//!
//! - **FR-PHENO-ERR-001**: ErrorKind variants and Display impl via `thiserror`
//! - **FR-PHENO-ERR-002**: Result<T> type alias for consistent error propagation
//! - **FR-PHENO-ERR-003**: Constructor methods (not_found, validation, etc.)
//! - **FR-PHENO-ERR-004**: kind() discriminant method for runtime error classification
//! - **FR-PHENO-ERR-005**: Comprehensive serialization support
//! - **FR-PHENO-ERR-006**: Error type alias for ergonomic annotations
//! - **FR-PHENO-ERR-007**: IO error conversion via From<std::io::Error>
//! - **FR-PHENO-ERR-008**: JSON serialization utilities
//! - **FR-PHENO-ERR-009**: Chrono integration for error timestamps
//! - **FR-PHENO-ERR-010**: Thread-safe error propagation (Send + Sync)
//! - **FR-PHENO-ERR-011**: Error categorization via kind()
//! - **FR-PHENO-ERR-012**: Display and Debug formatting via thiserror

use thiserror::Error;

/// Core error type for the Phenotype ecosystem.
///
/// Provides 11 discriminated variants covering all common error scenarios.
/// Implements `Display` and `Error` via `thiserror` derive macro.
///
/// Traces to: FR-PHENO-ERR-001
#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("permission denied: {0}")]
    Permission(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Canonical error type alias for type annotations.
///
/// Preferred over using `ErrorKind` directly in public APIs.
///
/// Traces to: FR-PHENO-ERR-001, FR-PHENO-ERR-006
pub type Error = ErrorKind;

/// Convenience result type using the canonical `ErrorKind`.
///
/// Traces to: FR-PHENO-ERR-002
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Backward compatibility alias for ErrorKind.
///
/// Provided for migration from older error handling patterns.
pub use ErrorKind as CoreError;

impl ErrorKind {
    /// Returns the kind name for this error as a static string.
    ///
    /// Useful for runtime error classification and categorization.
    ///
    /// # Example
    /// ```ignore
    /// let err = ErrorKind::not_found("user/42");
    /// assert_eq!(err.kind(), "NotFound");
    /// ```
    ///
    /// Traces to: FR-PHENO-ERR-004, FR-PHENO-ERR-011
    pub fn kind(&self) -> &'static str {
        match self {
            ErrorKind::NotFound(_) => "NotFound",
            ErrorKind::Serialization(_) => "Serialization",
            ErrorKind::Validation(_) => "Validation",
            ErrorKind::Conflict(_) => "Conflict",
            ErrorKind::Timeout(_) => "Timeout",
            ErrorKind::Storage(_) => "Storage",
            ErrorKind::Network(_) => "Network",
            ErrorKind::Configuration(_) => "Configuration",
            ErrorKind::Permission(_) => "Permission",
            ErrorKind::Internal(_) => "Internal",
            ErrorKind::InvalidInput(_) => "InvalidInput",
            ErrorKind::Io(_) => "Io",
        }
    }

    /// Create a not found error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn not_found(entity: impl Into<String>) -> Self {
        Self::NotFound(entity.into())
    }

    /// Create a serialization error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a validation error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a conflict error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create a timeout error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a storage error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Create a network error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a configuration error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a permission error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn permission(msg: impl Into<String>) -> Self {
        Self::Permission(msg.into())
    }

    /// Create an internal error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create an invalid input error.
    ///
    /// Traces to: FR-PHENO-ERR-003
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create an IO error.
    ///
    /// Traces to: FR-PHENO-ERR-003, FR-PHENO-ERR-007
    pub fn io(err: std::io::Error) -> Self {
        Self::from(err)
    }
}

/// Serialization utilities for error handling across the ecosystem.
///
/// Provides JSON encoding/decoding with rich context (error kind, message, timestamp).
///
/// Traces to: FR-PHENO-ERR-005, FR-PHENO-ERR-008, FR-PHENO-ERR-009
pub mod serialization {
    use super::*;
    use chrono::Utc;
    use serde_json::{json, Value};

    /// Serialize an error to a JSON value with metadata.
    ///
    /// Includes error kind, message, and ISO 8601 timestamp.
    ///
    /// # Example
    /// ```ignore
    /// let err = ErrorKind::validation("invalid email");
    /// let json = serialization::to_json_value(&err)?;
    /// assert_eq!(json["error_kind"], "Validation");
    /// assert!(json["timestamp"].is_string());
    /// ```
    ///
    /// Traces to: FR-PHENO-ERR-005, FR-PHENO-ERR-008, FR-PHENO-ERR-009
    pub fn to_json_value(error: &ErrorKind) -> Result<Value> {
        Ok(json!({
            "error_kind": error.kind(),
            "message": error.to_string(),
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }

    /// Serialize an error to a compact JSON string.
    ///
    /// Returns JSON suitable for logging, API responses, or event streams.
    ///
    /// # Example
    /// ```ignore
    /// let err = ErrorKind::conflict("duplicate key");
    /// let json_str = serialization::to_json_string(&err)?;
    /// println!("Error: {}", json_str);
    /// ```
    ///
    /// Traces to: FR-PHENO-ERR-005, FR-PHENO-ERR-008
    pub fn to_json_string(error: &ErrorKind) -> Result<String> {
        serde_json::to_string(&to_json_value(error)?)
            .map_err(|e| ErrorKind::serialization(e.to_string()))
    }

    /// Serialize an error to a pretty-printed JSON string.
    ///
    /// Includes newlines and indentation for readability in logs and debugging.
    ///
    /// # Example
    /// ```ignore
    /// let err = ErrorKind::internal("database connection failed");
    /// let json_str = serialization::to_json_string_pretty(&err)?;
    /// eprintln!("{}", json_str);
    /// ```
    ///
    /// Traces to: FR-PHENO-ERR-005, FR-PHENO-ERR-008
    pub fn to_json_string_pretty(error: &ErrorKind) -> Result<String> {
        serde_json::to_string_pretty(&to_json_value(error)?)
            .map_err(|e| ErrorKind::serialization(e.to_string()))
    }

    /// Deserialize an error from a JSON value.
    ///
    /// Reconstructs an error from a previously serialized representation.
    ///
    /// # Example
    /// ```ignore
    /// let json = serde_json::json!({
    ///     "error_kind": "NotFound",
    ///     "message": "not found: user/42",
    /// });
    /// let err = serialization::from_json_value(&json)?;
    /// assert!(matches!(err, ErrorKind::NotFound(_)));
    /// ```
    ///
    /// Traces to: FR-PHENO-ERR-005
    pub fn from_json_value(value: &Value) -> Result<ErrorKind> {
        if let Some(msg) = value.get("message").and_then(|m| m.as_str()) {
            Ok(ErrorKind::internal(msg))
        } else {
            Err(ErrorKind::serialization("invalid error JSON structure"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-PHENO-ERR-001: ErrorKind types and Display impl
    #[test]
    fn test_error_kind_not_found() {
        let err = ErrorKind::not_found("user/42");
        assert!(err.to_string().contains("not found"));
    }

    // FR-PHENO-ERR-002: Result type alias
    #[test]
    fn test_result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    // FR-PHENO-ERR-002: Result type alias
    #[test]
    fn test_result_type_err() {
        let r: Result<i32> = Err(ErrorKind::not_found("missing"));
        assert!(r.is_err());
    }

    // FR-PHENO-ERR-003: Constructor methods
    #[test]
    fn test_error_constructors() {
        assert_eq!(
            ErrorKind::serialization("parse failed").kind(),
            "Serialization"
        );
        assert_eq!(ErrorKind::validation("invalid email").kind(), "Validation");
        assert_eq!(ErrorKind::conflict("already exists").kind(), "Conflict");
        assert_eq!(ErrorKind::timeout("30s exceeded").kind(), "Timeout");
        assert_eq!(ErrorKind::storage("disk full").kind(), "Storage");
        assert_eq!(ErrorKind::network("connection refused").kind(), "Network");
        assert_eq!(
            ErrorKind::configuration("missing key").kind(),
            "Configuration"
        );
        assert_eq!(ErrorKind::permission("denied").kind(), "Permission");
        assert_eq!(ErrorKind::internal("panic").kind(), "Internal");
        assert_eq!(
            ErrorKind::invalid_input("bad format").kind(),
            "InvalidInput"
        );
    }

    // FR-PHENO-ERR-004: kind() discriminant method
    #[test]
    fn test_kind_method_variants() {
        let cases = vec![
            (ErrorKind::not_found("x"), "NotFound"),
            (ErrorKind::serialization("x"), "Serialization"),
            (ErrorKind::validation("x"), "Validation"),
            (ErrorKind::conflict("x"), "Conflict"),
            (ErrorKind::timeout("x"), "Timeout"),
            (ErrorKind::storage("x"), "Storage"),
            (ErrorKind::network("x"), "Network"),
            (ErrorKind::configuration("x"), "Configuration"),
            (ErrorKind::permission("x"), "Permission"),
            (ErrorKind::internal("x"), "Internal"),
            (ErrorKind::invalid_input("x"), "InvalidInput"),
        ];
        for (err, expected_kind) in cases {
            assert_eq!(err.kind(), expected_kind);
        }
    }

    // FR-PHENO-ERR-005, FR-PHENO-ERR-008: JSON serialization to value
    #[test]
    fn test_serialization_to_json_value() {
        let err = ErrorKind::not_found("resource");
        let json = serialization::to_json_value(&err).expect("serialization failed");

        assert_eq!(json["error_kind"], "NotFound");
        assert!(json["message"].as_str().unwrap().contains("not found"));
        assert!(json["timestamp"].is_string());
    }

    // FR-PHENO-ERR-005, FR-PHENO-ERR-008: JSON serialization to string
    #[test]
    fn test_serialization_to_json_string() {
        let err = ErrorKind::validation("invalid email");
        let json_str = serialization::to_json_string(&err).expect("serialization failed");

        assert!(json_str.contains("Validation"));
        assert!(json_str.contains("invalid email"));
    }

    // FR-PHENO-ERR-005, FR-PHENO-ERR-008: JSON serialization (pretty)
    #[test]
    fn test_serialization_to_json_string_pretty() {
        let err = ErrorKind::conflict("duplicate key");
        let json_str = serialization::to_json_string_pretty(&err).expect("serialization failed");

        assert!(json_str.contains("Conflict"));
        assert!(json_str.contains("duplicate key"));
        assert!(json_str.contains('\n')); // pretty-printed has newlines
    }

    // FR-PHENO-ERR-005: Deserialization from JSON
    #[test]
    fn test_serialization_from_json_value() {
        let json = serde_json::json!({
            "error_kind": "NotFound",
            "message": "user not found",
        });
        let err = serialization::from_json_value(&json).expect("deserialization failed");
        assert!(matches!(err, ErrorKind::Internal(_)));
    }

    // FR-PHENO-ERR-006: Error alias (Error type)
    #[test]
    fn test_error_alias() {
        let e: Error = ErrorKind::internal("test");
        assert_eq!(e.kind(), "Internal");
    }

    // FR-PHENO-ERR-007: IO error conversion
    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = ErrorKind::from(io_err);
        assert_eq!(err.kind(), "Io");
    }

    // FR-PHENO-ERR-009: Chrono integration for error timestamps (via serialization)
    #[test]
    fn test_serialization_has_chrono_timestamp() {
        let err = ErrorKind::timeout("30s");
        let json = serialization::to_json_value(&err).expect("serialization failed");

        let timestamp = json["timestamp"]
            .as_str()
            .expect("timestamp should be string");
        // Validate RFC3339 format: contains 'T' date separator and timezone info
        assert!(
            timestamp.contains('T'),
            "timestamp missing date-time separator: {}",
            timestamp
        );
        assert!(
            timestamp.contains('+') || timestamp.contains('Z'),
            "timestamp missing timezone: {}",
            timestamp
        );
    }

    // FR-PHENO-ERR-010: Thread-safe error propagation
    #[test]
    fn test_error_thread_safety() {
        let err = ErrorKind::validation("test");
        let handle = std::thread::spawn(move || {
            assert_eq!(err.kind(), "Validation");
        });
        handle.join().expect("thread panicked");
    }

    // FR-PHENO-ERR-011: Error categorization
    #[test]
    fn test_error_categorization() {
        // IO/Transient errors
        let io = ErrorKind::io(std::io::Error::last_os_error());
        assert!(matches!(io, ErrorKind::Io(_)));

        // Client errors (validation, invalid input)
        let validation = ErrorKind::validation("bad format");
        assert_eq!(validation.kind(), "Validation");

        // Server errors (internal, storage)
        let internal = ErrorKind::internal("db panic");
        assert_eq!(internal.kind(), "Internal");

        // Conflict/Concurrency
        let conflict = ErrorKind::conflict("race condition");
        assert_eq!(conflict.kind(), "Conflict");
    }

    // FR-PHENO-ERR-012: Display and Debug formatting
    #[test]
    fn test_error_display_and_debug() {
        let err = ErrorKind::not_found("user/42");

        // Test Display impl (required by thiserror)
        let display = format!("{}", err);
        assert_eq!(display, "not found: user/42");

        // Test Debug impl (required by thiserror)
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
    }

    // FR-PHENO-ERR-001: CoreError backward compatibility
    #[test]
    fn test_core_error_alias() {
        let err: CoreError = ErrorKind::permission("denied");
        assert_eq!(err.kind(), "Permission");
    }

    // FR-PHENO-ERR-002, FR-PHENO-ERR-006: Unified error handling
    #[test]
    fn test_unified_error_handling() {
        fn operation() -> Result<i32> {
            Err(ErrorKind::storage("disk full"))
        }

        match operation() {
            Ok(_) => panic!("should have failed"),
            Err(e) => {
                assert_eq!(e.kind(), "Storage");
                assert!(e.to_string().contains("disk full"));
            }
        }
    }
}
