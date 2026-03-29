# DUPLICATION Worklogs

Cross-project duplication audits, code reuse analysis, and library extraction tracking.

---

## 2026-03-29 - Cross-Project Duplication Audit (Comprehensive)

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Summary

Comprehensive audit of cross-project duplication in AgilePlus codebase. Identified 6 major duplication categories with 40+ specific instances.

### Categories Identified

| Category | Count | Priority |
|----------|-------|----------|
| Error Types | 36+ | High |
| Health Checks | 4 | Medium |
| Config Loading | 4 | High |
| Store Traits | 3 | Medium |
| In-Memory Backends | 4 | Low |
| Builder Patterns | 3 | Low |

### Error Type Duplications

```rust
// Pattern repeated in 10+ crates
pub enum SomeError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Storage error: {0}")]
    Storage(String),
}
```

### Files Affected

- `crates/agileplus-domain/src/error.rs`
- `crates/agileplus-api/src/error.rs`
- `crates/agileplus-sync/src/error.rs`
- `crates/agileplus-p2p/src/error.rs`
- `crates/agileplus-nats/src/error.rs`
- `crates/agileplus-events/src/lib.rs`
- `crates/agileplus-graph/src/lib.rs`
- `crates/agileplus-cache/src/lib.rs`
- `libs/nexus/src/error.rs`
- `libs/hexagonal-rs/src/lib.rs`

### Related
- FORK-002: Extract to `phenotype-error`

---

## 2026-03-29 - Error Type Pattern Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Summary

Analyzed error type patterns across all crates. Found 36+ distinct error enum definitions with significant semantic overlap.

### Duplication Matrix

| Error Variant | Present In |
|--------------|------------|
| `NotFound` | DomainError, ApiError, GraphError, EventError, NexusError |
| `Conflict` | DomainError, ApiError, SyncError |
| `Storage/Store` | DomainError::Storage, SyncError::Store, GraphError, EventError::StorageError, CacheError |
| `Serialization` | SyncError, P2P SyncError, CacheError, EventBusError |
| `Config/InvalidConfig` | ConfigError (multiple), NexusError, gauge Error |

### Specific Duplications

**SyncError (agileplus-sync/src/error.rs)**
```rust
pub enum SyncError {
    #[error("Store error: {0}")]
    Store(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Entity not found: {entity_type}/{entity_id}")]
    EntityNotFound { entity_type: String, entity_id: i64 },
}
```

**SyncError (agileplus-p2p/src/error.rs)**
```rust
pub enum SyncError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Event store error: {0}")]
    EventStore(String),
}
```

**EventBusError (agileplus-nats/src/bus.rs)**
```rust
pub enum EventBusError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
```

**CacheError (agileplus-cache/src/store.rs)**
```rust
pub enum CacheError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}
```

### Recommendation

Extract shared error variants to `phenotype-error` crate:
- `NotFound`
- `Conflict`
- `InvalidInput`
- `Serialization`
- `Storage`
- `Timeout`
- `Unauthorized`

---

## 2026-03-29 - Health Check Duplication Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Identified 4 health enum definitions with different semantics across crates.

### Current Definitions

**GraphHealth (agileplus-graph/src/health.rs)**
```rust
pub enum GraphHealth {
    Healthy,
    Unavailable,
}
```

**CacheHealth (agileplus-cache/src/health.rs)**
```rust
pub enum CacheHealth {
    Healthy,
    Unavailable,
}
```

**BusHealth (agileplus-nats/src/health.rs)**
```rust
pub enum BusHealth {
    Connected,
    Disconnected,
}
```

**HealthStatus (agileplus-domain/src/domain/service_health.rs)**
```rust
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}
```

### Issues

1. Different enum names for similar concepts
2. No consistency in health levels
3. No shared trait for health checks
4. Duplicated health check implementations

### Recommendation

Create `phenotype-health` with unified HealthLevel:
```rust
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unavailable,
}
```

---

