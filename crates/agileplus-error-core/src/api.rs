//! API error type for application layer errors.

use thiserror::Error;

/// Error type for API layer operations.
#[derive(Debug, Clone, Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal server error: {0}")]
    InternalServerError(String),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self { Self::BadRequest(msg.into()) }
    pub fn forbidden(msg: impl Into<String>) -> Self { Self::Forbidden(msg.into()) }
    pub fn not_found(entity: impl Into<String>) -> Self { Self::NotFound(entity.into()) }
    pub fn conflict(msg: impl Into<String>) -> Self { Self::Conflict(msg.into()) }
    pub fn internal_server_error(msg: impl Into<String>) -> Self { Self::InternalServerError(msg.into()) }
    pub fn service_unavailable(msg: impl Into<String>) -> Self { Self::ServiceUnavailable(msg.into()) }
    pub fn validation_error(msg: impl Into<String>) -> Self { Self::ValidationError(msg.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request_error() { let err = ApiError::bad_request("invalid json"); assert_eq!(err.to_string(), "bad request: invalid json"); }

    #[test]
    fn test_unauthorized_error() { let err = ApiError::Unauthorized; assert_eq!(err.to_string(), "unauthorized"); }

    #[test]
    fn test_forbidden_error() { let err = ApiError::forbidden("admin only"); assert_eq!(err.to_string(), "forbidden: admin only"); }

    #[test]
    fn test_not_found_error() { let err = ApiError::not_found("WorkPackage/wp-123"); assert_eq!(err.to_string(), "not found: WorkPackage/wp-123"); }

    #[test]
    fn test_conflict_error() { let err = ApiError::conflict("resource already exists"); assert_eq!(err.to_string(), "conflict: resource already exists"); }

    #[test]
    fn test_internal_server_error() { let err = ApiError::internal_server_error("database connection failed"); assert_eq!(err.to_string(), "internal server error: database connection failed"); }

    #[test]
    fn test_service_unavailable_error() { let err = ApiError::service_unavailable("maintenance in progress"); assert_eq!(err.to_string(), "service unavailable: maintenance in progress"); }

    #[test]
    fn test_validation_error() { let err = ApiError::validation_error("missing required fields"); assert_eq!(err.to_string(), "validation error: missing required fields"); }
}
