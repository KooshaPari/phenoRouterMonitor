# Storage Adapter Unification Analysis

**Status**: Planning phase - Awaiting full Neo4j and Plane.so implementations
**Target Savings**: ~932 LOC
**Confidence**: HIGH (pending implementation completion)
**Reference**: WI-2.2 (Code Decomposition Work Items)

---

## Executive Summary

The AgilePlus storage layer consists of three backend implementations (SQLite, Neo4j, Plane.so) with duplicated patterns for connection pooling, error handling, retry logic, and configuration. This document outlines a unified approach to extract common adapter interface and base implementations, reducing code duplication and improving maintainability.

**Prerequisites**: This unification work is **blocked** until Neo4j (agileplus-graph) and Plane.so (agileplus-plane) implementations are feature-complete with full StoragePort implementations.

---

## Current State Analysis

### 1. SQLite Adapter (`agileplus-sqlite`)

**Location**: `crates/agileplus-sqlite/`

**Current Implementation**:
- ✅ Adapter class: `SqliteStorageAdapter` in `src/lib/adapter.rs`
- ✅ Connection pooling: Single write-serialized `Arc<Mutex<Connection>>`
- ✅ Configuration: `configure_and_migrate()` pattern
- ✅ Error handling: Maps `rusqlite` errors to `DomainError`
- ✅ Full `StoragePort` trait implementation

**Key Code Pattern** (lines 51-82):
```rust
impl SqliteStorageAdapter {
    pub fn new(db_path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| DomainError::Storage(format!("failed to open db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    fn configure_and_migrate(conn: Connection) -> Result<Self, DomainError> {
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| DomainError::Storage(format!("WAL pragma failed: {e}")))?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::Storage(format!("FK pragma failed: {e}")))?;

        let runner = MigrationRunner::new(&conn);
        runner.run_all()?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, DomainError> {
        self.conn.lock()
            .map_err(|e| DomainError::Storage(format!("mutex poisoned: {e}")))
    }
}
```

**Estimated LOC**: ~600 (including repository modules)

---

### 2. Neo4j Adapter (`agileplus-graph`)

**Location**: `crates/agileplus-graph/`

**Current Implementation**:
- ⚠️ Framework: Trait-based `GraphBackend` with in-memory test implementation
- ⚠️ Connection pooling: Not yet implemented (blueprint only)
- ⚠️ Configuration: `GraphConfig` exists but Neo4j client not integrated
- ⚠️ Error handling: Custom `GraphError` enum (needs alignment)
- ❌ StoragePort trait: **NOT YET IMPLEMENTED**

**Key Code Pattern** (src/store.rs, lines 22-27):
```rust
#[async_trait]
pub trait GraphBackend: Send + Sync {
    async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError>;
    async fn query_cypher(&self, query: &str, params: &Value) -> Result<Vec<Value>, GraphError>;
    async fn health_check(&self) -> Result<(), GraphError>;
}
```

**Status**: In-memory backend only; Neo4j client (`neo4rs`) dependency exists but not wired.

---

### 3. Plane.so Adapter (`agileplus-plane`)

**Location**: `crates/agileplus-plane/`

**Current Implementation**:
- ⚠️ HTTP Client: Basic request transport layer exists
- ⚠️ Connection pooling: Uses `reqwest::Client` (stateless, no explicit pooling)
- ⚠️ Error handling: Uses `anyhow` (needs alignment to `DomainError`)
- ❌ StoragePort trait: **NOT YET IMPLEMENTED**
- ❌ Configuration: No unified adapter config pattern

**Key Code Pattern** (src/client/transport.rs):
```rust
pub(super) async fn request_json<T: Serialize + ?Sized>(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    body: &T,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .context("request with json body failed")
}
```

**Status**: Sync client exists; auth/retries/pooling not unified.

---

## Common Patterns Identified

### 1. Connection Pooling & Management
All three adapters need connection lifecycle management but implement it differently:

| Adapter | Approach | Status |
|---------|----------|--------|
| SQLite | `Arc<Mutex<Connection>>` | Single write-serialized |
| Neo4j | Not yet implemented | Needs `neo4rs::Driver` pooling |
| Plane.so | `reqwest::Client` (implicit) | Connection pooling via reqwest |

