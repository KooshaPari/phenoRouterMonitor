use crate::contract::{Contract, HttpInteraction, Interaction, ProviderState, QueryParams};
use crate::error::{ContractError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactMetadata {
    pub version: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactConsumer {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactProvider {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactInteraction {
    pub description: String,
    #[serde(default)]
    pub provider_state: Option<String>,
    pub request: PactRequest,
    pub response: PactResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub matching_rules: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactResponse {
    pub status: u16,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub matching_rules: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactFile {
    pub consumer: PactConsumer,
    pub provider: PactProvider,
    pub interactions: Vec<PactInteraction>,
    #[serde(default)]
    pub metadata: PactMetadata,
}

impl PactFile {
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| ContractError::PactError(format!("failed to parse JSON: {}", e)))
    }

    pub fn from_json_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ContractError::PactError(format!("failed to read file: {}", e)))?;
        Self::from_json(&content)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ContractError::PactError(format!("failed to serialize: {}", e)))
    }

    pub fn to_interactions(&self) -> Vec<HttpInteraction> {
        self.interactions
            .iter()
            .map(|i| {
                let query_params = i
                    .request
                    .query
                    .as_ref()
                    .map(|q| {
                        q.split('&')
                            .filter_map(|pair| {
                                let mut parts = pair.split('=');
                                Some((parts.next()?.to_string(), parts.next()?.to_string()))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                HttpInteraction {
                    description: i.description.clone(),
                    provider_state: i.provider_state.as_ref().map(|name| ProviderState {
                        name: Some(name.clone()),
                        params: None,
                    }),
                    request: crate::contract::HttpRequest {
                        method: i.request.method.clone(),
                        path: i.request.path.clone(),
                        query: if query_params.is_empty() {
                            None
                        } else {
                            Some(QueryParams(query_params))
                        },
                        headers: i
                            .request
                            .headers
                            .as_ref()
                            .map(|h| h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default(),
                        body: i.request.body.clone(),
                    },
                    response: crate::contract::HttpResponse {
                        status: i.response.status,
                        headers: i
                            .response
                            .headers
                            .as_ref()
                            .map(|h| h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default(),
                        body: i.response.body.clone(),
                    },
                }
            })
            .collect()
    }

    pub fn from_interactions(
        consumer_name: &str,
        provider_name: &str,
        interactions: Vec<HttpInteraction>,
    ) -> Self {
        let pact_interactions: Vec<PactInteraction> = interactions
            .into_iter()
            .map(|i| {
                let query_string = i.request.query.as_ref().map(|q| {
                    q.0.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&")
                });

                PactInteraction {
                    description: i.description,
                    provider_state: i.provider_state.as_ref().and_then(|p| p.name.clone()),
                    request: PactRequest {
                        method: i.request.method,
                        path: i.request.path,
                        query: query_string,
                        headers: if i.request.headers.is_empty() {
                            None
                        } else {
                            Some(i.request.headers.into_iter().collect())
                        },
                        body: i.request.body,
                        matching_rules: None,
                    },
                    response: PactResponse {
                        status: i.response.status,
                        headers: if i.response.headers.is_empty() {
                            None
                        } else {
                            Some(i.response.headers.into_iter().collect())
                        },
                        body: i.response.body,
                        matching_rules: None,
                    },
                }
            })
            .collect();

        Self {
            consumer: PactConsumer {
                name: consumer_name.to_string(),
            },
            provider: PactProvider {
                name: provider_name.to_string(),
            },
            interactions: pact_interactions,
            metadata: PactMetadata {
                version: "1.0.0".to_string(),
                created_at: Some(chrono::Utc::now().to_rfc3339()),
            },
        }
    }
}

impl From<PactFile> for Contract {
    fn from(pact: PactFile) -> Self {
        let interactions: Vec<Interaction> = pact
            .interactions
            .iter()
            .map(|i| {
                let query_params = i
                    .request
                    .query
                    .as_ref()
                    .map(|q| {
                        q.split('&')
                            .filter_map(|pair| {
                                let mut parts = pair.split('=');
                                Some((parts.next()?.to_string(), parts.next()?.to_string()))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                Interaction {
                    id: uuid::Uuid::new_v4(),
                    description: i.description.clone(),
                    method: i.request.method.clone(),
                    path: i.request.path.clone(),
                    query_params,
                    request_headers: i
                        .request
                        .headers
                        .as_ref()
                        .map(|h| h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default(),
                    request_body: i.request.body.clone(),
                    response_status: i.response.status,
                    response_headers: i
                        .response
                        .headers
                        .as_ref()
                        .map(|h| h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default(),
                    response_body: i.response.body.clone(),
                    provider_state: i.provider_state.clone(),
                }
            })
            .collect();

        Contract {
            consumer: pact.consumer.name,
            provider: pact.provider.name,
            interactions,
            metadata: crate::contract::ContractMetadata {
                version: pact.metadata.version,
                created_at: pact
                    .metadata
                    .created_at
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pact_file_from_json() {
        let json = r#"{
            "consumer": {"name": "consumer1"},
            "provider": {"name": "provider1"},
            "interactions": [
                {
                    "description": "get users",
                    "request": {"method": "GET", "path": "/api/users"},
                    "response": {"status": 200, "body": {"users": []}}
                }
            ],
            "metadata": {"version": "1.0.0"}
        }"#;

        let pact = PactFile::from_json(json).unwrap();
        assert_eq!(pact.consumer.name, "consumer1");
        assert_eq!(pact.provider.name, "provider1");
        assert_eq!(pact.interactions.len(), 1);
        assert_eq!(pact.interactions[0].description, "get users");
    }

    #[test]
    fn test_pact_to_interactions() {
        let json = r#"{
            "consumer": {"name": "c"},
            "provider": {"name": "p"},
            "interactions": [
                {
                    "description": "test",
                    "request": {"method": "POST", "path": "/api/data", "body": {"key": "value"}},
                    "response": {"status": 201}
                }
            ]
        }"#;

        let pact = PactFile::from_json(json).unwrap();
        let interactions = pact.to_interactions();
        assert_eq!(interactions.len(), 1);
        assert_eq!(interactions[0].description, "test");
        assert_eq!(interactions[0].request.method, "POST");
    }

    #[test]
    fn test_from_interactions() {
        let interaction = HttpInteraction {
            description: "test interaction".to_string(),
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
                body: Some(serde_json::json!({"result": "ok"})),
            },
        };

        let pact = PactFile::from_interactions("test-consumer", "test-provider", vec![interaction]);
        assert_eq!(pact.consumer.name, "test-consumer");
        assert_eq!(pact.provider.name, "test-provider");
        assert_eq!(pact.interactions.len(), 1);
    }
}
