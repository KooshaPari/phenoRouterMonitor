//! Anthropic provider implementation
//! Wraps: anthropic.com Claude API (2024)

use super::{CompletionRequest, CompletionResponse, LlmProvider};
use crate::config::ProviderType;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Anthropic API response structure
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

/// Content block from Anthropic response
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContent {
    text: String,
}

/// Token usage from Anthropic
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic provider implementation (Claude)
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Anthropic provider with custom base URL
    pub fn with_base_url(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    fn get_api_key_header(&self) -> String {
        self.api_key.clone()
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/messages", self.base_url);

        let payload = serde_json::json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "top_p": request.top_p,
            "messages": [
                {
                    "role": "user",
                    "content": request.prompt
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.get_api_key_header())
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::RequestFailed {
                reason: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(Error::RequestFailed {
                reason: format!("Anthropic API error: {}", response.status()),
            });
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| Error::InvalidResponse {
                reason: e.to_string(),
            })?;

        let first_content = api_response
            .content
            .first()
            .ok_or_else(|| Error::InvalidResponse {
                reason: "No content in response".to_string(),
            })?;

        Ok(CompletionResponse {
            text: first_content.text.clone(),
            prompt_tokens: api_response.usage.input_tokens,
            completion_tokens: api_response.usage.output_tokens,
            total_tokens: api_response.usage.input_tokens + api_response.usage.output_tokens,
        })
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn verify_credentials(&self) -> Result<bool> {
        // Anthropic doesn't have a direct credentials verification endpoint,
        // so we attempt a simple models list request
        let url = format!("{}/messages", self.base_url);

        let payload = serde_json::json!({
            "model": self.model,
            "max_tokens": 10,
            "messages": [
                {
                    "role": "user",
                    "content": "test"
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.get_api_key_header())
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::AuthenticationFailed {
                reason: e.to_string(),
            })?;

        // 401 or 403 = invalid credentials; 400 = invalid request (but auth OK)
        match response.status().as_u16() {
            401 | 403 => Ok(false),
            _ => Ok(response.status().is_success() || response.status().as_u16() == 400),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_new() {
        let provider = AnthropicProvider::new(
            "sk-ant-key".to_string(),
            "claude-3-opus-20240229".to_string(),
        );

        assert_eq!(provider.api_key, "sk-ant-key");
        assert_eq!(provider.model, "claude-3-opus-20240229");
        assert_eq!(provider.base_url, "https://api.anthropic.com/v1");
        assert_eq!(provider.provider_type(), ProviderType::Anthropic);
        assert_eq!(provider.model(), "claude-3-opus-20240229");
    }

    #[test]
    fn test_anthropic_with_custom_base_url() {
        let provider = AnthropicProvider::with_base_url(
            "sk-ant-key".to_string(),
            "claude-3-opus-20240229".to_string(),
            "https://custom-anthropic.example.com".to_string(),
        );

        assert_eq!(provider.base_url, "https://custom-anthropic.example.com");
    }

    #[test]
    fn test_anthropic_api_key_header() {
        let provider = AnthropicProvider::new(
            "test-ant-key-789".to_string(),
            "claude-3-opus-20240229".to_string(),
        );

        assert_eq!(provider.get_api_key_header(), "test-ant-key-789");
    }

    #[test]
    fn test_anthropic_provider_type() {
        let provider = AnthropicProvider::new(
            "sk-ant-key".to_string(),
            "claude-3-opus-20240229".to_string(),
        );

        assert_eq!(provider.provider_type(), ProviderType::Anthropic);
    }
}
