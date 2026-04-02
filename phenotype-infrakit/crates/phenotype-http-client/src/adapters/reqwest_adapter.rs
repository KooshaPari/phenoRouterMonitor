//! Reqwest HTTP adapter

use crate::{error::Result, types::*, HttpClientError};
use reqwest::Client;

/// Reqwest HTTP client adapter
#[derive(Debug, Clone)]
pub struct ReqwestAdapter {
    client: Client,
}

impl ReqwestAdapter {
    /// Create a new adapter
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Send a request
    pub async fn send(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        };

        let mut req_builder = self.client.request(method, &request.url);

        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| HttpClientError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| HttpClientError::DeserializationError(e.to_string()))?;

        Ok(HttpResponse::new(status, body))
    }
}

impl Default for ReqwestAdapter {
    fn default() -> Self {
        Self::new()
    }
}
