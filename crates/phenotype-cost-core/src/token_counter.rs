// Token counting utilities

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
        (text.len() / 4).max(1)
    }

    /// Count tokens in a single message
    pub fn count_message_tokens(message: &Message) -> usize {
        let content_tokens = Self::count_text_tokens(&message.content);
        let role_tokens = Self::count_text_tokens(&message.role);
        content_tokens + role_tokens + 4
    }

    /// Count tokens in a conversation (list of messages)
    pub fn count_messages_tokens(messages: &[Message]) -> usize {
        let message_tokens: usize = messages.iter().map(Self::count_message_tokens).sum();
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
        Self::count_text_tokens_with_overhead(prompt, 3)
    }

    /// Estimate output tokens based on max_tokens parameter
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
    }

    #[test]
    fn test_estimate_input_tokens() {
        let prompt = "This is a test prompt with some content";
        let tokens = TokenCounter::estimate_input_tokens(prompt);
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_count_breakdown() {
        let breakdown = TokenCountBreakdown::new(100, 50);
        assert_eq!(breakdown.prompt_tokens, 100);
        assert_eq!(breakdown.total_tokens, 150);
    }
}
