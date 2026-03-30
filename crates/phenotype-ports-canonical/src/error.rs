//! Unified error type for port operations.

/// Error type for all port operations.
///
/// Consolidates the separate `inbound::Error` and `outbound::Error` types
/// from `phenotype-contracts` into a single enum.
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    /// Entity or resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Resource already exists (conflict on create).
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Input validation failed.
    #[error("validation: {0}")]
    Validation(String),

    /// Conflict (optimistic concurrency, etc.).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Permission denied / authorization failure.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Connection or transport error.
    #[error("connection: {0}")]
    Connection(String),

    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Unclassified internal error.
    #[error("internal: {0}")]
    Internal(String),
}
