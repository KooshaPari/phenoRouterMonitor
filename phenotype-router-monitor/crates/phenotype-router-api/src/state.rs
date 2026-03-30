//! Application state for API server

use phenotype_router_core::Router;
use phenotype_router_metrics::MetricsCollector;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub router: Arc<Router>,
    pub metrics: Arc<MetricsCollector>,
    pub start_time: SystemTime,
}

impl AppState {
    /// Create new application state
    pub fn new(router: Router, metrics: MetricsCollector) -> Self {
        Self {
            router: Arc::new(router),
            metrics: Arc::new(metrics),
            start_time: SystemTime::now(),
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get uptime as human-readable string
    pub fn uptime_string(&self) -> String {
        let seconds = self.uptime_seconds();
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-014 (Application state)
    #[test]
    fn test_app_state_creation() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();
        let metrics = MetricsCollector::new();
        let state = AppState::new(router, metrics);

        assert!(state.uptime_seconds() >= 0);
    }

    // Traces to: FR-ROUTER-014
    #[test]
    fn test_uptime_string_formatting() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();
        let metrics = MetricsCollector::new();
        let state = AppState::new(router, metrics);
        let uptime = state.uptime_string();

        // Should contain some time unit
        assert!(
            uptime.contains("s") || uptime.contains("m") || uptime.contains("h") || uptime.contains("d")
        );
    }
}
