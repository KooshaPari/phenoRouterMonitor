//! OpenRouter provider implementation
//! Wraps: openrouter.ai API v1 (2024)

use super::{CompletionRequest, CompletionResponse, LlmProvider};
use crate::config::ProviderType;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// OpenRouter API response structure
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

/// Choice object from OpenRouter response
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

/// Message object from OpenRouter
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    content: String,
}

/// Token usage from OpenRouter
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenRouter provider implementation
#[derive(Debug, Clone)]
pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new OpenRouter provider with custom base URL
    pub fn with_base_url(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    fn get_authorization_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": request.prompt
                }
            ],
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
        });

        let response = self
            .client
            .post(&url)
            .header(
                "Authorization",
                self.get_authorization_header(),
            )
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::RequestFailed {
                reason: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(Error::RequestFailed {
                reason: format!("OpenRouter API error: {}", response.status()),
            });
        }

        let api_response: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| Error::InvalidResponse {
                reason: e.to_string(),
            })?;

        let first_choice = api_response
            .choices
            .first()
            .ok_or_else(|| Error::InvalidResponse {
                reason: "No choices in response".to_string(),
            })?;

        Ok(CompletionResponse {
            text: first_choice.message.content.clone(),
            prompt_tokens: api_response.usage.prompt_tokens,
            completion_tokens: api_response.usage.completion_tokens,
            total_tokens: api_response.usage.total_tokens,
        })
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenRouter
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn verify_credentials(&self) -> Result<bool> {
        let url = format!("{}/models", self.base_url);

        let response = self
            .client
            .get(&url)
            .header(
                "Authorization",
                self.get_authorization_header(),
            )
            .send()
            .await
            .map_err(|e| Error::AuthenticationFailed {
                reason: e.to_string(),
            })?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_provider_new() {
        let provider = OpenRouterProvider::new(
            "sk-key".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(provider.api_key, "sk-key");
        assert_eq!(provider.model, "gpt-4");
        assert_eq!(provider.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(provider.provider_type(), ProviderType::OpenRouter);
        assert_eq!(provider.model(), "gpt-4");
    }

    #[test]
    fn test_openrouter_with_custom_base_url() {
        let provider = OpenRouterProvider::with_base_url(
            "sk-key".to_string(),
            "gpt-4".to_string(),
            "https://custom.example.com".to_string(),
        );

        assert_eq!(provider.base_url, "https://custom.example.com");
    }

    #[test]
    fn test_openrouter_authorization_header() {
        let provider = OpenRouterProvider::new(
            "test-key-123".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(provider.get_authorization_header(), "Bearer test-key-123");
    }

    #[test]
    fn test_openrouter_provider_type() {
        let provider = OpenRouterProvider::new(
            "sk-key".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(provider.provider_type(), ProviderType::OpenRouter);
    }
}
