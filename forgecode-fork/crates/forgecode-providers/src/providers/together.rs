//! Together AI provider implementation
//! Wraps: together.ai API v1 (2024)

use super::{CompletionRequest, CompletionResponse, LlmProvider};
use crate::config::ProviderType;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Together API response structure
#[derive(Debug, Serialize, Deserialize)]
struct TogetherResponse {
    output: TogetherOutput,
    usage: TogetherUsage,
}

/// Output object from Together response
#[derive(Debug, Serialize, Deserialize)]
struct TogetherOutput {
    choices: Vec<TogetherChoice>,
}

/// Choice object from Together response
#[derive(Debug, Serialize, Deserialize)]
struct TogetherChoice {
    text: String,
}

/// Token usage from Together
#[derive(Debug, Serialize, Deserialize)]
struct TogetherUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Together AI provider implementation
#[derive(Debug, Clone)]
pub struct TogetherProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl TogetherProvider {
    /// Create a new Together AI provider
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            base_url: "https://api.together.xyz/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Together AI provider with custom base URL
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
impl LlmProvider for TogetherProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/completions", self.base_url);

        let payload = serde_json::json!({
            "model": self.model,
            "prompt": request.prompt,
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
                reason: format!("Together API error: {}", response.status()),
            });
        }

        let api_response: TogetherResponse = response
            .json()
            .await
            .map_err(|e| Error::InvalidResponse {
                reason: e.to_string(),
            })?;

        let first_choice = api_response
            .output
            .choices
            .first()
            .ok_or_else(|| Error::InvalidResponse {
                reason: "No choices in response".to_string(),
            })?;

        Ok(CompletionResponse {
            text: first_choice.text.clone(),
            prompt_tokens: api_response.usage.prompt_tokens,
            completion_tokens: api_response.usage.completion_tokens,
            total_tokens: api_response.usage.total_tokens,
        })
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Together
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
    fn test_together_provider_new() {
        let provider = TogetherProvider::new(
            "sk-together".to_string(),
            "meta-llama/Llama-2-70b".to_string(),
        );

        assert_eq!(provider.api_key, "sk-together");
        assert_eq!(provider.model, "meta-llama/Llama-2-70b");
        assert_eq!(provider.base_url, "https://api.together.xyz/v1");
        assert_eq!(provider.provider_type(), ProviderType::Together);
        assert_eq!(provider.model(), "meta-llama/Llama-2-70b");
    }

    #[test]
    fn test_together_with_custom_base_url() {
        let provider = TogetherProvider::with_base_url(
            "sk-together".to_string(),
            "meta-llama/Llama-2-70b".to_string(),
            "https://custom-together.example.com".to_string(),
        );

        assert_eq!(provider.base_url, "https://custom-together.example.com");
    }

    #[test]
    fn test_together_authorization_header() {
        let provider = TogetherProvider::new(
            "test-key-456".to_string(),
            "meta-llama/Llama-2-70b".to_string(),
        );

        assert_eq!(provider.get_authorization_header(), "Bearer test-key-456");
    }

    #[test]
    fn test_together_provider_type() {
        let provider = TogetherProvider::new(
            "sk-together".to_string(),
            "meta-llama/Llama-2-70b".to_string(),
        );

        assert_eq!(provider.provider_type(), ProviderType::Together);
    }
}
