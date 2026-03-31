use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub model: String,
    pub prompt: String,
    pub task_type: Option<String>,
    pub metadata: serde_json::Value,
}

impl RoutingRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            task_type: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_task(mut self, task: impl Into<String>) -> Self {
        self.task_type = Some(task.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata[key.into()] = value;
        self
    }
}

#[derive(Debug, Clone)]
pub struct RouterDecision {
    pub provider: String,
    pub reasoning: String,
    pub estimated_cost_usd: f64,
    pub estimated_latency_ms: u32,
    pub confidence: f64,
}

impl RouterDecision {
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            reasoning: String::new(),
            estimated_cost_usd: 0.0,
            estimated_latency_ms: 0,
            confidence: 0.0,
        }
    }

    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = reasoning.into();
        self
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost_usd = cost;
        self
    }

    pub fn with_latency(mut self, latency: u32) -> Self {
        self.estimated_latency_ms = latency;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResponse {
    pub decision: RouterDecision,
    pub timestamp_ms: u64,
    pub router_name: String,
}

impl RoutingResponse {
    pub fn new(decision: RouterDecision, router_name: impl Into<String>) -> Self {
        Self {
            decision,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            router_name: router_name.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f64,
    pub avg_cost_usd: f64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.failed_requests as f64 / self.total_requests as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_request_builder() {
        let request = RoutingRequest::new("gpt-4o", "Hello world")
            .with_task("chat")
            .with_metadata("user_id", serde_json::json!("123"));

        assert_eq!(request.model, "gpt-4o");
        assert_eq!(request.prompt, "Hello world");
        assert_eq!(request.task_type, Some("chat".to_string()));
        assert_eq!(request.metadata["user_id"], "123");
    }

    #[test]
    fn test_router_decision_builder() {
        let decision = RouterDecision::new("gpt-4o")
            .with_reasoning("Selected for analysis")
            .with_cost(0.003)
            .with_latency(500)
            .with_confidence(0.95);

        assert_eq!(decision.provider, "gpt-4o");
        assert_eq!(decision.reasoning, "Selected for analysis");
        assert_eq!(decision.estimated_cost_usd, 0.003);
        assert_eq!(decision.estimated_latency_ms, 500);
        assert_eq!(decision.confidence, 0.95);
    }

    #[test]
    fn test_metrics_hit_rate() {
        let mut metrics = Metrics::new();
        metrics.total_requests = 100;
        metrics.successful_requests = 90;
        metrics.failed_requests = 10;

        assert_eq!(metrics.hit_rate(), 0.9);
        assert_eq!(metrics.error_rate(), 0.1);
    }

    #[test]
    fn test_metrics_zero_requests() {
        let metrics = Metrics::new();
        assert_eq!(metrics.hit_rate(), 0.0);
        assert_eq!(metrics.error_rate(), 0.0);
    }
}
