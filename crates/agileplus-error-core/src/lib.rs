//! Unified error handling for the AgilePlus ecosystem.
//!
//! This crate consolidates error types scattered across AgilePlus into 5 canonical error types:
//! - `ParseError` — JSON/TOML/YAML parsing
//! - `DatabaseError` — SQLite/storage operations
//! - `ValidationError` — Input validation, constraints
//! - `ConfigError` — Configuration loading/merging
//! - `InternalError` — System/unrecoverable errors
//!
//! Additional domain-specific error types:
//! - `ApiError` — API layer operations
//! - `StorageError` — Persistent storage operations
//! - `SerializationError` — Serialization/deserialization
//! - `DomainError` — Domain layer operations
//! - `SyncError` — P2P and replication operations

pub mod api;
pub mod config;
pub mod database;
pub mod domain;
pub mod internal;
pub mod parse;
pub mod serialization;
pub mod storage;
pub mod sync;
pub mod traits;
pub mod validation;

// Re-exports for convenience
pub use api::ApiError;
pub use config::ConfigError;
pub use database::DatabaseError;
pub use domain::DomainError;
pub use internal::InternalError;
pub use parse::ParseError;
pub use serialization::SerializationError;
pub use storage::StorageError;
pub use sync::SyncError;
pub use traits::{ErrorKindProvider, NotFoundMarker};
pub use validation::ValidationError;

/// Result type alias using AgilePlus canonical error types.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Unified error type that can hold any of the canonical error types.
#[derive(Debug)]
pub enum AgileplusError {
    Parse(ParseError),
    Database(DatabaseError),
    Validation(ValidationError),
    Config(ConfigError),
    Internal(InternalError),
    Api(ApiError),
    Storage(StorageError),
    Serialization(SerializationError),
    Domain(DomainError),
    Sync(SyncError),
}

impl std::fmt::Display for AgileplusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{}", e),
            Self::Database(e) => write!(f, "{}", e),
            Self::Validation(e) => write!(f, "{}", e),
            Self::Config(e) => write!(f, "{}", e),
            Self::Internal(e) => write!(f, "{}", e),
            Self::Api(e) => write!(f, "{}", e),
            Self::Storage(e) => write!(f, "{}", e),
            Self::Serialization(e) => write!(f, "{}", e),
            Self::Domain(e) => write!(f, "{}", e),
            Self::Sync(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AgileplusError {}

// Conversions from individual error types
impl From<ParseError> for AgileplusError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<DatabaseError> for AgileplusError {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<ValidationError> for AgileplusError {
    fn from(e: ValidationError) -> Self {
        Self::Validation(e)
    }
}

impl From<ConfigError> for AgileplusError {
    fn from(e: ConfigError) -> Self {
        Self::Config(e)
    }
}

impl From<InternalError> for AgileplusError {
    fn from(e: InternalError) -> Self {
        Self::Internal(e)
    }
}

impl From<ApiError> for AgileplusError {
    fn from(e: ApiError) -> Self {
        Self::Api(e)
    }
}

impl From<StorageError> for AgileplusError {
    fn from(e: StorageError) -> Self {
        Self::Storage(e)
    }
}

impl From<SerializationError> for AgileplusError {
    fn from(e: SerializationError) -> Self {
        Self::Serialization(e)
    }
}

impl From<DomainError> for AgileplusError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

impl From<SyncError> for AgileplusError {
    fn from(e: SyncError) -> Self {
        Self::Sync(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_creation() {
        let e = StorageError::not_found("wp/1");
        assert_eq!(e.to_string(), "not found: wp/1");
    }

    #[test]
    fn test_parse_error_creation() {
        let e = ParseError::json("invalid json");
        assert_eq!(e.to_string(), "json parsing error: invalid json");
    }

    #[test]
    fn test_database_error_creation() {
        let e = DatabaseError::not_found("WorkPackage/wp-001");
        assert_eq!(e.to_string(), "not found: WorkPackage/wp-001");
    }

    #[test]
    fn test_validation_error_creation() {
        let e = ValidationError::missing_field("email");
        assert_eq!(e.to_string(), "missing required field: email");
    }

    #[test]
    fn test_config_error_creation() {
        let e = ConfigError::file_not_found("config.toml");
        assert_eq!(e.to_string(), "config file not found: config.toml");
    }

    #[test]
    fn test_internal_error_creation() {
        let e = InternalError::system("out of memory");
        assert_eq!(e.to_string(), "system error: out of memory");
    }

    #[test]
    fn test_api_error_creation() {
        let e = ApiError::not_found("Resource");
        assert_eq!(e.to_string(), "not found: Resource");
    }

    #[test]
    fn test_domain_error_creation() {
        let e = DomainError::invalid_state("transition not allowed");
        assert_eq!(e.to_string(), "invalid state: transition not allowed");
    }

    #[test]
    fn test_sync_error_creation() {
        let e = SyncError::conflict("merge conflict");
        assert_eq!(e.to_string(), "conflict: merge conflict");
    }

    #[test]
    fn test_unified_error_from_parse_error() {
        let parse_err = ParseError::json("bad json");
        let unified: AgileplusError = parse_err.into();
        assert!(matches!(unified, AgileplusError::Parse(_)));
    }

    #[test]
    fn test_unified_error_from_database_error() {
        let db_err = DatabaseError::not_found("table");
        let unified: AgileplusError = db_err.into();
        assert!(matches!(unified, AgileplusError::Database(_)));
    }

    #[test]
    fn test_unified_error_display() {
        let err = AgileplusError::Parse(ParseError::json("test"));
        assert_eq!(err.to_string(), "json parsing error: test");
    }

    #[test]
    fn test_serde_json_error_to_parse_error() {
        let serde_err = serde_json::from_str::<i32>("not json").unwrap_err();
        let parse_err: ParseError = serde_err.into();
        matches!(parse_err, ParseError::Json(_));
    }

    #[test]
    fn test_io_error_to_database_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let db_err: DatabaseError = io_err.into();
        matches!(db_err, DatabaseError::Io(_));
    }

    #[test]
    fn test_io_error_to_storage_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let storage_err: StorageError = io_err.into();
        matches!(storage_err, StorageError::Io(_));
    }

    #[test]
    fn test_all_canonical_error_types() {
        // Verify all 5 canonical types are accessible
        let _parse = ParseError::json("test");
        let _db = DatabaseError::not_found("test");
        let _validation = ValidationError::missing_field("test");
        let _config = ConfigError::file_not_found("test");
        let _internal = InternalError::system("test");

        // Verify additional types are accessible
        let _api = ApiError::bad_request("test");
        let _storage = StorageError::not_found("test");
        let _serialization = SerializationError::json("test");
        let _domain = DomainError::invalid_state("test");
        let _sync = SyncError::conflict("test");
    }
}