## 2026-03-29 - Configuration Loading Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Identified 4 configuration loading implementations with identical patterns.

### Duplicated Patterns

| Crate | File | Pattern |
|-------|------|---------|
| agileplus-domain | `src/config/loader.rs` | TOML, `dirs_next::home_dir()` |
| agileplus-dashboard | `src/routes.rs:137-170` | TOML, `std::env::var("HOME")` |
| agileplus-telemetry | `src/config.rs` | YAML |
| agileplus-subcmds | `src/sync/config.rs` | JSON |

### Repeated Code Pattern

```rust
impl AppConfig {
    pub fn config_path() -> PathBuf {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".agileplus")
            .join("config.toml")
    }
    
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content).map_err(ConfigError::from)
    }
}
```

### dirs_next::home_dir() Usage

Duplicated in:
- `crates/agileplus-telemetry/src/config.rs:209`
- `crates/agileplus-domain/src/config/core.rs:26`
- `crates/agileplus-domain/src/config/credentials.rs:32`
- `crates/agileplus-domain/src/config/loader.rs:24`

### Recommendation

Extract to `phenotype-config`:
```rust
pub fn home_dir() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".phenotype")
}
```

---

## 2026-03-29 - In-Memory Backend Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified 4 identical in-memory test store implementations.

### Current Implementations

| Crate | Type | File | Lines |
|-------|------|------|-------|
| agileplus-nats | `InMemoryBus` | `src/bus.rs:127` | ~50 |
| agileplus-graph | `InMemoryBackend` | `src/store.rs:106` | ~50 |
| agileplus-domain | `InMemoryCredentialStore` | `src/credentials/memory.rs:15` | ~30 |
| agileplus-sync | `InMemoryStore` | `src/store.rs:59` | ~40 |

### Common Pattern

```rust
#[derive(Default)]
pub struct InMemoryStore {
    data: HashMap<String, Vec<u8>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CacheStore for InMemoryStore {
    async fn get(&self, key: &str) -> Result<Option<T>, CacheError> {
        // ...
    }
}
```

### Recommendation

Extract base implementation to `phenotype-store`:
```rust
pub struct InMemoryStore<K, V> {
    data: RwLock<HashMap<K, V>>,
}

impl<K: Eq + Hash + Clone, V: Clone> InMemoryStore<K, V> {
    pub fn new() -> Self { /* ... */ }
}
```

---

## 2026-03-29 - Store Trait Pattern Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Analyzed async store traits across crates. Found 3 similar but incompatible trait definitions.

### Current Traits

**EventStore (agileplus-events/src/store.rs)**
```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, event: &Event) -> Result<i64, EventError>;
    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError>;
}
```

**CacheStore (agileplus-cache/src/store.rs)**
```rust
#[async_trait]
pub trait CacheStore: Send + Sync {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>;
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
}
```

**GraphBackend (agileplus-graph/src/store.rs)**
```rust
#[async_trait]
pub trait GraphBackend: Send + Sync {
    async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError>;
    async fn query_cypher(&self, query: &str, params: &Value) -> Result<Vec<Value>, GraphError>;
    async fn health_check(&self) -> Result<(), GraphError>;
}
```

### Common Pattern

All follow the same structure:
1. `#[async_trait]` derive
2. `Send + Sync` bounds
3. Async methods with `Result<T, Error>` returns

### Recommendation

Create base `Store` trait:
```rust
#[async_trait]
pub trait Store<K, V>: Send + Sync {
    type Error;
    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    async fn set(&self, key: &K, value: &V) -> Result<(), Self::Error>;
    async fn delete(&self, key: &K) -> Result<(), Self::Error>;
}
```

---

## 2026-03-29 - Builder Pattern Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified 3 configuration builder implementations with similar patterns.

### Current Builders

**GraphConfig (agileplus-graph/src/config.rs)**
```rust
impl GraphConfig {
    pub fn new(bolt_uri: String, username: String, password: String) -> Self { ... }
    pub fn with_database(mut self, db: String) -> Self { self.database = db; self }
}
```

