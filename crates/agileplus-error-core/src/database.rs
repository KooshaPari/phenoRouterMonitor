//! Database error type for SQLite and storage operations.

use thiserror::Error;

/// Error type for database and storage operations.
#[derive(Debug, Clone, Error)]
pub enum DatabaseError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("database error: {0}")]
    Internal(String),

    #[error("migration error: {0}")]
    Migration(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("transaction error: {0}")]
    Transaction(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("io error: {0}")]
    Io(String),
}

impl DatabaseError {
    /// Create a not found error.
    pub fn not_found(entity: impl Into<String>) -> Self {
        Self::NotFound(entity.into())
    }

    /// Create an already exists error.
    pub fn already_exists(entity: impl Into<String>) -> Self {
        Self::AlreadyExists(entity.into())
    }

    /// Create a constraint violation error.
    pub fn constraint_violation(msg: impl Into<String>) -> Self {
        Self::ConstraintViolation(msg.into())
    }

    /// Create an internal database error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a migration error.
    pub fn migration(msg: impl Into<String>) -> Self {
        Self::Migration(msg.into())
    }

    /// Create a connection error.
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a transaction error.
    pub fn transaction(msg: impl Into<String>) -> Self {
        Self::Transaction(msg.into())
    }

    /// Create a query error.
    pub fn query(msg: impl Into<String>) -> Self {
        Self::Query(msg.into())
    }

    /// Create an IO error.
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }
}

// Conversion from std::io::Error
impl From<std::io::Error> for DatabaseError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = DatabaseError::not_found("WorkPackage/wp-001");
        assert_eq!(err.to_string(), "not found: WorkPackage/wp-001");
    }

    #[test]
    fn test_already_exists_error() {
        let err = DatabaseError::already_exists("User/user-123");
        assert_eq!(err.to_string(), "already exists: User/user-123");
    }

    #[test]
    fn test_constraint_violation_error() {
        let err = DatabaseError::constraint_violation("unique constraint failed");
        assert_eq!(
            err.to_string(),
            "constraint violation: unique constraint failed"
        );
    }

    #[test]
    fn test_migration_error() {
        let err = DatabaseError::migration("migration v3 failed");
        assert_eq!(err.to_string(), "migration error: migration v3 failed");
    }

    #[test]
    fn test_connection_error() {
        let err = DatabaseError::connection("unable to connect to sqlite");
        assert_eq!(
            err.to_string(),
            "connection error: unable to connect to sqlite"
        );
    }

    #[test]
    fn test_transaction_error() {
        let err = DatabaseError::transaction("rollback failed");
        assert_eq!(err.to_string(), "transaction error: rollback failed");
    }

    #[test]
    fn test_query_error() {
        let err = DatabaseError::query("invalid sql");
        assert_eq!(err.to_string(), "query error: invalid sql");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let db_err: DatabaseError = io_err.into();
        matches!(db_err, DatabaseError::Io(_));
    }
}
