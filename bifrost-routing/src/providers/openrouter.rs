// OpenRouter provider implementation (aggregates multiple models)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{BifrostError, BifrostResult};
use crate::models::{LLMProvider, LLMRequest, LLMResponse, ProviderMetadata};
use crate::metrics::ProviderMetrics;

/// OpenRouter provider configuration
#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub base_url: String,
    pub http_referer: String,
    pub timeout_secs: u64,
}

impl OpenRouterConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            http_referer: "https://bifrost.local".to_string(),
            timeout_secs: 30,
        }
    }

    pub fn with_http_referer(mut self, referer: String) -> Self {
        self.http_referer = referer;
        self
    }
}

/// OpenRouter API request format
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

/// OpenRouter API response format
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterResponse {
    id: String,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterChoice {
    index: usize,
    message: OpenRouterMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

pub struct OpenRouterProvider {
    config: OpenRouterConfig,
    client: reqwest::Client,
    metrics: ProviderMetrics,
}

impl OpenRouterProvider {
    pub fn new(config: OpenRouterConfig) -> Self {
        let client = reqwest::Client::new();
        let metrics = ProviderMetrics::new("openrouter".to_string());

        Self {
            config,
            client,
            metrics,
        }
    }

    fn build_request(&self, bifrost_req: &LLMRequest) -> OpenRouterRequest {
        let messages = bifrost_req
            .messages
            .iter()
            .map(|m| OpenRouterMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        OpenRouterRequest {
            model: bifrost_req.model.clone(),
            messages,
            max_tokens: bifrost_req.max_tokens,
            temperature: bifrost_req.temperature,
            top_p: bifrost_req.top_p,
            stream: bifrost_req.stream,
        }
    }

    async fn call_api(&self, req: OpenRouterRequest) -> BifrostResult<OpenRouterResponse> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.api_key)
                .parse()
                .map_err(|_| {
                    BifrostError::ConfigurationError("Invalid API key format".to_string())
                })?,
        );
        headers.insert(
            "HTTP-Referer",
            self.config.http_referer.parse().map_err(|_| {
                BifrostError::ConfigurationError("Invalid HTTP referer".to_string())
            })?,
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .headers(headers)
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                BifrostError::ProviderError(format!("OpenRouter request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => BifrostError::AuthenticationError {
                    provider: "openrouter".to_string(),
                    reason: "Invalid API key".to_string(),
                },
                429 => BifrostError::RateLimited {
                    provider: "openrouter".to_string(),
                },
                400 => BifrostError::InvalidRequest(error_text),
                _ => BifrostError::ProviderError(format!(
                    "OpenRouter error {}: {}",
                    status, error_text
                )),
            });
        }

        response.json::<OpenRouterResponse>().await.map_err(|e| {
            BifrostError::SerializationError(format!(
                "Failed to parse OpenRouter response: {}",
                e
            ))
        })
    }

    /// Pricing for models on OpenRouter
    fn pricing(&self, model: &str) -> (f64, f64) {
        // Varies by model; these are approximations
        match model {
            "gpt-4-turbo" | "gpt-4-turbo-preview" => (0.01, 0.03),
            "gpt-4" => (0.03, 0.06),
            "gpt-3.5-turbo" => (0.0005, 0.0015),
            "claude-opus" => (0.015, 0.075),
            "claude-sonnet" => (0.003, 0.015),
            "llama-2-70b" => (0.0007, 0.0009),
            _ => (0.01, 0.03),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn is_available(&self) -> BifrostResult<bool> {
        self.list_models().await.map(|models| !models.is_empty())
    }

    async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse> {
        let start = Instant::now();
        let openrouter_req = self.build_request(request);

        match self.call_api(openrouter_req).await {
            Ok(openrouter_resp) => {
                let choice = openrouter_resp
                    .choices
                    .first()
                    .ok_or_else(|| {
                        BifrostError::ProviderError(
                            "No choices in OpenRouter response".to_string(),
                        )
                    })?;

                let latency_ms = start.elapsed().as_millis() as u64;

                let (input_price, output_price) = self.pricing(&openrouter_resp.model);
                let cost = (openrouter_resp.usage.prompt_tokens as f64 / 1000.0 * input_price)
                    + (openrouter_resp.usage.completion_tokens as f64 / 1000.0 * output_price);

                self.metrics.record_success(latency_ms, cost);

                Ok(LLMResponse {
                    response_id: openrouter_resp.id,
                    request_id: request.request_id.clone(),
                    content: choice.message.content.clone(),
                    model: openrouter_resp.model,
                    provider: "openrouter".to_string(),
                    prompt_tokens: openrouter_resp.usage.prompt_tokens,
                    completion_tokens: openrouter_resp.usage.completion_tokens,
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
            "Streaming not yet implemented for OpenRouter".to_string(),
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
        // Sample of available models on OpenRouter
        Ok(vec![
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
            "claude-opus".to_string(),
            "claude-sonnet".to_string(),
            "llama-2-70b".to_string(),
        ])
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "openrouter".to_string(),
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

    #[test]
    fn test_openrouter_config() {
        let config = OpenRouterConfig::new("sk-or-test".to_string());
        assert_eq!(config.api_key, "sk-or-test");
        assert_eq!(config.base_url, "https://openrouter.ai/api/v1");
    }

    #[test]
    fn test_openrouter_provider_creation() {
        let config = OpenRouterConfig::new("sk-or-test".to_string());
        let provider = OpenRouterProvider::new(config);
        assert_eq!(provider.name(), "openrouter");
    }

    #[test]
    fn test_openrouter_pricing() {
        let config = OpenRouterConfig::new("sk-or-test".to_string());
        let provider = OpenRouterProvider::new(config);

        let cost = provider.estimate_cost("gpt-4-turbo", 1000, 1000);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_openrouter_multi_model_pricing() {
        let config = OpenRouterConfig::new("sk-or-test".to_string());
        let provider = OpenRouterProvider::new(config);

        let gpt4_cost = provider.estimate_cost("gpt-4", 1000, 1000);
        let llama_cost = provider.estimate_cost("llama-2-70b", 1000, 1000);

        // GPT-4 is more expensive than Llama
        assert!(gpt4_cost > llama_cost);
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = OpenRouterConfig::new("sk-or-test".to_string());
        let provider = OpenRouterProvider::new(config);

        let models = provider.list_models().await.unwrap();
        assert!(models.len() > 3);
        assert!(models.contains(&"gpt-4".to_string()));
    }
}
