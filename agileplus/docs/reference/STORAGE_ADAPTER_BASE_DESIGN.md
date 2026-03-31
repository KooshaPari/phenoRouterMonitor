# Storage Adapter Base Framework Design

**Purpose**: Define the unified storage adapter architecture that will be shared across SQLite, Neo4j, and Plane.so implementations.

**Target Savings**: ~932 LOC through elimination of duplicate connection pooling, error handling, retry, and configuration logic.

---

## Design Principles

1. **Trait-Based Abstraction**: Each concern (pooling, config, error mapping, retry, health) is a separate trait
2. **Backend Agnostic**: Base framework makes no assumptions about storage backend
3. **Backward Compatible**: Existing SQLite adapter continues to work with minimal changes
4. **Composable**: Adapters can be extended with additional traits as needed
5. **Observable**: All adapters expose consistent health metrics and diagnostics

---

## Trait Architecture

### 1. StorageAdapterError — Unified Error Type

**File**: `crates/storage-adapter-base/src/error.rs`

```rust
use std::fmt;
use thiserror::Error;

/// Canonical error type for all storage operations.
/// Maps backend-specific errors to domain errors at the boundary.
#[derive(Debug, Error)]
pub enum StorageAdapterError {
    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("query failed: {0}")]
    QueryError(String),

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("authentication failed: {0}")]
    AuthenticationError(String),

    #[error("configuration invalid: {0}")]
    ConfigError(String),

    #[error("timeout after {duration_ms}ms")]
    TimeoutError { duration_ms: u64 },

    #[error("retry exhausted after {attempts} attempts: {reason}")]
    RetryExhausted { attempts: usize, reason: String },

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<StorageAdapterError> for agileplus_domain::error::DomainError {
    fn from(err: StorageAdapterError) -> Self {
        agileplus_domain::error::DomainError::Storage(err.to_string())
    }
}

/// Trait for mapping backend-specific errors to StorageAdapterError.
/// Each adapter implements this to translate its native error types.
pub trait ErrorMapper {
    /// Map a backend-specific error to StorageAdapterError.
    fn map_query_error(&self, err: impl fmt::Debug) -> StorageAdapterError;

    /// Map a backend connection error to StorageAdapterError.
    fn map_connection_error(&self, err: impl fmt::Debug) -> StorageAdapterError;

    /// Check if an error is transient (can be retried).
    fn is_transient(&self, err: &StorageAdapterError) -> bool;
}

/// Default error mapper implementation (can be overridden by adapters).
pub struct DefaultErrorMapper;

impl ErrorMapper for DefaultErrorMapper {
    fn map_query_error(&self, err: impl fmt::Debug) -> StorageAdapterError {
        StorageAdapterError::QueryError(format!("{:?}", err))
    }

    fn map_connection_error(&self, err: impl fmt::Debug) -> StorageAdapterError {
        StorageAdapterError::ConnectionError(format!("{:?}", err))
    }

    fn is_transient(&self, err: &StorageAdapterError) -> bool {
        matches!(err, StorageAdapterError::ConnectionError(_) | StorageAdapterError::TimeoutError { .. })
    }
}
```

---

### 2. StorageConnectionPool — Connection Lifecycle

**File**: `crates/storage-adapter-base/src/pool.rs`

