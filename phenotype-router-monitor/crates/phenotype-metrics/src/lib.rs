//! # phenotype-metrics
//!
//! High-performance metrics collection library with counters, gauges, and histograms.
//! Extracted from thegent-metrics with enhancements for centralized metric storage.
//!
//! ## Key Features
//! - Thread-safe Counter (Arc<Mutex<u64>>)
//! - Thread-safe Gauge (Arc<Mutex<f64>>)
//! - Histogram with percentile calculations
//! - Centralized MetricsRegistry with DashMap backend
//! - JSON serialization support
//! - Lock-free reads via Arc<Mutex<>> pattern

pub mod counter;
pub mod gauge;
pub mod histogram;
pub mod registry;
pub mod snapshot;
pub mod percentiles;

pub use counter::Counter;
pub use gauge::Gauge;
pub use histogram::Histogram;
pub use registry::MetricsRegistry;
pub use snapshot::MetricsSnapshot;
