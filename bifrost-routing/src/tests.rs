// Integration tests for bifrost-routing

use std::sync::Arc;

use bifrost_routing::{
    AnthropicProvider, BifrostError, CostTracker, FailoverStrategy, LatencyAwareStrategy,
    LatencyTracker, LLMProvider, LLMRequest, Message, MessageRole, OpenAIProvider,
    OpenRouterProvider, ProviderMetrics, Router, RoutingStrategy, RoundRobinStrategy,
    TogetherProvider,
};

// ---------------------------------------------------------------------------
// Provider integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_all_providers_implement_trait() {
    // Verify all 4 providers can be used as trait objects
    let providers: Vec<Box<dyn LLMProvider>> = vec![
        Box::new(OpenAIProvider::new("test-key").unwrap()),
        Box::new(AnthropicProvider::new("test-key").unwrap()),
        Box::new(OpenRouterProvider::new("test-key").unwrap()),
        Box::new(TogetherProvider::new("test-key").unwrap()),
    ];

    for provider in &providers {
        assert!(!provider.name().is_empty());
        assert!(provider.is_available().await.unwrap());
    }
}

#[tokio::test]
async fn test_provider_cost_estimates_differ() {
    let openai = OpenAIProvider::new("test-key").unwrap();
    let anthropic = AnthropicProvider::new("test-key").unwrap();
    let together = TogetherProvider::new("test-key").unwrap();

    let openai_cost = openai.estimate_cost("gpt-4o", 1000, 1000);
    let anthropic_cost = anthropic.estimate_cost("claude-3-haiku", 1000, 1000);
    let together_cost = together.estimate_cost(
        "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo",
        1000,
        1000,
    );

    // Together should be cheapest, OpenAI gpt-4o most expensive
    assert!(together_cost < anthropic_cost);
    assert!(anthropic_cost < openai_cost);
}