```rust
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use serde::Serialize;

/// Statistics about the state of a connection pool.
#[derive(Debug, Clone, Serialize)]
pub struct PoolStats {
    /// Total connections managed by the pool.
    pub total_connections: usize,

    /// Idle connections available for checkout.
    pub idle_connections: usize,

    /// Connections currently in use.
    pub active_connections: usize,

    /// Total checkouts since pool creation.
    pub total_checkouts: usize,

    /// Total checkins since pool creation.
    pub total_checkins: usize,

    /// Timestamp of last activity.
    pub last_activity: SystemTime,

    /// Cumulative time spent waiting for connections (ms).
    pub wait_time_ms: u64,
}

/// Trait for connection pool implementations.
/// Each adapter wraps its native connection mechanism (Arc<Mutex>, Driver, Client)
/// in an implementation of this trait.
#[async_trait]
pub trait StorageConnectionPool: Send + Sync {
    /// The connection type returned by this pool.
    type Connection: Send;

    /// Acquire a connection from the pool.
    /// If the pool is exhausted, this will block until one becomes available
    /// or a timeout is reached.
    async fn acquire(&self) -> Result<Self::Connection, StorageAdapterError>;

    /// Acquire a connection with an explicit timeout.
    async fn acquire_with_timeout(
        &self,
        timeout_ms: u64,
    ) -> Result<Self::Connection, StorageAdapterError> {
        // Default implementation: uses tokio::time::timeout
        let duration = std::time::Duration::from_millis(timeout_ms);
        tokio::time::timeout(duration, self.acquire())
            .await
            .map_err(|_| StorageAdapterError::TimeoutError {
                duration_ms: timeout_ms,
            })?
    }

    /// Return a connection to the pool.
    /// If the connection is damaged (poisoned/closed), the pool may discard it.
    fn release(&self, conn: Self::Connection) -> Result<(), StorageAdapterError>;

    /// Get pool statistics.
    fn stats(&self) -> PoolStats;

    /// Gracefully drain the pool and close all connections.
    async fn close(&self) -> Result<(), StorageAdapterError>;

    /// Health check: verify at least one connection is valid.
    /// Used on startup and periodically to detect pool degradation.
    async fn health_check(&self) -> Result<(), StorageAdapterError>;
}

/// Wrapper for concrete pool implementations.
pub struct PoolMetrics {
    pub total_checkouts: Arc<AtomicUsize>,
    pub total_checkins: Arc<AtomicUsize>,
    pub last_activity: Arc<parking_lot::Mutex<SystemTime>>,
}

impl PoolMetrics {
    pub fn new() -> Self {
        Self {
            total_checkouts: Arc::new(AtomicUsize::new(0)),
            total_checkins: Arc::new(AtomicUsize::new(0)),
            last_activity: Arc::new(parking_lot::Mutex::new(SystemTime::now())),
        }
    }

    pub fn record_checkout(&self) {
        self.total_checkouts.fetch_add(1, Ordering::Relaxed);
        *self.last_activity.lock() = SystemTime::now();
    }

    pub fn record_checkin(&self) {
        self.total_checkins.fetch_add(1, Ordering::Relaxed);
        *self.last_activity.lock() = SystemTime::now();
    }
}
```

---

### 3. StorageAdapterConfig — Unified Configuration

**File**: `crates/storage-adapter-base/src/config.rs`

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Trait for unified adapter configuration.
/// Each adapter implements this with its backend-specific settings.
pub trait StorageAdapterConfig: Send + Sync {
    /// Validate the configuration. Called on startup to detect invalid settings.
    fn validate(&self) -> Result<(), StorageAdapterError>;

    /// Get the adapter name (e.g., "sqlite", "neo4j", "plane.so").
    fn adapter_name(&self) -> &str;

    /// Get the backend URI/path (for diagnostics).
    fn backend_uri(&self) -> &str;

    /// Get base configuration common to all adapters.
    fn base_config(&self) -> &BaseAdapterConfig;

    /// Get diagnostic information as JSON.
    fn diagnostics(&self) -> serde_json::Value;
}

/// Configuration common to all storage adapters.
/// Adapters can extend this with backend-specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseAdapterConfig {
    /// Operation timeout (seconds).
    #[serde(default = "default_operation_timeout_secs")]
    pub operation_timeout_secs: u64,

    /// Maximum number of retries for transient failures.
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Initial backoff delay for exponential retry (ms).
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,

    /// Maximum backoff delay for exponential retry (ms).
    #[serde(default = "default_max_backoff_ms")]
    pub max_backoff_ms: u64,

    /// Enable health checks on startup.
    #[serde(default = "default_health_check_on_init")]
    pub health_check_on_init: bool,

    /// Enable periodic health checks (seconds, 0 = disabled).
    #[serde(default = "default_health_check_interval_secs")]
    pub health_check_interval_secs: u64,

    /// Enable query logging.
    #[serde(default = "default_enable_query_logging")]
    pub enable_query_logging: bool,
}

