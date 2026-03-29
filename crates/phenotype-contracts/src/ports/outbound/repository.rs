//! # Repository Port
//!
//! Outbound port for persistent storage operations.

use async_trait::async_trait;
use std::collections::HashMap;

/// Repository port for CRUD and query operations.
#[async_trait]
pub trait RepositoryPort<T>: Send + Sync
where
    T: Send + Sync,
{
    /// Save an entity.
    async fn save(&self, entity: &T) -> Result<(), RepositoryError>;

    /// Find an entity by ID.
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, RepositoryError>;

    /// Delete an entity by ID.
    async fn delete(&self, id: &str) -> Result<(), RepositoryError>;

    /// List all entities with optional pagination.
    async fn list(
        &self,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<T>, RepositoryError>;

    /// Find entities by a query filter.
    async fn find_by(
        &self,
        filter: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<T>, RepositoryError>;

    /// Count total entities matching a filter.
    async fn count(&self, filter: Option<HashMap<String, serde_json::Value>>)
        -> Result<usize, RepositoryError>;

    /// Check if an entity exists.
    async fn exists(&self, id: &str) -> Result<bool, RepositoryError>;
}

/// Repository operation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("duplicate entity: {0}")]
    Duplicate(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("constraint violation: {0}")]
    Constraint(String),

    #[error("operation failed: {0}")]
    Operation(String),
}
