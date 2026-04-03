# Retrospective Command Refactor - Migration Phases

**Document Type:** Implementation Roadmap
**Scope:** 3 Phases | **Total Duration:** 5-7 days | **Parallel Work:** 2-3 subagents per phase

---

## Overview: Three-Phase Migration

```
PHASE 1: PORT DEFINITION        PHASE 2: SERVICE IMPL        PHASE 3: CLI CUTOVER
2 days | 400 LOC                3 days | 800 LOC              2 days | 150 LOC
═════════════════════════════════════════════════════════════════════════════════

1. Traits defined            1. Services implemented       1. CLI refactored
2. Models created            2. Adapters built             2. Wiring complete
3. Error types defined       3. Tests written             3. Old code removed
4. Contract tests pass       4. Integration tests pass     4. Migration docs
5. Zero breaking changes     5. Backwards compatible       5. Success validation

█████████████ → █████████████ → █████████████
```

---

## Phase 1: Port & Contract Definition (2 Days)

### Objective
Establish the boundary contracts without touching existing code.

### Work Packages

#### WP1.1: Inbound Port Definition (4 hours)
**Deliverable:** `crates/phenotype-contracts/src/ports/inbound/retrospective.rs` (200 LOC)

**Tasks:**
1. Create `RetrospectiveService` trait with 5 core methods
2. Define `RetrospectiveConfig` struct
3. Define `ExportFormat` enum + helpers
4. Define `RetrospectiveError` error type with 9 variants
5. Add doc comments for all public items

**Acceptance Criteria:**
```bash
cargo build --package phenotype-contracts
cargo test --doc -p phenotype-contracts
```

**Testing:**
- Doc tests for error types
- No warnings on build

---

#### WP1.2: Outbound Ports Definition (4 hours)
**Deliverable:**
- `crates/phenotype-contracts/src/ports/outbound/repository.rs` (50 LOC)
- `crates/phenotype-contracts/src/ports/outbound/cache.rs` (40 LOC)
- `crates/phenotype-contracts/src/ports/outbound/event_bus.rs` (60 LOC)

**Tasks:**
1. Define `RetrospectiveRepository` trait (6 methods)
2. Define `RepositoryError` error type
3. Define `RetrospectiveCache` trait (5 methods)
4. Define `CacheError` error type
5. Define `RetrospectiveEventBus` trait (2 methods)
6. Define `RetrospectiveEvent` enum (5 variants)
7. Define `EventBusError` error type

**Acceptance Criteria:**
```bash
cargo build --package phenotype-contracts
cargo clippy --package phenotype-contracts -- -D warnings
```

**Testing:**
- No Clippy warnings
- All traits are `Send + Sync`

---

#### WP1.3: Domain Models (3 hours)
**Deliverable:** `crates/phenotype-contracts/src/models/retrospective.rs` (100 LOC)

**Tasks:**
1. Create `Retrospective` aggregate root
2. Create `TimeRange` value object
3. Create `Metric`, `Trend`, `Insight` entities
4. Create `AggregateResult` and `RetrospectiveMetadata`
5. Add `Serialize`/`Deserialize` derives
6. Implement utility methods (e.g., `TimeRange::parse()`)

**Acceptance Criteria:**
```bash
cargo build --package phenotype-contracts
serde_json::to_string(&retrospective).is_ok()
```

**Testing:**
- Serde serialization round-trip tests
- TimeRange validation tests

---

#### WP1.4: Module Organization & Testing (3 hours)
**Deliverable:**
- Updated `crates/phenotype-contracts/src/lib.rs`
- Contract tests in `crates/phenotype-contracts/src/tests/`

**Tasks:**
1. Export all ports and models from lib.rs
2. Create contract tests (mock implementations)
3. Create test fixtures for common scenarios
4. Document the port interface contract

**Acceptance Criteria:**
```bash
cargo test --package phenotype-contracts --lib
cargo test --package phenotype-contracts --doc
```

