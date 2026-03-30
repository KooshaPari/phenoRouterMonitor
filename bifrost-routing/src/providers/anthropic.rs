// Anthropic provider implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{BifrostError, BifrostResult};
use crate::models::{LLMProvider, LLMRequest, LLMResponse, ProviderMetadata};
use crate::metrics::ProviderMetrics;

/// Anthropic provider configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
    pub api_version: String,
    pub timeout_secs: u64,
}

impl AnthropicConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            api_version: "2024-06-01".to_string(),
            timeout_secs: 30,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    model: String,
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: reqwest::Client,
    metrics: ProviderMetrics,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Self {
        let client = reqwest::Client::new();
        let metrics = ProviderMetrics::new("anthropic".to_string());

        Self {
            config,
            client,
            metrics,
        }
    }

    fn build_request(&self, bifrost_req: &LLMRequest) -> AnthropicRequest {
        let messages = bifrost_req
            .messages
            .iter()
            .map(|m| AnthropicMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        AnthropicRequest {
            model: bifrost_req.model.clone(),
            max_tokens: bifrost_req.max_tokens.unwrap_or(2048),
            messages,
            temperature: bifrost_req.temperature,
            top_p: bifrost_req.top_p,
            system: None, // Could be extracted from messages if needed
        }
    }

    async fn call_api(&self, req: AnthropicRequest) -> BifrostResult<AnthropicResponse> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            self.config.api_key.parse().map_err(|_| {
                BifrostError::ConfigurationError("Invalid API key format".to_string())
            })?,
        );
        headers.insert(
            "anthropic-version",
            self.config.api_version.parse().map_err(|_| {
                BifrostError::ConfigurationError("Invalid API version".to_string())
            })?,
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = self
            .client
            .post(format!("{}/v1/messages", self.config.base_url))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                BifrostError::ProviderError(format!("Anthropic request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => BifrostError::AuthenticationError {
                    provider: "anthropic".to_string(),
                    reason: "Invalid API key".to_string(),
                },
                429 => BifrostError::RateLimited {
                    provider: "anthropic".to_string(),
                },
                400 => BifrostError::InvalidRequest(error_text),
                _ => BifrostError::ProviderError(format!(
                    "Anthropic error {}: {}",
                    status, error_text
                )),
            });
        }

        response.json::<AnthropicResponse>().await.map_err(|e| {
            BifrostError::SerializationError(format!("Failed to parse Anthropic response: {}", e))
        })
    }

    /// Pricing for Anthropic models (as of March 2026)
    fn pricing(&self, model: &str) -> (f64, f64) {
        // (input_per_1m_tokens, output_per_1m_tokens) in USD
        match model {
            "claude-opus" => (15.0, 75.0),
            "claude-sonnet" => (3.0, 15.0),
            "claude-haiku" => (0.8, 4.0),
            _ => (3.0, 15.0), // Default estimate
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn is_available(&self) -> BifrostResult<bool> {
        // Check by listing models
        self.list_models().await.map(|models| !models.is_empty())
    }

    async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse> {
        let start = Instant::now();
        let anthropic_req = self.build_request(request);

        match self.call_api(anthropic_req).await {
            Ok(anthropic_resp) => {
                let content = anthropic_resp
                    .content
                    .first()
                    .and_then(|c| c.text.clone())
                    .ok_or_else(|| {
                        BifrostError::ProviderError(
                            "No text content in Anthropic response".to_string(),
                        )
                    })?;

                let latency_ms = start.elapsed().as_millis() as u64;

                let (input_price, output_price) = self.pricing(&anthropic_resp.model);
                let cost = (anthropic_resp.usage.input_tokens as f64 / 1_000_000.0 * input_price)
                    + (anthropic_resp.usage.output_tokens as f64 / 1_000_000.0 * output_price);

                self.metrics.record_success(latency_ms, cost);

                Ok(LLMResponse {
                    response_id: anthropic_resp.id,
                    request_id: request.request_id.clone(),
                    content,
                    model: anthropic_resp.model,
                    provider: "anthropic".to_string(),
                    prompt_tokens: anthropic_resp.usage.input_tokens,
                    completion_tokens: anthropic_resp.usage.output_tokens,
                    cost_usd: cost,
                    latency_ms,
                    stop_reason: None,
                    finished_at: chrono::Utc::now(),
                })
            }
            Err(e) => {
                self.metrics.record_failure();
                Err(e)
            }
        }
    }

    async fn invoke_streaming(
        &self,
        _request: &LLMRequest,
    ) -> BifrostResult<Box<dyn std::any::Any>> {
        Err(BifrostError::ProviderError(
            "Streaming not yet implemented for Anthropic".to_string(),
        ))
    }

    fn estimate_cost(
        &self,
        model: &str,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> f64 {
        let (input_price, output_price) = self.pricing(model);
        (prompt_tokens as f64 / 1_000_000.0 * input_price)
            + (completion_tokens as f64 / 1_000_000.0 * output_price)
    }

    async fn list_models(&self) -> BifrostResult<Vec<String>> {
        // Known Anthropic models
        Ok(vec![
            "claude-opus".to_string(),
            "claude-sonnet".to_string(),
            "claude-haiku".to_string(),
        ])
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "anthropic".to_string(),
            available: true,
            latency_ms: self
                .metrics
                .latency_tracker
                .average_latency_ms()
                .into(),
            last_error: None,
            total_requests: self.metrics.total_requests(),
            total_cost_usd: self.metrics.cost_tracker.total_cost(),
            success_rate: self.metrics.success_rate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LLMProvider, Message};

    #[test]
    fn test_anthropic_config() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        assert_eq!(config.api_key, "sk-ant-test");
        assert_eq!(config.base_url, "https://api.anthropic.com");
    }

    #[test]
    fn test_anthropic_provider_creation() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        let provider = AnthropicProvider::new(config);
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_anthropic_pricing() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        let provider = AnthropicProvider::new(config);

        let cost = provider.estimate_cost("claude-opus", 1000, 1000);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_anthropic_pricing_tiers() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        let provider = AnthropicProvider::new(config);

        let opus_cost = provider.estimate_cost("claude-opus", 1000, 1000);
        let haiku_cost = provider.estimate_cost("claude-haiku", 1000, 1000);

        // Opus should be more expensive than Haiku
        assert!(opus_cost > haiku_cost);
    }

    #[test]
    fn test_build_request() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        let provider = AnthropicProvider::new(config);

        let messages = vec![Message {
            role: crate::models::MessageRole::User,
            content: "Hello".to_string(),
        }];
        let bifrost_req = LLMRequest::new("claude-opus".to_string(), messages)
            .with_max_tokens(512);
        let anthropic_req = provider.build_request(&bifrost_req);

        assert_eq!(anthropic_req.model, "claude-opus");
        assert_eq!(anthropic_req.max_tokens, 512);
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        let provider = AnthropicProvider::new(config);

        let models = provider.list_models().await.unwrap();
        assert!(models.len() > 0);
        assert!(models.contains(&"claude-opus".to_string()));
    }

    #[test]
    fn test_metadata() {
        let config = AnthropicConfig::new("sk-ant-test".to_string());
        let provider = AnthropicProvider::new(config);

        let metadata = provider.metadata();
        assert_eq!(metadata.name, "anthropic");
        assert!(metadata.available);
    }
}
