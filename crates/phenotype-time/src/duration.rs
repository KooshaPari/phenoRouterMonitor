//! Duration parsing and manipulation helpers

use crate::{Duration, TimeError};

#[derive(Debug, Clone, Copy, Default)]
pub struct DurationBuilder {
    hours: u64,
    minutes: u64,
    seconds: u64,
}

impl DurationBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn hours(mut self, h: u64) -> Self {
        self.hours = h;
        self
    }
    #[must_use]
    pub fn minutes(mut self, m: u64) -> Self {
        self.minutes = m;
        self
    }
    #[must_use]
    pub fn seconds(mut self, s: u64) -> Self {
        self.seconds = s;
        self
    }
    #[must_use]
    pub fn build(self) -> Duration {
        let total = self
            .hours
            .saturating_mul(3600)
            .saturating_add(self.minutes.saturating_mul(60))
            .saturating_add(self.seconds);
        Duration::from_secs(total)
    }
}

pub fn minutes(m: u64) -> Duration {
    Duration::from_secs(m * 60)
}

pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

pub fn days(d: u64) -> Duration {
    Duration::from_secs(d * 86400)
}

impl Duration {
    pub fn parse(input: &str) -> Result<Self, TimeError> {
        Self::from_human(input)
    }
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.as_secs() == 0
    }
    #[must_use]
    pub fn mul(&self, factor: u64) -> Self {
        Duration::from_secs(self.as_secs().saturating_mul(factor))
    }
    #[must_use]
    pub fn div(&self, divisor: u64) -> Self {
        assert_ne!(divisor, 0);
        Duration::from_secs(self.as_secs() / divisor)
    }
    #[must_use]
    pub fn as_millis(&self) -> u64 {
        self.inner().as_millis() as u64
    }
    #[must_use]
    pub fn as_minutes(&self) -> u64 {
        self.as_secs() / 60
    }
    #[must_use]
    pub fn as_hours(&self) -> u64 {
        self.as_secs() / 3600
    }
    #[must_use]
    pub fn as_days(&self) -> u64 {
        self.as_secs() / 86400
    }
    #[must_use]
    pub fn is_longer_than(&self, other: Duration) -> bool {
        self.as_secs() > other.as_secs()
    }
    #[must_use]
    pub fn is_shorter_than(&self, other: Duration) -> bool {
        self.as_secs() < other.as_secs()
    }
    #[must_use]
    pub fn is_equal_to(&self, other: Duration) -> bool {
        self.as_secs() == other.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_hours() {
        let d = DurationBuilder::new().hours(2).build();
        assert_eq!(d.as_secs(), 7200);
    }
    #[test]
    fn helper_minutes() {
        assert_eq!(minutes(30).as_secs(), 1800);
    }
    #[test]
    fn parse_hours() {
        assert_eq!(Duration::parse("2h").unwrap().as_secs(), 7200);
    }
    #[test]
    fn is_zero() {
        assert!(Duration::from_secs(0).is_zero());
    }
    #[test]
    fn mul() {
        assert_eq!(Duration::from_secs(5).mul(3).as_secs(), 15);
    }
    #[test]
    fn comparisons() {
        let d1 = Duration::from_secs(10);
        let d2 = Duration::from_secs(20);
        assert!(d2.is_longer_than(d1));
    }
}