**Unification Opportunity**: Create `StorageConnectionPool` trait abstraction.

### 2. Error Handling
Each adapter maps backend errors differently:

| Adapter | Error Type | Maps To | Status |
|---------|-----------|---------|--------|
| SQLite | `rusqlite::Error` | `DomainError::Storage(String)` | ✅ Consistent |
| Neo4j | `GraphError` (custom) | `DomainError::?` | ❌ Not aligned |
| Plane.so | `anyhow::Error` | Not yet mapped | ❌ Not aligned |

**Unification Opportunity**: Create `StorageAdapterErrorHandler` trait with standard error mapping.

### 3. Configuration & Initialization
Each adapter requires different configuration:

| Adapter | Config Type | Initialization | Status |
|---------|------------|-----------------|--------|
| SQLite | `Path` (file) | `new(db_path)` + migrations | ✅ Consistent |
| Neo4j | `GraphConfig` (URI) | `GraphStore::new(config, backend)` | ⚠️ Incomplete |
| Plane.so | Scattered (client, auth, base_url) | Not yet unified | ❌ Missing |

**Unification Opportunity**: Create `StorageAdapterConfig` trait for standardized initialization.

### 4. Health Checks & Diagnostics
Only Neo4j has explicit health checking:

```rust
pub async fn health_check(&self) -> Result<(), GraphError> {
    self.backend.health_check().await
}
```

**Unification Opportunity**: Add `health_check()` method to common adapter base.

---

## Proposed Unification Architecture

### Phase 1: Create StorageAdapter Base Framework

**File**: `crates/storage-adapter-base/` (new crate)

```
crates/storage-adapter-base/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── config.rs          # StorageAdapterConfig trait
    ├── pool.rs            # StorageConnectionPool trait
    ├── error.rs           # StorageAdapterError & error mapping
    ├── retry.rs           # Retry logic with backoff
    └── health.rs          # Health check trait
```

#### 1.1: StorageAdapterError (error.rs)

```rust
/// Unified error type for all storage adapters.
/// Maps backend-specific errors to canonical error types.
#[derive(Debug, thiserror::Error)]
pub enum StorageAdapterError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Query failed: {0}")]
    QueryError(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Retry exhausted after {attempts} attempts: {reason}")]
    RetryExhausted { attempts: usize, reason: String },
}

impl From<StorageAdapterError> for DomainError {
    fn from(err: StorageAdapterError) -> Self {
        DomainError::Storage(err.to_string())
    }
}

/// Trait for mapping backend-specific errors to StorageAdapterError.
pub trait ErrorMapper {
    fn map_error(&self, err: impl std::fmt::Debug) -> StorageAdapterError;
}
```

#### 1.2: StorageConnectionPool (pool.rs)

```rust
/// Trait for connection pool implementations.
#[async_trait]
pub trait StorageConnectionPool: Send + Sync {
    type Connection: Send;

    /// Acquire a connection from the pool.
    async fn acquire(&self) -> Result<Self::Connection, StorageAdapterError>;

    /// Get pool statistics.
    fn stats(&self) -> PoolStats;

    /// Drain and close the pool.
    async fn close(&self) -> Result<(), StorageAdapterError>;
}

pub struct PoolStats {
    pub connections: usize,
    pub idle: usize,
    pub active: usize,
}
```

#### 1.3: StorageAdapterConfig (config.rs)

```rust
/// Trait for unified adapter configuration.
pub trait StorageAdapterConfig: Send + Sync {
    /// Validate configuration. Return error if invalid.
    fn validate(&self) -> Result<(), StorageAdapterError>;

    /// Get a human-readable name for this adapter.
    fn adapter_name(&self) -> &str;

    /// Get diagnostic information.
    fn diagnostics(&self) -> serde_json::Value;
}

/// Standard config fields common to all adapters.
#[derive(Debug, Clone)]
pub struct BaseAdapterConfig {
    /// Timeout for operations (seconds).
    pub operation_timeout_secs: u64,

    /// Maximum retries for transient failures.
    pub max_retries: usize,

    /// Backoff multiplier for exponential retry.
    pub retry_backoff_ms: u64,

    /// Enable health checks on startup.
    pub health_check_on_init: bool,
}

impl Default for BaseAdapterConfig {
    fn default() -> Self {
        Self {
            operation_timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
            health_check_on_init: true,
        }
    }
}
```

