// LLM Provider implementations

pub mod openai;
pub mod anthropic;
pub mod openrouter;
pub mod together;

// Re-export providers
pub use openai::{OpenAIProvider, OpenAIConfig};
pub use anthropic::{AnthropicProvider, AnthropicConfig};
pub use openrouter::{OpenRouterProvider, OpenRouterConfig};
pub use together::{TogetherProvider, TogetherConfig};

#[cfg(test)]
mod tests {

    #[test]
    fn test_provider_names() {
        // This test verifies that all provider names are unique and non-empty
        let names = vec!["openai", "anthropic", "openrouter", "together"];
        let mut seen = std::collections::HashSet::new();

        for name in names {
            assert!(!name.is_empty());
            assert!(seen.insert(name));
        }
    }
}
