use crate::contract::HttpInteraction;
use crate::error::{ContractError, Result};

#[cfg(feature = "mock-server")]
use crate::mock_server::MockServerAdapter;

#[derive(Debug)]
pub struct ConsumerTest {
    pub consumer_name: String,
    pub provider_name: String,
}

impl ConsumerTest {
    pub fn new(consumer_name: &str, provider_name: &str) -> Self {
        Self {
            consumer_name: consumer_name.to_string(),
            provider_name: provider_name.to_string(),
        }
    }

    #[cfg(feature = "mock-server")]
    pub fn verify_interaction(
        &self,
        interaction: &HttpInteraction,
        mock_server: &MockServerAdapter,
    ) -> Result<()> {
        mock_server.register(interaction)
    }

    #[cfg(not(feature = "mock-server"))]
    pub fn verify_interaction(
        &self,
        _interaction: &HttpInteraction,
        _mock_server: &MockServerAdapter,
    ) -> Result<()> {
        Err(ContractError::MockServerError(
            "mock-server feature not enabled".to_string(),
        ))
    }

    pub fn build_interaction(description: &str) -> InteractionBuilder {
        InteractionBuilder::new(description)
    }
}

pub struct InteractionBuilder {
    description: String,
    method: String,
    path: String,
    request_body: Option<serde_json::Value>,
    response_status: u16,
    response_body: Option<serde_json::Value>,
    provider_state: Option<String>,
}

impl InteractionBuilder {
    fn new(description: &str) -> Self {
        Self {
            description: description.to_string(),
            method: "GET".to_string(),
            path: "/".to_string(),
            request_body: None,
            response_status: 200,
            response_body: None,
            provider_state: None,
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    pub fn request_body(mut self, body: serde_json::Value) -> Self {
        self.request_body = Some(body);
        self
    }

    pub fn response_status(mut self, status: u16) -> Self {
        self.response_status = status;
        self
    }

    pub fn response_body(mut self, body: serde_json::Value) -> Self {
        self.response_body = Some(body);
        self
    }

    pub fn provider_state(mut self, state: &str) -> Self {
        self.provider_state = Some(state.to_string());
        self
    }

    pub fn build(self) -> HttpInteraction {
        HttpInteraction {
            description: self.description,
            provider_state: self
                .provider_state
                .map(|name| crate::contract::ProviderState {
                    name: Some(name),
                    params: None,
                }),
            request: crate::contract::HttpRequest {
                method: self.method,
                path: self.path,
                query: None,
                headers: vec![],
                body: self.request_body,
            },
            response: crate::contract::HttpResponse {
                status: self.response_status,
                headers: vec![],
                body: self.response_body,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_builder() {
        let interaction = ConsumerTest::build_interaction("get users")
            .method("GET")
            .path("/api/users")
            .response_status(200)
            .response_body(serde_json::json!({"users": []}))
            .build();

        assert_eq!(interaction.description, "get users");
        assert_eq!(interaction.request.method, "GET");
        assert_eq!(interaction.request.path, "/api/users");
        assert_eq!(interaction.response.status, 200);
    }

    #[test]
    fn test_consumer_test_new() {
        let test = ConsumerTest::new("my-consumer", "my-provider");
        assert_eq!(test.consumer_name, "my-consumer");
        assert_eq!(test.provider_name, "my-provider");
    }
}