**Testing:**
- All doc tests pass
- All unit tests pass
- No dead code warnings

---

### Phase 1 Success Metrics

| Metric | Target | Verification |
|--------|--------|--------------|
| Code Compiles | ✓ | `cargo build --release` |
| No Warnings | ✓ | `cargo clippy` with `-D warnings` |
| Tests Pass | ✓ | `cargo test --package phenotype-contracts` |
| Documentation | 100% | `cargo doc --open` inspect all items |
| Trait Soundness | ✓ | All `Send + Sync + 'static` |
| No Breaking Changes | ✓ | Existing code unaffected |

---

## Phase 2: Service Implementation & Adapters (3 Days)

### Objective
Implement the business logic and concrete adapters.

### Work Packages

#### WP2.1: RetrospectiveServiceImpl (2 days)
**Deliverable:** `crates/phenotype-core/src/services/retrospective_service.rs` (300 LOC)

**Tasks:**
1. Create `RetrospectiveServiceImpl` struct
2. Implement `RetrospectiveService` trait methods:
   - `generate()` - Build + persist + publish event
   - `compute_aggregates()` - Query + cache + publish event
   - `export()` - Format conversion + publish event
   - `get_metadata()` - Lightweight lookup
   - `list_retrospectives()` - Pagination
   - `delete()` - Cascade delete + publish event
3. Implement pure business logic helper methods:
   - `build_retrospective()` - Compose from parts
   - `aggregate_metrics()` - Pure aggregation
   - `extract_metrics()` - Data collection
   - `compute_trends()` - Trend detection
   - `generate_insights()` - AI insights (stub for MVP)
4. Implement export formatters (pure functions):
   - `export_json()` - JSON serialization
   - `export_markdown()` - Markdown generation
   - `export_csv()` - CSV generation
   - `export_pdf()` - PDF generation (stub for MVP)

**Key Design Points:**
- Separate I/O from pure logic
- All public methods are async
- Ports used exclusively for I/O
- Error handling with proper context
- Event publishing on state changes

**Acceptance Criteria:**
```bash
cargo build --package phenotype-core
cargo clippy --package phenotype-core -- -D warnings
cargo test --package phenotype-core
```

**Testing:** (WP2.5)
- Unit tests for pure logic (50+ tests)
- Integration tests with mock ports (30+ tests)
- Error path coverage (15+ tests)

---

#### WP2.2: SQLite Repository Adapter (1 day)
**Deliverable:** `crates/phenotype-sqlite-adapter/src/retrospective_repository.rs` (150 LOC)

**Tasks:**
1. Implement `RetrospectiveRepository` trait for SQLite
2. Create database schema (migrations)
3. Implement `create()` - INSERT with generated ID
4. Implement `get()` - SELECT by ID
5. Implement `list_by_range()` - SELECT with date filter
6. Implement `update()` - UPDATE with timestamp
7. Implement `delete()` - DELETE cascade
8. Implement `count_by_range()` - COUNT for pagination

**Database Schema:**
```sql
CREATE TABLE retrospectives (
    id TEXT PRIMARY KEY,
    range_from TIMESTAMP NOT NULL,
    range_to TIMESTAMP NOT NULL,
    metrics_json TEXT NOT NULL,
    trends_json TEXT NOT NULL,
    insights_json TEXT NOT NULL,
    author TEXT,
    tags_json TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    INDEX idx_created_at (created_at),
    INDEX idx_range (range_from, range_to)
);
```

**Acceptance Criteria:**
```bash
cargo build --package phenotype-sqlite-adapter
cargo test --package phenotype-sqlite-adapter
```

**Testing:**
- CRUD operations (8+ tests)
- Index usage verification
- Constraint enforcement (3+ tests)

---

#### WP2.3: Redis Cache Adapter (6 hours)
**Deliverable:** `crates/phenotype-redis-adapter/src/retrospective_cache.rs` (100 LOC)

