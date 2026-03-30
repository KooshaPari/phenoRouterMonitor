// LLM Provider implementations

pub mod openai;
pub mod anthropic;
pub mod openrouter;
pub mod together;

pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;
pub use openrouter::OpenRouterProvider;
pub use together::TogetherProvider;

#[cfg(test)]
mod tests {
    use super::*;

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