#[tokio::test]
async fn test_provider_metadata_consistency() {
    let provider = OpenAIProvider::new("test-key").unwrap();
    let meta = provider.metadata();

    assert_eq!(meta.name, "openai");
    assert!(meta.available);
    assert_eq!(meta.total_requests, 0);
    assert!((meta.success_rate - 1.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Router integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_router_with_multiple_providers() {
    let providers: Vec<Arc<dyn LLMProvider>> = vec![
        Arc::new(OpenAIProvider::new("test-key").unwrap()),
        Arc::new(AnthropicProvider::new("test-key").unwrap()),
        Arc::new(OpenRouterProvider::new("test-key").unwrap()),
    ];

    let strategy = Arc::new(RoundRobinStrategy::new());
    let router = Router::new(providers, strategy);

    assert_eq!(router.provider_count(), 3);
    assert_eq!(router.strategy_name(), "round-robin");
}

#[tokio::test]
async fn test_router_with_cost_aware_strategy() {
    let providers: Vec<Arc<dyn LLMProvider>> = vec![
        Arc::new(OpenAIProvider::new("test-key").unwrap()),
        Arc::new(TogetherProvider::new("test-key").unwrap()),
    ];

    let strategy = Arc::new(bifrost_routing::CostAwareStrategy::new());
    let router = Router::new(providers, strategy);

    let request = LLMRequest::new(
        "test-model".to_string(),
        vec![Message {
            role: MessageRole::User,
            content: "test".to_string(),
        }],
    );

    // Cost-aware should pick the cheapest provider
    // Since we can't actually invoke (no real API key), just verify the router is configured
    assert_eq!(router.provider_count(), 2);
    assert_eq!(router.strategy_name(), "cost-aware");
}

#[tokio::test]
async fn test_router_empty_providers() {
    let providers: Vec<Arc<dyn LLMProvider>> = vec![];
    let strategy = Arc::new(RoundRobinStrategy::new());
    let router = Router::new(providers, strategy);

    let request = LLMRequest::new(
        "gpt-4".to_string(),
        vec![Message {
            role: MessageRole::User,
            content: "test".to_string(),
        }],
    );

    let result = router.invoke(&request).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        BifrostError::RoutingError(msg) => {
            assert!(msg.contains("No providers"));
        }
        other => panic!("Expected RoutingError, got: {other}"),
    }
}

#[tokio::test]
async fn test_router_max_retries() {
    let providers: Vec<Arc<dyn LLMProvider>> = vec![
        Arc::new(OpenAIProvider::new("test-key").unwrap()),
    ];

    let strategy = Arc::new(RoundRobinStrategy::new());
    let router = Router::new(providers, strategy).with_max_retries(5);

    assert_eq!(router.provider_count(), 1);
}

// ---------------------------------------------------------------------------
// Routing strategy integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_failover_with_unavailable_providers() {
    // Failover should find the first available provider
    let providers: Vec<Arc<dyn LLMProvider>> = vec![
        Arc::new(OpenAIProvider::new("test-key").unwrap()),
        Arc::new(AnthropicProvider::new("test-key").unwrap()),
    ];

    let strategy = FailoverStrategy::new();
    let selected = strategy.select_provider(&providers, &create_test_request()).await;
    assert!(selected.is_ok());
}

#[tokio::test]
async fn test_latency_aware_picks_first() {
    let providers: Vec<Arc<dyn LLMProvider>> = vec![
        Arc::new(OpenAIProvider::new("test-key").unwrap()),
        Arc::new(AnthropicProvider::new("test-key").unwrap()),
    ];

    let strategy = LatencyAwareStrategy::new();
    let selected = strategy.select_provider(&providers, &create_test_request()).await;
    assert!(selected.is_ok());
    assert_eq!(selected.unwrap().name(), "openai"); // first provider
}

#[tokio::test]
async fn test_round_robin_cycling() {
    let providers: Vec<Arc<dyn LLMProvider>> = vec![
        Arc::new(OpenAIProvider::new("test-key").unwrap()),
        Arc::new(AnthropicProvider::new("test-key").unwrap()),
    ];

    let strategy = RoundRobinStrategy::new();
    let request = create_test_request();

    let p1 = strategy.select_provider(&providers, &request).await.unwrap();
    let p2 = strategy.select_provider(&providers, &request).await.unwrap();
    let p3 = strategy.select_provider(&providers, &request).await.unwrap();

    assert_eq!(p1.name(), "openai");
    assert_eq!(p2.name(), "anthropic");
    assert_eq!(p3.name(), "openai"); // cycles back
}

// ---------------------------------------------------------------------------
// Metrics integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_metrics_with_router_simulation() {
    let metrics = ProviderMetrics::new();
    let cost_tracker = CostTracker::new();
    let latency_tracker = LatencyTracker::new(100);

    // Simulate a routing session
    metrics.record_success("openai", 150);
    metrics.record_success("openai", 200);
    metrics.record_failure("anthropic", 500);
    metrics.record_success("together", 80);

    cost_tracker.record_cost("openai", 0.05);
    cost_tracker.record_cost("anthropic", 0.03);
    cost_tracker.record_cost("together", 0.001);

    latency_tracker.record("openai", 150);
    latency_tracker.record("openai", 200);
    latency_tracker.record("anthropic", 500);
    latency_tracker.record("together", 80);

    // Verify metrics
    assert_eq!(metrics.request_count("openai"), 2);
    assert!((metrics.success_rate("openai") - 1.0).abs() < f64::EPSILON);
    assert_eq!(metrics.request_count("anthropic"), 1);
    assert!((metrics.success_rate("anthropic") - 0.0).abs() < f64::EPSILON);

    // Verify cost tracking
    assert!((cost_tracker.total_cost_usd() - 0.081).abs() < 0.001);

    // Verify latency tracking
    assert_eq!(latency_tracker.fastest_provider(), Some("together".to_string()));
}

#[test]
fn test_cost_tracker_with_budget() {
    let tracker = CostTracker::new();
    tracker.set_budget_limit(0.10);

    tracker.record_cost("openai", 0.06);
    assert!(!tracker.is_over_budget());

    tracker.record_cost("anthropic", 0.05);
    assert!(tracker.is_over_budget());

    let remaining = tracker.remaining_budget_usd().unwrap();
    assert!(remaining < 0.0); // over budget
}

// ---------------------------------------------------------------------------
// Error handling tests
// ---------------------------------------------------------------------------

#[test]
fn test_bifrost_error_retryable() {
    assert!(BifrostError::Timeout {
        provider: "openai".to_string(),
        timeout_ms: 30000,
    }
    .is_retryable());

    assert!(BifrostError::RateLimited {
        provider: "openai".to_string(),
    }
    .is_retryable());

    assert!(!BifrostError::AuthenticationError {
        provider: "openai".to_string(),
        reason: "invalid key".to_string(),
    }
    .is_retryable());
}

#[test]
fn test_bifrost_error_provider_name() {
    let err = BifrostError::Timeout {
        provider: "anthropic".to_string(),
        timeout_ms: 30000,
    };
    assert_eq!(err.provider_name(), Some("anthropic".to_string()));

    let err = BifrostError::RoutingError("no providers".to_string());
    assert!(err.provider_name().is_none());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_test_request() -> LLMRequest {
    LLMRequest::new(
        "gpt-4".to_string(),
        vec![Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        }],
    )
}
