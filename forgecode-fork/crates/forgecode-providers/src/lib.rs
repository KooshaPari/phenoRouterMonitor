//! forgecode-providers: Multi-provider LLM API abstraction with builder pattern
//!
//! This crate provides a unified interface for multiple LLM providers:
//! - OpenRouter
//! - Together AI
//! - Anthropic
//!
//! # Example
//!
//! ```ignore
//! use forgecode_providers::{ProviderBuilder, Provider};
//!
//! let provider = ProviderBuilder::new()
//!     .provider(Provider::OpenRouter)
//!     .api_key("sk-...")
//!     .model("gpt-4")
//!     .temperature(0.7)
//!     .max_tokens(2048)
//!     .build()
//!     .expect("valid configuration");
//! ```

pub mod error;
pub mod providers;
pub mod config;
pub mod builder;

pub use error::{Error, Result};
pub use providers::{Provider, LlmProvider};
pub use config::ProviderConfig;
pub use builder::ProviderBuilder;

/// Current version of forgecode-providers
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        assert_eq!(VERSION, "0.1.0");
    }
}
