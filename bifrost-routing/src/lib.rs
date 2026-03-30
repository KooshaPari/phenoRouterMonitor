// bifrost-routing: Unified LLM provider interface with intelligent routing
//
// Provides abstraction over multiple LLM providers (OpenAI, Anthropic, OpenRouter, Together)
// with intelligent routing strategies (round-robin, cost-aware, latency-aware, failover)

pub mod error;
pub mod models;
pub mod providers;
pub mod router;
pub mod metrics;

pub use error::BifrostError;
pub use models::{
    LLMRequest, LLMResponse, LLMProvider, ProviderMetadata, StreamingMessage,
};
pub use providers::{
    OpenAIProvider, AnthropicProvider, OpenRouterProvider, TogetherProvider,
};
pub use router::{Router, RoutingStrategy, RoutingStrategyType};
pub use metrics::{ProviderMetrics, CostTracker, LatencyTracker};

#[cfg(test)]
mod tests;
