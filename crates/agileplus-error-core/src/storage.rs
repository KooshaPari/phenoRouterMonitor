//! Storage error type - persistent storage operations.

use thiserror::Error;

/// Error type for storage and persistence operations.
#[derive(Debug, Clone, Error)]
pub enum StorageError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("storage error: {0}")]
    Internal(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

impl StorageError {
    pub fn not_found(entity: impl Into<String>) -> Self { Self::NotFound(entity.into()) }
    pub fn already_exists(entity: impl Into<String>) -> Self { Self::AlreadyExists(entity.into()) }
    pub fn internal(msg: impl Into<String>) -> Self { Self::Internal(msg.into()) }
    pub fn io(msg: impl Into<String>) -> Self { Self::Io(msg.into()) }
    pub fn serialization(msg: impl Into<String>) -> Self { Self::Serialization(msg.into()) }
    pub fn permission_denied(msg: impl Into<String>) -> Self { Self::PermissionDenied(msg.into()) }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self { Self::Io(e.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() { let err = StorageError::not_found("wp/1"); assert_eq!(err.to_string(), "not found: wp/1"); }

    #[test]
    fn test_already_exists_error() { let err = StorageError::already_exists("project/proj-1"); assert_eq!(err.to_string(), "already exists: project/proj-1"); }

    #[test]
    fn test_internal_error() { let err = StorageError::internal("corrupted data"); assert_eq!(err.to_string(), "storage error: corrupted data"); }

    #[test]
    fn test_io_error() { let err = StorageError::io("disk full"); assert_eq!(err.to_string(), "io error: disk full"); }

    #[test]
    fn test_serialization_error() { let err = StorageError::serialization("invalid utf-8"); assert_eq!(err.to_string(), "serialization error: invalid utf-8"); }

    #[test]
    fn test_permission_denied_error() { let err = StorageError::permission_denied("/var/data"); assert_eq!(err.to_string(), "permission denied: /var/data"); }
}
