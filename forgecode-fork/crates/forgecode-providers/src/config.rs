//! Configuration types for LLM providers

use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// OpenRouter provider (openrouter.ai)
    OpenRouter,
    /// Together AI provider (together.ai)
    Together,
    /// Anthropic provider (anthropic.com)
    Anthropic,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::OpenRouter => write!(f, "OpenRouter"),
            ProviderType::Together => write!(f, "Together"),
            ProviderType::Anthropic => write!(f, "Anthropic"),
        }
    }
}

/// Configuration for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type
    pub provider: ProviderType,

    /// API key for authentication
    pub api_key: String,

    /// Model identifier (e.g., "gpt-4", "meta-llama/Llama-2-70b")
    pub model: String,

    /// Temperature for response generation (0.0 to 2.0)
    pub temperature: f32,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// Top-p sampling parameter (0.0 to 1.0)
    pub top_p: f32,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Base URL for the API (optional for custom endpoints)
    pub base_url: Option<String>,
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new(provider: ProviderType, api_key: String, model: String) -> Self {
        Self {
            provider,
            api_key,
            model,
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            timeout_secs: 30,
            base_url: None,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(Error::MissingConfig {
                field: "api_key".to_string(),
            });
        }

        if self.model.is_empty() {
            return Err(Error::MissingConfig {
                field: "model".to_string(),
            });
        }

        if !(0.0..=2.0).contains(&self.temperature) {
            return Err(Error::InvalidConfig {
                field: "temperature".to_string(),
                reason: "must be between 0.0 and 2.0".to_string(),
            });
        }

        if self.max_tokens == 0 {
            return Err(Error::InvalidConfig {
                field: "max_tokens".to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        if !(0.0..=1.0).contains(&self.top_p) {
            return Err(Error::InvalidConfig {
                field: "top_p".to_string(),
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }

        Ok(())
    }

    /// Set the temperature parameter
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the top-p parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_new() {
        let config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(config.provider, ProviderType::OpenRouter);
        assert_eq!(config.api_key, "sk-key");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.base_url, None);
    }

    #[test]
    fn test_provider_config_validate_success() {
        let config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_validate_missing_api_key() {
        let config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "".to_string(),
            "gpt-4".to_string(),
        );
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_config_validate_invalid_temperature() {
        let mut config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        );
        config.temperature = 3.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_config_validate_invalid_top_p() {
        let mut config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        );
        config.top_p = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_config_with_temperature() {
        let config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        )
        .with_temperature(0.5);

        assert_eq!(config.temperature, 0.5);
    }

    #[test]
    fn test_provider_config_with_max_tokens() {
        let config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        )
        .with_max_tokens(4096);

        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn test_provider_config_with_base_url() {
        let config = ProviderConfig::new(
            ProviderType::OpenRouter,
            "sk-key".to_string(),
            "gpt-4".to_string(),
        )
        .with_base_url("https://custom.example.com".to_string());

        assert_eq!(config.base_url, Some("https://custom.example.com".to_string()));
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::OpenRouter.to_string(), "OpenRouter");
        assert_eq!(ProviderType::Together.to_string(), "Together");
        assert_eq!(ProviderType::Anthropic.to_string(), "Anthropic");
    }
}
