//! # phenotype-http-client-core
//!
//! HTTP client core for the Phenotype ecosystem.
//!
//! ## Features
//!
//! - HTTP client abstraction with retry support
//! - Request/Response types
//! - Error handling with detailed diagnostics

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, HttpClientError>;

/// HTTP client errors
#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("request failed: {0}")]
    RequestFailed(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("rate limited: retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("unauthorized")]
    Unauthorized,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Patch => write!(f, "PATCH"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl Request {
    /// Create a new GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: Method::Get,
            url: url.into(),
            headers: Vec::new(),
            body: None,
            timeout_secs: Some(30),
        }
    }

    /// Create a new POST request with JSON body
    pub fn post_json(url: impl Into<String>, body: impl Serialize) -> Self {
        Self {
            method: Method::Post,
            url: url.into(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: serde_json::to_string(&body).ok(),
            timeout_secs: Some(30),
        }
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Response {
    /// Check if status is success (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Parse body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_str(&self.body)
            .map_err(|e| HttpClientError::ParseError(e.to_string()))
    }
}

/// HTTP client trait for abstraction
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    /// Execute a request
    async fn execute(&self, request: Request) -> Result<Response>;

    /// Execute with automatic retry
    async fn execute_with_retry(&self, request: Request, max_retries: u32) -> Result<Response>;
}
