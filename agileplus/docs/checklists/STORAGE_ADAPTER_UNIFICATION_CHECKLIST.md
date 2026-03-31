# Storage Adapter Unification — Implementation Checklist

**Project**: AgilePlus Storage Layer Unification
**Target**: Reduce code duplication across SQLite, Neo4j, and Plane.so adapters
**Target Savings**: ~932 LOC
**Status**: Planning phase (blocked on Neo4j/Plane completion)

---

## Prerequisites (BLOCKING)

These must be completed before unification work begins.

### Neo4j Adapter Completion (WP05)

- [ ] **T028**: agileplus-graph crate scaffolded
  - [ ] Cargo.toml created with dependencies
  - [ ] Module structure established (config, store, nodes, relationships, queries, health)
  - [ ] lib.rs exports correct modules

- [ ] **T029**: Neo4j connection management
  - [ ] Neo4j client initialized (neo4rs::Driver)
  - [ ] Connection pool configured (min/max connections)
  - [ ] Health check implemented
  - [ ] Error mapping defined

- [ ] **T030**: Node CRUD operations
  - [ ] Feature nodes (create, read, update, delete)
  - [ ] WorkPackage nodes (create, read, update, delete)
  - [ ] Uniqueness constraints enforced
  - [ ] Tests passing for all node operations

- [ ] **T031**: Relationship operations
  - [ ] Create relationships between nodes
  - [ ] Query relationships
  - [ ] Delete relationships
  - [ ] Tests passing

- [ ] **T032**: Graph queries
  - [ ] Dependency traversal queries working
  - [ ] Project-scoped queries working
  - [ ] Performance acceptable (< 100ms for typical queries)

- [ ] **T033**: Integration tests
  - [ ] All agileplus-graph tests passing
  - [ ] Health checks reliable
  - [ ] No panics or unwraps in code

### Plane.so Adapter Completion (WP06 + WP08)

- [ ] **WP06 Part A**: Sync client implementation
  - [ ] Plane API client fully functional
  - [ ] All CRUD operations implemented
  - [ ] Error handling aligned
  - [ ] Tests passing

- [ ] **WP06 Part B**: StoragePort trait implementation
  - [ ] Create feature → create Plane work item
  - [ ] Update feature → update Plane work item
  - [ ] List features → query Plane API
  - [ ] All StoragePort methods implemented
  - [ ] Tests passing

- [ ] **WP08 Part A**: Retry logic
  - [ ] Exponential backoff implemented
  - [ ] Rate limiting detection and retry
  - [ ] Timeout handling
  - [ ] Tests passing

- [ ] **WP08 Part B**: Integration tests
  - [ ] All agileplus-plane tests passing
  - [ ] Bidirectional sync working
  - [ ] No data loss in round-trip sync

---

## Phase 1: Create Storage Adapter Base Framework

**Duration**: 2-3 hours
**Deliverable**: `crates/storage-adapter-base/` with all traits and utilities

### Step 1.1: Create Crate Skeleton

- [ ] Create directory: `crates/storage-adapter-base/`
- [ ] Create Cargo.toml with correct metadata and dependencies
  - [ ] async-trait = "0.1"
  - [ ] serde = { version = "1.0", features = ["derive"] }
  - [ ] serde_json = "1.0"
  - [ ] thiserror = "1.0"
  - [ ] tokio = { version = "1", features = ["time"] }
  - [ ] parking_lot = "0.12"
- [ ] Create src/lib.rs with module declarations
  - [ ] mod error
  - [ ] mod pool
  - [ ] mod config
  - [ ] mod retry
  - [ ] mod health
- [ ] Update workspace Cargo.toml to include new crate

### Step 1.2: Implement Error Types (src/error.rs)

- [ ] Define StorageAdapterError enum
  - [ ] ConnectionError variant
  - [ ] QueryError variant
  - [ ] ConstraintViolation variant
  - [ ] NotFound variant
  - [ ] AuthenticationError variant
  - [ ] ConfigError variant
  - [ ] TimeoutError variant
  - [ ] RetryExhausted variant
  - [ ] Internal variant

- [ ] Implement From<StorageAdapterError> → DomainError
  - [ ] Conversion preserves error message
  - [ ] Handles all variants

- [ ] Define ErrorMapper trait
  - [ ] map_query_error() method
  - [ ] map_connection_error() method
  - [ ] is_transient() method

