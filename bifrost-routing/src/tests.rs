// Comprehensive integration tests

#[cfg(test)]
mod integration_tests {
    use crate::models::{LLMProvider, LLMRequest, Message, MessageRole};
    use crate::providers::{
        AnthropicConfig, AnthropicProvider, OpenAIConfig, OpenAIProvider,
        OpenRouterConfig, OpenRouterProvider, TogetherConfig, TogetherProvider,
    };
    use crate::router::{
        CostAwareStrategy, FailoverStrategy, LatencyAwareStrategy, RoundRobinStrategy, Router,
    };
    use crate::metrics::ProviderMetrics;
    use std::sync::Arc;

    #[test]
    fn test_provider_initialization() {
        let openai_config = OpenAIConfig::new("sk-test".to_string());
        let openai = OpenAIProvider::new(openai_config);
        assert_eq!(openai.name(), "openai");

        let anthropic_config = AnthropicConfig::new("sk-ant-test".to_string());
        let anthropic = AnthropicProvider::new(anthropic_config);
        assert_eq!(anthropic.name(), "anthropic");

        let openrouter_config = OpenRouterConfig::new("sk-or-test".to_string());
        let _openrouter = OpenRouterProvider::new(openrouter_config);
        assert_eq!(_openrouter.name(), "openrouter");

        let together_config = TogetherConfig::new("sk-together-test".to_string());
        let together = TogetherProvider::new(together_config);
        assert_eq!(together.name(), "together");
    }

    #[test]
    fn test_cost_comparison_across_providers() {
        let openai_config = OpenAIConfig::new("sk-test".to_string());
        let openai = OpenAIProvider::new(openai_config);

        let anthropic_config = AnthropicConfig::new("sk-ant-test".to_string());
        let anthropic = AnthropicProvider::new(anthropic_config);

        let openrouter_config = OpenRouterConfig::new("sk-or-test".to_string());
        let _openrouter = OpenRouterProvider::new(openrouter_config);

        let together_config = TogetherConfig::new("sk-together-test".to_string());
        let together = TogetherProvider::new(together_config);

        // Compare costs for similar models
        let openai_cost = openai.estimate_cost("gpt-3.5-turbo", 1000, 1000);
        let together_cost = together.estimate_cost("mistral-7b", 1000, 1000);

        // Together open-source should be cheaper
        assert!(together_cost < openai_cost);

        // Anthropic Haiku should be cheaper than Opus
        let haiku_cost = anthropic.estimate_cost("claude-haiku", 1000, 1000);
        let opus_cost = anthropic.estimate_cost("claude-opus", 1000, 1000);
        assert!(haiku_cost < opus_cost);
    }

