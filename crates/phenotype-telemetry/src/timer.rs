//! Timer utilities for measuring span duration and recording in histograms.

use std::time::Instant;

use crate::registry::Histogram;

/// RAII guard that records elapsed time to a histogram on Drop.
pub struct SpanTimer {
    name: String,
    start: Instant,
    on_drop: Option<Box<dyn Fn(String, u128) + Send>>,
}

impl std::fmt::Debug for SpanTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpanTimer")
            .field("name", &self.name)
            .field("start", &self.start)
            .field("on_drop", &"<callback>")
            .finish()
    }
}

impl SpanTimer {
    /// Create a new span timer with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            on_drop: None,
        }
    }

    /// Set a callback to invoke on drop with (name, duration_ms).
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(String, u128) + Send + 'static,
    {
        self.on_drop = Some(Box::new(callback));
        self
    }

    /// Get the elapsed duration in milliseconds without consuming the timer.
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

impl Drop for SpanTimer {
    fn drop(&mut self) {
        if let Some(callback) = self.on_drop.take() {
            callback(self.name.clone(), self.elapsed_ms());
        }
    }
}

/// Timed function wrapper — records duration into an optional [`Histogram`].
///
/// When `histogram` is `Some`, the elapsed milliseconds are recorded as a
/// histogram observation via [`Histogram::observe`].  In all cases the result
/// of the future is returned unchanged.
pub async fn timed<F, Fut, T>(name: &str, histogram: Option<&Histogram>, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = f().await;
    let elapsed = start.elapsed().as_millis() as f64;
    tracing::debug!(name, elapsed_ms = elapsed, "timing recorded");
    if let Some(h) = histogram {
        h.observe(elapsed);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[test]
    fn span_timer_elapsed() {
        let timer = SpanTimer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }

    #[test]
    fn span_timer_callback() {
        let recorded = Arc::new(AtomicU64::new(0));
        let recorded_clone = recorded.clone();

        {
            let _timer = SpanTimer::new("test").with_callback(move |_name, duration| {
                let duration_u64 = duration.min(u64::MAX as u128) as u64;
                recorded_clone.store(duration_u64, Ordering::Relaxed);
            });
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let duration = recorded.load(Ordering::Relaxed);
        assert!(duration >= 10);
    }

    #[tokio::test]
    async fn timed_async() {
        let result = timed("async_op", None, || async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        })
        .await;

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn timed_async_with_histogram() {
        use crate::registry::{MetricsRegistry, TelemetryConfig};

        let registry = MetricsRegistry::new(TelemetryConfig {
            service_name: "test".into(),
            environment: "test".into(),
        });
        let histogram = registry.histogram("test");

        let result = timed("async_op", Some(&histogram), || async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            99
        })
        .await;

        assert_eq!(result, 99);
        assert_eq!(histogram.len(), 1);
        assert!(histogram.values()[0] >= 10.0);
    }
}