**CacheConfig (agileplus-cache/src/config.rs)**
```rust
impl CacheConfig {
    pub fn new(host: String, port: u16) -> Self { ... }
    pub fn with_pool_size(mut self, size: u32) -> Self { self.pool_size = size; self }
    pub fn with_default_ttl(mut self, secs: u64) -> Self { ... }
    pub fn redis_url(&self) -> String { ... }
}
```

**NatsConfig (agileplus-nats/src/config.rs)**
```rust
impl NatsConfig {
    pub fn new(url: impl Into<String>) -> Self { ... }
    pub fn with_auth(mut self, token: impl Into<String>) -> Self { ... }
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self { ... }
}
```

### Recommendation

Low priority - builders are crate-specific enough to justify duplication.

---

## 2026-03-29 - Library Crate Error Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P1

### Summary

Analyzed error types in `libs/` directory. Found errors mirroring crate errors.

### Library Error Types

| Library | Error | Similar To |
|---------|-------|------------|
| `libs/nexus/src/error.rs` | `NexusError::NotFound`, `InvalidConfig` | `DomainError::NotFound`, `ConfigError` |
| `libs/hexagonal-rs/src/lib.rs` | `HexagonalError::NotFound`, `Validation`, `Conflict` | Multiple crate errors |
| `libs/cipher/src/domain/error.rs` | Domain errors | Should be in domain crate |
| `libs/gauge/src/domain/error.rs` | `Error::InvalidConfig` | Duplicate of telemetry config |

### Issues

1. No shared error hierarchy
2. Errors duplicated between libs and crates
3. No conversion implementations between lib and crate errors

### Recommendation

1. Move library errors to shared `phenotype-error`
2. Add `From<LibError> for CrateError` implementations
3. Document error ownership

---

## 2026-03-29 - Error Propagation Gaps

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P1

### Summary

Identified inconsistent error conversion patterns across crates.

### Current Conversions

**agileplus-api/src/error.rs:56-66**
```rust
impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(s) => ApiError::NotFound(s),
            DomainError::InvalidInput(s) => ApiError::BadRequest(s),
            // ...
        }
    }
}
```

### Missing Conversions

| From | To | Status |
|------|-----|--------|
| DomainError | GraphError | Missing |
| DomainError | SyncError | Missing |
| SyncError | EventError | Missing |
| NexusError | DomainError | Missing |

### Recommendation

1. Add blanket conversion traits
2. Document conversion expectations
3. Add tests for error propagation

---

## 2026-03-29 - Config Format Inconsistencies

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified inconsistent configuration formats across crates.

### Current Formats

| Crate | Format | Error Handling |
|-------|--------|----------------|
| agileplus-domain | TOML | `ConfigError` enum with `thiserror` |
| agileplus-dashboard | TOML | `Box<dyn std::error::Error>` |
| agileplus-telemetry | YAML | `ConfigError` enum with `thiserror` |
| agileplus-subcmds | JSON | `anyhow::Result` |

### Problems

1. Different error handling strategies
2. No standard config format
3. Harder to share config between crates

### Recommendation

1. Standardize on TOML for file configs
2. Use `thiserror` consistently
3. Create shared `ConfigError` type

---

## 2026-03-29 - Duplicate Git Operations

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P1

### Summary

Identified multiple git operation implementations across projects.

### Current Locations

| Location | Pattern | LOC | Quality |
|----------|---------|-----|---------|
| `utils/git/` | Raw git commands | ~300 | Basic |
| `agileplus-git/` | Full git library | N/A | Better |
| `crates/*/src/` | Inline git calls | ~100 | Mixed |

### Duplicated Operations

```rust
// In utils/git/ and inline in various crates
fn run_git(args: &[&str]) -> Result<String> {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

fn git_status() -> Result<GitStatus> { /* ... */ }
fn git_branch() -> Result<String> { /* ... */ }
fn git_commit(msg: &str) -> Result<()> { /* ... */ }
```

