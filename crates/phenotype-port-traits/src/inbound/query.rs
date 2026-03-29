//! Query handler port for CQRS query processing.

use async_trait::async_trait;

/// Marker trait for query types.
pub trait Query: Send + Sync + Sized {}

/// Query handler port for processing read operations (CQRS).
#[async_trait]
pub trait QueryHandler<Q: Query, R: Send + Sync>: Send + Sync {
    /// Handle the given query and return results.
    async fn handle(&self, query: Q) -> Result<R, QueryError>;
}

/// Errors that can occur during query handling.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("invalid query: {0}")]
    InvalidQuery(String),

    #[error("not found")]
    NotFound,

    #[error("internal error: {0}")]
    Internal(String),
}
