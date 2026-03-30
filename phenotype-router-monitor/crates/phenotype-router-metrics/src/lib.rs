//! High-performance metrics collection for thegent.

use dashmap::DashMap;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    value: Arc<Mutex<u64>>,
}

impl Counter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: Arc::new(Mutex::new(0)),
        }
    }

    pub fn inc(&self, delta: u64) {
        let mut v = self.value.lock().expect("lock poisoned");
        *v += delta;
    }

    pub fn value(&self) -> u64 {
        *self.value.lock().expect("lock poisoned")
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    value: Arc<Mutex<f64>>,
}

impl Gauge {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn set(&self, val: f64) {
        *self.value.lock().expect("lock poisoned") = val;
    }

    pub fn value(&self) -> f64 {
        *self.value.lock().expect("lock poisoned")
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    values: Arc<Mutex<Vec<u64>>>,
    _buckets: usize,
}

impl Histogram {
    pub fn new(name: &str, buckets: usize) -> Self {
        Self {
            name: name.to_string(),
            values: Arc::new(Mutex::new(Vec::new())),
            _buckets: buckets,
        }
    }

    pub fn record(&self, value: u64) {
        let mut v = self.values.lock().expect("lock poisoned");
        v.push(value);
        v.sort_unstable();
    }

    pub fn count(&self) -> usize {
        self.values.lock().expect("lock poisoned").len()
    }

    pub fn p50(&self) -> f64 {
        self.percentile(50)
    }

    pub fn p99(&self) -> f64 {
        self.percentile(99)
    }

    fn percentile(&self, pct: usize) -> f64 {
        let values = self.values.lock().expect("lock poisoned");
        if values.is_empty() {
            return 0.0;
        }
        let idx = (values.len() * pct) / 100;
        let idx = idx.min(values.len() - 1);
        values[idx] as f64
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct MetricsRegistry {
    counters: DashMap<String, u64>,
    gauges: DashMap<String, f64>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
            gauges: DashMap::new(),
        }
    }

    pub fn add_counter(&self, name: &str, value: u64) {
        self.counters.insert(name.to_string(), value);
    }

    pub fn add_gauge(&self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn snapshot(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::new();

        for entry in self.counters.iter() {
            result.push(json!({
                "name": entry.key(),
                "type": "counter",
                "value": entry.value()
            }));
        }

        for entry in self.gauges.iter() {
            result.push(json!({
                "name": entry.key(),
                "type": "gauge",
                "value": entry.value()
            }));
        }

        result
    }

    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        for entry in self.counters.iter() {
            output.push_str(&format!("{} {}\n", entry.key(), entry.value()));
        }

        for entry in self.gauges.iter() {
            output.push_str(&format!("{} {}\n", entry.key(), entry.value()));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @trace FR-MET-001
    #[test]
    fn test_counter_increment() {
        let counter = Counter::new("requests_total");

        counter.inc(1);
        assert_eq!(counter.value(), 1);

        counter.inc(5);
        assert_eq!(counter.value(), 6);
    }

    /// @trace FR-MET-002
    #[test]
    fn test_gauge_set() {
        let gauge = Gauge::new("memory_usage_bytes");

        gauge.set(1024.0);
        assert_eq!(gauge.value(), 1024.0);

        gauge.set(2048.0);
        assert_eq!(gauge.value(), 2048.0);
    }

    /// @trace FR-MET-003
    #[test]
    fn test_histogram_record() {
        let histogram = Histogram::new("latency_ms", 10);

        histogram.record(5);
        histogram.record(15);
        histogram.record(25);

        assert_eq!(histogram.count(), 3);
        assert!((histogram.p50() - 15.0).abs() < f64::EPSILON);
    }

    /// @trace FR-MET-003
    #[test]
    fn test_histogram_p99() {
        let histogram = Histogram::new("latency_ms", 10);

        for i in 1..=100 {
            histogram.record(i);
        }

        assert!(histogram.p99() >= 99.0);
    }

    /// @trace FR-MET-004
    #[test]
    fn test_registry_collect() {
        let registry = MetricsRegistry::new();

        registry.add_counter("requests", 100);
        registry.add_gauge("memory", 1024.0);

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 2);
    }

    /// @trace FR-MET-005
    #[test]
    fn test_registry_export_prometheus() {
        let registry = MetricsRegistry::new();

        registry.add_counter("requests", 42);

        let output = registry.export_prometheus();
        assert!(output.contains("requests 42"));
    }

    /// @trace FR-MET-001
    #[test]
    fn test_counter_clone_shares_state() {
        let counter = Counter::new("test");
        let clone = counter.clone();

        counter.inc(10);
        assert_eq!(clone.value(), 10);
    }

    /// @trace FR-MET-003
    #[test]
    fn test_histogram_empty() {
        let histogram = Histogram::new("empty", 10);
        assert_eq!(histogram.count(), 0);
        assert_eq!(histogram.p50(), 0.0);
        assert_eq!(histogram.p99(), 0.0);
    }
}