### Recommendation

1. Fork `agileplus-git` to `phenotype-git`
2. Deprecate `utils/git/`
3. Migrate all inline git calls

---

## 2026-03-29 - PTY/Process Management Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P0

### Summary

Identified PTY management in `utils/pty/` used by heliosCLI.

### Current Implementation

- Location: `utils/pty/`
- Lines: ~750
- Used by: heliosCLI
- Could benefit: Any project needing terminal emulation

### Core Components

```rust
pub struct PtyProcess { /* ... */ }
pub struct PtyReader { /* ... */ }
pub struct PtyWriter { /* ... */ }

impl PtyProcess {
    pub fn spawn(cmd: &[&str], cwd: Option<&Path>) -> Result<Self>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    pub fn write(&mut self, buf: &[u8]) -> Result<usize>;
}
```

### Recommendation

Extract to `phenotype-process`:
- FORK-001: Highest priority
- Reuse across projects needing process management

---

## 2026-03-29 - CLI Command Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Analyzed CLI commands across agileplus-cli, heliosCLI, and thegent.

### Shared Commands

| Command | agileplus-cli | heliosCLI | thegent |
|---------|--------------|-----------|---------|
| init | Yes | Yes | No |
| config | Yes | Yes | No |
| login | Yes | Yes | Yes |
| status | Yes | Yes | Yes |

### Duplicated Logic

```rust
// Similar patterns in multiple CLIs
async fn login(creds: Credentials) -> Result<Token> {
    let resp = client.post("/auth/login")
        .json(&creds)
        .send()
        .await?;
    // ... parse token
}
```

### Recommendation

1. Create shared `phenotype-cli-core`
2. Extract common auth/config handling
3. Keep project-specific commands separate

---

## 2026-03-29 - Serialization Pattern Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified repeated serialization logic across stores.

### Current Patterns

```rust
// Repeated in multiple stores
fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|e| Error::Serialization(e.to_string()))
}

fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(bytes).map_err(|e| Error::Serialization(e.to_string()))
}
```

### Locations

- `crates/agileplus-sync/src/error.rs`
- `crates/agileplus-p2p/src/error.rs`
- `crates/agileplus-nats/src/bus.rs`
- `crates/agileplus-cache/src/store.rs`

### Recommendation

Extract to `phenotype-serialization`:
```rust
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, SerializationError> {
    serde_json::to_vec(value).map_err(SerializationError::from)
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SerializationError> {
    serde_json::from_slice(bytes).map_err(SerializationError::from)
}
```

---

## 2026-03-29 - Telemetry Duplicate Implementations

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified duplicated telemetry/logging setup across crates.

### Current Setup

| Crate | Tracing | Metrics | Logging |
|-------|---------|---------|---------|
| agileplus-domain | tracing | metrics | tracing |
| agileplus-api | tracing | metrics | tracing |
| agileplus-sync | tracing | metrics | tracing |
| agileplus-cli | tracing | metrics | tracing |
| heliosCLI | tracing | metrics | tracing |

### Repeated Setup Code

```rust
// In each binary
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

let metrics_handle = metrics::launch_metrics_server();
let tracer = opentelemetry::sdk::trace::TracerProvider::builder()
    // ...
    .build();
```

### Recommendation

1. Create `phenotype-telemetry` for setup
2. Add `telemetry::init()` helper
3. Standardize on one logging format

---

## 2026-03-29 - Database Pool Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified duplicated database connection pool setup.

### Current Patterns

```rust
// Repeated in multiple crates
let pool = PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;
```

### Locations

- `crates/agileplus-domain/src/database.rs`
- `crates/agileplus-api/src/database.rs`
- `crates/agileplus-sync/src/database.rs`

### Recommendation

Extract to `phenotype-database`:
```rust
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.timeout)
        .connect(&config.url)
        .await
        .map_err(DatabaseError::from)
}
```

