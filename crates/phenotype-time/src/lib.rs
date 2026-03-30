//! Time utilities for the Phenotype ecosystem.
//!
//! Provides [`Timestamp`], [`Duration`], [`TimeRange`], and a [`Clock`] trait
//! for testable time abstractions.

use std::fmt;
use std::sync::atomic::{AtomicI64, Ordering};

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced by time parsing operations.
#[derive(Debug, Error)]
pub enum TimeError {
    /// A human-readable duration string could not be parsed.
    #[error("invalid duration string: {0}")]
    InvalidDuration(String),

    /// A millisecond timestamp could not be converted.
    #[error("invalid millis timestamp: {0}")]
    InvalidMillis(i64),

    /// The time range is inverted (start > end).
    #[error("inverted time range: start ({start}) is after end ({end})")]
    InvertedRange { start: Timestamp, end: Timestamp },
}

/// A UTC timestamp wrapping [`chrono::DateTime<Utc>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_millis(ms: i64) -> Result<Self, TimeError> {
        Utc.timestamp_millis_opt(ms)
            .single()
            .map(Self)
            .ok_or(TimeError::InvalidMillis(ms))
    }

    #[must_use]
    pub fn to_millis(self) -> i64 {
        self.0.timestamp_millis()
    }

    #[must_use]
    pub fn inner(self) -> DateTime<Utc> {
        self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self { Self(dt) }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self { ts.0 }
}

/// A duration wrapping [`std::time::Duration`] with human-readable display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Duration(#[serde(with = "serde_duration")] std::time::Duration);

mod serde_duration {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(d: &std::time::Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<std::time::Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(std::time::Duration::from_secs(secs))
    }
}

impl Duration {
    #[must_use]
    pub fn from_secs(secs: u64) -> Self { Self(std::time::Duration::from_secs(secs)) }

    #[must_use]
    pub fn from_millis(ms: u64) -> Self { Self(std::time::Duration::from_millis(ms)) }

    pub fn from_human(input: &str) -> Result<Self, TimeError> {
        let s = input.trim();
        if s.is_empty() { return Err(TimeError::InvalidDuration(input.to_owned())); }
        let mut total_secs: u64 = 0;
        let mut num_buf = String::new();
        let mut found_unit = false;
        for ch in s.chars() {
            if ch.is_ascii_digit() { num_buf.push(ch); }
            else {
                let n: u64 = num_buf.parse().map_err(|_| TimeError::InvalidDuration(input.to_owned()))?;
                num_buf.clear();
                found_unit = true;
                match ch {
                    'h' | 'H' => total_secs += n * 3600,
                    'm' | 'M' => total_secs += n * 60,
                    's' | 'S' => total_secs += n,
                    _ => return Err(TimeError::InvalidDuration(input.to_owned())),
                }
            }
        }
        if !found_unit || !num_buf.is_empty() { return Err(TimeError::InvalidDuration(input.to_owned())); }
        Ok(Self(std::time::Duration::from_secs(total_secs)))
    }

    #[must_use]
    pub fn as_secs(&self) -> u64 { self.0.as_secs() }

    #[must_use]
    pub fn inner(self) -> std::time::Duration { self.0 }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.0.as_secs();
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        let mut parts = Vec::new();
        if h > 0 { parts.push(format!("{h}h")); }
        if m > 0 { parts.push(format!("{m}m")); }
        if s > 0 || parts.is_empty() { parts.push(format!("{s}s")); }
        write!(f, "{}", parts.join(" "))
    }
}

impl From<std::time::Duration> for Duration {
    fn from(d: std::time::Duration) -> Self { Self(d) }
}

impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> Self { d.0 }
}

/// A half-open time range `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange { start: Timestamp, end: Timestamp }

impl TimeRange {
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self, TimeError> {
        if start > end { return Err(TimeError::InvertedRange { start, end }); }
        Ok(Self { start, end })
    }

    #[must_use]
    pub fn contains(&self, ts: Timestamp) -> bool { ts >= self.start && ts < self.end }

    #[must_use]
    pub fn overlaps(&self, other: &TimeRange) -> bool { self.start < other.end && other.start < self.end }

    #[must_use]
    pub fn duration(&self) -> Duration {
        let diff = self.end.inner() - self.start.inner();
        Duration::from_millis(diff.num_milliseconds().unsigned_abs())
    }

    #[must_use]
    pub fn start(&self) -> Timestamp { self.start }

    #[must_use]
    pub fn end(&self) -> Timestamp { self.end }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

/// Abstraction over time sources for testability.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// The real system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp { Timestamp::now() }
}

