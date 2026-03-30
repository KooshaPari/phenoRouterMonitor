// Token counting utilities for accurate cost calculation

use crate::error::{CostError, CostResult};
use serde::{Deserialize, Serialize};

/// Message for token counting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Token counter for various input formats
#[derive(Debug, Clone, Default)]
pub struct TokenCounter;

impl TokenCounter {
    /// Count tokens in a text string (4 chars per token heuristic)
    pub fn count_text_tokens(text: &str) -> usize {
        // Standard approximation: ~4 characters per token
        (text.len() / 4).max(1)
    }

    /// Count tokens in a single message
    pub fn count_message_tokens(message: &Message) -> usize {
        let content_tokens = Self::count_text_tokens(&message.content);
        let role_tokens = Self::count_text_tokens(&message.role);

        // Add overhead for message structure (role, formatting, etc.)
        content_tokens + role_tokens + 4
    }

    /// Count tokens in a conversation (list of messages)
    pub fn count_messages_tokens(messages: &[Message]) -> usize {
        let message_tokens: usize = messages.iter().map(Self::count_message_tokens).sum();

        // Add overhead for conversation structure and separators
        message_tokens + (messages.len() * 2)
    }

    /// Count tokens in raw text with padding for overhead
    pub fn count_text_tokens_with_overhead(text: &str, overhead_percent: usize) -> usize {
        let base = Self::count_text_tokens(text);
        let overhead = (base * overhead_percent) / 100;
        base + overhead
    }

    /// Estimate input tokens for a request (text + overhead)
    pub fn estimate_input_tokens(prompt: &str) -> usize {
        // 3% overhead for request structure, headers, etc.
        Self::count_text_tokens_with_overhead(prompt, 3)
    }

    /// Estimate output tokens based on max_tokens parameter
    /// If max_tokens is None, estimate based on typical response
    pub fn estimate_output_tokens(max_tokens: Option<usize>) -> usize {
        max_tokens.unwrap_or(256)
    }

    /// Count tokens in JSON-serialized data
    pub fn count_json_tokens(json: &str) -> CostResult<usize> {
        let _ = serde_json::from_str::<serde_json::Value>(json)?;
        Ok(Self::count_text_tokens(json))
    }

    /// Validate token count is within reasonable bounds
    pub fn validate_token_count(tokens: i64) -> CostResult<usize> {
        if tokens < 0 {
            return Err(CostError::InvalidTokenCount(tokens));
        }

        // Max 1M tokens for a single request (reasonable limit)
        if tokens > 1_000_000 {
            return Err(CostError::CalculationError(format!(
                "Token count {} exceeds maximum of 1,000,000",
                tokens
            )));
        }

        Ok(tokens as usize)
    }
}

/// Token count with breakdown by type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenCountBreakdown {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

impl TokenCountBreakdown {
    /// Create a new token count breakdown
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }

    /// Calculate ratio of completion to prompt tokens
    pub fn completion_ratio(&self) -> f64 {
        if self.prompt_tokens == 0 {
            0.0
        } else {
            self.completion_tokens as f64 / self.prompt_tokens as f64
        }
    }

    /// Check if breakdown is valid
    pub fn is_valid(&self) -> bool {
        self.prompt_tokens > 0 && self.completion_tokens > 0 && self.total_tokens > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_text_tokens() {
        let text = "Hello world! This is a test.";
        let tokens = TokenCounter::count_text_tokens(text);
        assert!(tokens > 0);
        assert_eq!(tokens, (text.len() / 4).max(1));
    }

    #[test]
    fn test_count_text_tokens_short_text() {
        let text = "Hi";
        let tokens = TokenCounter::count_text_tokens(text);
        assert_eq!(tokens, 1); // min 1
    }

    #[test]
    fn test_count_message_tokens() {
        let msg = Message {
            role: "user".to_string(),
            content: "What is the meaning of life?".to_string(),
        };
        let tokens = TokenCounter::count_message_tokens(&msg);
        assert!(tokens > 0);
    }

    #[test]
    fn test_count_messages_tokens() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
        ];
        let tokens = TokenCounter::count_messages_tokens(&messages);
        assert!(tokens > 0);
    }

    #[test]
    fn test_estimate_input_tokens() {
        let prompt = "This is a test prompt with some content";
        let tokens = TokenCounter::estimate_input_tokens(prompt);
        assert!(tokens > 0);
    }

    #[test]
    fn test_estimate_output_tokens_with_max() {
        let tokens = TokenCounter::estimate_output_tokens(Some(512));
        assert_eq!(tokens, 512);
    }

    #[test]
    fn test_estimate_output_tokens_default() {
        let tokens = TokenCounter::estimate_output_tokens(None);
        assert_eq!(tokens, 256);
    }

    #[test]
    fn test_count_json_tokens() {
        let json = r#"{"name": "John", "age": 30}"#;
        let tokens = TokenCounter::count_json_tokens(json).unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_count_invalid_json_tokens() {
        let json = r#"{"invalid json"#;
        let result = TokenCounter::count_json_tokens(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_count_valid() {
        let result = TokenCounter::validate_token_count(1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_validate_token_count_negative() {
        let result = TokenCounter::validate_token_count(-100);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_count_exceeds_max() {
        let result = TokenCounter::validate_token_count(1_001_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_count_breakdown_new() {
        let breakdown = TokenCountBreakdown::new(100, 50);
        assert_eq!(breakdown.prompt_tokens, 100);
        assert_eq!(breakdown.completion_tokens, 50);
        assert_eq!(breakdown.total_tokens, 150);
    }

    #[test]
    fn test_token_count_breakdown_completion_ratio() {
        let breakdown = TokenCountBreakdown::new(100, 50);
        let ratio = breakdown.completion_ratio();
        assert!((ratio - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_token_count_breakdown_completion_ratio_zero_prompt() {
        let breakdown = TokenCountBreakdown::new(0, 50);
        assert_eq!(breakdown.completion_ratio(), 0.0);
    }

    #[test]
    fn test_token_count_breakdown_is_valid() {
        let valid = TokenCountBreakdown::new(100, 50);
        assert!(valid.is_valid());

        let invalid = TokenCountBreakdown::new(0, 0);
        assert!(!invalid.is_valid());
    }
}
