// OpenAI provider implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{BifrostError, BifrostResult};
use crate::models::{LLMProvider, LLMRequest, LLMResponse, ProviderMetadata};
use crate::metrics::ProviderMetrics;

/// OpenAI provider configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,
    pub org_id: Option<String>,
    pub timeout_secs: u64,
}

impl OpenAIConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            org_id: None,
            timeout_secs: 30,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }
}

/// OpenAI API request format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    index: usize,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

pub struct OpenAIProvider {
    config: OpenAIConfig,
    client: reqwest::Client,
    metrics: ProviderMetrics,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Self {
        let client = reqwest::Client::new();
        let metrics = ProviderMetrics::new("openai".to_string());

        Self {
            config,
            client,
            metrics,
        }
    }

    fn build_request(&self, bifrost_req: &LLMRequest) -> OpenAIRequest {
        let messages = bifrost_req
            .messages
            .iter()
            .map(|m| OpenAIMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        OpenAIRequest {
            model: bifrost_req.model.clone(),
            messages,
            max_tokens: bifrost_req.max_tokens,
            temperature: bifrost_req.temperature,
            top_p: bifrost_req.top_p,
            stream: bifrost_req.stream,
        }
    }

    async fn call_api(&self, req: OpenAIRequest) -> BifrostResult<OpenAIResponse> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.api_key)
                .parse()
                .map_err(|_| {
                    BifrostError::ConfigurationError("Invalid API key format".to_string())
                })?,
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        if let Some(org_id) = &self.config.org_id {
            headers.insert(
                "OpenAI-Organization",
                org_id.parse().map_err(|_| {
                    BifrostError::ConfigurationError("Invalid org ID format".to_string())
                })?,
            );
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                BifrostError::ProviderError(format!("OpenAI request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => BifrostError::AuthenticationError {
                    provider: "openai".to_string(),
                    reason: "Invalid API key".to_string(),
                },
                429 => BifrostError::RateLimited {
                    provider: "openai".to_string(),
                },
                400 => BifrostError::InvalidRequest(error_text),
                _ => BifrostError::ProviderError(format!(
                    "OpenAI error {}: {}",
                    status, error_text
                )),
            });
        }

        response.json::<OpenAIResponse>().await.map_err(|e| {
            BifrostError::SerializationError(format!("Failed to parse OpenAI response: {}", e))
        })
    }

    /// Pricing for OpenAI models (as of March 2026)
    fn pricing(&self, model: &str) -> (f64, f64) {
        // (input_per_1k_tokens, output_per_1k_tokens) in USD
        match model {
            "gpt-4o" => (0.005, 0.015),
            "gpt-4-turbo" => (0.01, 0.03),
            "gpt-4" => (0.03, 0.06),
            "gpt-3.5-turbo" => (0.0005, 0.0015),
            _ => (0.01, 0.03), // Default estimate
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn is_available(&self) -> BifrostResult<bool> {
        // Simple health check by listing models
        self.list_models().await.map(|models| !models.is_empty())
    }

    async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse> {
        let start = Instant::now();
        let openai_req = self.build_request(request);

        match self.call_api(openai_req).await {
            Ok(openai_resp) => {
                let choice = openai_resp
                    .choices
                    .first()
                    .ok_or_else(|| {
                        BifrostError::ProviderError(
                            "No choices in OpenAI response".to_string(),
                        )
                    })?;

                let latency_ms = start.elapsed().as_millis() as u64;

                let (input_price, output_price) = self.pricing(&openai_resp.model);
                let cost = (openai_resp.usage.prompt_tokens as f64 / 1000.0 * input_price)
                    + (openai_resp.usage.completion_tokens as f64 / 1000.0 * output_price);

                self.metrics.record_success(latency_ms, cost);

                Ok(LLMResponse {
                    response_id: openai_resp.id,
                    request_id: request.request_id.clone(),
                    content: choice.message.content.clone(),
                    model: openai_resp.model,
                    provider: "openai".to_string(),
                    prompt_tokens: openai_resp.usage.prompt_tokens,
                    completion_tokens: openai_resp.usage.completion_tokens,
                    cost_usd: cost,
                    latency_ms,
                    stop_reason: choice.finish_reason.clone(),
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
            "Streaming not yet implemented for OpenAI".to_string(),
        ))
    }

    fn estimate_cost(
        &self,
        model: &str,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> f64 {
        let (input_price, output_price) = self.pricing(model);
        (prompt_tokens as f64 / 1000.0 * input_price)
            + (completion_tokens as f64 / 1000.0 * output_price)
    }

    async fn list_models(&self) -> BifrostResult<Vec<String>> {
        // Known OpenAI models (in production, fetch from API)
        Ok(vec![
            "gpt-4o".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ])
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "openai".to_string(),
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
    fn test_openai_config() {
        let config = OpenAIConfig::new("sk-test".to_string());
        assert_eq!(config.api_key, "sk-test");
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_openai_provider_creation() {
        let config = OpenAIConfig::new("sk-test".to_string());
        let provider = OpenAIProvider::new(config);
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_openai_pricing() {
        let config = OpenAIConfig::new("sk-test".to_string());
        let provider = OpenAIProvider::new(config);

        let cost = provider.estimate_cost("gpt-4o", 1000, 1000);
        assert!(cost > 0.0);
        assert!(cost < 1.0); // Reasonable upper bound
    }

    #[test]
    fn test_openai_pricing_models() {
        let config = OpenAIConfig::new("sk-test".to_string());
        let provider = OpenAIProvider::new(config);

        let gpt4o_cost = provider.estimate_cost("gpt-4o", 1000, 1000);
        let gpt35_cost = provider.estimate_cost("gpt-3.5-turbo", 1000, 1000);

        // GPT-4o should be more expensive than GPT-3.5
        assert!(gpt4o_cost > gpt35_cost);
    }

    #[test]
    fn test_build_request() {
        let config = OpenAIConfig::new("sk-test".to_string());
        let provider = OpenAIProvider::new(config);

        let messages = vec![Message {
            role: crate::models::MessageRole::User,
            content: "Hello".to_string(),
        }];
        let bifrost_req = LLMRequest::new("gpt-4o".to_string(), messages);
        let openai_req = provider.build_request(&bifrost_req);

        assert_eq!(openai_req.model, "gpt-4o");
        assert_eq!(openai_req.messages.len(), 1);
        assert!(!openai_req.stream);
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = OpenAIConfig::new("sk-test".to_string());
        let provider = OpenAIProvider::new(config);

        let models = provider.list_models().await.unwrap();
        assert!(models.len() > 0);
        assert!(models.contains(&"gpt-4o".to_string()));
    }

    #[test]
    fn test_metadata() {
        let config = OpenAIConfig::new("sk-test".to_string());
        let provider = OpenAIProvider::new(config);

        let metadata = provider.metadata();
        assert_eq!(metadata.name, "openai");
        assert!(metadata.available);
    }
}
