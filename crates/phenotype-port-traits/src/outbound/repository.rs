//! Repository port for persistence operations.

use async_trait::async_trait;
use std::time::Duration;

/// Entity identifier type.
pub trait EntityId: Send + Sync + std::fmt::Display + std::fmt::Debug {}

/// Entity type.
pub trait Entity: Send + Sync {
    type Id: EntityId;
    fn id(&self) -> &Self::Id;
}

/// Repository port for entity persistence operations.
#[async_trait]
pub trait Repository<E: Entity, I: EntityId>: Send + Sync {
    /// Save an entity (insert or update).
    async fn save(&self, entity: &E) -> Result<(), RepositoryError>;

    /// Find an entity by its ID.
    async fn find_by_id(&self, id: &I) -> Result<Option<E>, RepositoryError>;

    /// Delete an entity by its ID.
    async fn delete(&self, id: &I) -> Result<(), RepositoryError>;

    /// List all entities with optional pagination.
    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<E>, RepositoryError>;

    /// Count total entities.
    async fn count(&self) -> Result<usize, RepositoryError>;
}

/// Unit of Work pattern for transactional operations.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin a new transaction.
    async fn begin(&mut self) -> Result<(), RepositoryError>;

    /// Commit the current transaction.
    async fn commit(&mut self) -> Result<(), RepositoryError>;

    /// Rollback the current transaction.
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}

/// Repository errors.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("not found: {entity} {id}")]
    NotFound { entity: String, id: String },

    #[error("duplicate: {entity} {id}")]
    Duplicate { entity: String, id: String },

    #[error("connection error: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("transaction error: {0}")]
    Transaction(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// Cache TTL constants for common use cases.
pub mod ttl {
    use std::time::Duration;

    pub const ONE_MINUTE: Duration = Duration::from_secs(60);
    pub const FIVE_MINUTES: Duration = Duration::from_secs(300);
    pub const FIFTEEN_MINUTES: Duration = Duration::from_secs(900);
    pub const THIRTY_MINUTES: Duration = Duration::from_secs(1800);
    pub const ONE_HOUR: Duration = Duration::from_secs(3600);
    pub const ONE_DAY: Duration = Duration::from_secs(86400);
    pub const ONE_WEEK: Duration = Duration::from_secs(604800);
}