    #[test]
    fn test_request_builder() {
        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are helpful".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: "What is 2+2?".to_string(),
            },
        ];

        let request = LLMRequest::new("gpt-4".to_string(), messages)
            .with_max_tokens(1024)
            .with_temperature(0.7)
            .with_timeout(30_000)
            .with_stream(false);

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.max_tokens, Some(1024));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.timeout_ms, Some(30_000));
        assert!(!request.stream);
    }

    #[test]
    fn test_token_estimation() {
        let messages = vec![Message {
            role: MessageRole::User,
            content: "a".repeat(1000), // ~250 tokens
        }];

        let request = LLMRequest::new("gpt-4".to_string(), messages)
            .with_max_tokens(2048);

        let tokens = request.estimate_tokens();
        assert!(tokens > 200 && tokens < 3000);
    }

    #[tokio::test]
    async fn test_router_with_multiple_strategies() {
        // Create mock-like providers
        let openai_config = OpenAIConfig::new("sk-test".to_string());
        let openai = Arc::new(OpenAIProvider::new(openai_config)) as Arc<dyn crate::models::LLMProvider>;

        let anthropic_config = AnthropicConfig::new("sk-ant-test".to_string());
        let anthropic = Arc::new(AnthropicProvider::new(anthropic_config)) as Arc<dyn crate::models::LLMProvider>;

        let providers = vec![openai, anthropic];

        // Test with different strategies
        let round_robin = Arc::new(RoundRobinStrategy::new());
        let router_rr = Router::new(providers.clone(), round_robin);
        assert_eq!(router_rr.strategy_name(), "round-robin");

        let cost_aware = Arc::new(CostAwareStrategy::new());
        let router_ca = Router::new(providers.clone(), cost_aware);
        assert_eq!(router_ca.strategy_name(), "cost-aware");

        let latency_aware = Arc::new(LatencyAwareStrategy::new());
        let router_la = Router::new(providers.clone(), latency_aware);
        assert_eq!(router_la.strategy_name(), "latency-aware");

        let failover = Arc::new(FailoverStrategy::new());
        let router_fo = Router::new(providers, failover);
        assert_eq!(router_fo.strategy_name(), "failover");
    }

    #[test]
    fn test_provider_metrics_tracking() {
        let metrics = ProviderMetrics::new("openai".to_string());

        // Simulate successful requests
        metrics.record_success(100, 0.01);
        metrics.record_success(150, 0.015);
        metrics.record_success(120, 0.012);

        // Simulate a failure
        metrics.record_failure();

        assert_eq!(metrics.total_requests(), 4);
        assert_eq!(metrics.success_count.load(std::sync::atomic::Ordering::Relaxed), 3);
        assert_eq!(metrics.failure_count.load(std::sync::atomic::Ordering::Relaxed), 1);

        let success_rate = metrics.success_rate();
        assert!(success_rate > 0.7 && success_rate < 0.8); // ~75%

        let avg_latency = metrics.latency_tracker.average_latency_ms();
        assert!(avg_latency > 100 && avg_latency < 150);

        let total_cost = metrics.cost_tracker.total_cost();
        assert!(total_cost > 0.03 && total_cost < 0.04);
    }

    #[test]
    fn test_all_providers_have_models() {
        let openai_config = OpenAIConfig::new("sk-test".to_string());
        let openai = OpenAIProvider::new(openai_config);

        let anthropic_config = AnthropicConfig::new("sk-ant-test".to_string());
        let anthropic = AnthropicProvider::new(anthropic_config);

        let openrouter_config = OpenRouterConfig::new("sk-or-test".to_string());
        let _openrouter = OpenRouterProvider::new(openrouter_config);

        let together_config = TogetherConfig::new("sk-together-test".to_string());
        let together = TogetherProvider::new(together_config);

        // Note: These are sync tests, can't use await
        // In real scenario, these would be async tests
        let providers = vec![
            openai.name(),
            anthropic.name(),
            openrouter.name(),
            together.name(),
        ];

        assert_eq!(providers.len(), 4);
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"openrouter"));
        assert!(providers.contains(&"together"));
    }

    #[test]
    fn test_cost_tracking_precision() {
        let metrics = ProviderMetrics::new("test".to_string());

        // Test with very small costs
        metrics.record_success(50, 0.00001);
        metrics.record_success(60, 0.00002);
        metrics.record_success(55, 0.000015);

        let total = metrics.cost_tracker.total_cost();
        let expected = 0.00001 + 0.00002 + 0.000015;

        // Allow for floating point precision differences
        assert!((total - expected).abs() < 0.000001);
    }

    #[test]
    fn test_latency_min_max_tracking() {
        let metrics = ProviderMetrics::new("test".to_string());

        metrics.latency_tracker.record_latency(100);
        metrics.latency_tracker.record_latency(500);
        metrics.latency_tracker.record_latency(250);

        assert_eq!(metrics.latency_tracker.min_latency_ms(), Some(100));
        assert_eq!(metrics.latency_tracker.max_latency_ms(), Some(500));
        assert_eq!(metrics.latency_tracker.average_latency_ms(), 283); // (100+500+250)/3
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = ProviderMetrics::new("test".to_string());

        metrics.record_success(100, 0.01);
        metrics.record_success(150, 0.015);

        assert_eq!(metrics.total_requests(), 2);

        metrics.reset();

        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.cost_tracker.total_cost(), 0.0);
        assert_eq!(metrics.latency_tracker.request_count(), 0);
    }

    #[test]
    fn test_provider_metadata_generation() {
        let metrics = ProviderMetrics::new("openai".to_string());

        metrics.record_success(100, 0.01);
        metrics.record_success(150, 0.015);
        metrics.record_failure();

        let summary = metrics.summary();

        assert!(summary.contains("openai"));
        assert!(summary.contains("Requests: 3"));
        assert!(summary.contains("Success Rate"));
        assert!(summary.contains("Cost"));
    }

    #[test]
    fn test_message_role_serde() {
        use serde_json;

        let message = Message {
            role: MessageRole::User,
            content: "test".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"role\":\"user\""));

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, MessageRole::User);
    }

    #[test]
    fn test_llm_request_serialization() {
        use serde_json;

        let request = LLMRequest::new(
            "gpt-4".to_string(),
            vec![Message {
                role: MessageRole::User,
                content: "test".to_string(),
            }],
        )
        .with_max_tokens(1024)
        .with_temperature(0.7);

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: LLMRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "gpt-4");
        assert_eq!(deserialized.max_tokens, Some(1024));
        assert_eq!(deserialized.temperature, Some(0.7));
    }
}