/// A mock clock for tests.
#[derive(Debug)]
pub struct MockClock { millis: AtomicI64 }

impl MockClock {
    #[must_use]
    pub fn new(millis: i64) -> Self { Self { millis: AtomicI64::new(millis) } }

    pub fn advance(&self, ms: i64) { self.millis.fetch_add(ms, Ordering::Relaxed); }

    pub fn set(&self, millis: i64) { self.millis.store(millis, Ordering::Relaxed); }
}

impl Clock for MockClock {
    fn now(&self) -> Timestamp {
        let ms = self.millis.load(Ordering::Relaxed);
        Timestamp::from_millis(ms).expect("MockClock millis out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_roundtrip_millis() {
        let ms: i64 = 1_700_000_000_000;
        let ts = Timestamp::from_millis(ms).unwrap();
        assert_eq!(ts.to_millis(), ms);
    }

    #[test]
    fn timestamp_display_rfc3339() {
        let ts = Timestamp::from_millis(0).unwrap();
        assert!(ts.to_string().contains("1970"));
    }

    #[test]
    fn timestamp_ordering() {
        let a = Timestamp::from_millis(1000).unwrap();
        let b = Timestamp::from_millis(2000).unwrap();
        assert!(a < b);
    }

    #[test]
    fn timestamp_serde_roundtrip() {
        let ts = Timestamp::from_millis(1_700_000_000_000).unwrap();
        let json = serde_json::to_string(&ts).unwrap();
        let back: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(ts, back);
    }

    #[test]
    fn duration_display() {
        let d = Duration::from_secs(3600 * 2 + 60 * 30 + 15);
        assert_eq!(d.to_string(), "2h 30m 15s");
    }

    #[test]
    fn duration_display_zero() {
        let d = Duration::from_secs(0);
        assert_eq!(d.to_string(), "0s");
    }

    #[test]
    fn duration_from_human_parse() {
        let d = Duration::from_human("2h30m").unwrap();
        assert_eq!(d.as_secs(), 2 * 3600 + 30 * 60);
    }

    #[test]
    fn duration_from_human_full() {
        let d = Duration::from_human("1h2m3s").unwrap();
        assert_eq!(d.as_secs(), 3600 + 120 + 3);
    }

    #[test]
    fn duration_from_human_invalid() {
        assert!(Duration::from_human("").is_err());
        assert!(Duration::from_human("abc").is_err());
        assert!(Duration::from_human("10").is_err());
        assert!(Duration::from_human("10x").is_err());
    }

    #[test]
    fn time_range_contains() {
        let s = Timestamp::from_millis(1000).unwrap();
        let e = Timestamp::from_millis(5000).unwrap();
        let r = TimeRange::new(s, e).unwrap();
        assert!(r.contains(Timestamp::from_millis(1000).unwrap()));
        assert!(r.contains(Timestamp::from_millis(3000).unwrap()));
        assert!(!r.contains(Timestamp::from_millis(5000).unwrap()));
        assert!(!r.contains(Timestamp::from_millis(999).unwrap()));
    }

    #[test]
    fn time_range_overlaps() {
        let r1 = TimeRange::new(Timestamp::from_millis(1000).unwrap(), Timestamp::from_millis(5000).unwrap()).unwrap();
        let r2 = TimeRange::new(Timestamp::from_millis(4000).unwrap(), Timestamp::from_millis(8000).unwrap()).unwrap();
        let r3 = TimeRange::new(Timestamp::from_millis(5000).unwrap(), Timestamp::from_millis(9000).unwrap()).unwrap();
        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn time_range_duration() {
        let r = TimeRange::new(Timestamp::from_millis(1000).unwrap(), Timestamp::from_millis(4000).unwrap()).unwrap();
        assert_eq!(r.duration().as_secs(), 3);
    }

    #[test]
    fn time_range_inverted() {
        assert!(TimeRange::new(Timestamp::from_millis(5000).unwrap(), Timestamp::from_millis(1000).unwrap()).is_err());
    }

    #[test]
    fn system_clock_returns_recent() {
        assert!(SystemClock.now().to_millis() > 1_577_836_800_000);
    }

    #[test]
    fn mock_clock_fixed() {
        let c = MockClock::new(42_000);
        assert_eq!(c.now().to_millis(), 42_000);
    }

    #[test]
    fn mock_clock_advance() {
        let c = MockClock::new(1000);
        c.advance(500);
        assert_eq!(c.now().to_millis(), 1500);
    }

    #[test]
    fn mock_clock_set() {
        let c = MockClock::new(1000);
        c.set(9999);
        assert_eq!(c.now().to_millis(), 9999);
    }
}
