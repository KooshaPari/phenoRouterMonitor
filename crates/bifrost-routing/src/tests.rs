// Tests for bifrost-routing router strategies

#[cfg(test)]
mod tests {
    use crate::router::{RoundRobinStrategy, CostAwareStrategy, LatencyAwareStrategy, RoutingStrategy};
    use crate::models::{LLMRequest, Message, MessageRole};
    use crate::error::BifrostError;

    fn create_test_request() -> LLMRequest {
        LLMRequest::new(
            "gpt-4".to_string(),
            vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
        )
    }

    #[test]
    fn test_round_robin_strategy_name() {
        let strategy = RoundRobinStrategy::new();
        assert_eq!(strategy.name(), "round-robin");
    }

    #[test]
    fn test_cost_aware_strategy_name() {
        let strategy = CostAwareStrategy::new();
        assert_eq!(strategy.name(), "cost-aware");
    }

    #[test]
    fn test_latency_aware_strategy_name() {
        let strategy = LatencyAwareStrategy::new();
        assert_eq!(strategy.name(), "latency-aware");
    }

    #[test]
    fn test_routing_request_builder() {
        let request = create_test_request();
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_round_robin_empty_providers() {
        let strategy = RoundRobinStrategy::new();
        let request = create_test_request();
        let result = strategy.select_provider(&[], &request).await;
        assert!(result.is_err());
    }
}
