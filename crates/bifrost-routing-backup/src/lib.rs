// bifrost-routing: Unified LLM provider interface with intelligent routing
//
// Provides abstraction over multiple LLM providers (OpenAI, Anthropic, OpenRouter, Together)
// with intelligent routing strategies (round-robin, cost-aware, latency-aware, failover)

pub mod error;
pub mod models;
pub mod router;
pub mod tests;

pub use error::{BifrostError, BifrostResult};
pub use models::{
    Message, MessageRole, LLMRequest, LLMResponse, LLMProvider, ProviderMetadata, StreamingMessage,
    RoutingRequest,
};
pub use router::{Router, RoutingStrategy, RoutingStrategyType};
pub use tests as router_tests;
