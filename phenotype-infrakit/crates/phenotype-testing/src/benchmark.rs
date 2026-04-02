//! Performance benchmarking utilities
//!
//! This module provides infrastructure for creating and running
//! performance benchmarks, including statistical analysis of results.

use std::time::{Duration, Instant};

// ============================================================================
// BenchmarkConfig - Benchmark configuration
// ============================================================================

/// Configuration for benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations to run
    pub iterations: usize,
    /// Warmup iterations (not counted in results)
    pub warmup_iterations: usize,
    /// Confidence level for statistics (e.g., 0.95 for 95%)
    pub confidence_level: f64,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            confidence_level: 0.95,
            verbose: false,
        }
    }
}

impl BenchmarkConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set number of iterations
    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// Set warmup iterations
    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_iterations = warmup;
        self
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set confidence level
    pub fn with_confidence(mut self, level: f64) -> Self {
        self.confidence_level = level;
        self
    }
}

// ============================================================================
// BenchmarkResult - Results of a benchmark run
// ============================================================================

/// Results from a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Average time per iteration
    pub mean: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// Minimum time
    pub min: Duration,
    /// Maximum time
    pub max: Duration,
    /// Median time
    pub median: Duration,
    /// Number of samples
    pub samples: usize,
    /// Iterations per second
    pub iterations_per_second: f64,
}

impl BenchmarkResult {
    /// Calculate from a set of durations
    pub fn from_durations(durations: &[Duration]) -> Self {
        if durations.is_empty() {
            return Self {
                mean: Duration::from_secs(0),
                std_dev: Duration::from_secs(0),
                min: Duration::from_secs(0),
                max: Duration::from_secs(0),
                median: Duration::from_secs(0),
                samples: 0,
                iterations_per_second: 0.0,
            };
        }

        let mut sorted = durations.to_vec();
        sorted.sort();

        let total_ns: u64 = sorted.iter().map(|d| d.as_nanos() as u64).sum();
        let mean_ns = total_ns / durations.len() as u64;

        let variance: f64 = sorted
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_ns as f64;
                diff * diff
            })
            .sum::<f64>()
            / durations.len() as f64;
        let std_dev_ns = variance.sqrt() as u64;

        let min = sorted.first().copied().unwrap_or_default();
        let max = sorted.last().copied().unwrap_or_default();
        let median = if sorted.len() % 2 == 0 {
            let mid = sorted.len() / 2;
            Duration::from_nanos(
                (sorted[mid - 1].as_nanos() as u64 + sorted[mid].as_nanos() as u64) / 2,
            )
        } else {
            sorted[sorted.len() / 2]
        };

        let mean = Duration::from_nanos(mean_ns);
        let std_dev = Duration::from_nanos(std_dev_ns);

        let iterations_per_second = if mean.as_secs_f64() > 0.0 {
            1.0 / mean.as_secs_f64()
        } else {
            0.0
        };

        Self {
            mean,
            std_dev,
            min,
            max,
            median,
            samples: durations.len(),
            iterations_per_second,
        }
    }

    /// Format as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "Benchmark Results:\n\
             ├─ Mean:     {}\n\
             ├─ Std Dev:  {}\n\
             ├─ Min:      {}\n\
             ├─ Max:      {}\n\
             ├─ Median:   {}\n\
             ├─ Samples:  {}\n\
             └─ Ops/sec:  {:.2}",
            format_duration(self.mean),
            format_duration(self.std_dev),
            format_duration(self.min),
            format_duration(self.max),
            format_duration(self.median),
            self.samples,
            self.iterations_per_second,
        )
    }
}

/// Format a duration in a human-readable way
pub fn format_duration(d: Duration) -> String {
    let total_ns = d.as_nanos();

    if total_ns < 1_000 {
        format!("{} ns", total_ns)
    } else if total_ns < 1_000_000 {
        format!("{:.2} µs", total_ns as f64 / 1_000.0)
    } else if total_ns < 1_000_000_000 {
        format!("{:.2} ms", total_ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", d.as_secs_f64())
    }
}

// ============================================================================
// BenchmarkRunner - Utility for running benchmarks
// ============================================================================

/// Runner for executing benchmarks with measurement
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    measurements: Vec<Duration>,
}