**Tasks:**
1. Implement `RetrospectiveCache` trait for Redis
2. Implement `cache_aggregates()` - SET with TTL
3. Implement `get_aggregates()` - GET with deserialize
4. Implement `invalidate()` - DEL by pattern
5. Implement `clear_all()` - FLUSHDB (admin only)
6. Implement `stats()` - INFO command parsing
7. Add error handling for connection failures

**Key Design Points:**
- Fail-open for cache misses (return Ok(None))
- Fail-closed for cache writes (return Err)
- Exponential backoff on connection retry
- TTL configurable per operation
- Stats collected for observability

**Acceptance Criteria:**
```bash
cargo build --package phenotype-redis-adapter
cargo test --package phenotype-redis-adapter
```

**Testing:**
- Cache hit/miss scenarios (6+ tests)
- TTL expiration (2+ tests)
- Connection failure handling (4+ tests)

---

#### WP2.4: Event Bus Adapter (6 hours)
**Deliverable:** `crates/phenotype-event-bus-adapter/src/retrospective_events.rs` (120 LOC)

**Tasks:**
1. Implement `RetrospectiveEventBus` trait
2. Choose backend (start with SQLite append-only log, optional Kafka)
3. Implement `publish()` - Persist + queue for subscribers
4. Implement `subscribe()` - Return event stream
5. Add retry logic (exponential backoff, 3 attempts)
6. Add event versioning support
7. Add tracing/observability

**Event Store Schema:**
```sql
CREATE TABLE retrospective_events (
    id TEXT PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSON NOT NULL,
    created_at TIMESTAMP NOT NULL,
    version INTEGER NOT NULL,
    INDEX idx_aggregate_id (aggregate_id),
    INDEX idx_created_at (created_at)
);
```

**Acceptance Criteria:**
```bash
cargo build --package phenotype-event-bus-adapter
cargo test --package phenotype-event-bus-adapter
```

**Testing:**
- Event persistence (4+ tests)
- Event stream ordering (3+ tests)
- Retry logic (3+ tests)

---

#### WP2.5: Comprehensive Testing (1 day)
**Deliverable:**
- `crates/phenotype-core/src/services/tests/` (200 LOC)
- Integration tests
- Test fixtures

**Test Categories:**

**Unit Tests (Pure Logic)** - 50+ tests
```rust
#[test]
fn test_aggregate_metrics_empty() { }
#[test]
fn test_aggregate_metrics_single() { }
#[test]
fn test_aggregate_metrics_multiple() { }
#[test]
fn test_compute_trends_up() { }
#[test]
fn test_compute_trends_down() { }
#[test]
fn test_time_range_validation_invalid() { }
#[test]
fn test_export_json_roundtrip() { }
// ... many more
```

**Integration Tests** - 30+ tests
```rust
#[tokio::test]
async fn test_generate_persists_to_repository() { }
#[tokio::test]
async fn test_generate_publishes_event() { }
#[tokio::test]
async fn test_aggregates_cached() { }
#[tokio::test]
async fn test_export_all_formats() { }
// ... many more
```

**Error Path Tests** - 15+ tests
```rust
#[tokio::test]
async fn test_generate_repository_failure() { }
#[tokio::test]
async fn test_compute_aggregates_not_found() { }
#[tokio::test]
async fn test_export_invalid_format() { }
// ... many more
```

**Acceptance Criteria:**
```bash
cargo test --package phenotype-core --lib -- --test-threads=1
cargo tarpaulin -p phenotype-core --out Html
# Check coverage >= 80%
```

**Testing:**
- 95+ tests written
- 80%+ code coverage
- All error paths covered
- No flaky tests

---

### Phase 2 Success Metrics

| Metric | Target | Verification |
|--------|--------|--------------|
| Code Compiles | ✓ | `cargo build --release` |
| No Warnings | ✓ | `cargo clippy` with `-D warnings` |
| Tests Pass | ✓ | `cargo test --all` |
| Coverage | ≥80% | `cargo tarpaulin` |
| Performance | <500ms | Aggregation benchmark |
| Thread-safe | ✓ | Checked by `Send + Sync` bounds |
| Backwards Compatible | ✓ | No changes to public API |

