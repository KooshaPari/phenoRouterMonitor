//! Drop-based span timer that records duration to a histogram.

use std::sync::Arc;
use std::time::Instant;

use crate::registry::{Histogram, MetricsRegistry};

/// Measures wall-clock duration from creation to drop and records the elapsed
/// milliseconds into the associated histogram.
///
/// Create via [`timed`] or manually with [`SpanTimer::new`].
#[derive(Debug)]
pub struct SpanTimer {
    start: Instant,
    histogram: Arc<Histogram>,
}

impl SpanTimer {
    /// Start a new timer that will record into `histogram` on drop.
    pub fn new(histogram: Arc<Histogram>) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }

    /// Elapsed time since the timer was created.
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Drop for SpanTimer {
    fn drop(&mut self) {
        let ms = self.elapsed_ms();
        self.histogram.observe(ms);
        tracing::trace!(elapsed_ms = ms, "span timer recorded");
    }
}

/// Default histogram buckets for duration measurements (milliseconds).
const DEFAULT_DURATION_BUCKETS: &[f64] = &[
    1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
];

/// Convenience: start a [`SpanTimer`] that records to a histogram named
/// `"{name}_duration_ms"` in the given registry.
pub fn timed(name: &str, registry: &MetricsRegistry) -> SpanTimer {
    let hist_name = format!("{name}_duration_ms");
    let histogram = registry.histogram(&hist_name, DEFAULT_DURATION_BUCKETS.to_vec());
    SpanTimer::new(histogram)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::TelemetryConfig;

    #[test]
    fn timer_records_on_drop() {
        let r = MetricsRegistry::new(TelemetryConfig::default());
        {
            let _t = timed("db_query", &r);
            // simulate work
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let h = r.histogram("db_query_duration_ms", vec![]);
        assert_eq!(h.count(), 1);
        assert!(h.sum() >= 4.0, "expected >= 4ms, got {}", h.sum());
    }
}
