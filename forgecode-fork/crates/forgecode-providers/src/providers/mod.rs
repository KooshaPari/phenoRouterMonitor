//! Provider implementations for various LLM services

pub mod openrouter;
pub mod together;
pub mod anthropic;

use crate::config::ProviderType;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub use self::openrouter::OpenRouterProvider;
pub use self::together::TogetherProvider;
pub use self::anthropic::AnthropicProvider;

/// Request payload for LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The prompt/messages to send
    pub prompt: String,

    /// Temperature for response generation
    pub temperature: f32,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// Top-p sampling parameter
    pub top_p: f32,
}

/// Response from LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The generated completion text
    pub text: String,

    /// Number of tokens used in prompt
    pub prompt_tokens: u32,

    /// Number of tokens used in completion
    pub completion_tokens: u32,

    /// Total tokens used
    pub total_tokens: u32,
}

impl CompletionResponse {
    /// Create a new completion response
    pub fn new(text: String, prompt_tokens: u32, completion_tokens: u32) -> Self {
        let total_tokens = prompt_tokens + completion_tokens;
        Self {
            text,
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }
}

/// Trait for LLM provider implementations
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a completion request to the provider
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Get the model name
    fn model(&self) -> &str;

    /// Verify the API key is valid
    async fn verify_credentials(&self) -> Result<bool>;
}

/// Provider factory enum
#[derive(Debug, Clone)]
pub enum Provider {
    /// OpenRouter provider
    OpenRouter(OpenRouterProvider),
    /// Together AI provider
    Together(TogetherProvider),
    /// Anthropic provider
    Anthropic(AnthropicProvider),
}

impl Provider {
    /// Get the provider type
    pub fn provider_type(&self) -> ProviderType {
        match self {
            Provider::OpenRouter(p) => p.provider_type(),
            Provider::Together(p) => p.provider_type(),
            Provider::Anthropic(p) => p.provider_type(),
        }
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        match self {
            Provider::OpenRouter(p) => p.model(),
            Provider::Together(p) => p.model(),
            Provider::Anthropic(p) => p.model(),
        }
    }

    /// Send a completion request
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        match self {
            Provider::OpenRouter(p) => p.complete(request).await,
            Provider::Together(p) => p.complete(request).await,
            Provider::Anthropic(p) => p.complete(request).await,
        }
    }

    /// Verify the API key is valid
    pub async fn verify_credentials(&self) -> Result<bool> {
        match self {
            Provider::OpenRouter(p) => p.verify_credentials().await,
            Provider::Together(p) => p.verify_credentials().await,
            Provider::Anthropic(p) => p.verify_credentials().await,
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.provider_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_response_new() {
        let response = CompletionResponse::new(
            "Hello, world!".to_string(),
            10,
            5,
        );

        assert_eq!(response.text, "Hello, world!");
        assert_eq!(response.prompt_tokens, 10);
        assert_eq!(response.completion_tokens, 5);
        assert_eq!(response.total_tokens, 15);
    }

    #[test]
    fn test_completion_request_creation() {
        let request = CompletionRequest {
            prompt: "Say hello".to_string(),
            temperature: 0.7,
            max_tokens: 100,
            top_p: 0.9,
        };

        assert_eq!(request.prompt, "Say hello");
        assert_eq!(request.temperature, 0.7);
        assert_eq!(request.max_tokens, 100);
    }
}
