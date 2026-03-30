//! Core metrics registry and metric types.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for telemetry initialisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Logical service name (e.g. `"agileplus-api"`).
    pub service_name: String,
    /// Deployment environment (e.g. `"production"`, `"staging"`).
    pub environment: String,
    /// How often metrics should be exported (advisory; no backend in this crate).
    #[serde(with = "duration_millis")]
    pub export_interval: Duration,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "unknown".into(),
            environment: "development".into(),
            export_interval: Duration::from_secs(30),
        }
    }
}

// ---------------------------------------------------------------------------
// Metric types (lock-free atomics)
// ---------------------------------------------------------------------------

/// A monotonically increasing counter (u64).
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Increment by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by an arbitrary amount.
    pub fn inc_by(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Current value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A gauge that can go up or down (i64).
#[derive(Debug)]
pub struct Gauge {
    value: AtomicI64,
}

impl Gauge {
    fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }

    /// Set to an absolute value.
    pub fn set(&self, v: i64) {
        self.value.store(v, Ordering::Relaxed);
    }

    /// Increment by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement by 1.
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Current value.
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A histogram that records observations into pre-defined buckets.
#[derive(Debug)]
pub struct Histogram {
    /// Upper-bound for each bucket (sorted ascending, last is +Inf conceptually).
    buckets: Vec<f64>,
    /// Counts per bucket (index-aligned with `buckets`), plus one overflow bucket.
    counts: Vec<AtomicU64>,
    /// Running sum of all observed values (stored as bits).
    sum_bits: AtomicU64,
    /// Total observation count.
    total: AtomicU64,
}

impl Histogram {
    fn new(buckets: Vec<f64>) -> Self {
        let mut sorted = buckets;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = sorted.len() + 1; // +1 for the +Inf overflow bucket
        let counts = (0..len).map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets: sorted,
            counts,
            sum_bits: AtomicU64::new(0.0f64.to_bits()),
            total: AtomicU64::new(0),
        }
    }

    /// Record an observation.
    pub fn observe(&self, value: f64) {
        self.total.fetch_add(1, Ordering::Relaxed);

        // Atomically add to sum via CAS loop on the bit representation.
        loop {
            let old_bits = self.sum_bits.load(Ordering::Relaxed);
            let old = f64::from_bits(old_bits);
            let new = old + value;
            if self
                .sum_bits
                .compare_exchange_weak(
                    old_bits,
                    new.to_bits(),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        // Increment the first bucket whose bound >= value, or the overflow.
        let idx = self
            .buckets
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(self.buckets.len());
        self.counts[idx].fetch_add(1, Ordering::Relaxed);
    }

    /// Total observations.
    pub fn count(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    /// Sum of all observations.
    pub fn sum(&self) -> f64 {
        f64::from_bits(self.sum_bits.load(Ordering::Relaxed))
    }

    /// Bucket boundaries (sorted ascending).
    pub fn bucket_bounds(&self) -> &[f64] {
        &self.buckets
    }

    /// Count per bucket (index-aligned with [`bucket_bounds`], plus one trailing
    /// overflow entry).
    pub fn bucket_counts(&self) -> Vec<u64> {
        self.counts
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Metric enum (for snapshot serialisation)
// ---------------------------------------------------------------------------

/// Discriminated metric value for snapshot purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Metric {
    Counter { name: String, value: u64 },
    Gauge { name: String, value: i64 },
    Histogram {
        name: String,
        count: u64,
        sum: f64,
        buckets: Vec<f64>,
        bucket_counts: Vec<u64>,
    },
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Central, thread-safe metrics registry.
///
/// All metric handles are behind `Arc` so callers can hold cheap references.
#[derive(Debug)]
pub struct MetricsRegistry {
    config: TelemetryConfig,
    counters: DashMap<String, Arc<Counter>>,
    gauges: DashMap<String, Arc<Gauge>>,
    histograms: DashMap<String, Arc<Histogram>>,
}

impl MetricsRegistry {
    /// Create a new registry with the given configuration.
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            counters: DashMap::new(),
            gauges: DashMap::new(),
            histograms: DashMap::new(),
        }
    }

    /// Configuration used to create this registry.
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }

    /// Get or create a counter.
    pub fn counter(&self, name: &str) -> Arc<Counter> {
        self.counters
            .entry(name.to_owned())
            .or_insert_with(|| Arc::new(Counter::new()))
            .clone()
    }

    /// Get or create a gauge.
    pub fn gauge(&self, name: &str) -> Arc<Gauge> {
        self.gauges
            .entry(name.to_owned())
            .or_insert_with(|| Arc::new(Gauge::new()))
            .clone()
    }

    /// Get or create a histogram with the given bucket boundaries.
    ///
    /// If the histogram already exists, the existing instance is returned and
    /// `buckets` is ignored.
    pub fn histogram(&self, name: &str, buckets: Vec<f64>) -> Arc<Histogram> {
        self.histograms
            .entry(name.to_owned())
            .or_insert_with(|| Arc::new(Histogram::new(buckets)))
            .clone()
    }

    /// Collect a point-in-time snapshot of every registered metric.
    pub fn snapshot(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();

        for entry in self.counters.iter() {
            metrics.push(Metric::Counter {
                name: entry.key().clone(),
                value: entry.value().get(),
            });
        }
        for entry in self.gauges.iter() {
            metrics.push(Metric::Gauge {
                name: entry.key().clone(),
                value: entry.value().get(),
            });
        }
        for entry in self.histograms.iter() {
            let h = entry.value();
            metrics.push(Metric::Histogram {
                name: entry.key().clone(),
                count: h.count(),
                sum: h.sum(),
                buckets: h.bucket_bounds().to_vec(),
                bucket_counts: h.bucket_counts(),
            });
        }

        metrics
    }
}

// ---------------------------------------------------------------------------
// Serde helper for Duration as milliseconds
// ---------------------------------------------------------------------------

mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_millis() as u64)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_increments() {
        let r = MetricsRegistry::new(TelemetryConfig::default());
        let c = r.counter("req_total");
        c.inc();
        c.inc_by(5);
        assert_eq!(c.get(), 6);
    }

    #[test]
    fn gauge_up_down() {
        let r = MetricsRegistry::new(TelemetryConfig::default());
        let g = r.gauge("active_conns");
        g.inc();
        g.inc();
        g.dec();
        assert_eq!(g.get(), 1);
        g.set(42);
        assert_eq!(g.get(), 42);
    }

    #[test]
    fn histogram_observations() {
        let r = MetricsRegistry::new(TelemetryConfig::default());
        let h = r.histogram("latency_ms", vec![10.0, 50.0, 100.0, 500.0]);
        h.observe(5.0);
        h.observe(25.0);
        h.observe(75.0);
        h.observe(999.0);
        assert_eq!(h.count(), 4);
        assert!((h.sum() - 1104.0).abs() < f64::EPSILON);
        let counts = h.bucket_counts();
        // buckets: [10, 50, 100, 500, +Inf]
        assert_eq!(counts, vec![1, 1, 1, 0, 1]);
    }

    #[test]
    fn snapshot_serialises() {
        let r = MetricsRegistry::new(TelemetryConfig::default());
        r.counter("a").inc();
        r.gauge("b").set(7);
        r.histogram("c", vec![1.0]).observe(0.5);
        let snap = r.snapshot();
        let json = serde_json::to_string(&snap).expect("serialise");
        assert!(json.contains("\"type\":\"counter\""));
        assert!(json.contains("\"type\":\"gauge\""));
        assert!(json.contains("\"type\":\"histogram\""));
    }

    #[test]
    fn same_name_returns_same_handle() {
        let r = MetricsRegistry::new(TelemetryConfig::default());
        let c1 = r.counter("x");
        c1.inc();
        let c2 = r.counter("x");
        assert_eq!(c2.get(), 1);
    }
}