impl BenchmarkRunner {
    /// Create a new runner with config
    pub fn new(config: BenchmarkConfig) -> Self {
        let iterations = config.iterations;
        Self {
            config,
            measurements: Vec::with_capacity(iterations),
        }
    }

    /// Run a benchmark
    pub fn run<F>(&mut self, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            f();
        }

        // Measure
        self.measurements.clear();
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            f();
            let elapsed = start.elapsed();
            self.measurements.push(elapsed);
        }

        BenchmarkResult::from_durations(&self.measurements)
    }

    /// Run an async benchmark
    #[cfg(feature = "tokio-rt")]
    pub async fn run_async<F, Fut>(&mut self, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            f().await;
        }

        // Measure
        self.measurements.clear();
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            f().await;
            let elapsed = start.elapsed();
            self.measurements.push(elapsed);
        }

        BenchmarkResult::from_durations(&self.measurements)
    }

    /// Get measurements
    pub fn measurements(&self) -> &[Duration] {
        &self.measurements
    }

    /// Clear measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new(BenchmarkConfig::default())
    }
}

// ============================================================================
// BenchmarkTimer - Simple timing utility
// ============================================================================

/// Simple timer for measuring elapsed time
#[derive(Debug)]
pub struct BenchmarkTimer {
    start: Instant,
}

impl BenchmarkTimer {
    /// Start a new timer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

impl Default for BenchmarkTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BenchmarkTimer {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Don't print on panic
            return;
        }
        eprintln!("Timer elapsed: {}", format_duration(self.elapsed()));
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.warmup_iterations, 10);
        assert_eq!(config.confidence_level, 0.95);
    }

    #[test]
    fn test_benchmark_config_builder() {
        let config = BenchmarkConfig::new()
            .with_iterations(50)
            .with_warmup(5)
            .with_verbose(true);

        assert_eq!(config.iterations, 50);
        assert_eq!(config.warmup_iterations, 5);
        assert!(config.verbose);
    }

    #[test]
    fn test_benchmark_result_empty() {
        let result = BenchmarkResult::from_durations(&[]);
        assert_eq!(result.samples, 0);
        assert_eq!(result.mean, Duration::from_secs(0));
    }

    #[test]
    fn test_benchmark_result_single() {
        let durations = vec![Duration::from_millis(10)];
        let result = BenchmarkResult::from_durations(&durations);
        assert_eq!(result.samples, 1);
        assert_eq!(result.mean, Duration::from_millis(10));
        assert_eq!(result.min, Duration::from_millis(10));
        assert_eq!(result.max, Duration::from_millis(10));
    }

    #[test]
    fn test_benchmark_result_multiple() {
        let durations = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(15),
        ];
        let result = BenchmarkResult::from_durations(&durations);
        assert_eq!(result.samples, 3);
        assert_eq!(result.min, Duration::from_millis(10));
        assert_eq!(result.max, Duration::from_millis(20));
    }

    #[test]
    fn test_benchmark_runner() {
        let config = BenchmarkConfig::new().with_iterations(10).with_warmup(0);
        let mut runner = BenchmarkRunner::new(config);

        let result = runner.run(|| {
            // Simple computation
            let _ = (0..100).sum::<i32>();
        });

        assert_eq!(result.samples, 10);
    }

    #[test]
    fn test_format_duration() {
        assert!(format_duration(Duration::from_nanos(500)).contains("ns"));
        assert!(format_duration(Duration::from_micros(500)).contains("µs"));
        assert!(format_duration(Duration::from_millis(500)).contains("ms"));
        assert!(format_duration(Duration::from_secs(1)).contains("s"));
    }

    #[test]
    fn test_benchmark_timer() {
        let mut timer = BenchmarkTimer::new();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));

        timer.reset();
        let new_elapsed = timer.elapsed();
        assert!(new_elapsed < Duration::from_millis(1));
    }
}
