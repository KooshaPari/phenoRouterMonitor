// Routing logic for selecting providers and handling failover

use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::error::{BifrostError, BifrostResult};
use crate::models::{LLMProvider, LLMRequest, LLMResponse};

/// Routing strategy trait
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// Select a provider from the available list
    async fn select_provider<'a>(
        &self,
        providers: &'a [Arc<dyn LLMProvider>],
        request: &LLMRequest,
    ) -> BifrostResult<&'a Arc<dyn LLMProvider>>;

    /// Get strategy name
    fn name(&self) -> &str;
}

/// Available routing strategy types
#[derive(Debug, Clone, Copy)]
pub enum RoutingStrategyType {
    /// Round-robin: cycle through providers
    RoundRobin,
    /// Cost-aware: prefer cheapest provider
    CostAware,
    /// Latency-aware: prefer fastest provider
    LatencyAware,
    /// Failover: use first available, switch on failure
    Failover,
    /// Priority order with simple per-provider rate limiting
    PriorityRateLimited,
}

/// Round-robin routing strategy
pub struct RoundRobinStrategy {
    counter: Arc<AtomicUsize>,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RoutingStrategy for RoundRobinStrategy {
    async fn select_provider<'a>(
        &self,
        providers: &'a [Arc<dyn LLMProvider>],
        _request: &LLMRequest,
    ) -> BifrostResult<&'a Arc<dyn LLMProvider>> {
        if providers.is_empty() {
            return Err(BifrostError::RoutingError("No providers available".to_string()));
        }

        let index = self.counter.fetch_add(1, Ordering::Relaxed) % providers.len();
        Ok(&providers[index])
    }

    fn name(&self) -> &str {
        "round-robin"
    }
}

/// Cost-aware routing strategy
pub struct CostAwareStrategy;

impl CostAwareStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CostAwareStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RoutingStrategy for CostAwareStrategy {
    async fn select_provider<'a>(
        &self,
        providers: &'a [Arc<dyn LLMProvider>],
        request: &LLMRequest,
    ) -> BifrostResult<&'a Arc<dyn LLMProvider>> {
        if providers.is_empty() {
            return Err(BifrostError::RoutingError("No providers available".to_string()));
        }

        let prompt_tokens = request.estimate_tokens();
        let completion_tokens = request.max_tokens.unwrap_or(2048);

        let mut best_provider = &providers[0];
        let mut best_cost = f64::INFINITY;

        for provider in providers {
            let cost = provider.estimate_cost(&request.model, prompt_tokens, completion_tokens);
            if cost < best_cost {
                best_cost = cost;
                best_provider = provider;
            }
        }

        Ok(best_provider)
    }

    fn name(&self) -> &str {
        "cost-aware"
    }
}

/// Latency-aware routing strategy
pub struct LatencyAwareStrategy;

impl LatencyAwareStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LatencyAwareStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RoutingStrategy for LatencyAwareStrategy {
    async fn select_provider<'a>(
        &self,
        providers: &'a [Arc<dyn LLMProvider>],
        _request: &LLMRequest,
    ) -> BifrostResult<&'a Arc<dyn LLMProvider>> {
        if providers.is_empty() {
            return Err(BifrostError::RoutingError("No providers available".to_string()));
        }

        // For now, return first provider
        // In real implementation, would track historical latencies
        Ok(&providers[0])
    }

    fn name(&self) -> &str {
        "latency-aware"
    }
}

/// Failover routing strategy
pub struct FailoverStrategy;

impl FailoverStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FailoverStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RoutingStrategy for FailoverStrategy {
    async fn select_provider<'a>(
        &self,
        providers: &'a [Arc<dyn LLMProvider>],
        _request: &LLMRequest,
    ) -> BifrostResult<&'a Arc<dyn LLMProvider>> {
        if providers.is_empty() {
            return Err(BifrostError::RoutingError("No providers available".to_string()));
        }

        // Return first available provider
        for provider in providers {
            if provider.is_available().await.unwrap_or(false) {
                return Ok(provider);
            }
        }

        Err(BifrostError::RoutingError("No available providers".to_string()))
    }

    fn name(&self) -> &str {
        "failover"
    }
}