---

## Phase 3: CLI Refactor & Cutover (2 Days)

### Objective
Replace old 630 LOC monolith with thin 150 LOC handler.

### Work Packages

#### WP3.1: CLI Command Refactor (1 day)
**Deliverable:** `crates/agileplus-cli/src/commands/retrospective.rs` (150 LOC)

**Tasks:**
1. Create `RetrospectiveCmd` struct with subcommands
2. Implement 4 subcommands:
   - `gen` - Generate retrospective
   - `aggregate` - Compute aggregates
   - `export` - Export to file
   - `info` - Show metadata
3. Implement error handling (map service errors → CLI errors)
4. Implement output formatting (JSON, Markdown, Table)
5. Add command help text and examples

**New Handler Structure:**
```rust
pub async fn execute(self, ctx: &CliContext) -> Result<()> {
    let service = ctx.retrospective_service();

    match self.action {
        RetrospectiveAction::Gen { ... } =>
            self.cmd_generate(service, ...).await,
        RetrospectiveAction::Aggregate { ... } =>
            self.cmd_aggregate(service, ...).await,
        RetrospectiveAction::Export { ... } =>
            self.cmd_export(service, ...).await,
        RetrospectiveAction::Info { ... } =>
            self.cmd_info(service, ...).await,
    }
}
```

**Acceptance Criteria:**
```bash
cargo build --package agileplus-cli
cargo clippy --package agileplus-cli -- -D warnings
agileplus retrospective gen --from 2026-03-01 --to 2026-03-31
```

**Testing:**
- CLI integration tests (10+ tests)
- Help text validation
- Argument parsing tests

---

#### WP3.2: Context Wiring (6 hours)
**Deliverable:** Updated `crates/agileplus-cli/src/context.rs`

**Tasks:**
1. Add service creation to `CliContext`
2. Wire service dependencies (repo, cache, event bus)
3. Add service builder for testing
4. Update documentation

**Code:**
```rust
impl CliContext {
    pub fn retrospective_service(&self) -> Arc<dyn RetrospectiveService> {
        let repo = Arc::new(SqliteRepository::new(&self.db_pool));
        let cache = Arc::new(RedisCache::new(&self.redis_client));
        let event_bus = Arc::new(EventBusAdapter::new(&self.event_db));

        Arc::new(RetrospectiveServiceImpl::new(repo, cache, event_bus))
    }
}
```

**Acceptance Criteria:**
```bash
cargo build --package agileplus-cli
cargo test --package agileplus-cli
```

**Testing:**
- Context initialization (3+ tests)
- Service wiring validation

---

#### WP3.3: Old Code Removal (4 hours)
**Deliverable:** Clean repo with old code removed

**Tasks:**
1. Delete old `retrospective.rs` (630 LOC)
2. Update imports in dependent modules
3. Remove old test files
4. Run full test suite to verify no regressions
5. Update CHANGELOG

**Verification:**
```bash
git diff HEAD~1 -- crates/agileplus-cli/src/commands/
# Should show: -630 LOC old code, +150 LOC new code
```

**Testing:**
- Full CLI test suite passes
- No dead imports
- No compilation errors

---

#### WP3.4: Migration Documentation (4 hours)
**Deliverable:**
- `docs/reference/RETROSPECTIVE_MIGRATION_GUIDE.md`
- Code comments for future maintainers
- Updated API documentation

**Content:**
1. Before/After architecture diagrams
2. File layout changes
3. Testing migration guide
4. Troubleshooting section
5. Performance comparison

**Acceptance Criteria:**
```bash
cargo doc --open
# Verify all public items documented
grep -r "TODO\|FIXME" crates/agileplus-cli/src/commands/
# Should be empty
```

