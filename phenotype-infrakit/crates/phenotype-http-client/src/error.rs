//! HTTP errors

use thiserror::Error;

/// HTTP error
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("request failed: {0}")]
    RequestFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        HttpError::RequestFailed(e.to_string())
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(err: reqwest::Error) -> Self {
        HttpError::RequestFailed(err.to_string())
    }
}

/// HTTP result
pub type Result<T> = std::result::Result<T, HttpError>;
