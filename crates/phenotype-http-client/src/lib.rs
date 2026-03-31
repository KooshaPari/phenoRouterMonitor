//! # Phenotype HTTP Client
//!
//! Shared HTTP client patterns with retry, timeout, and observability.

mod builder;
mod error;
mod interceptors;

pub use builder::{ClientBuilder, RetryConfig};
pub use error::{HttpClientError, HttpResult};

use reqwest::{Client, Method, RequestBuilder, Response};
use serde::Serialize;
use std::time::Duration;

/// HTTP client wrapper with shared defaults.
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Creates a new client with default settings.
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    /// Creates a client using the provided builder.
    pub fn from_builder(builder: ClientBuilder) -> Self {
        Self {
            client: builder.build().expect("failed to build HTTP client"),
        }
    }

    /// Perform a GET request.
    pub async fn get(&self, url: &str) -> HttpResult<Response> {
        self.request(Method::GET, url).send().await
    }

    /// Perform a POST request with JSON body.
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> HttpResult<Response> {
        self.request(Method::POST, url)
            .json(body)
            .send()
            .await
    }

    /// Perform a PUT request with JSON body.
    pub async fn put_json<T: Serialize>(&self, url: &str, body: &T) -> HttpResult<Response> {
        self.request(Method::PUT, url)
            .json(body)
            .send()
            .await
    }

    /// Perform a DELETE request.
    pub async fn delete(&self, url: &str) -> HttpResult<Response> {
        self.request(Method::DELETE, url).send().await
    }

    /// Generic request builder.
    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client.request(method, url)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = HttpClient::new();
        assert!(client.client.timeout().is_some());
    }

    #[tokio::test]
    async fn test_get_request_builds() {
        let client = HttpClient::new();
        let rb = client.request(Method::GET, "https://example.com");
        assert!(rb.try_build().is_ok());
    }
}