- [ ] Implement DefaultErrorMapper
  - [ ] Generic error mapping
  - [ ] Transient detection heuristics

- [ ] Write unit tests
  - [ ] Error variant creation
  - [ ] DomainError conversion
  - [ ] DefaultErrorMapper behavior
  - [ ] Transient detection

### Step 1.3: Implement Connection Pool (src/pool.rs)

- [ ] Define PoolStats struct
  - [ ] total_connections field
  - [ ] idle_connections field
  - [ ] active_connections field
  - [ ] total_checkouts field
  - [ ] total_checkins field
  - [ ] last_activity field
  - [ ] wait_time_ms field

- [ ] Define StorageConnectionPool trait
  - [ ] associated type Connection
  - [ ] acquire() method
  - [ ] acquire_with_timeout() method with default impl
  - [ ] release() method
  - [ ] stats() method
  - [ ] close() method
  - [ ] health_check() method

- [ ] Implement PoolMetrics helper
  - [ ] record_checkout() with atomic counter
  - [ ] record_checkin() with atomic counter
  - [ ] last_activity tracking

- [ ] Write unit tests
  - [ ] Metrics tracking
  - [ ] Stats calculation
  - [ ] Timeout handling

### Step 1.4: Implement Configuration (src/config.rs)

- [ ] Define StorageAdapterConfig trait
  - [ ] validate() method
  - [ ] adapter_name() method
  - [ ] backend_uri() method
  - [ ] base_config() method
  - [ ] diagnostics() method

- [ ] Implement BaseAdapterConfig struct
  - [ ] operation_timeout_secs field (default: 30)
  - [ ] max_retries field (default: 3)
  - [ ] retry_backoff_ms field (default: 100)
  - [ ] max_backoff_ms field (default: 5000)
  - [ ] health_check_on_init field (default: true)
  - [ ] health_check_interval_secs field (default: 60)
  - [ ] enable_query_logging field (default: false)

- [ ] Add helper methods to BaseAdapterConfig
  - [ ] operation_timeout() → Duration
  - [ ] health_check_interval() → Option<Duration>

- [ ] Add serde support
  - [ ] Derive Serialize, Deserialize
  - [ ] Default field values work with #[serde(default)]

- [ ] Write unit tests
  - [ ] Config creation
  - [ ] Validation logic
  - [ ] Default values
  - [ ] Serde round-trip

### Step 1.5: Implement Retry Policy (src/retry.rs)

- [ ] Define RetryPolicy trait
  - [ ] execute_with_retry() method
  - [ ] Supports generic operation returning Result<T, StorageAdapterError>

- [ ] Implement ExponentialBackoffRetry
  - [ ] max_retries field
  - [ ] initial_backoff_ms field
  - [ ] max_backoff_ms field
  - [ ] error_mapper field

- [ ] Implement backoff calculation
  - [ ] calculate_backoff(attempt: usize) → u64
  - [ ] Exponential formula: initial * 2^attempt
  - [ ] Capped by max_backoff_ms

- [ ] Implement execute_with_retry logic
  - [ ] Loop up to max_retries times
  - [ ] Check if error is transient
  - [ ] Sleep with backoff between retries
  - [ ] Return final error if all retries exhausted

- [ ] Implement NoRetryPolicy for testing
  - [ ] execute_with_retry() executes once, returns result

- [ ] Write unit tests
  - [ ] Successful operation on first try
  - [ ] Retry succeeds after transient failure
  - [ ] Retry exhausted on persistent error
  - [ ] Backoff calculation correct
  - [ ] No retry on non-transient errors

### Step 1.6: Implement Health Check (src/health.rs)

- [ ] Define HealthStatus struct
  - [ ] healthy: bool field
  - [ ] latency_ms: u128 field
  - [ ] message: String field
  - [ ] checked_at: u64 field (Unix timestamp)
  - [ ] adapter_version: String field

- [ ] Implement HealthStatus::new() constructor
  - [ ] Calculate checked_at from SystemTime
  - [ ] Accept healthy, latency_ms, message, adapter_version

- [ ] Define StorageAdapterHealthCheck trait
  - [ ] health_check() → HealthStatus
  - [ ] adapter_version() → &str

- [ ] Implement SimpleHealthCheck
  - [ ] Uses StorageConnectionPool::health_check()
  - [ ] Measures latency
  - [ ] Returns appropriate status

