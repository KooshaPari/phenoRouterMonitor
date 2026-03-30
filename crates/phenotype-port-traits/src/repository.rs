//! Generic repository port for CRUD operations on aggregates.
//!
//! The Repository pattern abstracts data access, allowing domain logic to remain
//! independent of storage implementation details.

use async_trait::async_trait;
use std::fmt::Debug;

/// Errors that can occur during repository operations.
#[derive(Debug, Clone)]
pub enum RepositoryError {
    NotFound,
    AlreadyExists,
    InvalidData(String),
    StorageError(String),
    SerializationError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Entity not found"),
            RepositoryError::AlreadyExists => write!(f, "Entity already exists"),
            RepositoryError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            RepositoryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            RepositoryError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

/// Generic repository port for CRUD operations on aggregates.
///
/// # Type Parameters
///
/// - `T`: The aggregate root type being stored. Must be `Send + Sync`.
/// - `Id`: The identifier type for the aggregate. Must be `Send + Sync + Clone`.
///
/// # Example
///
/// ```ignore
/// struct User {
///     id: String,
///     name: String,
/// }
///
/// impl Repository<User, String> for PostgresUserRepository {
///     type Error = RepositoryError;
///
///     async fn find_by_id(&self, id: &String) -> Result<Option<User>, Self::Error> {
///         // Query database...
///     }
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait Repository<T: Send + Sync, Id: Send + Sync + Clone>: Send + Sync + Debug {
    /// Error type returned by repository operations.
    type Error: std::error::Error + Send + Sync + Debug;

    /// Retrieve an aggregate by its identifier.
    ///
    /// Returns `Ok(None)` if the aggregate does not exist.
    async fn find_by_id(&self, id: &Id) -> Result<Option<T>, Self::Error>;

    /// Retrieve all aggregates.
    ///
    /// # Note
    ///
    /// Implementations should consider pagination for large datasets.
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;

    /// Persist an aggregate (insert or update).
    async fn save(&self, entity: &T) -> Result<(), Self::Error>;

    /// Remove an aggregate by its identifier.
    async fn delete(&self, id: &Id) -> Result<(), Self::Error>;

    /// Check if an aggregate exists.
    async fn exists(&self, id: &Id) -> Result<bool, Self::Error>;
}
