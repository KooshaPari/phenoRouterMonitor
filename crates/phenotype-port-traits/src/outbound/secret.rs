//! Secret storage port for sensitive data.

use async_trait::async_trait;

/// Secret port for accessing sensitive data.
#[async_trait]
pub trait SecretPort: Send + Sync {
    /// Get a secret by name.
    async fn get(&self, name: &str) -> Result<Option<String>, SecretError>;

    /// Set a secret.
    async fn set(&self, name: &str, value: &str) -> Result<(), SecretError>;

    /// Delete a secret.
    async fn delete(&self, name: &str) -> Result<(), SecretError>;

    /// List all secret names.
    async fn list(&self) -> Result<Vec<String>, SecretError>;
}

/// Versioned secret port for secrets with version history.
#[async_trait]
pub trait VersionedSecretPort: Send + Sync {
    /// Get a specific version of a secret.
    async fn get_version(&self, name: &str, version: u32) -> Result<Option<String>, SecretError>;

    /// Get the latest version of a secret.
    async fn get_latest(&self, name: &str) -> Result<Option<(String, u32)>, SecretError>;

    /// Set a new version of a secret.
    async fn set_versioned(&self, name: &str, value: &str) -> Result<u32, SecretError>;

    /// List all versions of a secret.
    async fn list_versions(&self, name: &str) -> Result<Vec<u32>, SecretError>;

    /// Delete a specific version of a secret.
    async fn delete_version(&self, name: &str, version: u32) -> Result<(), SecretError>;
}

/// Secret rotation port for automated secret rotation.
#[async_trait]
pub trait SecretRotator: Send + Sync {
    /// Rotate a secret (create new version, optionally disable old).
    async fn rotate(&self, name: &str) -> Result<u32, SecretError>;

    /// Check if rotation is needed based on age.
    async fn needs_rotation(&self, name: &str, max_age_days: u32) -> Result<bool, SecretError>;
}

/// Secret errors.
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("operation failed: {0}")]
    OperationFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_error_not_found_display() {
        let err = SecretError::NotFound("api-key".into());
        assert_eq!(err.to_string(), "not found: api-key");
    }

    #[test]
    fn secret_error_permission_denied_display() {
        let err = SecretError::PermissionDenied("admin required".into());
        assert_eq!(err.to_string(), "permission denied: admin required");
    }

    #[test]
    fn secret_error_connection_display() {
        let err = SecretError::Connection("vault unreachable".into());
        assert_eq!(err.to_string(), "connection error: vault unreachable");
    }

    #[test]
    fn secret_error_operation_failed_display() {
        let err = SecretError::OperationFailed("rotation failed".into());
        assert_eq!(err.to_string(), "operation failed: rotation failed");
    }

    #[test]
    fn secret_error_internal_display() {
        let err = SecretError::Internal("unexpected".into());
        assert_eq!(err.to_string(), "internal error: unexpected");
    }

    #[test]
    fn secret_error_debug() {
        let err = SecretError::NotFound("key".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("key"));
    }
}