- [ ] Write unit tests
  - [ ] HealthStatus creation
  - [ ] Healthy adapter reporting
  - [ ] Unhealthy adapter reporting
  - [ ] Latency calculation

### Step 1.7: Integration & Documentation

- [ ] Write module-level documentation
  - [ ] Describe purpose of each module
  - [ ] Add examples of trait usage
  - [ ] Document error handling patterns

- [ ] Write README.md for base crate
  - [ ] Quick start guide
  - [ ] Architecture overview
  - [ ] Usage examples

- [ ] Run tests
  - [ ] `cargo test -p storage-adapter-base` passes
  - [ ] `cargo check -p storage-adapter-base` zero errors
  - [ ] `cargo clippy -p storage-adapter-base` clean

- [ ] Update workspace docs
  - [ ] Add to CRATES.md if exists
  - [ ] Link to detailed design docs

---

## Phase 2: Refactor SQLite Adapter

**Duration**: 1-2 hours
**Baseline**: ~600 LOC → Target: ~480 LOC (save ~120)

### Step 2.1: Create SQLite Config

- [ ] Create src/config.rs in agileplus-sqlite
- [ ] Implement StorageAdapterConfig for SQLite
  - [ ] db_path field
  - [ ] base: BaseAdapterConfig field
  - [ ] validate() checks db_path accessible
  - [ ] adapter_name() returns "sqlite"
  - [ ] diagnostics() returns schema version, migration count

- [ ] Create helper to load config from environment/file
  - [ ] SQLITE_DB_PATH env var
  - [ ] Optional config file support

- [ ] Write tests
  - [ ] Config creation with valid path
  - [ ] Config validation rejects missing path
  - [ ] Diagnostics output correct

### Step 2.2: Create SQLite Pool Wrapper

- [ ] Create src/pool.rs in agileplus-sqlite
- [ ] Implement StorageConnectionPool for existing Arc<Mutex<Connection>>
  - [ ] acquire() clones Arc (cheap)
  - [ ] release() is no-op for Arc
  - [ ] stats() returns metrics from PoolMetrics
  - [ ] close() closes underlying connection
  - [ ] health_check() runs PRAGMA integrity_check

- [ ] Wire PoolMetrics into existing lock() method
  - [ ] record_checkout() on acquire
  - [ ] record_checkin() on release

- [ ] Write tests
  - [ ] Pool acquire/release
  - [ ] Stats calculation
  - [ ] Health check passing
  - [ ] Close properly closes connection

### Step 2.3: Update SqliteStorageAdapter

