//! Error types for the Router API.
//!
//! Provides structured error handling for router API operations.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

/// Router API error type.
#[derive(Debug, Error)]
pub enum RouterApiError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("server error: {0}")]
    ServerError(String),

    #[error("internal error: {0}")]
    InternalError(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}

impl IntoResponse for RouterApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RouterApiError::InvalidConfig(msg) => (StatusCode::BAD_REQUEST, msg),
            RouterApiError::AgentNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            RouterApiError::ServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            RouterApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            RouterApiError::Timeout(msg) => (StatusCode::REQUEST_TIMEOUT, msg),
            RouterApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

pub type RouterResult<T> = Result<T, RouterApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RouterApiError::AgentNotFound("agent-1".to_string());
        assert_eq!(err.to_string(), "agent not found: agent-1");
    }

    #[test]
    fn test_error_config_invalid() {
        let err = RouterApiError::InvalidConfig("missing port".to_string());
        assert_eq!(err.to_string(), "invalid configuration: missing port");
    }

    #[test]
    fn test_error_server_error() {
        let err = RouterApiError::ServerError("bind failed".to_string());
        assert_eq!(err.to_string(), "server error: bind failed");
    }

    #[test]
    fn test_error_timeout() {
        let err = RouterApiError::Timeout("health check timeout".to_string());
        assert_eq!(err.to_string(), "timeout: health check timeout");
    }

    #[test]
    fn test_error_validation() {
        let err = RouterApiError::ValidationError("invalid agent name".to_string());
        assert_eq!(err.to_string(), "validation error: invalid agent name");
    }

    #[test]
    fn test_error_internal() {
        let err = RouterApiError::InternalError("unknown".to_string());
        assert_eq!(err.to_string(), "internal error: unknown");
    }
}
