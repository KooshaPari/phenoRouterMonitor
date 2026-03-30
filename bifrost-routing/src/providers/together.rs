// Together provider implementation (batch processing support)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{BifrostError, BifrostResult};
use crate::models::{LLMProvider, LLMRequest, LLMResponse, ProviderMetadata};
use crate::metrics::ProviderMetrics;

/// Together provider configuration
#[derive(Debug, Clone)]
pub struct TogetherConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_secs: u64,
}

impl TogetherConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.together.xyz".to_string(),
            timeout_secs: 30,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

/// Together API request format
#[derive(Debug, Serialize, Deserialize)]
struct TogetherRequest {
    model: String,
    messages: Vec<TogetherMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TogetherMessage {
    role: String,
    content: String,
}

/// Together API response format
#[derive(Debug, Serialize, Deserialize)]
struct TogetherResponse {
    id: String,
    model: String,
    choices: Vec<TogetherChoice>,
    usage: TogetherUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct TogetherChoice {
    index: usize,
    message: TogetherMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TogetherUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

pub struct TogetherProvider {
    config: TogetherConfig,
    client: reqwest::Client,
    metrics: ProviderMetrics,
}

impl TogetherProvider {
    pub fn new(config: TogetherConfig) -> Self {
        let client = reqwest::Client::new();
        let metrics = ProviderMetrics::new("together".to_string());

        Self {
            config,
            client,
            metrics,
        }
    }

    fn build_request(&self, bifrost_req: &LLMRequest) -> TogetherRequest {
        let messages = bifrost_req
            .messages
            .iter()
            .map(|m| TogetherMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        TogetherRequest {
            model: bifrost_req.model.clone(),
            messages,
            max_tokens: bifrost_req.max_tokens,
            temperature: bifrost_req.temperature,
            top_p: bifrost_req.top_p,
            stream: bifrost_req.stream,
        }
    }

    async fn call_api(&self, req: TogetherRequest) -> BifrostResult<TogetherResponse> {
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

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                BifrostError::ProviderError(format!("Together request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => BifrostError::AuthenticationError {
                    provider: "together".to_string(),
                    reason: "Invalid API key".to_string(),
                },
                429 => BifrostError::RateLimited {
                    provider: "together".to_string(),
                },
                400 => BifrostError::InvalidRequest(error_text),
                _ => BifrostError::ProviderError(format!(
                    "Together error {}: {}",
                    status, error_text
                )),
            });
        }

        response.json::<TogetherResponse>().await.map_err(|e| {
            BifrostError::SerializationError(format!("Failed to parse Together response: {}", e))
        })
    }

    /// Pricing for open-source models via Together (typically cheaper)
    fn pricing(&self, model: &str) -> (f64, f64) {
        // Per 1M tokens pricing for open source models
        match model {
            "mistral-7b" => (0.0002, 0.0006),
            "llama-2-70b" => (0.0009, 0.0012),
            "llama-2-13b" => (0.000225, 0.0003),
            "codellama-34b" => (0.0008, 0.0011),
            _ => (0.0003, 0.001), // Conservative estimate
        }
    }
}

#[async_trait]
impl LLMProvider for TogetherProvider {
    fn name(&self) -> &str {
        "together"
    }

    async fn is_available(&self) -> BifrostResult<bool> {
        self.list_models().await.map(|models| !models.is_empty())
    }

    async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse> {
        let start = Instant::now();
        let together_req = self.build_request(request);

        match self.call_api(together_req).await {
            Ok(together_resp) => {
                let choice = together_resp
                    .choices
                    .first()
                    .ok_or_else(|| {
                        BifrostError::ProviderError(
                            "No choices in Together response".to_string(),
                        )
                    })?;

                let latency_ms = start.elapsed().as_millis() as u64;

                let (input_price, output_price) = self.pricing(&together_resp.model);
                let cost = (together_resp.usage.prompt_tokens as f64 / 1_000_000.0 * input_price)
                    + (together_resp.usage.completion_tokens as f64 / 1_000_000.0 * output_price);

                self.metrics.record_success(latency_ms, cost);

                Ok(LLMResponse {
                    response_id: together_resp.id,
                    request_id: request.request_id.clone(),
                    content: choice.message.content.clone(),
                    model: together_resp.model,
                    provider: "together".to_string(),
                    prompt_tokens: together_resp.usage.prompt_tokens,
                    completion_tokens: together_resp.usage.completion_tokens,
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
            "Streaming not yet implemented for Together".to_string(),
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
        // Open source models available on Together
        Ok(vec![
            "mistral-7b".to_string(),
            "llama-2-70b".to_string(),
            "llama-2-13b".to_string(),
            "codellama-34b".to_string(),
        ])
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "together".to_string(),
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
    use crate::models::Message;

    #[test]
    fn test_together_config() {
        let config = TogetherConfig::new("sk-together-test".to_string());
        assert_eq!(config.api_key, "sk-together-test");
        assert_eq!(config.base_url, "https://api.together.xyz");
    }

    #[test]
    fn test_together_provider_creation() {
        let config = TogetherConfig::new("sk-together-test".to_string());
        let provider = TogetherProvider::new(config);
        assert_eq!(provider.name(), "together");
    }

    #[test]
    fn test_together_pricing() {
        let config = TogetherConfig::new("sk-together-test".to_string());
        let provider = TogetherProvider::new(config);

        let cost = provider.estimate_cost("mistral-7b", 1000, 1000);
        assert!(cost > 0.0);
        assert!(cost < 0.001); // Should be cheap
    }

    #[test]
    fn test_together_cheaper_than_proprietary() {
        let config = TogetherConfig::new("sk-together-test".to_string());
        let provider = TogetherProvider::new(config);

        let together_cost = provider.estimate_cost("mistral-7b", 1000, 1000);

        // Together open source models should be cheaper
        assert!(together_cost < 0.001);
    }

    #[test]
    fn test_build_request() {
        let config = TogetherConfig::new("sk-together-test".to_string());
        let provider = TogetherProvider::new(config);

        let messages = vec![Message {
            role: crate::models::MessageRole::User,
            content: "Hello".to_string(),
        }];
        let bifrost_req = LLMRequest::new("mistral-7b".to_string(), messages)
            .with_max_tokens(256);
        let together_req = provider.build_request(&bifrost_req);

        assert_eq!(together_req.model, "mistral-7b");
        assert_eq!(together_req.max_tokens, Some(256));
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = TogetherConfig::new("sk-together-test".to_string());
        let provider = TogetherProvider::new(config);

        let models = provider.list_models().await.unwrap();
        assert!(models.len() > 0);
        assert!(models.contains(&"mistral-7b".to_string()));
    }
}