impl Default for BaseAdapterConfig {
    fn default() -> Self {
        Self {
            operation_timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
            max_backoff_ms: 5000,
            health_check_on_init: true,
            health_check_interval_secs: 60,
            enable_query_logging: false,
        }
    }
}

fn default_operation_timeout_secs() -> u64 { 30 }
fn default_max_retries() -> usize { 3 }
fn default_retry_backoff_ms() -> u64 { 100 }
fn default_max_backoff_ms() -> u64 { 5000 }
fn default_health_check_on_init() -> bool { true }
fn default_health_check_interval_secs() -> u64 { 60 }
fn default_enable_query_logging() -> bool { false }

impl BaseAdapterConfig {
    pub fn operation_timeout(&self) -> Duration {
        Duration::from_secs(self.operation_timeout_secs)
    }

    pub fn health_check_interval(&self) -> Option<Duration> {
        if self.health_check_interval_secs > 0 {
            Some(Duration::from_secs(self.health_check_interval_secs))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let cfg = BaseAdapterConfig::default();
        assert_eq!(cfg.operation_timeout_secs, 30);
        assert_eq!(cfg.max_retries, 3);
        assert!(cfg.health_check_on_init);
    }
}
```

---

### 4. RetryPolicy — Exponential Backoff Retry

**File**: `crates/storage-adapter-base/src/retry.rs`

```rust
use async_trait::async_trait;
use std::future::Future;
use std::time::Duration;

/// Trait for retry policies.
#[async_trait]
pub trait RetryPolicy: Send + Sync {
    /// Execute an async operation with retry logic.
    /// If the operation fails, it will be retried according to the policy.
    async fn execute_with_retry<F, Fut, T>(
        &self,
        mut operation: F,
    ) -> Result<T, StorageAdapterError>
    where
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, StorageAdapterError>> + Send,
        T: Send;
}

/// Exponential backoff retry implementation.
/// Retries up to `max_retries` times with exponential backoff.
pub struct ExponentialBackoffRetry {
    pub max_retries: usize,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub error_mapper: Box<dyn ErrorMapper>,
}

impl ExponentialBackoffRetry {
    pub fn new(
        max_retries: usize,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
        error_mapper: Box<dyn ErrorMapper>,
    ) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
            max_backoff_ms,
            error_mapper,
        }
    }

    fn calculate_backoff(&self, attempt: usize) -> u64 {
        let exponential = self.initial_backoff_ms * 2_u64.pow(attempt as u32);
        exponential.min(self.max_backoff_ms)
    }
}

#[async_trait]
impl RetryPolicy for ExponentialBackoffRetry {
    async fn execute_with_retry<F, Fut, T>(
        &self,
        mut operation: F,
    ) -> Result<T, StorageAdapterError>
    where
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, StorageAdapterError>> + Send,
        T: Send,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    // Check if error is transient and we have retries left
                    if !self.error_mapper.is_transient(&err) || attempt >= self.max_retries {
                        last_error = Some(err);
                        break;
                    }

                    // Calculate backoff and sleep
                    let backoff_ms = self.calculate_backoff(attempt);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or(StorageAdapterError::Internal(
            "retry loop exited without error or result".into(),
        )))
    }
}

/// No-retry policy (for testing or when retries are not desired).
pub struct NoRetryPolicy;

