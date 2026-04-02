//! HTTP errors

use thiserror::Error;

/// HTTP error
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("request failed: {0}")]
    RequestFailed(String),
}

/// HTTP result
pub type Result<T> = std::result::Result<T, HttpError>;
