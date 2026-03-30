//! Error types and HTTP response handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// API error type
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Route error: {0}")]
    RouteError(#[from] phenotype_router_core::RouterError),
}

/// Result type for API operations
pub type Result<T> = std::result::Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::ConfigError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::InternalError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::RouteError(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-013 (API error handling)
    #[test]
    fn test_not_found_error() {
        let err = ApiError::NotFound("users".to_string());
        assert!(err.to_string().contains("users"));
    }

    // Traces to: FR-ROUTER-013
    #[test]
    fn test_bad_request_error() {
        let err = ApiError::BadRequest("invalid input".to_string());
        assert!(err.to_string().contains("invalid input"));
    }

    // Traces to: FR-ROUTER-013
    #[test]
    fn test_internal_error() {
        let err = ApiError::InternalError("database error".to_string());
        assert!(err.to_string().contains("database error"));
    }
}
