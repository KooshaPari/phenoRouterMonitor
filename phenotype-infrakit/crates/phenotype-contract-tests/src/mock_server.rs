#[cfg(feature = "mock-server")]
use wiremock::{
    Match, Mock, MockServer, Request as WireRequest, ResponseTemplate,
    http::{Method, Uri, HeaderName, HeaderValue},
};
#[cfg(feature = "mock-server")]
use std::str::FromStr;

use crate::contract::HttpInteraction;
use crate::error::{ContractError, Result};

#[cfg(feature = "mock-server")]
pub struct MockServerAdapter {
    server: MockServer,
}

#[cfg(feature = "mock-server")]
impl MockServerAdapter {
    pub async fn start(port: u16) -> Result<Self> {
        let server = MockServer::start(format!("127.0.0.1:{}", port))
            .await
            .map_err(|e| ContractError::MockServerError(e.to_string()))?;
        Ok(Self { server })
    }

    pub fn url(&self) -> String {
        self.server.uri()
    }

    pub fn register(&mut self, interaction: &HttpInteraction) -> Result<()> {
        let method = Method::from_str(&interaction.request.method)
            .map_err(|e| ContractError::MockServerError(format!("invalid method: {}", e)))?;

        let uri = Uri::from_str(&interaction.request.path)
            .map_err(|e| ContractError::MockServerError(format!("invalid path: {}", e)))?;

        let mut mock = Mock::given(method).and(wiremock::matchers::path(uri.to_string().as_ref()));

        if let Some(ref body) = interaction.request.body {
            let body_str = body.to_string();
            mock = mock.and(wiremock::matchers::body_string(&body_str));
        }

        for (key, value) in &interaction.request.headers {
            let header_name = HeaderName::from_str(key)
                .map_err(|e| ContractError::MockServerError(format!("invalid header: {}", e)))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| ContractError::MockServerError(format!("invalid header value: {}", e)))?;
            mock = mock.and(wiremock::matchers::header(key.as_str(), value.as_str()));
        }

        let response = ResponseTemplate::new(interaction.response.status);

        if let Some(ref body) = interaction.response.body {
            let body_str = body.to_string();
            Mock::given(move |req: &WireRequest| {
                req.method == method && req.url.path() == uri.path()
            })
            .respond_with(response.set_body_string(body_str))
            .mount(&self.server)
            .map_err(|e| ContractError::MockServerError(e.to_string()))?;
        } else {
            Mock::given(move |req: &WireRequest| {
                req.method == method && req.url.path() == uri.path()
            })
            .respond_with(response)
            .mount(&self.server)
            .map_err(|e| ContractError::MockServerError(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(feature = "mock-server")]
impl Drop for MockServerAdapter {
    fn drop(&mut self) {
        // wiremock handles cleanup
    }
}

#[cfg(not(feature = "mock-server"))]
pub struct MockServerAdapter;

#[cfg(not(feature = "mock-server"))]
impl MockServerAdapter {
    pub async fn start(_port: u16) -> Result<Self> {
        Err(ContractError::MockServerError(
            "mock-server feature not enabled".to_string(),
        ))
    }

    pub fn url(&self) -> String {
        String::new()
    }

    pub fn register(&mut self, _interaction: &HttpInteraction) -> Result<()> {
        Err(ContractError::MockServerError(
            "mock-server feature not enabled".to_string(),
        ))
    }
}

#[cfg(feature = "mock-server")]
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::WireMock;

    #[tokio::test]
    async fn test_mock_server_start() {
        let adapter = MockServerAdapter::start(0).await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_mock_server_register() {
        let mut adapter = MockServerAdapter::start(0).await.unwrap();

        let interaction = HttpInteraction {
            description: "test".to_string(),
            provider_state: None,
            request: crate::contract::HttpRequest {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query: None,
                headers: vec![],
                body: None,
            },
            response: crate::contract::HttpResponse {
                status: 200,
                headers: vec![],
                body: Some(serde_json::json!({"status": "ok"})),
            },
        };

        let result = adapter.register(&interaction);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_server_url() {
        let adapter = MockServerAdapter::start(0).await.unwrap();
        assert!(adapter.url().starts_with("http://127.0.0.1:"));
    }
}
