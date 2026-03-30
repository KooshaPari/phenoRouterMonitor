//! HTTP client wrapper with auth headers and error handling.
//!
//! Provides a reusable HTTP client wrapper that handles:
//! - Authorization header injection
//! - JSON serialization/deserialization
//! - Error handling with anyhow
//! - Reusable across Plane.so, GitHub, and other API clients
//!
//! # Example
//!
//! ```rust,ignore
//! use phenotype_http_client::HttpClient;
//!
//! let client = HttpClient::new("https://api.example.com")
//!     .with_auth("Bearer", "token")
//!     .build();
//!
//! let response: MyType = client.get("/endpoint").await?;
//! ```

use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};

/// HTTP client with auth and JSON support.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
    auth_type: Option<String>,
    auth_value: Option<String>,
}

impl HttpClient {
    /// Create a new HTTP client.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
            auth_type: None,
            auth_value: None,
        }
    }

    /// Set Bearer token auth.
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.auth_type = Some("Bearer".to_string());
        self.auth_value = Some(token.into());
        self
    }

    /// Set custom auth header.
    pub fn with_auth(mut self, auth_type: impl Into<String>, value: impl Into<String>) -> Self {
        self.auth_type = Some(auth_type.into());
        self.auth_value = Some(value.into());
        self
    }

    /// Build the client.
    pub fn build(self) -> Self {
        self
    }

    /// Build with a custom reqwest client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    fn auth_header(&self) -> Option<(String, String)> {
        self.auth_type.as_ref().zip(self.auth_value.as_ref()).map(|(t, v)| {
            if t == "Bearer" {
                (format!("Authorization"), format!("Bearer {}", v))
            } else {
                (t.clone(), v.clone())
            }
        })
    }

    /// GET request with JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        if let Some((key, val)) = self.auth_header() {
            req = req.header(&key, &val);
        }
        let resp = req.send().await.context("GET request failed")?;
        self.handle_response(resp).await
    }

    /// POST request with JSON body and response.
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.post(&url).json(body);
        if let Some((key, val)) = self.auth_header() {
            req = req.header(&key, &val);
        }
        let resp = req.send().await.context("POST request failed")?;
        self.handle_response(resp).await
    }

    /// PATCH request with JSON body and response.
    pub async fn patch<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.patch(&url).json(body);
        if let Some((key, val)) = self.auth_header() {
            req = req.header(&key, &val);
        }
        let resp = req.send().await.context("PATCH request failed")?;
        self.handle_response(resp).await
    }

    async fn handle_response<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP error {status}: {body}");
        }
        resp.json().await.context("Failed to parse JSON response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpClient::new("https://api.example.com")
            .with_bearer_token("test-token")
            .build();
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_auth_header_bearer() {
        let client = HttpClient::new("https://api.example.com")
            .with_bearer_token("my-token")
            .build();
        let header = client.auth_header().unwrap();
        assert_eq!(header.0, "Authorization");
        assert_eq!(header.1, "Bearer my-token");
    }
}
