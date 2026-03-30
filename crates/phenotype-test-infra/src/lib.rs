//! # Phenotype Test Infrastructure
pub use phenotype_error_core::CoreError;
pub use std::io::ErrorKind;
//!
//! ## Features
//!
//! - [`TempDir`] — RAII temporary directory that cleans up on drop.
//! - [`TestFixture`] — Trait for setup/teardown test patterns.
//! - [`MockClock`] — Controllable clock for deterministic time-dependent tests.
//! - [`capture_logs`] — Capture `tracing` output during tests.
//! - [`assert_err_contains!`] — Assert that an error's display message contains a substring.
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_test_infra::{TempDir, assert_err_contains};
//!
//! let dir = TempDir::new("my-test").expect("create temp dir");
//! assert!(dir.path().exists());
//!
//! let err: Result<(), String> = Err("validation failed: bad input".into());
//! assert_err_contains!(err, "bad input");
//! ```

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ValidationError => write!(f, "validation error"),
            ErrorKind::IOError => write!(f, "io error"),
            ErrorKind::NotFound => write!(f, "not found"),
            ErrorKind::Timeout => write!(f, "timeout"),
            ErrorKind::Internal => write!(f, "internal error"),
        }
    }
}

/// RAII temporary directory that cleans up on drop.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Create a new temporary directory.
    pub fn new(prefix: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()));
        fs::create_dir_all(&path)?;
        Ok(TempDir { path })
    }

    /// Get the path to the temporary directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Trait for test fixtures with setup/teardown.
pub trait TestFixture: Sized {
    /// Setup the fixture.
    fn setup() -> std::io::Result<Self>;

    /// Teardown the fixture.
    fn teardown(&mut self) -> std::io::Result<()>;
}

/// Mock clock for deterministic time-dependent tests.
pub struct MockClock {
    current_time: Arc<AtomicU64>,
}

impl MockClock {
    /// Create a new mock clock starting at the given timestamp.
    pub fn new(millis: u64) -> Self {
        MockClock {
            current_time: Arc::new(AtomicU64::new(millis)),
        }
    }

    /// Advance the clock by the given duration.
    pub fn advance(&self, duration: Duration) {
        let millis = duration.as_millis() as u64;
        self.current_time.fetch_add(millis, Ordering::SeqCst);
    }

    /// Get the current time as a u64 millisecond value.
    pub fn now_millis(&self) -> u64 {
        self.current_time.load(Ordering::SeqCst)
    }
}

/// Assert that an error's display message contains a substring.
///
/// # Example
///
/// ```
/// # use phenotype_test_infra::assert_err_contains;
/// let err: Result<(), String> = Err("validation failed: bad input".into());
/// assert_err_contains!(err, "bad input");
/// ```
#[macro_export]
macro_rules! assert_err_contains {
    ($result:expr, $substr:expr) => {
        match $result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => {
                let msg = format!("{}", e);
                assert!(
                    msg.contains($substr),
                    "Error message '{}' does not contain '{}'",
                    msg,
                    $substr
                );
            }
        }
    };
}

/// Capture logs emitted during a test using `tracing`.
///
/// Returns the captured log output as a string.
pub fn capture_logs<F: FnOnce() -> R, R>(f: F) -> (R, String) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let (writer, handle) = tracing_test::trace_init();
    let result = f();
    let logs = handle.finish();
    (result, logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dir_creates_and_cleans_up() {
        let dir = TempDir::new("test").expect("create temp dir");
        assert!(dir.path().exists());
        let path = dir.path().to_path_buf();
        drop(dir);
        assert!(!path.exists());
    }

    #[test]
    fn mock_clock_advances() {
        let clock = MockClock::new(0);
        assert_eq!(clock.now_millis(), 0);
        clock.advance(Duration::from_millis(100));
        assert_eq!(clock.now_millis(), 100);
    }

    #[test]
    #[should_panic(expected = "Expected error but got Ok")]
    fn assert_err_contains_panics_on_ok() {
        let result: Result<(), String> = Ok(());
        assert_err_contains!(result, "foo");
    }

    #[test]
    fn assert_err_contains_succeeds() {
        let result: Result<(), String> = Err("validation failed".into());
        assert_err_contains!(result, "validation");
    }
}