- [ ] Update lib.rs to use new config/pool
  - [ ] Add config field
  - [ ] Add pool field
  - [ ] Keep existing conn field for backward compat (but don't use)

- [ ] Update ::new() to accept SqliteConfig
  - [ ] Call config.validate()
  - [ ] Create pool from config
  - [ ] If health_check_on_init, run health check

- [ ] Keep adapter methods the same (backward compatible)
  - [ ] StoragePort trait impl unchanged
  - [ ] All methods still work as before

- [ ] Wire up error mapper
  - [ ] Implement ErrorMapper for rusqlite errors
  - [ ] Create SqliteErrorMapper struct
  - [ ] map_query_error: CONSTRAINT_UNIQUE → ConstraintViolation
  - [ ] map_query_error: IOERR → ConnectionError
  - [ ] is_transient: IOERR, CANTOPEN → true

- [ ] Integrate retry policy
  - [ ] Create ExponentialBackoffRetry in ::new()
  - [ ] Wrap key operations (create, update, delete) with retry
  - [ ] Read operations don't need retry

- [ ] Write tests
  - [ ] Config-based initialization
  - [ ] Error mapping behavior
  - [ ] Retry logic with transient errors
  - [ ] Health checks

### Step 2.4: Migration & Cleanup

- [ ] Run existing test suite
  - [ ] `cargo test -p agileplus-sqlite` all pass
  - [ ] No regressions vs baseline

- [ ] Benchmark performance
  - [ ] Measure latency before/after
  - [ ] Ensure no >5% regression

- [ ] Clean up old code
  - [ ] Remove duplicate error handling
  - [ ] Remove old config logic

---

## Phase 3: Complete & Integrate Neo4j Adapter

**Duration**: 3-4 hours
**Scope**: Finish Neo4j implementation + base framework integration

### Step 3.1: Complete Neo4j Client Backend

- [ ] Implement Neo4j connection management
  - [ ] neo4rs::Driver initialization
  - [ ] Connection pooling configuration
  - [ ] Authentication (URI with credentials)
  - [ ] TLS setup if needed

- [ ] Implement GraphBackend for Neo4j
  - [ ] run_cypher() executes mutations
  - [ ] query_cypher() executes reads
  - [ ] health_check() via neo4j.cypher("RETURN 1")

- [ ] Implement node operations
  - [ ] Feature nodes: create, read, update, delete
  - [ ] WorkPackage nodes: create, read, update, delete
  - [ ] Agent/Label/Project nodes
  - [ ] Uniqueness constraints on id fields

- [ ] Implement relationship operations
  - [ ] Create relationships
  - [ ] Query relationships
  - [ ] Delete relationships by type

- [ ] Write error mapper
  - [ ] Neo4j error → StorageAdapterError
  - [ ] Transient: network errors, timeouts
  - [ ] Permanent: constraint violations, auth errors

- [ ] Write tests
  - [ ] Node CRUD working
  - [ ] Relationships working
  - [ ] Error handling correct
  - [ ] Health check working

### Step 3.2: Create Neo4j Config

- [ ] Create src/config.rs
- [ ] Implement StorageAdapterConfig
  - [ ] uri: String (neo4j://... or neo4j+s://...)
  - [ ] username: String
  - [ ] password: String
  - [ ] base: BaseAdapterConfig
  - [ ] max_connections: usize (default: 10)

- [ ] Implement validate()
  - [ ] URI format validation
  - [ ] Credentials non-empty
  - [ ] Test connection on init if health_check_on_init

- [ ] Write tests
  - [ ] Config with valid URI
  - [ ] Config validation rejects invalid URI
  - [ ] Credentials handling

### Step 3.3: Create Neo4j Pool Wrapper

- [ ] Create src/pool.rs
- [ ] Implement StorageConnectionPool
  - [ ] Wrap neo4rs::Driver
  - [ ] acquire() gets connection from driver
  - [ ] release() closes connection gracefully
  - [ ] stats() returns driver stats
  - [ ] close() closes driver
  - [ ] health_check() via health() method

- [ ] Wire metrics
  - [ ] Track checkout/checkin counts
  - [ ] Track pool exhaustion events

- [ ] Write tests
  - [ ] Pool acquire/release
  - [ ] Connection reuse
  - [ ] Pool closure

### Step 3.4: Implement StoragePort for Neo4j

- [ ] Create src/storage_port.rs
- [ ] Implement full StoragePort trait
  - [ ] Feature CRUD methods
  - [ ] WorkPackage CRUD methods
  - [ ] All query methods
  - [ ] Delegate to nodes/relationships/queries modules

- [ ] Wire retry policy
  - [ ] Wrap operations with ExponentialBackoffRetry
  - [ ] Exclude non-idempotent mutations from retry (or use idempotency keys)

- [ ] Wire error mapping
  - [ ] All errors go through NeoErrorMapper

- [ ] Write comprehensive tests
  - [ ] Feature create/read/update/delete
  - [ ] WorkPackage operations
  - [ ] Error handling
  - [ ] Retry behavior

### Step 3.5: Integration Tests

- [ ] Test against real Neo4j instance (Docker)
  - [ ] Full CRUD cycle
  - [ ] Concurrent operations
  - [ ] Connection pool exhaustion
  - [ ] Error scenarios

- [ ] Run full test suite
  - [ ] `cargo test -p agileplus-graph` all pass

---

## Phase 4: Complete & Integrate Plane.so Adapter

**Duration**: 3-4 hours
**Scope**: Finish Plane.so implementation + base framework integration

### Step 4.1: Create PlaneStorageAdapter

- [ ] Create src/adapter.rs
- [ ] Define PlaneStorageAdapter struct
  - [ ] config: Arc<PlaneConfig>
  - [ ] client: reqwest::Client
  - [ ] pool: Arc<PlanePool>
  - [ ] retry_policy: Arc<dyn RetryPolicy>
  - [ ] error_mapper: Box<dyn ErrorMapper>

- [ ] Implement ::new() constructor
  - [ ] Validate config
  - [ ] Create HTTP client
  - [ ] Initialize pool
  - [ ] Create retry policy
  - [ ] Run health check if configured

- [ ] Implement ::from_env() helper
  - [ ] Read PLANE_API_KEY env var
  - [ ] Read PLANE_WORKSPACE_ID env var
  - [ ] Read PLANE_BASE_URL env var (optional, default to plane.so)

### Step 4.2: Create Plane Config

- [ ] Create src/config.rs
- [ ] Implement StorageAdapterConfig
  - [ ] api_key: String (sensitive, don't debug)
  - [ ] base_url: String (default: "https://api.plane.so")
  - [ ] workspace_id: String
  - [ ] project_id: String (or use workspace_id?)
  - [ ] base: BaseAdapterConfig
  - [ ] max_concurrent_requests: usize (default: 10)

- [ ] Implement validate()
  - [ ] api_key non-empty
  - [ ] base_url is valid URL
  - [ ] workspace_id non-empty
  - [ ] Test connection if health_check_on_init

- [ ] Implement diagnostics()
  - [ ] Hide api_key
  - [ ] Show base_url, workspace_id

- [ ] Write tests
  - [ ] Config creation
  - [ ] Validation

### Step 4.3: Create Plane Pool Wrapper

- [ ] Create src/pool.rs
- [ ] Implement StorageConnectionPool
  - [ ] Wrap reqwest::Client
  - [ ] acquire() returns client (stateless)
  - [ ] release() is no-op
  - [ ] stats() returns request counts
  - [ ] close() is no-op
  - [ ] health_check() via Plane health endpoint

- [ ] Track metrics
  - [ ] Total requests
  - [ ] Active requests
  - [ ] Last activity timestamp

- [ ] Write tests
  - [ ] Pool acquire/release
  - [ ] Stats tracking

### Step 4.4: Implement StoragePort for Plane

- [ ] Create src/storage_port.rs
- [ ] Implement full StoragePort trait
  - [ ] Feature CRUD → Work Item CRUD in Plane
  - [ ] WorkPackage CRUD → ? (may map to issues)
  - [ ] Cycles CRUD → ? (need to understand Plane model)
  - [ ] Modules CRUD → ? (may not exist in Plane)
  - [ ] All query methods → Plane API queries

- [ ] Handle mapping challenges
  - [ ] AgilePlus entities may not map 1:1 to Plane
  - [ ] May need synthetic/virtual entities
  - [ ] Document mapping in code comments

- [ ] Wire retry policy
  - [ ] Wrap all HTTP operations
  - [ ] Special handling for rate limits (429 → retry)
  - [ ] Exponential backoff for transient errors

- [ ] Wire error mapping
  - [ ] HTTP status → StorageAdapterError
  - [ ] 401/403 → AuthenticationError
  - [ ] 404 → NotFound
  - [ ] 409 → ConstraintViolation (duplicate)
  - [ ] 429 → ConnectionError (rate limit)
  - [ ] 5xx → transient

- [ ] Write comprehensive tests
  - [ ] Feature create/read/update/delete
  - [ ] WorkPackage operations
  - [ ] Error handling
  - [ ] Retry behavior
  - [ ] Rate limit handling

### Step 4.5: Integration Tests

- [ ] Test against Plane sandbox/staging
  - [ ] Full CRUD cycle
  - [ ] Concurrent operations
  - [ ] Rate limit handling
  - [ ] Error scenarios

- [ ] Test bidirectional sync
  - [ ] Push changes to Plane
  - [ ] Fetch changes from Plane
  - [ ] Conflict resolution

- [ ] Run full test suite
  - [ ] `cargo test -p agileplus-plane` all pass

---

## Phase 5: Cross-Adapter Integration & Testing

**Duration**: 2-3 hours
**Scope**: Verify unified behavior, measure savings, document patterns

### Step 5.1: Create Unified Test Suite

- [ ] Create tests/storage_adapter_compat.rs (in a shared test crate?)
  - [ ] Tests that all three adapters implement StoragePort correctly
  - [ ] Same test suite runs against all three backends
  - [ ] Feature CRUD tests
  - [ ] WorkPackage CRUD tests
  - [ ] Query tests
  - [ ] Error handling tests
  - [ ] Retry behavior tests
  - [ ] Health check tests

- [ ] Parameterize tests
  - [ ] Create test fixtures for each adapter
  - [ ] Use shared test logic
  - [ ] Document expected behavior for each adapter

- [ ] Run tests
  - [ ] All three adapters pass all tests
  - [ ] No adapter-specific failures

### Step 5.2: Measure Code Savings

- [ ] Count LOC before/after
  - [ ] `wc -l crates/agileplus-sqlite/src/**/*.rs`
  - [ ] `wc -l crates/agileplus-graph/src/**/*.rs`
  - [ ] `wc -l crates/agileplus-plane/src/**/*.rs`
  - [ ] `wc -l crates/storage-adapter-base/src/**/*.rs`

- [ ] Calculate net reduction
  - [ ] Total LOC before: sum of three adapters
  - [ ] Total LOC after: sum of three adapters + base
  - [ ] Reduction percentage
  - [ ] Target: ~932 LOC savings

- [ ] Breakdown by category
  - [ ] Connection pooling reduction
  - [ ] Error handling reduction
  - [ ] Retry logic reduction
  - [ ] Config initialization reduction

### Step 5.3: Performance Benchmarking

- [ ] Create benches/storage_adapter_bench.rs
  - [ ] Benchmark feature create operation
  - [ ] Benchmark feature query operation
  - [ ] Benchmark with/without retries
  - [ ] Measure latency for each adapter

- [ ] Compare before/after
  - [ ] Ensure no >5% regression
  - [ ] Flag any significant differences

- [ ] Document results
  - [ ] Add to docs/reports/STORAGE_ADAPTER_UNIFICATION_COMPLETE.md

### Step 5.4: Documentation

- [ ] Update architecture docs
  - [ ] Add diagrams showing unified architecture
  - [ ] Document trait relationships

- [ ] Write adapter selection guide
  - [ ] When to use SQLite (single machine, embedded)
  - [ ] When to use Neo4j (graph queries, relationships)
  - [ ] When to use Plane.so (sync with external workspace)

- [ ] Update CLAUDE.md
  - [ ] Document new adapter patterns
  - [ ] Add examples of implementing new storage backends

- [ ] Write migration guide
  - [ ] How to migrate from old adapter pattern
  - [ ] How to implement new adapters using base framework

### Step 5.5: Final Verification

- [ ] All adapters pass tests
  - [ ] `cargo test` in all three adapter crates
  - [ ] Cross-adapter compatibility tests
  - [ ] No regressions vs baseline

- [ ] Code review
  - [ ] All new code reviewed
  - [ ] Design decisions documented
  - [ ] Edge cases handled

- [ ] Clean up
  - [ ] Remove old dead code
  - [ ] Update comments for new patterns
  - [ ] Ensure no warnings from clippy

- [ ] Commit changes
  - [ ] One commit per logical unit
  - [ ] Clear commit messages referencing design docs
  - [ ] Include benchmark results in commit

---

## Success Criteria (Overall)

- [ ] All three adapters implement StoragePort correctly
- [ ] All three adapters pass unified test suite
- [ ] Code reduced by ≥920 LOC
- [ ] No performance regression (within 5%)
- [ ] All existing tests pass
- [ ] New integration tests added
- [ ] Documentation updated
- [ ] Design patterns documented in code

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Breaking SQLite | Backward-compatible wrapper, comprehensive tests |
| Neo4j incomplete | Phase 1 cannot start until WP05 fully done |
| Plane.so incomplete | Phase 1 cannot start until WP06+WP08 fully done |
| Trait indirection overhead | Benchmark carefully, inline where needed |
| Error mapping missed edges | Comprehensive error scenario tests |
| Retry logic too aggressive | Conservative defaults, configurable |

---

## Timeline Estimate

| Phase | Duration | Status |
|-------|----------|--------|
| Prerequisites | TBD | 🚫 Blocking (WP05, WP06, WP08 not done) |
| Phase 1: Base framework | 2-3 hours | ⏳ Ready when prerequisites met |
| Phase 2: SQLite refactor | 1-2 hours | ⏳ Depends on Phase 1 |
| Phase 3: Neo4j integration | 3-4 hours | ⏳ Depends on Phase 1 + WP05 completion |
| Phase 4: Plane.so integration | 3-4 hours | ⏳ Depends on Phase 1 + WP06/WP08 completion |
| Phase 5: Testing & validation | 2-3 hours | ⏳ Depends on Phases 2-4 |
| **TOTAL** | **~14-18 hours** | ⏳ |

**Wall-clock estimate**: 2-3 days of focused work (once prerequisites complete)

---

## Sign-Off

- [ ] Architect approval: _________________
- [ ] Lead engineer approval: _________________
- [ ] QA lead sign-off: _________________
- [ ] Date completed: _________________
