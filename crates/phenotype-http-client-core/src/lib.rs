//! # HTTP Client Core
//!
//! Core traits and retry logic for HTTP clients in the Phenotype ecosystem.
//! Minimal, no external HTTP crate dependency — consumers bring their own (reqwest, hyper, etc).

pub mod auth;
pub mod error;
pub mod retry;

pub use auth::{AuthCredentials, AuthMiddleware};
pub use error::{ErrorKind, TransportError};
pub use retry::RetryPolicy;

/// Result type for HTTP transport operations.
pub type Result<T> = std::result::Result<T, TransportError>;

/// Core HTTP transport trait. Implementors wrap their preferred HTTP client.
#[async_trait::async_trait]
pub trait HttpTransport: Send + Sync {
    async fn execute(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
    ) -> Result<HttpResponse>;
}

/// Simplified HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn body_as_str(&self) -> std::result::Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }
}
