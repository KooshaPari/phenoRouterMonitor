use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams(pub Vec<(String, String)>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub query: Option<QueryParams>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>
    pub updated_at: Option<DateTime<Utc>
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpInteraction {
    pub description: String,
    #[serde(default)]
    pub provider_state: Option<ProviderState>,
    pub request: HttpRequest,
    pub response: HttpResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Uuid,
    pub description: String,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub query_params: Vec<(String, String)>,
    #[serde(default)]
    pub request_headers: Vec<(String, String)>,
    #[serde(default)]
    pub request_body: Option<serde_json::Value>,
    pub response_status: u16,
    #[serde(default)]
    pub response_headers: Vec<(String, String)>,
    #[serde(default)]
    pub response_body: Option<serde_json::Value>,
    #[serde(default)]
    pub provider_state: Option<String>,
}

impl Interaction {
    pub fn new(description: &str, method: &str, path: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            query_params: Vec::new(),
            request_headers: Vec::new(),
            request_body: None,
            response_status: 200,
            response_headers: Vec::new(),
            response_body: None,
            provider_state: None,
        }
    }

    pub fn with_request_body(mut self, body: serde_json::Value) -> Self {
        self.request_body = Some(body);
        self
    }

    pub fn with_response_body(mut self, body: serde_json::Value) -> Self {
        self.response_body = Some(body);
        self
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.response_status = status;
        self
    }

    pub fn with_provider_state(mut self, state: &str) -> Self {
        self.provider_state = Some(state.to_string());
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.request_headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_response_header(mut self, key: &str, value: &str) -> Self {
        self.response_headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_query_param(mut self, key: &str, value: &str) -> Self {
        self.query_params.push((key.to_string(), value.to_string()));
        self
    }

    pub fn into_http_interaction(self) -> HttpInteraction {
        let request = HttpRequest {
            method: self.method.clone(),
            path: self.path.clone(),
            query: if self.query_params.is_empty() {
                None
            } else {
                Some(QueryParams(self.query_params))
            },
            headers: self.request_headers,
            body: self.request_body,
        };
        let response = HttpResponse {
            status: self.response_status,
            headers: self.response_headers,
            body: self.response_body,
        };
        HttpInteraction {
            description: self.description,
            provider_state: self.provider_state.map(|name| ProviderState {
                name: Some(name),
                params: None,
            }),
            request,
            response,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub version: String,
    #[serde(default],
    pub created_at: Option<DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub consumer: String,
    pub provider: String,
    pub interactions: Vec<Interaction>,
    #[serde(default)]
    pub metadata: ContractMetadata,
}

impl Contract {
    pub fn new(consumer: &str, provider: &str) -> Self {
        Self {
            consumer: consumer.to_string(),
            provider: provider.to_string(),
            interactions: Vec::new(),
            metadata: ContractMetadata {
                version: "1.0.0".to_string(),
                created_at: Some(Utc::now()),
            },
        }
    }

    pub fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }

    pub fn into_http_interactions(self) -> Vec<HttpInteraction> {
        self.interactions.into_iter().map(|i| i.into_http_interaction()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_builder() {
        let interaction = Interaction::new("test interaction", "GET", "/api/users")
            .with_status(200)
            .with_response_body(serde_json::json!({"users": []}));

        assert_eq!(interaction.method, "GET");
        assert_eq!(interaction.path, "/api/users");
        assert_eq!(interaction.response_status, 200);
        assert!(interaction.response_body.is_some());
    }

    #[test]
    fn test_contract_new() {
        let contract = Contract::new("consumer", "provider");
        assert_eq!(contract.consumer, "consumer");
        assert_eq!(contract.provider, "provider");
        assert!(contract.interactions.is_empty());
    }

    #[test]
    fn test_interaction_into_http_interaction() {
        let interaction = Interaction::new("test", "POST", "/api/data")
            .with_request_body(serde_json::json!({"key": "value"}))
            .with_response_body(serde_json::json!({"status": "ok"}))
            .with_status(201);

        let http: HttpInteraction = interaction.into_http_interaction();
        assert_eq!(http.description, "test");
        assert_eq!(http.request.method, "POST");
        assert_eq!(http.response.status, 201);
    }
}
