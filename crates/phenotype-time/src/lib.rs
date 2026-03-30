//! Time utilities for the Phenotype ecosystem.
//!
//! This crate provides comprehensive time handling utilities including:
//! - **Timestamps**: Millisecond-precision UTC timestamps with serialization
//! - **Durations**: Human-readable duration parsing and manipulation
//! - **Intervals**: Configurable intervals with fixed, exponential, and linear backoff strategies
//! - **Scheduling**: Simple task scheduler for recurring work
//! - **Clocks**: Testable clock abstraction (System and Mock implementations)

pub mod clock;
pub mod duration;
pub mod interval;
pub mod scheduler;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// Re-export commonly used items
pub use clock::{Clock, FlexClock, MockClock, SystemClock};
pub use duration::DurationBuilder;
pub use interval::Interval;
pub use scheduler::Scheduler;

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("invalid duration string: {0}")]
    InvalidDuration(String),
    #[error("invalid millis timestamp: {0}")]
    InvalidMillis(i64),
    #[error("inverted time range: start ({start}) is after end ({end})")]
    InvertedRange { start: Timestamp, end: Timestamp },
}

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
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}
impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

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
    pub fn from_secs(secs: u64) -> Self {
        Self(std::time::Duration::from_secs(secs))
    }
    #[must_use]
    pub fn from_millis(ms: u64) -> Self {
        Self(std::time::Duration::from_millis(ms))
    }
    #[must_use]
    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }
    #[must_use]
    pub fn inner(self) -> std::time::Duration {
        self.0
    }

    pub fn from_human(input: &str) -> Result<Self, TimeError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(TimeError::InvalidDuration(input.to_owned()));
        }
        let mut total: u64 = 0;
        let mut buf = String::new();
        let mut found = false;
        for ch in s.chars() {
            if ch.is_ascii_digit() {
                buf.push(ch);
            } else {
                let n: u64 = buf
                    .parse()
                    .map_err(|_| TimeError::InvalidDuration(input.to_owned()))?;
                buf.clear();
                found = true;
                match ch {
                    'h' | 'H' => total += n * 3600,
                    'm' | 'M' => total += n * 60,
                    's' | 'S' => total += n,
                    _ => return Err(TimeError::InvalidDuration(input.to_owned())),
                }
            }
        }
        if !found || !buf.is_empty() {
            return Err(TimeError::InvalidDuration(input.to_owned()));
        }
        Ok(Self(std::time::Duration::from_secs(total)))
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.0.as_secs();
        let h = t / 3600;
        let m = (t % 3600) / 60;
        let s = t % 60;
        let mut p = Vec::new();
        if h > 0 {
            p.push(format!("{h}h"));
        }
        if m > 0 {
            p.push(format!("{m}m"));
        }
        if s > 0 || p.is_empty() {
            p.push(format!("{s}s"));
        }
        write!(f, "{}", p.join(" "))
    }
}

impl From<std::time::Duration> for Duration {
    fn from(d: std::time::Duration) -> Self {
        Self(d)
    }
}
impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> Self {
        d.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    start: Timestamp,
    end: Timestamp,
}

impl TimeRange {
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self, TimeError> {
        if start > end {
            return Err(TimeError::InvertedRange { start, end });
        }
        Ok(Self { start, end })
    }
    #[must_use]
    pub fn contains(&self, ts: Timestamp) -> bool {
        ts >= self.start && ts < self.end
    }
    #[must_use]
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start < other.end && other.start < self.end
    }
    #[must_use]
    pub fn duration(&self) -> Duration {
        Duration::from_millis(
            (self.end.inner() - self.start.inner())
                .num_milliseconds()
                .unsigned_abs(),
        )
    }
    #[must_use]
    pub fn start(&self) -> Timestamp {
        self.start
    }
    #[must_use]
    pub fn end(&self) -> Timestamp {
        self.end
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ts_roundtrip() {
        let ms = 1_700_000_000_000i64;
        assert_eq!(Timestamp::from_millis(ms).unwrap().to_millis(), ms);
    }
    #[test]
    fn ts_display() {
        assert!(Timestamp::from_millis(0)
            .unwrap()
            .to_string()
            .contains("1970"));
    }
    #[test]
    fn ts_ord() {
        assert!(Timestamp::from_millis(1000).unwrap() < Timestamp::from_millis(2000).unwrap());
    }
    #[test]
    fn ts_serde() {
        let ts = Timestamp::from_millis(1_700_000_000_000).unwrap();
        let j = serde_json::to_string(&ts).unwrap();
        assert_eq!(serde_json::from_str::<Timestamp>(&j).unwrap(), ts);
    }
    #[test]
    fn dur_display() {
        assert_eq!(
            Duration::from_secs(2 * 3600 + 30 * 60 + 15).to_string(),
            "2h 30m 15s"
        );
    }
    #[test]
    fn dur_zero() {
        assert_eq!(Duration::from_secs(0).to_string(), "0s");
    }
    #[test]
    fn dur_human() {
        assert_eq!(
            Duration::from_human("2h30m").unwrap().as_secs(),
            2 * 3600 + 30 * 60
        );
    }
    #[test]
    fn dur_human_full() {
        assert_eq!(
            Duration::from_human("1h2m3s").unwrap().as_secs(),
            3600 + 120 + 3
        );
    }
    #[test]
    fn dur_human_err() {
        assert!(Duration::from_human("").is_err());
        assert!(Duration::from_human("10").is_err());
        assert!(Duration::from_human("10x").is_err());
    }
    #[test]
    fn range_contains() {
        let r = TimeRange::new(
            Timestamp::from_millis(1000).unwrap(),
            Timestamp::from_millis(5000).unwrap(),
        )
        .unwrap();
        assert!(r.contains(Timestamp::from_millis(1000).unwrap()));
        assert!(!r.contains(Timestamp::from_millis(5000).unwrap()));
    }
    #[test]
    fn range_overlaps() {
        let r1 = TimeRange::new(
            Timestamp::from_millis(1000).unwrap(),
            Timestamp::from_millis(5000).unwrap(),
        )
        .unwrap();
        let r2 = TimeRange::new(
            Timestamp::from_millis(4000).unwrap(),
            Timestamp::from_millis(8000).unwrap(),
        )
        .unwrap();
        assert!(r1.overlaps(&r2));
        let r3 = TimeRange::new(
            Timestamp::from_millis(5000).unwrap(),
            Timestamp::from_millis(9000).unwrap(),
        )
        .unwrap();
        assert!(!r1.overlaps(&r3));
    }
    #[test]
    fn range_dur() {
        assert_eq!(
            TimeRange::new(
                Timestamp::from_millis(1000).unwrap(),
                Timestamp::from_millis(4000).unwrap()
            )
            .unwrap()
            .duration()
            .as_secs(),
            3
        );
    }
    #[test]
    fn range_inv() {
        assert!(TimeRange::new(
            Timestamp::from_millis(5000).unwrap(),
            Timestamp::from_millis(1000).unwrap()
        )
        .is_err());
    }
    #[test]
    fn sys_clock() {
        assert!(SystemClock.now().to_millis() > 1_577_836_800_000);
    }
    #[test]
    fn mock_fixed() {
        assert_eq!(MockClock::new(42_000).now().to_millis(), 42_000);
    }
    #[test]
    fn mock_adv() {
        let c = MockClock::new(1000);
        c.advance(500);
        assert_eq!(c.now().to_millis(), 1500);
    }
    #[test]
    fn mock_set() {
        let c = MockClock::new(1000);
        c.set(9999);
        assert_eq!(c.now().to_millis(), 9999);
    }
}