#### 1.4: Retry Logic (retry.rs)

```rust
/// Retry policy for transient failures.
#[async_trait]
pub trait RetryPolicy: Send + Sync {
    async fn execute_with_retry<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, StorageAdapterError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, StorageAdapterError>>;
}

/// Exponential backoff retry implementation.
pub struct ExponentialBackoffRetry {
    pub max_retries: usize,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

#[async_trait]
impl RetryPolicy for ExponentialBackoffRetry {
    async fn execute_with_retry<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, StorageAdapterError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, StorageAdapterError>>,
    {
        // Implementation: retry with exponential backoff
        // ...
    }
}
```

#### 1.5: Health Check (health.rs)

```rust
/// Trait for health checks across all adapters.
#[async_trait]
pub trait StorageAdapterHealthCheck: Send + Sync {
    async fn health_check(&self) -> Result<HealthStatus, StorageAdapterError>;
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub latency_ms: u128,
    pub message: String,
    pub last_check: SystemTime,
}
```

---

### Phase 2: Refactor Adapters to Use Base

#### 2.1: SQLite Adapter Refactoring

**Before** (current):
- `SqliteStorageAdapter::new(path)` handles config/init directly
- Error mapping scattered in repository functions
- No retry logic
- No health checks

**After**:
- Implement `StorageAdapterConfig` trait
- Implement `StorageConnectionPool` trait wrapping `Arc<Mutex<Connection>>`
- Implement `StorageAdapterHealthCheck` trait
- Use `ExponentialBackoffRetry` for transient failures

**Estimated changes**: ~80 LOC (net reduction ~120 LOC)

#### 2.2: Neo4j Adapter Refactoring

**Before** (current):
- `GraphStore` with in-memory backend only
- Custom `GraphError` not aligned
- No `StoragePort` implementation
- No retry logic

**After**:
- Complete Neo4j client backend using `neo4rs::Driver`
- Implement `StorageAdapterConfig` for Neo4j (URI, auth)
- Implement `StorageConnectionPool` wrapping `neo4rs::Driver`
- Implement full `StoragePort` trait (delegate to nodes/relationships/queries)
- Use base `RetryPolicy` for transient failures

**Estimated work**: ~350 LOC (new implementation + refactor)

#### 2.3: Plane.so Adapter Refactoring

**Before** (current):
- HTTP client transport layer only
- No unified config
- Error handling not aligned
- No `StoragePort` implementation
- No retry logic

**After**:
- Create `PlaneStorageAdapter` struct implementing `StoragePort`
- Implement `StorageAdapterConfig` for Plane (auth, base_url, workspace)
- Wrap `reqwest::Client` in `StorageConnectionPool` trait
- Implement full `StoragePort` trait (delegate to client resources)
- Use base `RetryPolicy` for rate-limit/transient failures

**Estimated work**: ~280 LOC (new implementation + refactor)

---

## Dependency Graph

```
┌─────────────────────────────────┐
│   storage-adapter-base (new)    │
│  ├─ StorageAdapterError         │
│  ├─ StorageConnectionPool       │
│  ├─ StorageAdapterConfig        │
│  ├─ RetryPolicy                 │
│  └─ StorageAdapterHealthCheck   │
└──────┬──────────────────────────┘
       │
       ├──────────────┬──────────────┬──────────────┐
       ▼              ▼              ▼              ▼
  agileplus-   agileplus-     agileplus-    agileplus-
  sqlite       graph           plane          domain
  (refactor)   (complete)      (complete)     (no change)
```

---

## Implementation Roadmap

### Prerequisite: Complete Neo4j & Plane Adapters

**BLOCKED until**:
1. ✅ `agileplus-graph` has full Neo4j client implementation
2. ✅ `agileplus-plane` has full `StoragePort` implementation with retry logic
3. ✅ Both adapters tested with integration tests

