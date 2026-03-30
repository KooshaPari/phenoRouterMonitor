//! Point-in-time metrics snapshot for health endpoints and reporting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::registry::{Metric, MetricsRegistry};

/// A timestamped, JSON-serializable snapshot of all registered metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Service that produced the snapshot.
    pub service_name: String,
    /// Environment tag.
    pub environment: String,
    /// When the snapshot was taken.
    pub timestamp: DateTime<Utc>,
    /// All metric values at snapshot time.
    pub metrics: Vec<Metric>,
}

impl MetricsSnapshot {
    /// Capture a snapshot from the given registry right now.
    pub fn capture(registry: &MetricsRegistry) -> Self {
        Self {
            service_name: registry.config().service_name.clone(),
            environment: registry.config().environment.clone(),
            timestamp: Utc::now(),
            metrics: registry.snapshot(),
        }
    }

    /// Serialize to a JSON string (convenience wrapper).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::TelemetryConfig;

    #[test]
    fn snapshot_roundtrips_json() {
        let r = MetricsRegistry::new(TelemetryConfig {
            service_name: "test-svc".into(),
            environment: "test".into(),
            ..Default::default()
        });
        r.counter("hits").inc_by(42);
        r.gauge("queue_depth").set(3);

        let snap = MetricsSnapshot::capture(&r);
        let json = snap.to_json().expect("serialise");
        let decoded: MetricsSnapshot = serde_json::from_str(&json).expect("deserialise");

        assert_eq!(decoded.service_name, "test-svc");
        assert_eq!(decoded.metrics.len(), 2);
    }
}
