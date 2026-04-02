use std::sync::Arc;
use async_trait::async_trait;
use crate::contract::{Contract, HttpInteraction, Interaction};
use crate::error::{ContractError, Result};

#[async_trait]
pub trait HttpClientPort: Send + Sync {
    async fn execute(&self, request: HttpRequest) -> std::result::Result<HttpResponse, String>;
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct ProviderVerifier {
    client: Arc<dyn HttpClientPort>,
}

impl ProviderVerifier {
    pub fn new(client: Arc<dyn HttpClientPort>) -> Self {
        Self { client }
    }

    pub async fn verify_contract(&self, contract: &Contract, base_url: &str) -> Result<()> {
        for interaction in &contract.interactions {
            self.verify_interaction(interaction, base_url).await?;
        }
        Ok(())
    }

    pub async fn verify_http_interactions(
        &self,
        interactions: &[HttpInteraction],
        base_url: &str,
    ) -> Result<()> {
        for interaction in interactions {
            self.verify_http_interaction(interaction, base_url).await?;
        }
        Ok(())
    }

    pub async fn verify_interaction(&self, interaction: &Interaction, base_url: &str) -> Result<()> {
        let mut uri = format!("{}{}", base_url.trim_end_matches('/'), interaction.path);
        if !interaction.query_params.is_empty() {
            let query_string = interaction
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            uri = format!("{}?{}", uri, query_string);
        }

        let request = HttpRequest {
            method: interaction.method.clone(),
            uri,
            headers: interaction.request_headers.clone(),
            body: interaction.request_body.as_ref().map(|b| b.to_string().into_bytes()),
        };

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| ContractError::ProviderVerificationFailed(e))?;

        if response.status != interaction.response_status {
            return Err(ContractError::InteractionMismatch {
                expected: interaction.response_status.to_string(),
                actual: response.status.to_string(),
            });
        }

        if let Some(expected_body) = &interaction.response_body {
            let actual_body: serde_json::Value = serde_json::from_slice(&response.body)
                .map_err(|e| ContractError::ProviderVerificationFailed(format!("invalid JSON: {}", e)))?;
            if expected_body != &actual_body {
                return Err(ContractError::InteractionMismatch {
                    expected: expected_body.to_string(),
                    actual: actual_body.to_string(),
                });
            }
        }

        Ok(())
    }

    pub async fn verify_http_interaction(
        &self,
        interaction: &HttpInteraction,
        base_url: &str,
    ) -> Result<()> {
        let mut uri = format!("{}{}", base_url.trim_end_matches('/'), interaction.request.path);
        if let Some(ref query) = interaction.request.query {
            let query_string = query.0
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            uri = format!("{}?{}", uri, query_string);
        }

        let request = HttpRequest {
            method: interaction.request.method.clone(),
            uri,
            headers: interaction.request.headers.clone(),
            body: interaction.request.body.as_ref().map(|b| b.to_string().into_bytes()),
        };

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| ContractError::ProviderVerificationFailed(e))?;

        if response.status != interaction.response.status {
            return Err(ContractError::InteractionMismatch {
                expected: interaction.response.status.to_string(),
                actual: response.status.to_string(),
            });
        }

        if let Some(ref expected_body) = interaction.response.body {
            let actual_body: serde_json::Value = serde_json::from_slice(&response.body)
                .map_err(|e| ContractError::ProviderVerificationFailed(format!("invalid JSON: {}", e)))?;
            if expected_body != &actual_body {
                return Err(ContractError::InteractionMismatch {
                    expected: expected_body.to_string(),
                    actual: actual_body.to_string(),
                });
            }
        }

        Ok(())
    }
}

pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClientPort for ReqwestHttpClient {
    async fn execute(&self, request: HttpRequest) -> std::result::Result<HttpResponse, String> {
        let mut req = match request.method.to_uppercase().as_str() {
            "GET" => self.client.get(&request.uri),
            "POST" => self.client.post(&request.uri),
            "PUT" => self.client.put(&request.uri),
            "DELETE" => self.client.delete(&request.uri),
            "PATCH" => self.client.patch(&request.uri),
            "HEAD" => self.client.head(&request.uri),
            _ => return Err(format!("unsupported method: {}", request.method)),
        };

        for (key, value) in &request.headers {
            req = req.header(key, value);
        }

        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }

        let response = req.send().await.map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().await.map_err(|e| e.to_string())?.to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{HttpInteraction, HttpRequest as ContractHttpRequest, HttpResponse as ContractHttpResponse, ProviderState};

    struct MockClient {
        response: HttpResponse,
    }

    impl MockClient {
        fn new(response: HttpResponse) -> Self {
            Self { response }
        }
    }

    #[async_trait]
    impl HttpClientPort for MockClient {
        async fn execute(&self, _request: HttpRequest) -> std::result::Result<HttpResponse, String> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_verify_interaction_success() {
        let response = HttpResponse {
            status: 200,
            headers: vec![],
            body: r#"{"users":[]}"#.as_bytes().to_vec(),
        };
        let client = Arc::new(MockClient::new(response));
        let verifier = ProviderVerifier::new(client);

        let interaction = Interaction::new("test", "GET", "/api/users")
            .with_response_body(serde_json::json!({"users": []}));

        let result = verifier.verify_interaction(&interaction, "http://localhost:8080").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_interaction_status_mismatch() {
        let response = HttpResponse {
            status: 500,
            headers: vec![],
            body: vec![],
        };
        let client = Arc::new(MockClient::new(response));
        let verifier = ProviderVerifier::new(client);

        let interaction = Interaction::new("test", "GET", "/api/users")
            .with_status(200);

        let result = verifier.verify_interaction(&interaction, "http://localhost:8080").await;
        assert!(matches!(result, Err(ContractError::InteractionMismatch { .. })));
    }

    #[tokio::test]
    async fn test_verify_http_interaction() {
        let response = HttpResponse {
            status: 200,
            headers: vec![],
            body: r#"{"data":"test"}"#.as_bytes().to_vec(),
        };
        let client = Arc::new(MockClient::new(response));
        let verifier = ProviderVerifier::new(client);

        let interaction = HttpInteraction {
            description: "test http".to_string(),
            provider_state: None,
            request: ContractHttpRequest {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query: None,
                headers: vec![],
                body: None,
            },
            response: ContractHttpResponse {
                status: 200,
                headers: vec![],
                body: Some(serde_json::json!({"data": "test"})),
            },
        };

        let result = verifier.verify_http_interaction(&interaction, "http://localhost:8080").await;
        assert!(result.is_ok());
    }
}
