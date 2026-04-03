//! HTTP client for Phenotype

use thiserror::Error;

/// HTTP client error types
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("invalid url: {0}")]
    InvalidUrl(String),
}

/// HTTP client
pub struct HttpClient;

impl HttpClient {
    /// Create a new HTTP client
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