---

## 2026-03-29 - Validation Logic Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified duplicated input validation across crates.

### Repeated Validators

```rust
// Pattern repeated in multiple places
fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.contains('@') {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail)
    }
}

fn validate_url(url: &str) -> Result<(), ValidationError> {
    Url::parse(url).map_err(|_| ValidationError::InvalidUrl)?;
    Ok(())
}
```

### Locations

- `crates/agileplus-domain/src/validation.rs`
- `crates/agileplus-api/src/middleware/validation.rs`
- `libs/cipher/src/domain/validation.rs`

### Recommendation

Extract to `phenotype-validation`:
```rust
pub mod validators {
    pub fn email(s: &str) -> Result<(), ValidationError>;
    pub fn url(s: &str) -> Result<(), ValidationError>;
    pub fn non_empty(s: &str) -> Result<(), ValidationError>;
    pub fn max_len(s: &str, len: usize) -> Result<(), ValidationError>;
}
```

---

## 2026-03-29 - DateTime Handling Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified inconsistent DateTime handling across crates.

### Current Usage

| Crate | Type | Format |
|-------|------|--------|
| agileplus-domain | `DateTime<Utc>` | ISO 8601 |
| agileplus-events | `DateTime<Utc>` | ISO 8601 |
| agileplus-api | `DateTime<Utc>` | ISO 8601 |
| heliosCLI | `chrono::DateTime` | Mixed |

### Repeated Patterns

```rust
fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| Error::InvalidTimestamp)
}
```

### Recommendation

Low priority - standardization would help but low impact.

---

## 2026-03-29 - Async Runtime Setup Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified duplicated async runtime setup.

### Repeated Code

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::select! {
        result = server.run() => result?,
        signal = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
            Ok(())
        }
    }
}
```

### Recommendation

Low priority - #[tokio::main] is standard enough.

---

## 2026-03-29 - Logging Macros Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified custom logging macro patterns.

### Current Patterns

```rust
// Various forms used
tracing::info!("Processing {}", entity);
log::info!("Processing {}", entity);
eprintln!("Error: {}", err);
```

### Recommendation

Standardize on `tracing` throughout.

---

## 2026-03-29 - Test Utility Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified duplicated test utilities.

### Repeated Test Helpers

```rust
// In various test modules
fn test_entity() -> Entity {
    Entity {
        id: 1,
        name: "Test".into(),
        created_at: Utc::now(),
    }
}

fn create_test_db() -> TestDatabase {
    // ... setup test database
}

#[fixture]
fn mock_client() -> MockClient { /* ... */ }
```

### Recommendation

Create `phenotype-test-utils`:
```rust
pub mod fixtures {
    pub fn test_entity() -> Entity { /* ... */ }
    pub fn test_config() -> Config { /* ... */ }
}

pub struct TestDatabase { /* ... */ }
impl TestDatabase {
    pub async fn new() -> Self { /* ... */ }
    pub async fn cleanup(self) { /* ... */ }
}
```

---

## 2026-03-29 - Metric Name Inconsistencies

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified inconsistent metric naming across crates.

### Current Naming

| Crate | Counter | Histogram |
|-------|---------|-----------|
| agileplus-domain | `domain_events_total` | `domain_operation_duration_seconds` |
| agileplus-api | `api_requests_total` | `api_request_duration_seconds` |
| agileplus-sync | `sync_operations_total` | `sync_duration_seconds` |

### Recommendation

Create naming conventions document:
```rust
// Consistent prefixes
const METRIC_PREFIX: &str = "agileplus";

