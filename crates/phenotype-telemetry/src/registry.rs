//! Metrics registry for collecting and storing telemetry data.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Telemetry service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub environment: String,
}

/// Metric types supported by the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Metric {
    #[serde(rename = "counter")]
    Counter { value: u64 },
    #[serde(rename = "gauge")]
    Gauge { value: f64 },
    #[serde(rename = "histogram")]
    Histogram { values: Vec<f64> },
}

/// Counter metric — incremental counter.
#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    value: Arc<std::sync::atomic::AtomicU64>,
}

impl Counter {
    /// Get the metric name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Increment the counter by 1.
    pub fn inc(&self) {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increment the counter by `delta`.
    pub fn add(&self, delta: u64) {
        self.value
            .fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get the current value.
    pub fn value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Gauge metric — point-in-time measurement.
#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    value: Arc<std::sync::Mutex<f64>>,
}

impl Gauge {
    /// Get the metric name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the gauge to a specific value.
    pub fn set(&self, value: f64) {
        *self.value.lock().unwrap() = value;
    }

    /// Get the current value.
    pub fn value(&self) -> f64 {
        *self.value.lock().unwrap()
    }
}

/// Histogram metric — distribution of values.
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    values: Arc<std::sync::Mutex<Vec<f64>>>,
}

impl Histogram {
    /// Metric name as registered in the registry.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Record a value in the histogram.
    pub fn observe(&self, value: f64) {
        self.values.lock().unwrap().push(value);
    }

    /// Get all recorded values.
    pub fn values(&self) -> Vec<f64> {
        self.values.lock().unwrap().clone()
    }
}

/// Metrics registry for centralized metric management.
#[derive(Debug)]
pub struct MetricsRegistry {
    config: TelemetryConfig,
    counters: DashMap<String, Counter>,
    gauges: DashMap<String, Gauge>,
    histograms: DashMap<String, Histogram>,
}

impl MetricsRegistry {
    /// Get the telemetry config.
    #[must_use]
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }

    /// Create a new metrics registry.
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            counters: DashMap::new(),
            gauges: DashMap::new(),
            histograms: DashMap::new(),
        }
    }

    /// Get or create a counter.
    pub fn counter(&self, name: impl Into<String>) -> Counter {
        let name = name.into();
        self.counters
            .entry(name.clone())
            .or_insert_with(|| Counter {
                name: name.clone(),
                value: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            })
            .clone()
    }

    /// Get or create a gauge.
    pub fn gauge(&self, name: impl Into<String>) -> Gauge {
        let name = name.into();
        self.gauges
            .entry(name.clone())
            .or_insert_with(|| Gauge {
                name: name.clone(),
                value: Arc::new(std::sync::Mutex::new(0.0)),
            })
            .clone()
    }

    /// Get or create a histogram.
    pub fn histogram(&self, name: impl Into<String>) -> Histogram {
        let name = name.into();
        self.histograms
            .entry(name.clone())
            .or_insert_with(|| Histogram {
                name: name.clone(),
                values: Arc::new(std::sync::Mutex::new(Vec::new())),
            })
            .clone()
    }

    /// Get all metrics as a map.
    pub fn snapshot(&self) -> HashMap<String, Metric> {
        let mut snapshot = HashMap::new();

        for entry in self.counters.iter() {
            snapshot.insert(
                entry.key().clone(),
                Metric::Counter {
                    value: entry.value().value(),
                },
            );
        }

        for entry in self.gauges.iter() {
            snapshot.insert(
                entry.key().clone(),
                Metric::Gauge {
                    value: entry.value().value(),
                },
            );
        }

        for entry in self.histograms.iter() {
            snapshot.insert(
                entry.key().clone(),
                Metric::Histogram {
                    values: entry.value().values(),
                },
            );
        }

        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_increment() {
        let counter = Counter {
            name: "test".into(),
            value: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        };
        counter.inc();
        assert_eq!(counter.value(), 1);
        counter.add(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn gauge_operations() {
        let gauge = Gauge {
            name: "test".into(),
            value: Arc::new(std::sync::Mutex::new(0.0)),
        };
        gauge.set(42.5);
        assert_eq!(gauge.value(), 42.5);
    }

    #[test]
    #[test]
    fn histogram_observe() {
        let histogram = Histogram {
            name: "test".into(),
            values: Arc::new(std::sync::Mutex::new(Vec::new())),
        };
        assert_eq!(histogram.name(), "test");
        histogram.observe(1.0);
        histogram.observe(2.5);
        let values = histogram.values();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], 1.0);
        assert_eq!(values[1], 2.5);
    }

    #[test]
    fn registry_metrics() {
        let config = TelemetryConfig {
            service_name: "test-service".into(),
            environment: "test".into(),
        };
        let registry = MetricsRegistry::new(config);

        let counter = registry.counter("requests");
        counter.inc();
        counter.inc();

        let gauge = registry.gauge("memory");
        gauge.set(256.0);

        let histogram = registry.histogram("latency");
        histogram.observe(50.0);

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 3);
    }
}
