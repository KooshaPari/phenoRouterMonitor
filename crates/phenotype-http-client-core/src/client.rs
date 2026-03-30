//! HTTP client implementation using reqwest.

use async_trait::async_trait;
use reqwest::Client as ReqwestClient;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, warn};

use crate::{HttpResponse, HttpTransport, Result, TransportError};

/// Default HTTP client timeout.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP client wrapper around reqwest with retry and error handling.
#[derive(Debug, Clone)]
pub struct HttpClient {
    /// The underlying reqwest client.
    pub client: ReqwestClient,
    /// Request timeout duration.
    pub timeout: Duration,
    /// Default headers to send with each request.
    pub default_headers: HashMap<String, String>,
}

impl HttpClient {
    /// Create a new HTTP client with default settings.
    pub fn new() -> Self {
        Self::with_timeout(DEFAULT_TIMEOUT)
    }

    /// Create a new HTTP client with a custom timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        let client = ReqwestClient::builder()
            .timeout(timeout)
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            timeout,
            default_headers: HashMap::new(),
        }
    }

    /// Set a default header that will be sent with all requests.
    pub fn with_default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Perform a GET request and return the response as JSON.
    pub async fn get(&self, url: &str) -> Result<Value> {
        debug!("GET request to: {}", url);

        let mut request = self.client.get(url);

        // Add default headers
        for (key, value) in &self.default_headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                debug!("GET response status: {}", status);

                if !response.status().is_success() {
                    return Err(TransportError::Server {
                        status,
                        message: format!("HTTP request failed: {}", url),
                    });
                }

                match response.json::<Value>().await {
                    Ok(body) => Ok(body),
                    Err(e) => {
                        warn!("Failed to parse JSON response: {}", e);
                        Err(TransportError::Serialization(format!(
                            "Failed to parse JSON response: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                warn!("GET request failed: {}", e);
                Err(self.convert_reqwest_error(e))
            }
        }
    }

    /// Perform a POST request and return the response as JSON.
    pub async fn post(&self, url: &str, body: Value) -> Result<Value> {
        debug!("POST request to: {}, body: {}", url, body);

        let mut request = self.client.post(url).json(&body);

        // Add default headers
        for (key, value) in &self.default_headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                debug!("POST response status: {}", status);

                if !response.status().is_success() {
                    return Err(TransportError::Server {
                        status,
                        message: format!("HTTP request failed: {}", url),
                    });
                }

                match response.json::<Value>().await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        warn!("Failed to parse JSON response: {}", e);
                        Err(TransportError::Serialization(format!(
                            "Failed to parse JSON response: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                warn!("POST request failed: {}", e);
                Err(self.convert_reqwest_error(e))
            }
        }
    }

    /// Perform a PUT request and return the response as JSON.
    pub async fn put(&self, url: &str, body: Value) -> Result<Value> {
        debug!("PUT request to: {}, body: {}", url, body);

        let mut request = self.client.put(url).json(&body);

        for (key, value) in &self.default_headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                if !response.status().is_success() {
                    return Err(TransportError::Server {
                        status,
                        message: format!("HTTP request failed: {}", url),
                    });
                }

                match response.json::<Value>().await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        warn!("Failed to parse JSON response: {}", e);
                        Err(TransportError::Serialization(format!(
                            "Failed to parse JSON response: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                warn!("PUT request failed: {}", e);
                Err(self.convert_reqwest_error(e))
            }
        }
    }

    /// Perform a DELETE request.
    pub async fn delete(&self, url: &str) -> Result<Value> {
        debug!("DELETE request to: {}", url);

        let mut request = self.client.delete(url);

        for (key, value) in &self.default_headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                if !response.status().is_success() {
                    return Err(TransportError::Server {
                        status,
                        message: format!("HTTP request failed: {}", url),
                    });
                }

                // Handle empty responses
                if response.status() == 204 {
                    Ok(json!({}))
                } else {
                    match response.json::<Value>().await {
                        Ok(result) => Ok(result),
                        Err(e) => {
                            warn!("Failed to parse JSON response: {}", e);
                            Err(TransportError::Serialization(format!(
                                "Failed to parse JSON response: {}",
                                e
                            )))
                        }
                    }
                }
            }
            Err(e) => {
                warn!("DELETE request failed: {}", e);
                Err(self.convert_reqwest_error(e))
            }
        }
    }

    /// Convert reqwest errors to TransportError.
    fn convert_reqwest_error(&self, err: reqwest::Error) -> TransportError {
        if err.is_timeout() {
            TransportError::Timeout(format!("HTTP request timeout: {}", err))
        } else if err.is_connect() {
            TransportError::Connection(format!("Failed to connect: {}", err))
        } else if err.is_request() {
            TransportError::Request(format!("Invalid request: {}", err))
        } else if err.is_status() {
            TransportError::Request(format!("HTTP error: {}", err))
        } else if err.is_redirect() {
            TransportError::Request(format!("Too many redirects: {}", err))
        } else {
            TransportError::Unknown(format!("Network error: {}", err))
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpTransport for HttpClient {
    async fn execute(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
    ) -> Result<HttpResponse> {
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            "HEAD" => self.client.head(url),
            _ => {
                return Err(TransportError::Request(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        // Add default headers first
        for (key, value) in &self.default_headers {
            request_builder = request_builder.header(key.as_str(), value.as_str());
        }

        // Add custom headers (override defaults if provided)
        for (key, value) in headers {
            request_builder = request_builder.header(key.as_str(), value.as_str());
        }

        // Add body if provided
        if let Some(body_bytes) = body {
            request_builder = request_builder.body(body_bytes.to_vec());
        }

        match request_builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let mut response_headers = Vec::new();

                for (key, value) in response.headers() {
                    if let Ok(value_str) = value.to_str() {
                        response_headers.push((key.to_string(), value_str.to_string()));
                    }
                }

                match response.bytes().await {
                    Ok(body_bytes) => Ok(HttpResponse {
                        status,
                        headers: response_headers,
                        body: body_bytes.to_vec(),
                    }),
                    Err(e) => Err(TransportError::Request(format!(
                        "Failed to read response body: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(self.convert_reqwest_error(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();
        assert_eq!(client.timeout, DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_http_client_with_timeout() {
        let timeout = Duration::from_secs(60);
        let client = HttpClient::with_timeout(timeout);
        assert_eq!(client.timeout, timeout);
    }

    #[test]
    fn test_http_client_default() {
        let client = HttpClient::default();
        assert_eq!(client.timeout, DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_http_client_with_default_header() {
        let client = HttpClient::new().with_default_header("Authorization", "Bearer token123");
        assert_eq!(
            client.default_headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_http_client_multiple_default_headers() {
        let client = HttpClient::new()
            .with_default_header("Authorization", "Bearer token123")
            .with_default_header("Content-Type", "application/json");
        assert_eq!(client.default_headers.len(), 2);
        assert!(client.default_headers.contains_key("Authorization"));
        assert!(client.default_headers.contains_key("Content-Type"));
    }
}
