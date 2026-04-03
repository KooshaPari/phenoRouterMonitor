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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_error_invalid_display() {
        let err = QueryError::InvalidQuery("missing filter".into());
        assert_eq!(err.to_string(), "invalid query: missing filter");
    }

    #[test]
    fn query_error_not_found_display() {
        let err = QueryError::NotFound;
        assert_eq!(err.to_string(), "not found");
    }

    #[test]
    fn query_error_internal_display() {
        let err = QueryError::Internal("timeout".into());
        assert_eq!(err.to_string(), "internal error: timeout");
    }

    #[test]
    fn query_error_debug() {
        let err = QueryError::NotFound;
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
    }
}
