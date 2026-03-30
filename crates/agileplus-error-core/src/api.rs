use crate::traits::NotFoundMarker;
use thiserror::Error;

/// HTTP-oriented API errors for AgilePlus HTTP adapters (no framework types here).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl NotFoundMarker for ApiError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

impl From<ApiError> for phenotype_error_core::ErrorKind {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::NotFound(m) => Self::not_found(m),
            ApiError::BadRequest(m) => Self::validation(m),
            ApiError::Internal(m) => Self::internal(m),
        }
    }
}
