//! Error types for the router core.

use thiserror::Error;

/// Result type alias for router operations.
pub type RouterResult<T> = Result<T, RouterError>;

/// Error types for router operations.
#[derive(Debug, Error, Clone)]
pub enum RouterError {
    #[error("No route found for path: {0}")]
    RouteNotFound(String),

    #[error("No healthy backends available for route: {0}")]
    NoHealthyBackends(String),

    #[error("Invalid route configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid path pattern: {0}")]
    InvalidPattern(String),

    #[error("Backend not found: {0}")]
    BackendNotFound(String),

    #[error("Route already exists: {0}")]
    DuplicateRoute(String),

    #[error("Invalid regex pattern: {0}")]
    RegexError(String),

    #[error("Empty backend list for route: {0}")]
    EmptyBackendList(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid matcher strategy: {0}")]
    InvalidMatcher(String),

    #[error("Invalid load balancing strategy: {0}")]
    InvalidBalancer(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_not_found_error() {
        let err = RouterError::RouteNotFound("/api/v1".to_string());
        assert!(err.to_string().contains("No route found"));
    }

    #[test]
    fn test_no_healthy_backends_error() {
        let err = RouterError::NoHealthyBackends("api-route".to_string());
        assert!(err.to_string().contains("No healthy backends"));
    }

    #[test]
    fn test_invalid_config_error() {
        let err = RouterError::InvalidConfig("missing threshold".to_string());
        assert!(err.to_string().contains("Invalid route configuration"));
    }

    #[test]
    fn test_duplicate_route_error() {
        let err = RouterError::DuplicateRoute("api".to_string());
        assert!(err.to_string().contains("Route already exists"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RouterError>();
    }
}
