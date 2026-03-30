// Core data models for LLM routing

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::error::BifrostResult;

/// Represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

/// LLM request with configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    /// Unique request identifier
    pub request_id: String,

    /// Messages to send to the model
    pub messages: Vec<Message>,

    /// Model identifier (e.g., "gpt-4", "claude-opus")
    pub model: String,

    /// Maximum tokens in response
    pub max_tokens: Option<usize>,

    /// Temperature (randomness) 0.0-2.0
    pub temperature: Option<f32>,

    /// Top-p (nucleus sampling)
    pub top_p: Option<f32>,

    /// Whether to stream the response
    pub stream: bool,

    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Custom metadata
    pub metadata: Option<serde_json::Value>,
}

impl LLMRequest {
    pub fn new(model: String, messages: Vec<Message>) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            messages,
            model,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: false,
            timeout_ms: None,
            metadata: None,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Approximate token count (simple heuristic: ~4 chars per token)
    pub fn estimate_tokens(&self) -> usize {
        let content_tokens: usize = self
            .messages
            .iter()
            .map(|m| (m.content.len() / 4).max(1))
            .sum();

        let max_token_overhead = self.max_tokens.unwrap_or(2048);
        (content_tokens + max_token_overhead).min(32000) // Realistic upper bound
    }
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// Response ID
    pub response_id: String,

    /// Request ID this response is for
    pub request_id: String,

    /// Generated content
    pub content: String,

    /// Model that generated the response
    pub model: String,

    /// Provider that served the request
    pub provider: String,

    /// Tokens used in prompt
    pub prompt_tokens: usize,

    /// Tokens used in completion
    pub completion_tokens: usize,

    /// Cost in USD
    pub cost_usd: f64,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Stop reason
    pub stop_reason: Option<String>,

    /// Finish timestamp
    pub finished_at: chrono::DateTime<chrono::Utc>,
}

/// Streaming message during streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMessage {
    pub request_id: String,
    pub delta: String,
    pub index: usize,
    pub finish_reason: Option<String>,
}

/// Provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub available: bool,
    pub latency_ms: Option<u64>,
    pub last_error: Option<String>,
    pub total_requests: u64,
    pub total_cost_usd: f64,
    pub success_rate: f64,
}

/// Core trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Check if provider is available
    async fn is_available(&self) -> BifrostResult<bool>;

    /// Send a request to the LLM
    async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse>;

    /// Send a streaming request
    async fn invoke_streaming(
        &self,
        request: &LLMRequest,
    ) -> BifrostResult<Box<dyn std::any::Any>>;

    /// Get cost for a request (estimated before execution)
    fn estimate_cost(
        &self,
        model: &str,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> f64;

    /// List available models
    async fn list_models(&self) -> BifrostResult<Vec<String>>;

    /// Get metadata about this provider
    fn metadata(&self) -> ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_request_creation() {
        let messages = vec![Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        }];
        let request = LLMRequest::new("gpt-4".to_string(), messages);

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
        assert!(!request.stream);
    }

    #[test]
    fn test_llm_request_builder() {
        let messages = vec![Message {
            role: MessageRole::User,
            content: "Test message".to_string(),
        }];
        let request = LLMRequest::new("gpt-4".to_string(), messages)
            .with_max_tokens(1024)
            .with_temperature(0.7)
            .with_stream(true);

        assert_eq!(request.max_tokens, Some(1024));
        assert_eq!(request.temperature, Some(0.7));
        assert!(request.stream);
    }

    #[test]
    fn test_estimate_tokens() {
        let messages = vec![Message {
            role: MessageRole::User,
            content: "a".repeat(100), // 100 chars = ~25 tokens
        }];
        let request = LLMRequest::new("gpt-4".to_string(), messages)
            .with_max_tokens(1024);

        let tokens = request.estimate_tokens();
        assert!(tokens > 25 && tokens < 2000);
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::System.to_string(), "system");
    }
}