pub fn counter(name: &str) -> String {
    format!("{}_{}", METRIC_PREFIX, name)
}
```

---

## 2026-03-29 - gRPC Service Definition Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified gRPC service definitions that could be consolidated.

### Current Services

| Proto | Services | File |
|-------|----------|------|
| `schemas/agileplus.proto` | AgilePlusService | Main API |
| `schemas/event.proto` | EventService | Events |
| `schemas/sync.proto` | SyncService | Sync |

### Recommendation

1. Consolidate into single proto package
2. Use package versioning
3. Add proto lint rules

---

## 2026-03-29 - Domain Event Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified domain events that are semantically similar.

### Event Comparison

| Event | Crate | Fields |
|-------|-------|--------|
| `EntityCreated` | domain | id, name, created_at |
| `ObjectCreated` | p2p | id, name, timestamp |
| `ResourceCreated` | sync | id, name, created |

### Recommendation

1. Create shared event types
2. Use event upcasting for compatibility
3. Document event ownership

---

## 2026-03-29 - Port/Trait Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified trait/port definitions that could be unified.

### Current Ports

```rust
// In various crates
pub trait EntityRepository { /* ... */ }
pub trait ProjectStore { /* ... */ }
pub trait SpecStore { /* ... */ }
pub trait CredentialStore { /* ... */ }
```

### Recommendation

Consider hexagonal architecture adoption:
```rust
pub trait Repository<T: AggregateRoot> {
    async fn save(&self, aggregate: &T) -> Result<(), Error>;
    async fn find(&self, id: &T::Id) -> Result<Option<T>, Error>;
}
```

---

## 2026-03-29 - Error Message String Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified duplicated error message strings.

### Repeated Messages

```rust
"Not found: {}"
"Invalid input: {}"
"Operation failed: {}"
"Connection error: {}"
```

### Recommendation

Extract to error constants:
```rust
pub mod errors {
    pub const NOT_FOUND: &str = "Not found: {}";
    pub const INVALID_INPUT: &str = "Invalid input: {}";
}
```

---

## 2026-03-29 - ID Generation Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified multiple ID generation patterns.

### Current Patterns

```rust
// Various ID generation
let id = Ulid::new();
let id =Uuid::new_v4();
let id: i64 = rand::random();
```

### Recommendation

Standardize on ULID for entities:
```rust
pub fn new_id() -> Ulid {
    Ulid::new()
}
```

---

## 2026-03-29 - Authentication Middleware Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified duplicated authentication middleware.

### Repeated Pattern

```rust
async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, Error> {
    let token = req.headers()
        .get("Authorization")?
        .to_str()?;
    validate_token(token)?;
    next.run(req).await
}
```

### Recommendation

Extract to `phenotype-auth`:
```rust
pub async fn auth_middleware<B>(
    req: Request<B>,
    next: Next,
) -> Result<Response, AuthError> {
    // Standardized auth logic
}
```

---

## 2026-03-29 - Rate Limiting Implementation Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified rate limiting logic that could be shared.

### Current Implementation

```rust
// Repeated pattern
async fn check_rate_limit(
    client_id: &str,
    limit: u32,
) -> Result<(), RateLimitError> {
    let count = redis.incr(client_id).await?;
    if count > limit {
        return Err(RateLimitError::Exceeded);
    }
    Ok(())
}
```

### Recommendation

Extract to `phenotype-rate-limit`:
```rust
pub struct RateLimiter {
    redis: RedisPool,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub async fn check(&self, key: &str) -> Result<(), RateLimitError>;
}
```

---

## 2026-03-29 - Retry Logic Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified duplicated retry/backoff logic.

### Repeated Pattern

```rust
async fn with_retry<F, T, E>(mut f: F) -> Result<T, E>
where
    F: FnMut() -> future::Future<Output = Result<T, E>>,
{
    let mut backoff = Duration::from_millis(100);
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                tokio::time::sleep(backoff).await;
                backoff *= 2;
                if backoff > Duration::from_secs(30) {
                    return Err(e);
                }
            }
        }
    }
}
```

### Recommendation

Extract to `phenotype-retry`:
```rust
pub async fn retry<F, T, E>(
    f: impl Fn() -> F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>;
```

---

## 2026-03-29 - Circuit Breaker Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified circuit breaker patterns that could be shared.

### Current State

No shared circuit breaker - each service implements its own.

### Recommendation

Use existing crate (`opossum`) or extract:
```rust
pub struct CircuitBreaker {
    state: Arc<AtomicBool>,
    failures: Arc<AtomicUsize>,
    threshold: usize,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T, CircuitOpen>;
}
```

---

## 2026-03-29 - Feature Flag Check Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified feature flag checks repeated across codebase.

### Repeated Pattern

```rust
fn is_feature_enabled(name: &str) -> bool {
    std::env::var(format!("FEATURE_{}", name.to_uppercase()))
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

if is_feature_enabled("new_sync") {
    // new implementation
} else {
    // legacy
}
```

### Recommendation

Extract to `phenotype-features`:
```rust
pub struct FeatureFlags {
    inner: HashMap<String, bool>,
}

impl FeatureFlags {
    pub fn is_enabled(&self, name: &str) -> bool;
}
```

---

## 2026-03-29 - Timeout Handling Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified repeated timeout configuration.

### Repeated Pattern

```rust
let result = tokio::time::timeout(
    Duration::from_secs(30),
    operation()
).await?;
```

### Recommendation

Standardize on shared timeout config:
```rust
pub struct Timeouts {
    pub default: Duration,
    pub database: Duration,
    pub http: Duration,
    pub grpc: Duration,
}
```

---

## 2026-03-29 - Request ID Propagation Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified request ID handling that could be unified.

### Current Pattern

```rust
fn get_request_id(headers: &HeaderMap) -> Option<String> {
    headers.get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn with_request_id<B>(mut req: Request<B>, id: String) -> Request<B> {
    req.headers_mut().insert("X-Request-ID", HeaderValue::from_str(&id).unwrap());
    req
}
```

### Recommendation

Extract to `phenotype-request-context`:
```rust
pub struct RequestContext {
    pub request_id: Ulid,
    pub trace_id: TraceId,
}

impl RequestContext {
    pub fn current() -> Option<ContextGuard<Self>>;
}
```

---

## 2026-03-29 - Pagination Logic Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P2

### Summary

Identified pagination patterns that could be shared.

### Repeated Pattern

```rust
pub struct Pagination {
    pub offset: u64,
    pub limit: u64,
}

async fn paginate_query<T>(
    query: &str,
    pagination: &Pagination,
) -> Result<Vec<T>, Error> {
    let offset = pagination.offset;
    let limit = pagination.limit;
    // execute with LIMIT/OFFSET
}
```

### Recommendation

Extract to `phenotype-pagination`:
```rust
pub trait PaginatedQuery<T> {
    async fn execute(&self, page: Page) -> Result<PagedResult<T>, Error>;
}

pub struct Page {
    pub number: u64,
    pub size: u64,
}
```

---

## 2026-03-29 - File Path Handling Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified path handling that could be standardized.

### Repeated Pattern

```rust
fn project_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agileplus")
}
```

### Recommendation

Extract to `phenotype-paths`:
```rust
pub mod paths {
    pub fn project_root() -> PathBuf;
    pub fn config_dir() -> PathBuf;
    pub fn cache_dir() -> PathBuf;
    pub fn data_dir() -> PathBuf;
}
```

---

## 2026-03-29 - Environment Variable Parsing Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** pending
**Priority:** P3

### Summary

Identified repeated environment variable parsing.

### Repeated Pattern

```rust
fn get_env<T: FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok()?.parse().ok()
}

fn require_env<T: FromStr>(key: &str) -> Result<T, Error> {
    get_env(key).ok_or_else(|| Error::MissingEnv(key.to_string()))
}
```

### Recommendation

Extract to `phenotype-env`:
```rust
pub fn get<T: FromStr>(key: &str) -> Option<T>;
pub fn require<T: FromStr + Display>(key: &str) -> Result<T, MissingEnvError>;
```
