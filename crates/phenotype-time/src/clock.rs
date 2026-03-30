//! Clock trait and implementations for testable time handling

use crate::Timestamp;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

#[derive(Debug)]
pub struct MockClock {
    millis: Arc<AtomicI64>,
}

impl MockClock {
    #[must_use]
    pub fn new(millis: i64) -> Self {
        Self {
            millis: Arc::new(AtomicI64::new(millis)),
        }
    }
    pub fn advance(&self, ms: i64) {
        self.millis.fetch_add(ms, Ordering::Relaxed);
    }
    pub fn set(&self, millis: i64) {
        self.millis.store(millis, Ordering::Relaxed);
    }
    #[must_use]
    pub fn millis(&self) -> i64 {
        self.millis.load(Ordering::Relaxed)
    }
}

impl Clone for MockClock {
    fn clone(&self) -> Self {
        Self {
            millis: Arc::clone(&self.millis),
        }
    }
}

impl Clock for MockClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(self.millis.load(Ordering::Relaxed))
            .expect("MockClock millis out of range")
    }
}

#[derive(Debug)]
pub enum FlexClock {
    System,
    Mock(MockClock),
}

impl FlexClock {
    #[must_use]
    pub fn system() -> Self {
        Self::System
    }
    #[must_use]
    pub fn mock(initial_millis: i64) -> Self {
        Self::Mock(MockClock::new(initial_millis))
    }
    #[must_use]
    pub fn as_mock(&self) -> Option<&MockClock> {
        match self {
            Self::Mock(clock) => Some(clock),
            Self::System => None,
        }
    }
}

impl Clock for FlexClock {
    fn now(&self) -> Timestamp {
        match self {
            Self::System => SystemClock.now(),
            Self::Mock(clock) => clock.now(),
        }
    }
}

impl Clone for FlexClock {
    fn clone(&self) -> Self {
        match self {
            Self::System => Self::System,
            Self::Mock(clock) => Self::Mock(clock.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_clock() {
        let c = SystemClock;
        assert!(c.now().to_millis() > 0);
    }
    #[test]
    fn mock_clock_new() {
        let c = MockClock::new(42000);
        assert_eq!(c.now().to_millis(), 42000);
    }
    #[test]
    fn mock_clock_advance() {
        let c = MockClock::new(1000);
        c.advance(500);
        assert_eq!(c.now().to_millis(), 1500);
    }
    #[test]
    fn flex_clock_system() {
        let c = FlexClock::system();
        assert!(c.now().to_millis() > 0);
        assert!(c.as_mock().is_none());
    }
    #[test]
    fn flex_clock_mock() {
        let c = FlexClock::mock(5000);
        assert_eq!(c.now().to_millis(), 5000);
        assert!(c.as_mock().is_some());
    }
}
