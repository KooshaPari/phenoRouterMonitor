//! # Phenotype Telemetry Framework
//!
//! Generic telemetry trait consolidating 15+ trait definitions across the Phenotype ecosystem.
//! Provides duration tracking, status management, and async-safe telemetry collection.

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use phenotype_errors::Result;

/// Status of a telemetry event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryStatus {
    /// Operation has started
    Started,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
}

/// Core telemetry trait - generic over result type
///
/// Tracks operation lifecycle including start time, duration, and final status.
/// Can be used with both sync and async operations.
pub trait Telemetry<T: Send + Sync + 'static>: Send + Sync {
    /// Start recording telemetry
    fn start(&mut self);

    /// End recording with result
    fn end(&mut self, result: &Result<T>);

    /// Get recorded duration
    fn duration(&self) -> Duration;

    /// Get current status
    fn status(&self) -> TelemetryStatus;

    /// Get error message if failed (optional)
    fn error(&self) -> Option<&str> {
        None
    }
}

/// Basic duration tracker implementing Telemetry
///
/// Tracks start time, duration, status, and optional error message.
/// Suitable for synchronous operations and testing.
#[derive(Debug, Clone)]
pub struct DurationTracker {
    started_at: Option<Instant>,
    duration: Duration,
    status: TelemetryStatus,
    error: Option<String>,
}

impl DurationTracker {
    /// Create a new duration tracker
    pub fn new() -> Self {
        Self {
            started_at: None,
            duration: Duration::ZERO,
            status: TelemetryStatus::Started,
            error: None,
        }
    }

    /// Start tracking duration
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
        self.status = TelemetryStatus::Started;
        self.error = None;
    }

    /// End tracking with success/failure status
    pub fn end(&mut self, success: bool, error: Option<String>) {
        if let Some(start) = self.started_at {
            self.duration = start.elapsed();
        }
        self.status = if success {
            TelemetryStatus::Completed
        } else {
            TelemetryStatus::Failed
        };
        self.error = error;
    }
}

impl<T: Send + Sync + 'static> Telemetry<T> for DurationTracker {
    fn start(&mut self) {
        DurationTracker::start(self);
    }

    fn end(&mut self, result: &Result<T>) {
        let (success, error) = match result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        DurationTracker::end(self, success, error);
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn status(&self) -> TelemetryStatus {
        self.status
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Default for DurationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Async-safe telemetry wrapper using Arc/Mutex
///
/// Wraps a DurationTracker in Arc<Mutex<>> for safe sharing across async tasks.
/// Implements Clone to support arc cloning.
pub struct AsyncTelemetry<T> {
    inner: Arc<Mutex<DurationTracker>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> AsyncTelemetry<T> {
    /// Create a new async telemetry instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DurationTracker::new())),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Start recording telemetry (thread-safe)
    pub fn start(&self) {
        self.inner.lock().start();
    }

    /// End recording telemetry (thread-safe)
    pub fn end(&self, result: &Result<T>) {
        let mut guard = self.inner.lock();
        let (success, error) = match result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        guard.end(success, error);
    }

    /// Get the recorded duration (thread-safe)
    pub fn duration(&self) -> Duration {
        self.inner.lock().duration
    }

    /// Get the current status (thread-safe)
    pub fn status(&self) -> TelemetryStatus {
        self.inner.lock().status
    }

    /// Get the underlying Arc for advanced use cases
    pub fn clone_ref(&self) -> Arc<Mutex<DurationTracker>> {
        Arc::clone(&self.inner)
    }
}

impl<T: Send + Sync + 'static> Clone for AsyncTelemetry<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Default for AsyncTelemetry<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Telemetry context for recording multiple events
///
/// Aggregates multiple named events with their durations.
/// Useful for tracking composite operations with multiple phases.
#[derive(Debug, Clone)]
pub struct TelemetryContext {
    events: Vec<(String, Duration)>,
}

impl TelemetryContext {
    /// Create a new telemetry context
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Record an event with its duration
    pub fn record(&mut self, name: String, duration: Duration) {
        self.events.push((name, duration));
    }

    /// Get the total duration across all events
    pub fn total_duration(&self) -> Duration {
        self.events.iter().map(|(_, d)| d).sum()
    }

    /// Get all recorded events
    pub fn events(&self) -> &[(String, Duration)] {
        &self.events
    }

    /// Get the number of recorded events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for TelemetryContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_tracker_start_end() {
        let mut tracker = DurationTracker::new();
        DurationTracker::start(&mut tracker);
        std::thread::sleep(Duration::from_millis(10));
        DurationTracker::end(&mut tracker, true, None);
        assert!(tracker.duration >= Duration::from_millis(10));
        assert_eq!(tracker.status, TelemetryStatus::Completed);
    }

    #[test]
    fn test_duration_tracker_failure() {
        let mut tracker = DurationTracker::new();
        DurationTracker::start(&mut tracker);
        let error_msg = "test error".to_string();
        DurationTracker::end(&mut tracker, false, Some(error_msg.clone()));
        assert_eq!(tracker.status, TelemetryStatus::Failed);
        assert_eq!(tracker.error, Some(error_msg));
    }


    #[test]
    fn test_async_telemetry_new() {
        let telem = AsyncTelemetry::<i32>::new();
        telem.start();
        std::thread::sleep(Duration::from_millis(10));
        telem.end(&Ok(42));
        assert!(telem.duration() >= Duration::from_millis(10));
        assert_eq!(telem.status(), TelemetryStatus::Completed);
    }

    #[test]
    fn test_async_telemetry_clone() {
        let telem = AsyncTelemetry::<i32>::new();
        let telem2 = telem.clone();
        telem.start();
        std::thread::sleep(Duration::from_millis(5));
        telem.end(&Ok(42));

        // Both should see the same telemetry
        assert!(telem2.duration() >= Duration::from_millis(5));
        assert_eq!(telem2.status(), TelemetryStatus::Completed);
    }

    #[test]
    fn test_async_telemetry_default() {
        let telem = AsyncTelemetry::<String>::default();
        assert_eq!(telem.status(), TelemetryStatus::Started);
    }

    #[test]
    fn test_telemetry_context_single_event() {
        let mut ctx = TelemetryContext::new();
        ctx.record("event1".to_string(), Duration::from_millis(10));
        assert_eq!(ctx.total_duration(), Duration::from_millis(10));
        assert_eq!(ctx.event_count(), 1);
    }

    #[test]
    fn test_telemetry_context_multiple_events() {
        let mut ctx = TelemetryContext::new();
        ctx.record("event1".to_string(), Duration::from_millis(10));
        ctx.record("event2".to_string(), Duration::from_millis(20));
        ctx.record("event3".to_string(), Duration::from_millis(30));
        assert_eq!(ctx.total_duration(), Duration::from_millis(60));
        assert_eq!(ctx.event_count(), 3);
    }

    #[test]
    fn test_telemetry_context_events() {
        let mut ctx = TelemetryContext::new();
        ctx.record("event1".to_string(), Duration::from_millis(10));
        ctx.record("event2".to_string(), Duration::from_millis(20));
        let events = ctx.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, "event1");
        assert_eq!(events[1].1, Duration::from_millis(20));
    }

    #[test]
    fn test_telemetry_context_default() {
        let ctx = TelemetryContext::default();
        assert_eq!(ctx.event_count(), 0);
        assert_eq!(ctx.total_duration(), Duration::ZERO);
    }
}
