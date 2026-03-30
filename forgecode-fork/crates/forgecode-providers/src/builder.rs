//! Builder pattern for provider configuration and instantiation

use crate::config::{ProviderConfig, ProviderType};
use crate::error::{Error, Result};
use crate::providers::{
    AnthropicProvider, OpenRouterProvider, Provider, TogetherProvider,
};

/// Builder for creating LLM providers with fluent API
#[derive(Debug, Clone)]
pub struct ProviderBuilder {
    provider_type: Option<ProviderType>,
    api_key: Option<String>,
    model: Option<String>,
    temperature: f32,
    max_tokens: u32,
    top_p: f32,
    timeout_secs: u64,
    base_url: Option<String>,
}

impl Default for ProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderBuilder {
    /// Create a new provider builder with default values
    pub fn new() -> Self {
        Self {
            provider_type: None,
            api_key: None,
            model: None,
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            timeout_secs: 30,
            base_url: None,
        }
    }

    /// Set the provider type
    pub fn provider(mut self, provider: ProviderType) -> Self {
        self.provider_type = Some(provider);
        self
    }

    /// Set the API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the model identifier
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the temperature parameter (0.0 to 2.0)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set the maximum tokens in response
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the top-p sampling parameter (0.0 to 1.0)
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    /// Set the request timeout in seconds
    pub fn timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set a custom base URL for the API
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Build and validate the provider configuration
    pub fn build(self) -> Result<Provider> {
        let provider_type = self.provider_type.ok_or_else(|| Error::MissingConfig {
            field: "provider".to_string(),
        })?;

        let api_key = self.api_key.ok_or_else(|| Error::MissingConfig {
            field: "api_key".to_string(),
        })?;

        let model = self.model.ok_or_else(|| Error::MissingConfig {
            field: "model".to_string(),
        })?;

        // Validate temperature
        if !(0.0..=2.0).contains(&self.temperature) {
            return Err(Error::InvalidConfig {
                field: "temperature".to_string(),
                reason: "must be between 0.0 and 2.0".to_string(),
            });
        }

        // Validate max_tokens
        if self.max_tokens == 0 {
            return Err(Error::InvalidConfig {
                field: "max_tokens".to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate top_p
        if !(0.0..=1.0).contains(&self.top_p) {
            return Err(Error::InvalidConfig {
                field: "top_p".to_string(),
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }

        // Create provider based on type
        let provider = match provider_type {
            ProviderType::OpenRouter => {
                let openrouter = if let Some(base_url) = &self.base_url {
                    OpenRouterProvider::with_base_url(api_key, model, base_url.clone())
                } else {
                    OpenRouterProvider::new(api_key, model)
                };
                Provider::OpenRouter(openrouter)
            }
            ProviderType::Together => {
                let together = if let Some(base_url) = &self.base_url {
                    TogetherProvider::with_base_url(api_key, model, base_url.clone())
                } else {
                    TogetherProvider::new(api_key, model)
                };
                Provider::Together(together)
            }
            ProviderType::Anthropic => {
                let anthropic = if let Some(base_url) = &self.base_url {
                    AnthropicProvider::with_base_url(api_key, model, base_url.clone())
                } else {
                    AnthropicProvider::new(api_key, model)
                };
                Provider::Anthropic(anthropic)
            }
        };

        Ok(provider)
    }

    /// Build the configuration without creating a provider
    pub fn build_config(self) -> Result<ProviderConfig> {
        let provider_type = self.provider_type.ok_or_else(|| Error::MissingConfig {
            field: "provider".to_string(),
        })?;

        let api_key = self.api_key.ok_or_else(|| Error::MissingConfig {
            field: "api_key".to_string(),
        })?;

        let model = self.model.ok_or_else(|| Error::MissingConfig {
            field: "model".to_string(),
        })?;

        let config = ProviderConfig::new(provider_type, api_key, model)
            .with_temperature(self.temperature)
            .with_max_tokens(self.max_tokens)
            .with_top_p(self.top_p)
            .with_timeout(self.timeout_secs);

        let config = if let Some(base_url) = self.base_url {
            config.with_base_url(base_url)
        } else {
            config
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = ProviderBuilder::new();
        assert_eq!(builder.temperature, 0.7);
        assert_eq!(builder.max_tokens, 2048);
        assert_eq!(builder.top_p, 0.9);
        assert_eq!(builder.timeout_secs, 30);
    }

    #[test]
    fn test_builder_openrouter_full() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-openrouter-key")
            .model("gpt-4")
            .temperature(0.8)
            .max_tokens(4096)
            .top_p(0.95)
            .timeout(60)
            .build();

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_type(), ProviderType::OpenRouter);
        assert_eq!(provider.model(), "gpt-4");
    }

    #[test]
    fn test_builder_together_full() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::Together)
            .api_key("sk-together-key")
            .model("meta-llama/Llama-2-70b")
            .build();

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_type(), ProviderType::Together);
        assert_eq!(provider.model(), "meta-llama/Llama-2-70b");
    }

    #[test]
    fn test_builder_anthropic_full() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::Anthropic)
            .api_key("sk-ant-key")
            .model("claude-3-opus-20240229")
            .build();

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_type(), ProviderType::Anthropic);
    }

    #[test]
    fn test_builder_missing_provider() {
        let result = ProviderBuilder::new()
            .api_key("sk-key")
            .model("gpt-4")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_api_key() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .model("gpt-4")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_model() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_invalid_temperature() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .model("gpt-4")
            .temperature(3.0)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_invalid_max_tokens() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .model("gpt-4")
            .max_tokens(0)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_invalid_top_p() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .model("gpt-4")
            .top_p(1.5)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_custom_base_url() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .model("gpt-4")
            .base_url("https://custom.example.com")
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_build_config() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .model("gpt-4")
            .temperature(0.5)
            .max_tokens(1024)
            .build_config();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 1024);
    }

    #[test]
    fn test_builder_config_invalid_temperature() {
        let result = ProviderBuilder::new()
            .provider(ProviderType::OpenRouter)
            .api_key("sk-key")
            .model("gpt-4")
            .temperature(2.5)
            .build_config();

        assert!(result.is_err());
    }
}