/// Priority-based strategy with per-provider rate limiting
pub struct PriorityRateLimitedStrategy {
    /// Provider names in priority order.
    priority: Vec<String>,
    /// Maximum requests allowed per provider in a window.
    limit: usize,
    /// Window duration in seconds.
    window_secs: u64,
    /// Requests counters, per provider: (count, window_start_timestamp)
    counters: std::sync::Arc<dashmap::DashMap<String, (usize, Instant)>>,
}

impl PriorityRateLimitedStrategy {
    pub fn new(priority: Vec<String>, limit: usize, window_secs: u64) -> Self {
        Self {
            priority,
            limit,
            window_secs,
            counters: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    fn check_limit(&self, provider_name: &str) -> bool {
        let now = Instant::now();
        let mut entry = self
            .counters
            .entry(provider_name.to_string())
            .or_insert((0, now));

        let (count, start) = *entry;
        if now.duration_since(*start).as_secs() >= self.window_secs {
            *entry = (1, now);
            true
        } else if count < self.limit {
            *entry = (count + 1, *start);
            true
        } else {
            false
        }
    }
}

#[async_trait]
impl RoutingStrategy for PriorityRateLimitedStrategy {
    async fn select_provider<'a>(
        &self,
        providers: &'a [Arc<dyn LLMProvider>],
        _request: &LLMRequest,
    ) -> BifrostResult<&'a Arc<dyn LLMProvider>> {
        if providers.is_empty() {
            return Err(BifrostError::RoutingError("No providers available".to_string()));
        }

        // First attempt priority-ordered providers.
        for preferred in &self.priority {
            for provider in providers {
                if provider.name().eq_ignore_ascii_case(preferred) {
                    if provider.is_available().await.unwrap_or(false) {
                        if self.check_limit(provider.name()) {
                            return Ok(provider);
                        }
                    }
                }
            }
        }

        // Fallback to any available provider respecting rates if possible.
        for provider in providers {
            if provider.is_available().await.unwrap_or(false)
                && self.check_limit(provider.name())
            {
                return Ok(provider);
            }
        }

        Err(BifrostError::RateLimited {
            provider: "all".to_string(),
        })
    }

    fn name(&self) -> &str {
        "priority-rate-limited"
    }
}

/// Router for selecting and invoking providers
pub struct Router {
    providers: Vec<Arc<dyn LLMProvider>>,
    strategy: Arc<dyn RoutingStrategy>,
    max_retries: usize,
}

impl Router {
    pub fn new(
        providers: Vec<Arc<dyn LLMProvider>>,
        strategy: Arc<dyn RoutingStrategy>,
    ) -> Self {
        Self {
            providers,
            strategy,
            max_retries: 3,
        }
    }

    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn strategy_name(&self) -> &str {
        self.strategy.name()
    }

    pub async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse> {
        if self.providers.is_empty() {
            return Err(BifrostError::RoutingError("No providers available".to_string()));
        }

        let provider = self.strategy.select_provider(&self.providers, request).await?;
        provider.invoke(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LLMProvider, Message, MessageRole, LLMRequest};

    // Mock provider for testing
    struct MockProvider {
        name: String,
        available: bool,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                available: true,
            }
        }
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn is_available(&self) -> BifrostResult<bool> {
            Ok(self.available)
        }

        async fn invoke(&self, request: &LLMRequest) -> BifrostResult<LLMResponse> {
            Ok(LLMResponse {
                response_id: "resp-123".to_string(),
                request_id: request.request_id.clone(),
                content: "Hello from ".to_string() + &self.name,
                model: request.model.clone(),
                provider: self.name.clone(),
                prompt_tokens: request.estimate_tokens(),
                completion_tokens: 10,
                cost_usd: 0.01,
                latency_ms: 100,
                stop_reason: None,
                finished_at: chrono::Utc::now(),
            })
        }

