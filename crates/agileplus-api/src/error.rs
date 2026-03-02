//! API error type mapped to HTTP status codes.
//!
//! Traceability: WP15-T086

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Errors returned by API handlers.
///
/// Each variant maps to a specific HTTP status code and produces a JSON body:
/// `{"error": "<message>"}`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Conflict(String),
    #[error("internal server error")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            ApiError::Internal(m) => {
                tracing::error!("internal API error: {m}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<agileplus_domain::error::DomainError> for ApiError {
    fn from(e: agileplus_domain::error::DomainError) -> Self {
        match e {
            agileplus_domain::error::DomainError::NotFound(m) => ApiError::NotFound(m),
            agileplus_domain::error::DomainError::Conflict(m) => ApiError::Conflict(m),
            agileplus_domain::error::DomainError::InvalidTransition { from, to, reason } => {
                ApiError::Conflict(format!("invalid transition {from} -> {to}: {reason}"))
            }
            other => ApiError::Internal(other.to_string()),
        }
    }
}