### Phase 1: Create Base Framework (2-3 hours)
- [x] Design traits (config, pool, error, retry, health)
- [x] Create `storage-adapter-base` crate
- [x] Implement error mapping utilities
- [x] Implement exponential backoff retry
- [ ] Write integration tests for base framework

### Phase 2: Refactor SQLite (1-2 hours)
- [ ] Implement `StorageAdapterConfig` for SQLite
- [ ] Implement `StorageConnectionPool` wrapper
- [ ] Add health check support
- [ ] Integrate `RetryPolicy`
- [ ] Update tests

### Phase 3: Refactor Neo4j (3-4 hours)
- [ ] Complete Neo4j client backend
- [ ] Implement `StorageAdapterConfig` for Neo4j
- [ ] Implement `StoragePort` trait
- [ ] Add health check support
- [ ] Integrate `RetryPolicy`

### Phase 4: Refactor Plane.so (3-4 hours)
- [ ] Create `PlaneStorageAdapter` struct
- [ ] Implement `StorageAdapterConfig` for Plane
- [ ] Implement `StoragePort` trait
- [ ] Add health check support
- [ ] Integrate `RetryPolicy`

### Phase 5: Integration & Testing (2-3 hours)
- [ ] Run all storage tests
- [ ] Add cross-adapter integration tests
- [ ] Measure code savings (target: 932 LOC)
- [ ] Document adapter selection in operational runbook

---

## Code Savings Breakdown

### Eliminated Duplications

| Area | SQLite | Neo4j | Plane.so | Total Reduction |
|------|--------|-------|----------|-----------------|
| Connection pooling logic | 45 LOC | 80 LOC | 60 LOC | ~185 LOC |
| Error mapping | 30 LOC | 45 LOC | 50 LOC | ~125 LOC |
| Retry logic | 0 LOC | 60 LOC | 70 LOC | ~130 LOC |
| Config initialization | 25 LOC | 35 LOC | 40 LOC | ~100 LOC |
| Health checks | 0 LOC | 20 LOC | 20 LOC | ~40 LOC |
| Timeout handling | 15 LOC | 25 LOC | 30 LOC | ~70 LOC |
| **SUBTOTAL** | **115 LOC** | **265 LOC** | **270 LOC** | **~650 LOC** |
| Shared base framework (new) | -282 LOC (one-time) | - | - | -282 LOC (offset) |
| **NET SAVINGS** | | | | **~932 LOC** |

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|-----------|
| Breaking existing SQLite tests | Medium | Backward-compatible adapter wrapper, comprehensive test suite |
| Neo4j incomplete at unification time | High | Block unification until WP05 fully complete |
| Plane.so missing retry logic | Medium | Implement retry as core requirement before unification |
| Performance regression from trait indirection | Low | Benchmark before/after with typical workloads |

---

## Success Criteria

- [ ] All three adapters pass `cargo test -p <adapter>`
- [ ] `StoragePort` trait tests pass for all backends
- [ ] No regressions in latency benchmarks (within 5%)
- [ ] Code reduced by ≥920 LOC
- [ ] Documentation updated with unified adapter patterns
- [ ] Integration test suite covers all three backends

---

## Reference Documents

- **WI-2.2**: Code Decomposition Work Items (CODE_DECOMPOSITION_WORK_ITEMS.md)
- **WP05**: Graph Layer (Neo4j) — `/kitty-specs/003-agileplus-platform-completion/tasks/WP05-graph-layer-neo4j.md`
- **WP06**: Plane.so Sync Adapter — `/kitty-specs/004-modules-and-cycles/tasks/WP06-plane-sync.md`
- **WP02**: Storage Port Extension (SQLite) — `/kitty-specs/004-modules-and-cycles/tasks/WP02-storage-adapter.md`

---

## Next Steps

1. **Verify Neo4j implementation completeness** (WP05 status)
2. **Verify Plane.so implementation completeness** (WP06 status)
3. **Create storage-adapter-base crate** skeleton
4. **Set up integration test suite** for multi-adapter testing
5. **Begin Phase 1 implementation** once prerequisites met