        async fn invoke_streaming(
            &self,
            _request: &LLMRequest,
        ) -> BifrostResult<Box<dyn std::any::Any>> {
            unimplemented!()
        }

        fn estimate_cost(
            &self,
            _model: &str,
            _prompt_tokens: usize,
            _completion_tokens: usize,
        ) -> f64 {
            0.01
        }

        async fn list_models(&self) -> BifrostResult<Vec<String>> {
            Ok(vec![request.model.clone()])
        }

        fn metadata(&self) -> crate::models::ProviderMetadata {
            crate::models::ProviderMetadata {
                name: self.name.clone(),
                available: self.available,
                latency_ms: Some(100),
                last_error: None,
                total_requests: 0,
                total_cost_usd: 0.0,
                success_rate: 1.0,
            }
        }
    }

    #[tokio::test]
    async fn test_round_robin_strategy_creation() {
        let strategy = RoundRobinStrategy::new();
        assert_eq!(strategy.name(), "round-robin");
    }

    #[tokio::test]
    async fn test_cost_aware_strategy_creation() {
        let strategy = CostAwareStrategy::new();
        assert_eq!(strategy.name(), "cost-aware");
    }

    #[tokio::test]
    async fn test_latency_aware_strategy_creation() {
        let strategy = LatencyAwareStrategy::new();
        assert_eq!(strategy.name(), "latency-aware");
    }

    #[tokio::test]
    async fn test_failover_strategy_creation() {
        let strategy = FailoverStrategy::new();
        assert_eq!(strategy.name(), "failover");
    }

    #[tokio::test]
    async fn test_router_creation() {
        let providers = vec![Arc::new(MockProvider::new("test")) as Arc<dyn LLMProvider>];
        let strategy = Arc::new(RoundRobinStrategy::new());
        let router = Router::new(providers, strategy);

        assert_eq!(router.provider_count(), 1);
        assert_eq!(router.strategy_name(), "round-robin");
    }

    #[tokio::test]
    async fn test_router_invoke() {
        let providers = vec![Arc::new(MockProvider::new("test")) as Arc<dyn LLMProvider>];
        let strategy = Arc::new(RoundRobinStrategy::new());
        let router = Router::new(providers, strategy);

        let request = LLMRequest::new(
            "gpt-4".to_string(),
            vec![Message {
                role: MessageRole::User,
                content: "test".to_string(),
            }],
        );

        let response = router.invoke(&request).await.unwrap();
        assert_eq!(response.provider, "test");
        assert_eq!(response.content, "Hello from test");
    }

    #[tokio::test]
    async fn test_priority_rate_limited_strategy() {
        let p1 = Arc::new(MockProvider::new("coderabbit")) as Arc<dyn LLMProvider>;
        let p2 = Arc::new(MockProvider::new("gca")) as Arc<dyn LLMProvider>;

        let providers = vec![p1.clone(), p2.clone()];
        let strategy = Arc::new(PriorityRateLimitedStrategy::new(
            vec!["coderabbit".to_string(), "gca".to_string()],
            2,
            60,
        ));

        let router = Router::new(providers, strategy);

        let request = LLMRequest::new(
            "gpt-4".to_string(),
            vec![Message {
                role: MessageRole::User,
                content: "test".to_string(),
            }],
        );

        // First request should use coderabbit as priority
        let response1 = router.invoke(&request).await.unwrap();
        assert_eq!(response1.provider, "coderabbit");

        // Second request should also use coderabbit due to within limit.
        let response2 = router.invoke(&request).await.unwrap();
        assert_eq!(response2.provider, "coderabbit");

        // Third request should failover to gca since coderabbit limit reached.
        let response3 = router.invoke(&request).await.unwrap();
        assert_eq!(response3.provider, "gca");
    }
}