#[async_trait]
impl RetryPolicy for NoRetryPolicy {
    async fn execute_with_retry<F, Fut, T>(
        &self,
        mut operation: F,
    ) -> Result<T, StorageAdapterError>
    where
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, StorageAdapterError>> + Send,
        T: Send,
    {
        operation().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_exponential_backoff_succeeds_on_retry() {
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let attempt_count_clone = attempt_count.clone();

        let retry = ExponentialBackoffRetry::new(
            3,
            10,
            100,
            Box::new(DefaultErrorMapper),
        );

        let result = retry
            .execute_with_retry(|| {
                let attempt_count = attempt_count_clone.clone();
                async move {
                    let attempts = attempt_count.fetch_add(1, Ordering::SeqCst);
                    if attempts < 2 {
                        Err(StorageAdapterError::ConnectionError("temporary".into()))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let retry = ExponentialBackoffRetry::new(
            2,
            10,
            100,
            Box::new(DefaultErrorMapper),
        );

        let result = retry
            .execute_with_retry(|| async {
                Err(StorageAdapterError::ConnectionError("persistent".into()))
            })
            .await;

        assert!(result.is_err());
    }
}
```

---

### 5. StorageAdapterHealthCheck — Health Monitoring

**File**: `crates/storage-adapter-base/src/health.rs`

```rust
use async_trait::async_trait;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Health status of a storage adapter.
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    /// Is the adapter healthy?
    pub healthy: bool,

    /// Latency of the health check (ms).
    pub latency_ms: u128,

    /// Human-readable status message.
    pub message: String,

    /// Timestamp of the check (seconds since epoch).
    pub checked_at: u64,

    /// Version of the adapter (for compatibility tracking).
    pub adapter_version: String,
}

impl HealthStatus {
    pub fn new(
        healthy: bool,
        latency_ms: u128,
        message: impl Into<String>,
        adapter_version: impl Into<String>,
    ) -> Self {
        let checked_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            healthy,
            latency_ms,
            message: message.into(),
            checked_at,
            adapter_version: adapter_version.into(),
        }
    }
}

/// Trait for health checks across all adapters.
#[async_trait]
pub trait StorageAdapterHealthCheck: Send + Sync {
    /// Run a health check. Returns status and latency.
    async fn health_check(&self) -> Result<HealthStatus, StorageAdapterError>;

    /// Get adapter version string.
    fn adapter_version(&self) -> &str;
}

/// Simple health check implementation: execute a no-op query.
pub struct SimpleHealthCheck {
    pub adapter_name: String,
    pub adapter_version: String,
    pub pool: Box<dyn StorageConnectionPool<Connection = ()>>,
}

#[async_trait]
impl StorageAdapterHealthCheck for SimpleHealthCheck {
    async fn health_check(&self) -> Result<HealthStatus, StorageAdapterError> {
        let start = std::time::Instant::now();
        match self.pool.acquire().await {
            Ok(_) => {
                let latency = start.elapsed().as_millis();
                Ok(HealthStatus::new(
                    true,
                    latency,
                    format!("{} is healthy", self.adapter_name),
                    &self.adapter_version,
                ))
            }
            Err(e) => {
                let latency = start.elapsed().as_millis();
                Ok(HealthStatus::new(
                    false,
                    latency,
                    format!("{} is unhealthy: {}", self.adapter_name, e),
                    &self.adapter_version,
                ))
            }
        }
    }

    fn adapter_version(&self) -> &str {
        &self.adapter_version
    }
}
```

---

## Integration Pattern

### How Adapters Use the Base Framework

Each adapter follows this pattern:

```rust
// In agileplus-sqlite/src/lib.rs
use storage_adapter_base::{
    StorageAdapterConfig, StorageConnectionPool, RetryPolicy,
    ExponentialBackoffRetry, ErrorMapper, DefaultErrorMapper,
};

pub struct SqliteConfig {
    base: BaseAdapterConfig,
    db_path: PathBuf,
}

impl StorageAdapterConfig for SqliteConfig {
    fn validate(&self) -> Result<(), StorageAdapterError> {
        // Validate db_path is accessible
        // ...
    }

    fn adapter_name(&self) -> &str { "sqlite" }
    fn backend_uri(&self) -> &str { self.db_path.to_string_lossy().as_ref() }
    fn base_config(&self) -> &BaseAdapterConfig { &self.base }
    fn diagnostics(&self) -> serde_json::Value {
        // Return pool stats, schema version, etc.
    }
}

pub struct SqlitePool {
    conn: Arc<Mutex<Connection>>,
    metrics: PoolMetrics,
}

#[async_trait]
impl StorageConnectionPool for SqlitePool {
    type Connection = Arc<Mutex<Connection>>;

    async fn acquire(&self) -> Result<Self::Connection, StorageAdapterError> {
        self.metrics.record_checkout();
        Ok(self.conn.clone())
    }

    fn release(&self, _conn: Self::Connection) -> Result<(), StorageAdapterError> {
        self.metrics.record_checkin();
        Ok(())
    }

    fn stats(&self) -> PoolStats { /* ... */ }
    async fn close(&self) -> Result<(), StorageAdapterError> { /* ... */ }
    async fn health_check(&self) -> Result<(), StorageAdapterError> { /* ... */ }
}

pub struct SqliteStorageAdapter {
    pool: Arc<SqlitePool>,
    config: Arc<SqliteConfig>,
    retry_policy: Arc<dyn RetryPolicy>,
    error_mapper: Box<dyn ErrorMapper>,
}

impl SqliteStorageAdapter {
    pub async fn new(config: SqliteConfig) -> Result<Self, DomainError> {
        config.validate()?;

        let pool = Arc::new(SqlitePool::new(&config)?);
        let retry_policy = Arc::new(ExponentialBackoffRetry::new(
            config.base.max_retries,
            config.base.retry_backoff_ms,
            config.base.max_backoff_ms,
            Box::new(SqliteErrorMapper),
        ));

        if config.base.health_check_on_init {
            pool.health_check().await?;
        }

        Ok(Self {
            pool,
            config: Arc::new(config),
            retry_policy,
            error_mapper: Box::new(SqliteErrorMapper),
        })
    }
}

impl StoragePort for SqliteStorageAdapter {
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        self.retry_policy
            .execute_with_retry(|| {
                let pool = self.pool.clone();
                let feature = feature.clone();
                async move {
                    let conn = pool.acquire().await?;
                    features::create_feature(&conn, &feature)
                        .map_err(|e| self.error_mapper.map_query_error(e))
                }
            })
            .await
            .map_err(|e| e.into())
    }
    // ... other trait methods
}
```

---

## Cargo.toml Structure

**storage-adapter-base/Cargo.toml**:
```toml
[package]
name = "storage-adapter-base"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tokio = { version = "1", features = ["time"] }
parking_lot = "0.12"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

---

## Testing Strategy

### Unit Tests
- Error mapper behavior (transient detection)
- Retry backoff calculation
- Config validation
- Pool metrics tracking

### Integration Tests
- Each adapter's pool implements `StorageConnectionPool` correctly
- Retry policy works with real backends
- Health checks detect unhealthy adapters
- Timeout handling works correctly

### Cross-Adapter Tests
- All three adapters pass the same `StoragePort` test suite
- Consistent error mapping across backends
- Consistent retry behavior

---

## Migration Path

1. **Create base framework** (no breaking changes to existing code)
2. **Implement traits in each adapter** (backward-compatible wrappers)
3. **Update initialization paths** to use new config system
4. **Run comprehensive test suite** against all backends
5. **Migrate metrics/monitoring** to use unified health check interface
6. **Deprecate old patterns** in favor of trait-based approach

---

## Performance Considerations

- **Trait indirection**: Minimal overhead (inlined in release builds)
- **Lock contention**: No change (pools handle locking as before)
- **Memory overhead**: +100-200 bytes per adapter instance for metrics
- **Latency**: <1ms per operation (retry/config lookup cached)

---

## Future Extensions

- **Circuit breaker pattern**: Add to `RetryPolicy` trait
- **Bulkhead isolation**: Separate pools for read vs write workloads
- **Metrics collection**: Prometheus exporter support
- **Rate limiting**: Per-adapter or per-operation limits
- **Connection warming**: Pre-warm pools on startup
