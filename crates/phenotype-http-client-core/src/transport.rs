//! HTTP transport implementations and utilities.
//!
//! This module provides traits and adapters for different HTTP client backends.
//! Consumers can implement the `HttpTransport` trait for their preferred HTTP library.

use crate::{error::TransportError, HttpResponse};

/// Core HTTP transport trait. Implementors wrap their preferred HTTP client.
///
/// # Example
///
/// ```ignore
/// use phenotype_http_client_core::{HttpTransport, HttpResponse};
/// use async_trait::async_trait;
///
/// struct MyHttpClient;
///
/// #[async_trait]
/// impl HttpTransport for MyHttpClient {
///     async fn execute(
///         &self,
///         method: &str,
///         url: &str,
///         headers: &[(String, String)],
///         body: Option<&[u8]>,
///     ) -> Result<HttpResponse> {
///         // Implementation here
/// #       todo!()
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait HttpTransport: Send + Sync {
    /// Execute an HTTP request.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
    /// * `url` - Full request URL
    /// * `headers` - Request headers as (name, value) pairs
    /// * `body` - Optional request body
    ///
    /// # Returns
    ///
    /// A `Result<HttpResponse>` containing the response or an error.
    async fn execute(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
    ) -> crate::Result<HttpResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock HTTP transport for testing.
    struct MockTransport {
        status: u16,
        response_body: Vec<u8>,
    }

    #[async_trait::async_trait]
    impl HttpTransport for MockTransport {
        async fn execute(
            &self,
            _method: &str,
            _url: &str,
            _headers: &[(String, String)],
            _body: Option<&[u8]>,
        ) -> crate::Result<HttpResponse> {
            Ok(HttpResponse {
                status: self.status,
                headers: vec![],
                body: self.response_body.clone(),
            })
        }
    }

    #[tokio::test]
    async fn mock_transport_success() {
        let transport = MockTransport {
            status: 200,
            response_body: b"Hello, World!".to_vec(),
        };

        let response = transport
            .execute("GET", "http://example.com", &[], None)
            .await
            .unwrap();

        assert!(response.is_success());
        assert_eq!(response.body_as_str().unwrap(), "Hello, World!");
    }

    #[tokio::test]
    async fn mock_transport_error_status() {
        let transport = MockTransport {
            status: 500,
            response_body: b"Internal Server Error".to_vec(),
        };

        let response = transport
            .execute("GET", "http://example.com", &[], None)
            .await
            .unwrap();

        assert!(!response.is_success());
        assert_eq!(response.status, 500);
    }
}
