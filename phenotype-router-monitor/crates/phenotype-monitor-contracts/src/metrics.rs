//! Metrics domain contracts

use serde::{Deserialize, Serialize};

/// Core metrics collector trait
pub trait MetricCollector: Send + Sync {
    fn counter(&self, name: &str) -> crate::error::Result<Counter>;
    fn gauge(&self, name: &str) -> crate::error::Result<Gauge>;
    fn histogram(&self, name: &str, buckets: usize) -> crate::error::Result<Histogram>;
    fn export(&self) -> crate::error::Result<MetricsSnapshot>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    pub name: String,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gauge {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub name: String,
    pub count: u64,
    pub sum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: String,
    pub counters: std::collections::HashMap<String, u64>,
    pub gauges: std::collections::HashMap<String, f64>,
}
