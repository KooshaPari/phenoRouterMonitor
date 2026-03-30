//! # Phenotype Test Infrastructure
//!
//! Shared test utilities for the Phenotype ecosystem.
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

pub use phenotype_error_core::ErrorKind;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use tracing_subscriber::fmt::MakeWriter;

// ---------------------------------------------------------------------------
// Result / Error aliases
// ---------------------------------------------------------------------------

/// Result type for test operations.
pub type Result<T> = std::result::Result<T, TestError>;

/// Common errors that can occur in tests.
pub type TestError = ErrorKind;

// ---------------------------------------------------------------------------
// TempDir
// ---------------------------------------------------------------------------

/// RAII temporary directory.
///
/// Creates a unique directory under `std::env::temp_dir()` on construction
/// and removes it (recursively) when dropped.
///
/// # Examples
///
/// ```rust
/// use phenotype_test_infra::TempDir;
///
/// let tmp = TempDir::new("example").unwrap();
/// std::fs::write(tmp.path().join("hello.txt"), b"hi").unwrap();
/// assert!(tmp.path().join("hello.txt").exists());
/// // directory is removed when `tmp` goes out of scope
/// ```
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Create a new temporary directory with the given prefix.
    ///
    /// The actual directory name is `{prefix}-{pid}-{counter}` to avoid
    /// collisions across concurrent tests.
    pub fn new(prefix: &str) -> Result<Self> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("{prefix}-{}-{id}", std::process::id());
        let path = std::env::temp_dir().join(name);
        fs::create_dir_all(&path).map_err(ErrorKind::io)?;
        Ok(Self { path })
    }

    /// Return a reference to the directory path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Consume the `TempDir` **without** deleting it, returning the path.
    #[must_use]
    pub fn into_path(self) -> PathBuf {
        let path = self.path.clone();
        std::mem::forget(self);
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

impl fmt::Debug for TempDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TempDir")
            .field("path", &self.path)
            .finish()
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

// ---------------------------------------------------------------------------
// TestFixture trait
// ---------------------------------------------------------------------------

/// Trait for reusable test fixture setup and teardown.
///
/// Implementors provide `setup` to build the fixture state and optionally
/// override `teardown` for cleanup beyond what `Drop` provides.
///
/// # Examples
///
/// ```rust
/// use phenotype_test_infra::TestFixture;
///
/// struct DbFixture { url: String }
///
/// impl TestFixture for DbFixture {
///     type Context = String;
///     type Error = String;
///
///     fn setup() -> std::result::Result<Self, Self::Error> {
///         Ok(DbFixture { url: "sqlite::memory:".into() })
///     }
///
///     fn context(&self) -> Self::Context {
///         self.url.clone()
///     }
/// }
///
/// let f = DbFixture::setup().unwrap();
/// assert_eq!(f.context(), "sqlite::memory:");
/// ```
pub trait TestFixture: Sized {
    /// The context value exposed to the test body.
    type Context;
    /// The error type returned from setup / teardown.
    type Error: fmt::Debug;

    /// Build and return the fixture. Called once per test.
    fn setup() -> std::result::Result<Self, Self::Error>;

    /// Return a reference or value the test body needs.
    fn context(&self) -> Self::Context;

    /// Optional cleanup. The default implementation is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if teardown fails.
    fn teardown(self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// assert_err_contains! macro
// ---------------------------------------------------------------------------

/// Assert that a `Result` is `Err` and its display string contains the
/// given substring.
///
/// # Examples
///
/// ```rust
/// use phenotype_test_infra::assert_err_contains;
///
/// let res: Result<(), String> = Err("validation failed: bad input".into());
/// assert_err_contains!(res, "bad input");
/// ```
///
/// Panics with a descriptive message when the result is `Ok` or the error
/// message does not contain the expected substring.
#[macro_export]
macro_rules! assert_err_contains {
    ($expr:expr, $substr:expr) => {
        match $expr {
            Ok(val) => panic!(
                "assert_err_contains: expected Err, got Ok({:?})",
                val
            ),
            Err(ref e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains($substr),
                    "assert_err_contains: error message {:?} does not contain {:?}",
                    msg,
                    $substr,
                );
            }
        }
    };
}

// ---------------------------------------------------------------------------
// MockClock
// ---------------------------------------------------------------------------

/// A controllable clock for deterministic time-dependent tests.
///
/// `MockClock` stores an internal `Duration` that can be advanced manually.
/// It is `Clone` and `Send + Sync` so it can be shared across threads.
///
/// # Examples
///
/// ```rust
/// use phenotype_test_infra::MockClock;
/// use std::time::Duration;
///
/// let clock = MockClock::new();
/// assert_eq!(clock.now(), Duration::ZERO);
///
/// clock.advance(Duration::from_secs(5));
/// assert_eq!(clock.now(), Duration::from_secs(5));
/// ```
#[derive(Debug, Clone)]
pub struct MockClock {
    nanos: Arc<AtomicU64>,
}

impl MockClock {
    /// Create a new `MockClock` starting at zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nanos: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a `MockClock` starting at the given duration.
    #[must_use]
    pub fn starting_at(d: Duration) -> Self {
        let nanos = d.as_nanos().min(u64::MAX as u128) as u64;
        Self {
            nanos: Arc::new(AtomicU64::new(nanos)),
        }
    }

    /// Return the current mock time as a `Duration`.
    #[must_use]
    pub fn now(&self) -> Duration {
        Duration::from_nanos(self.nanos.load(Ordering::Acquire))
    }

    /// Advance the clock by `d`.
    pub fn advance(&self, d: Duration) {
        let delta = d.as_nanos().min(u64::MAX as u128) as u64;
        self.nanos.fetch_add(delta, Ordering::Release);
    }

    /// Set the clock to an absolute value.
    pub fn set(&self, d: Duration) {
        let nanos = d.as_nanos().min(u64::MAX as u128) as u64;
        self.nanos.store(nanos, Ordering::Release);
    }
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Log capture
// ---------------------------------------------------------------------------

/// Captured log output from a [`capture_logs`] invocation.
#[derive(Debug, Clone)]
pub struct CapturedLogs {
    /// The raw captured output.
    pub output: String,
}

impl CapturedLogs {
    /// Check whether any captured line contains the given substring.
    #[must_use]
    pub fn contains(&self, substr: &str) -> bool {
        self.output.contains(substr)
    }

    /// Return captured output lines.
    #[must_use]
    pub fn lines(&self) -> Vec<&str> {
        self.output.lines().collect()
    }
}

/// Shared buffer writer for capturing tracing output.
#[derive(Clone)]
struct BufWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for BufWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf
            .lock()
            .expect("lock poisoned")
            .extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for BufWriter {
    type Writer = BufWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Run a closure while capturing all `tracing` output, then return it.
///
/// This installs a temporary tracing subscriber scoped to the closure.
/// It does **not** interfere with the global subscriber.
///
/// # Examples
///
/// ```rust
/// use phenotype_test_infra::capture_logs;
///
/// let logs = capture_logs(|| {
///     tracing::info!("hello from test");
/// });
/// assert!(logs.contains("hello from test"));
/// ```
pub fn capture_logs<F: FnOnce()>(f: F) -> CapturedLogs {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let writer = BufWriter { buf: buf.clone() };

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .with_level(true)
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::with_default(subscriber, f);

    let raw = buf.lock().expect("lock poisoned").clone();
    CapturedLogs {
        output: String::from_utf8_lossy(&raw).into_owned(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-TEST-001
    #[test]
    fn test_temp_dir_creates_and_cleans_up() {
        let path;
        {
            let tmp = TempDir::new("test-cleanup").expect("create temp dir");
            path = tmp.path().to_path_buf();
            assert!(path.exists(), "temp dir should exist during lifetime");
            fs::write(path.join("file.txt"), b"data").unwrap();
        }
        assert!(!path.exists(), "temp dir should be removed on drop");
    }

    // Traces to: FR-TEST-001
    #[test]
    fn test_temp_dir_into_path_preserves() {
        let tmp = TempDir::new("test-persist").expect("create temp dir");
        let path = tmp.into_path();
        assert!(path.exists(), "into_path should preserve directory");
        // Manual cleanup.
        fs::remove_dir_all(&path).ok();
    }

    // Traces to: FR-TEST-002
    #[test]
    fn test_fixture_trait() {
        struct Simple {
            val: u32,
        }
        impl TestFixture for Simple {
            type Context = u32;
            type Error = String;
            fn setup() -> std::result::Result<Self, Self::Error> {
                Ok(Simple { val: 42 })
            }
            fn context(&self) -> u32 {
                self.val
            }
        }

        let f = Simple::setup().unwrap();
        assert_eq!(f.context(), 42);
        f.teardown().unwrap();
    }

    // Traces to: FR-TEST-003
    #[test]
    fn test_assert_err_contains_matches() {
        let res: std::result::Result<(), String> =
            Err("validation failed: bad input".into());
        assert_err_contains!(res, "bad input");
    }

    // Traces to: FR-TEST-003
    #[test]
    #[should_panic(expected = "does not contain")]
    fn test_assert_err_contains_mismatch_panics() {
        let res: std::result::Result<(), String> =
            Err("something else".into());
        assert_err_contains!(res, "not present");
    }

    // Traces to: FR-TEST-003
    #[test]
    #[should_panic(expected = "expected Err")]
    fn test_assert_err_contains_ok_panics() {
        let res: std::result::Result<i32, String> = Ok(1);
        assert_err_contains!(res, "anything");
    }

    // Traces to: FR-TEST-004
    #[test]
    fn test_mock_clock_default() {
        let clock = MockClock::new();
        assert_eq!(clock.now(), Duration::ZERO);
    }

    // Traces to: FR-TEST-004
    #[test]
    fn test_mock_clock_advance() {
        let clock = MockClock::new();
        clock.advance(Duration::from_secs(3));
        clock.advance(Duration::from_millis(500));
        assert_eq!(clock.now(), Duration::from_millis(3500));
    }

    // Traces to: FR-TEST-004
    #[test]
    fn test_mock_clock_set() {
        let clock = MockClock::new();
        clock.set(Duration::from_secs(100));
        assert_eq!(clock.now(), Duration::from_secs(100));
    }

    // Traces to: FR-TEST-004
    #[test]
    fn test_mock_clock_starting_at() {
        let clock = MockClock::starting_at(Duration::from_secs(10));
        assert_eq!(clock.now(), Duration::from_secs(10));
    }

    // Traces to: FR-TEST-004
    #[test]
    fn test_mock_clock_clone_shares_state() {
        let a = MockClock::new();
        let b = a.clone();
        a.advance(Duration::from_secs(5));
        assert_eq!(b.now(), Duration::from_secs(5));
    }

    // Traces to: FR-TEST-005
    #[test]
    fn test_capture_logs_captures_output() {
        let logs = capture_logs(|| {
            tracing::info!("hello from test");
        });
        assert!(
            logs.contains("hello from test"),
            "captured: {:?}",
            logs.output
        );
    }

    // Traces to: FR-TEST-005
    #[test]
    fn test_capture_logs_lines() {
        let logs = capture_logs(|| {
            tracing::warn!("line one");
            tracing::error!("line two");
        });
        assert!(logs.lines().len() >= 2, "expected >=2 lines: {:?}", logs.output);
    }

    // Traces to: FR-TEST-005
    #[test]
    fn test_captured_logs_contains_returns_false() {
        let logs = capture_logs(|| {
            tracing::info!("actual message");
        });
        assert!(!logs.contains("nonexistent string"));
    }
}