---

### Phase 3 Success Metrics

| Metric | Target | Verification |
|--------|--------|--------------|
| CLI Tests Pass | ✓ | `cargo test --package agileplus-cli` |
| Functional Parity | ✓ | All original features work |
| Code Size | 150 LOC | `wc -l retrospective.rs` |
| No Regressions | ✓ | Full test suite passes |
| Documentation | Complete | `cargo doc` + guide |
| Performance | Same/Better | Benchmark comparison |

---

## Cross-Cutting Concerns

### Dependency Injection Setup

All three phases require proper DI. Create a `ServiceProvider` trait:

```rust
// File: crates/phenotype-core/src/service_provider.rs

pub trait ServiceProvider {
    fn retrospective_service(&self) -> Arc<dyn RetrospectiveService>;
    fn plan_service(&self) -> Arc<dyn PlanService>;
    fn review_service(&self) -> Arc<dyn ReviewService>;
}

pub struct DefaultServiceProvider {
    repository_pool: Arc<dyn RepositoryPool>,
    cache_client: Arc<dyn CacheClient>,
    event_bus: Arc<dyn EventBus>,
}
```

### Error Handling Convention

All services return `Result<T, ServiceError>` with consistent structure:

```rust
#[derive(thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Event bus error: {0}")]
    EventBus(#[from] EventBusError),
}
```

### Testing Infrastructure

All phases use shared test infrastructure:

```rust
// Test fixture factory
pub struct TestContext {
    repository: Arc<MockRepository>,
    cache: Arc<MockCache>,
    event_bus: Arc<MockEventBus>,
}

impl TestContext {
    pub fn new() -> Self { }
    pub fn with_failure_mode(failure: FailureMode) -> Self { }
    pub fn build_service(&self) -> Arc<dyn RetrospectiveService> { }
}
```

---

## Parallel Execution Strategy

**Phase 1:** Single agent (serialized, 2 days)
- Must be sequential (later work depends on traits)

**Phase 2:** 3 agents in parallel (3 days)
- Agent A: Service implementation
- Agent B: Repository + Cache adapters
- Agent C: Event bus adapter + Testing

**Phase 3:** 2 agents in parallel (2 days)
- Agent A: CLI refactor + context wiring
- Agent B: Migration docs + old code removal

**Total wall-clock time:** ~7 days (2 + 3 + 2)
**Total effort:** ~15-20 tool calls per phase × 3 phases

---

## Rollback Plan

If issues arise, rollback points are:

**After Phase 1:** No production impact (traits only)
**After Phase 2:** No production impact (new code, old still works)
**After Phase 3:** Production cutover (old code deleted)

To rollback Phase 3:
1. Revert CLI handler commit
2. Keep services as internal API
3. No data migration needed (same DB schema)

---

## Success Criteria Summary

### Phase 1: Contracts Defined
- Traits compile without errors
- All doc tests pass
- No warnings with Clippy
- Models are serializable
- No breaking changes to existing code

### Phase 2: Services Implemented
- Service impl passes 80%+ tests
- All adapters functional
- Integration tests pass
- Performance benchmarks established
- Event sourcing works end-to-end

### Phase 3: CLI Operational
- New handler < 150 LOC
- All original functionality preserved
- Performance same or better
- Full test coverage
- Documentation complete
- Old code removed

---

## Next: Roadmap for plan.rs and review.rs

Once Phase 3 is complete, apply the same pattern:

```
RETROSPECTIVE      PLAN               REVIEW
✓ Traits defined   ─► Traits defined  ─► Traits defined
✓ Service impl     ─► Service impl    ─► Service impl
✓ CLI refactored   ─► CLI refactored  ─► CLI refactored

Result: 3× thin handlers + 3× reusable services + shared ports
Total LOC: 450 (3 handlers) + 1500 (3 services) = 1950 vs ~1800 monolithic
Architecture: Fully modular, testable, reusable
```
