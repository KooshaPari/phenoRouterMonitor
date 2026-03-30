//! # HTTP Client Core
//!
//! Core traits and retry logic for HTTP clients in the Phenotype ecosystem.
//! Minimal, no external HTTP crate dependency — consumers bring their own (reqwest, hyper, etc).
//!
//! # Features
//!
//! - **HttpTransport trait**: Core abstraction for HTTP implementations
//! - **Retry policy**: Exponential backoff with jitter
//! - **Error types**: Unified error handling across transports
//! - **Auth helpers**: Support for Bearer, API Key, and Basic auth

pub mod auth;
pub mod error;
pub mod retry;
pub mod transport;

pub use auth::{AuthCredentials, AuthMiddleware};
pub use error::{ErrorKind, TransportError};
pub use retry::{retry_with_policy, RetryPolicy};
pub use transport::HttpTransport;

/// Result type for HTTP transport operations.
pub type Result<T> = std::result::Result<T, TransportError>;

/// Simplified HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Check if the HTTP status indicates success (200-299).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Convert the response body to a string.
    pub fn body_as_str(&self) -> std::result::Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    /// Get a header value by name (case-insensitive).
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let lower_name = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower_name)
            .map(|(_, v)| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_response_success() {
        let response = HttpResponse {
            status: 200,
            headers: vec![],
            body: b"OK".to_vec(),
        };
        assert!(response.is_success());
    }

    #[test]
    fn http_response_redirect() {
        let response = HttpResponse {
            status: 301,
            headers: vec![],
            body: vec![],
        };
        assert!(!response.is_success());
    }

    #[test]
    fn http_response_error() {
        let response = HttpResponse {
            status: 500,
            headers: vec![],
            body: vec![],
        };
        assert!(!response.is_success());
    }

    #[test]
    fn http_response_get_header_case_insensitive() {
        let response = HttpResponse {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Content-Length".to_string(), "42".to_string()),
            ],
            body: vec![],
        };

        assert_eq!(response.get_header("content-type"), Some("application/json"));
        assert_eq!(response.get_header("Content-Type"), Some("application/json"));
        assert_eq!(response.get_header("CONTENT-TYPE"), Some("application/json"));
        assert_eq!(response.get_header("content-length"), Some("42"));
        assert_eq!(response.get_header("missing"), None);
    }

    #[test]
    fn http_response_body_as_str() {
        let response = HttpResponse {
            status: 200,
            headers: vec![],
            body: b"Hello, World!".to_vec(),
        };
        assert_eq!(response.body_as_str().unwrap(), "Hello, World!");
    }

    #[test]
    fn http_response_body_invalid_utf8() {
        let response = HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![0xFF, 0xFE],
        };
        assert!(response.body_as_str().is_err());
    }
}
