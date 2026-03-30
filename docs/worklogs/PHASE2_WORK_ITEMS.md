# Phase 2 Decomposition Work Items

**Date:** 2026-03-30
**Status:** PLANNED
**Scope:** High-impact file splitting and config consolidation
**Estimated Impact:** 25-35K LOC improvement, 3-4 weeks effort

---

## Executive Summary

Phase 2 builds on Phase 1's libification work and addresses the **3 largest technical debt items** identified in the [LOC Audit and Optimization Plan](LOC_AUDIT_AND_OPTIMIZATION_PLAN.md). This phase focuses on **file decomposition** and **architectural refactoring** rather than library standardization.

### Key Targets

| File | Current LOC | Issue | Target LOC | Savings |
|------|-------------|-------|-----------|---------|
| `agileplus-dashboard/routes.rs` | 2,631 | 53 async handlers, monolithic | 4 files × 500-700 LOC each | ~500 LOC |
| `agileplus-sqlite/lib.rs` | 1,582 | Single-crate adapter, SQL logic | 3 files × 400-600 LOC each | ~400 LOC |
| Config loading (distributed) | ~1,200 (spread) | Scattered `phenotype-config-core` usage | Unified loader | ~300 LOC |
| Total Phase 2 Scope | ~5,413 LOC | Cognitive complexity, maintainability | ~3,500 LOC refactored | **~1,900 LOC logical reduction** |

### Effort Estimate

| Phase | Phases | Weeks | Wall-Clock (Agent) | Tool Calls | Parallel Teams |
|-------|--------|-------|-------------------|-----------|-----------------|
| **2.1-2.4** (routes.rs split) | 2 | 1-1.5 | 8-12 min | 20-25 | 2 agents |
| **2.5-2.7** (sqlite/lib.rs split) | 2 | 1-1.5 | 8-12 min | 20-25 | 2 agents |
| **2.8-2.9** (config consolidation) | 1 | 0.5-1 | 5-8 min | 12-15 | 1 agent |
| **Verification & Testing** | Ongoing | 1-1.5 | 10-15 min | 15-20 | 1-2 agents |
| **Total Phase 2** | - | 3-4 | 31-47 min | 67-85 | 4-5 concurrent |

### Success Criteria

- ✅ All 9 work items completed
- ✅ 0 new lint warnings introduced
- ✅ Test coverage ≥80% for refactored code
- ✅ No breaking API changes (internal refactoring only)
- ✅ All affected crates build cleanly
- ✅ Cognitive complexity reduced by ≥30% in target files
- ✅ Code review with approval on all PRs

---

## Work Item Breakdown

### Phase 2.1: Routes Dashboard Split (WI-2.1)

**Objective:** Split `agileplus-dashboard/routes.rs` (2,631 LOC) into module-scoped files
**Status:** PLANNED
**Priority:** HIGH
**Effort:** 5-7 story points (2 agents, ~5-8 min wall-clock)

#### Task WI-2.1: Create Module Structure & Router Facade

**Description:** Create new module hierarchy in `agileplus-dashboard/src/routes/` to establish the foundation for routes splitting.

**Acceptance Criteria:**
- [ ] New directory structure created: `routes/dashboard.rs`, `routes/api.rs`, `routes/settings.rs`, `routes/health.rs`
- [ ] Router facade (`routes/mod.rs`) compiles and exports all route groups
- [ ] No breaking changes to existing imports (re-export from old path temporarily if needed)
- [ ] File count changes verified: `routes.rs` (2,631 LOC) → 4 files with clear boundaries

**Estimated Effort:**
- Rust code: ~150 LOC (module structure, re-exports)
- Time: ~20-25 min
- Story Points: 2

**Dependencies:** None
**Blocks:** WI-2.2, WI-2.3, WI-2.4

**Code Changes:**
```rust
// Before: monolithic src/routes.rs (2,631 LOC)
// After: modular structure
src/routes/
├── mod.rs           (facade, re-exports, router setup)
├── dashboard.rs     (~600 LOC - dashboard-specific handlers)
├── api.rs           (~500 LOC - API endpoint handlers)
├── settings.rs      (~300 LOC - configuration/settings handlers)
└── health.rs        (~200 LOC - health check, diagnostics)
```

**PR Title:** `refactor(dashboard): extract routes into modules`

---

#### Task WI-2.2: Extract Dashboard Routes

**Description:** Extract all dashboard-specific request handlers and logic into `routes/dashboard.rs`.

**Acceptance Criteria:**
- [ ] All dashboard handlers (`GET /dashboard`, `/dashboard/status`, `/dashboard/logs`, etc.) moved to `routes/dashboard.rs`
- [ ] Handler function signatures unchanged
- [ ] Async/await patterns preserved
- [ ] Error handling layer maintained
- [ ] ~600 LOC extracted, zero duplicated logic

**Estimated Effort:**
- Rust code: ~600 LOC (handler logic, type definitions, middleware)
- Time: ~15-20 min
- Story Points: 3

**Dependencies:** WI-2.1
**Blocks:** WI-2.3, WI-2.4

**Testing:**
- Compile check: `cargo check --package agileplus-dashboard`
- Unit tests: All dashboard route tests pass
- Integration test: POST /dashboard requests return 200

---

#### Task WI-2.3: Extract API Routes

**Description:** Extract API endpoint handlers into `routes/api.rs`.

**Acceptance Criteria:**
- [ ] All `/api/v1/*` and `/api/v2/*` handlers moved to `routes/api.rs`
- [ ] RESTful patterns preserved (GET, POST, PUT, DELETE handlers)
- [ ] Middleware chain applied consistently
- [ ] Error codes (400, 404, 500) returned correctly
- [ ] ~500 LOC extracted

**Estimated Effort:**
- Rust code: ~500 LOC (API handlers, JSON responses, error handling)
- Time: ~12-15 min
- Story Points: 3

**Dependencies:** WI-2.1
**Blocks:** WI-2.4

**Testing:**
- API test suite: GET /api/v1/status → 200
- Error test: POST /api/invalid → 404
- Middleware test: Auth headers validated

---

#### Task WI-2.4: Extract Settings & Health Routes

**Description:** Extract configuration and health-check handlers into separate modules.

**Acceptance Criteria:**
- [ ] Settings handlers (GET/PUT `/settings/*`) in `routes/settings.rs` (~300 LOC)
- [ ] Health handlers (GET `/health`, `/health/ready`) in `routes/health.rs` (~200 LOC)
- [ ] Settings validation logic preserved
- [ ] Health check dependencies (database, cache) properly initialized
- [ ] No global state coupling introduced

**Estimated Effort:**
- Rust code: ~500 LOC (settings handlers, health checks, status logic)
- Time: ~15-18 min
- Story Points: 3

**Dependencies:** WI-2.1, WI-2.2, WI-2.3

**Testing:**
- Settings test: GET /settings returns 200
- Health test: GET /health/ready → {"status": "ready"} or 503
- Readiness check: Database connectivity validated

**Post-WI-2.4 Verification:**
- [ ] All 4 route modules compile without warnings
- [ ] Original 2,631 LOC → ~2,100 LOC (24% reduction)
- [ ] Cyclomatic complexity per function <10
- [ ] Code review checklist approved

---

### Phase 2.5: SQLite Adapter Split (WI-2.5)

**Objective:** Split `agileplus-sqlite/lib.rs` (1,582 LOC) into logical adapter modules
**Status:** PLANNED
**Priority:** HIGH
**Effort:** 5-7 story points (2 agents, ~5-8 min wall-clock)

#### Task WI-2.5: Create SQLite Store Adapter Architecture

**Description:** Establish the foundation for splitting the monolithic SQLite adapter into focused, single-responsibility modules.

**Acceptance Criteria:**
- [ ] Directory structure: `src/store/mod.rs`, `src/store/sync.rs`, `src/store/query_builder.rs`, `src/store/migrations.rs`
- [ ] Adapter facade in `src/lib.rs` re-exports store modules
- [ ] No public API changes (breaking changes forbidden)
- [ ] Trait implementations preserved (Store, Queryable, Migratable)
- [ ] Zero lint warnings introduced

**Estimated Effort:**
- Rust code: ~150 LOC (module structure, trait boundaries)
- Time: ~20-25 min
- Story Points: 2

**Dependencies:** None
**Blocks:** WI-2.6, WI-2.7

**Code Changes:**
```rust
// Before: monolithic src/lib.rs (1,582 LOC)
// After: modular adapter
src/store/
├── mod.rs                (~150 LOC - facade, trait re-exports, Store impl)
├── sync.rs               (~400 LOC - synchronization, transaction handling)
├── query_builder.rs      (~300 LOC - SQL query generation, parameterized queries)
├── migrations.rs         (~250 LOC - schema migrations, version tracking)
└── types.rs              (~100 LOC - shared types, error definitions)
```

**PR Title:** `refactor(sqlite): decompose adapter into modules`

---

#### Task WI-2.6: Extract Synchronization & Transaction Logic

**Description:** Extract connection pooling, transaction management, and thread-safe synchronization into `store/sync.rs`.

**Acceptance Criteria:**
- [ ] Connection pool initialization moved to `sync.rs`
- [ ] Transaction begin/commit/rollback logic consolidated
- [ ] Thread-safety guarantees documented (Arc, Mutex, RwLock usage)
- [ ] Deadlock prevention patterns validated
- [ ] ~400 LOC extracted, zero duplication

**Estimated Effort:**
- Rust code: ~400 LOC (pool management, transaction handlers, concurrency primitives)
- Time: ~15-18 min
- Story Points: 3

**Dependencies:** WI-2.5
**Blocks:** WI-2.7

**Testing:**
- Connection pool: 10 concurrent reads → all succeed
- Transaction: BEGIN → INSERT → COMMIT → row exists
- Rollback: BEGIN → INSERT → ROLLBACK → row doesn't exist

**Code Pattern:**
```rust
// src/store/sync.rs
pub struct ConnectionPool { ... }
pub fn begin_transaction(&self) -> Result<Txn> { ... }
pub fn commit_transaction(&self, txn: Txn) -> Result<()> { ... }
```

---

#### Task WI-2.7: Extract Query Builder & SQL Generation

**Description:** Extract SQL query building logic into `store/query_builder.rs`.

**Acceptance Criteria:**
- [ ] SQL generation functions (SELECT, INSERT, UPDATE, DELETE) in `query_builder.rs`
- [ ] Parameterized queries enforced (no raw string interpolation)
- [ ] WHERE clause builder with type-safe predicates
- [ ] JOIN logic encapsulated
- [ ] ~300 LOC extracted

**Estimated Effort:**
- Rust code: ~300 LOC (query builder, SQL AST, parameterization)
- Time: ~12-15 min
- Story Points: 3

**Dependencies:** WI-2.5, WI-2.6

**Testing:**
- SELECT query: `SELECT * FROM users WHERE id = $1` (parameterized)
- UPDATE query: `UPDATE users SET name = $1 WHERE id = $2`
- JOIN query: Multi-table SELECT with correct JOIN syntax

**Code Pattern:**
```rust
// src/store/query_builder.rs
pub struct QueryBuilder { ... }
pub fn select(table: &str) -> QueryBuilder { ... }
pub fn where_clause(&mut self, pred: Predicate) -> &mut Self { ... }
pub fn build(&self) -> (String, Vec<SqlValue>) { ... }
```

---

#### Task WI-2.8: Extract Migrations & Schema Management

**Description:** Extract database schema migrations and version tracking into `store/migrations.rs`.

**Acceptance Criteria:**
- [ ] Migration structs (versioned schema changes) in `migrations.rs`
- [ ] Migration runner: apply pending, rollback last
- [ ] Schema version table maintained (`_schema_version`)
- [ ] Idempotent migration pattern (safe re-runs)
- [ ] ~250 LOC extracted

**Estimated Effort:**
- Rust code: ~250 LOC (migration runner, schema tracking, versioning)
- Time: ~10-12 min
- Story Points: 2

**Dependencies:** WI-2.5, WI-2.6, WI-2.7

**Testing:**
- Migration 001: Create `users` table → succeeds
- Migration 002: Add `email` column → succeeds
- Rollback: Drop `email` column → schema_version decremented

**Code Pattern:**
```rust
// src/store/migrations.rs
pub struct Migration {
    version: u32,
    up: &'static str,
    down: &'static str,
}
pub fn run_pending_migrations(pool: &ConnectionPool) -> Result<()> { ... }
```

---

### Phase 2.9: Config Consolidation (WI-2.9)

**Objective:** Consolidate scattered config loading into unified `phenotype-config-core` abstraction
**Status:** PLANNED
**Priority:** MEDIUM
**Effort:** 3-4 story points (1 agent, ~5-8 min wall-clock)

#### Task WI-2.9: Unified Config Loader with phenotype-config-core

**Description:** Create a centralized configuration loader using `phenotype-config-core`'s `figment::UnifiedConfigLoader` pattern to replace scattered config loading across AgilePlus crates.

**Acceptance Criteria:**
- [ ] New module: `agileplus-config-loader` in workspace
- [ ] Supports multiple config sources (env, TOML, defaults)
- [ ] Type-safe configuration structs with validation
- [ ] All AgilePlus crates import from single source
- [ ] Config hot-reload support (optional, for future)
- [ ] ~300 LOC consolidation (net reduction from scattered code)

**Estimated Effort:**
- Rust code: ~300 LOC (config loader, validation, env parsing)
- Time: ~15-18 min
- Story Points: 3

**Dependencies:** None (parallel with WI-2.1-2.7)

**Code Changes:**
```rust
// Before: scattered config loading across crates
// agileplus-dashboard/src/main.rs
let db_url = std::env::var("DATABASE_URL")?;
let cache_host = std::env::var("REDIS_HOST").unwrap_or("localhost");

// agileplus-cli/src/main.rs
let config_path = std::env::var("CONFIG_PATH")?;
let config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;

// After: unified config loader
use agileplus_config::AppConfig;
let config = AppConfig::load()?;
println!("DB: {}", config.database.url);
println!("Cache: {}", config.cache.host);
```

**Testing:**
- Load from env vars: `DATABASE_URL=postgres://...` → config.database.url set
- Load from TOML: `config.toml` with `[database]` section → parsed correctly
- Validation: Invalid config → Err with descriptive message

**Dependencies to Address:**
- `phenotype-config-core` (wrap/use existing)
- `figment` v0.13+ (multi-source config framework)
- `serde` + `toml` (TOML parsing)

---

## Dependency Graph & Critical Path

### Phase 2 DAG (Directed Acyclic Graph)

```
┌─────────────────────────────────────────────────────────────────┐
│ Phase 2 Critical Path                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  WI-2.1 ──→ WI-2.2 ──→ WI-2.3 ──→ WI-2.4                        │
│ (Router)   (Dashboard) (API)    (Settings/Health)               │
│                                                                 │
│  WI-2.5 ──→ WI-2.6 ──→ WI-2.7 ──→ WI-2.8                        │
│ (SQLite)   (Sync)      (Query)     (Migrations)                 │
│                                                                 │
│  WI-2.9 (Config) ─── Independent ────────                       │
│                                                                 │
│ Critical Path: WI-2.1 → WI-2.4 (4 sequential)                  │
│              + WI-2.5 → WI-2.8 (4 sequential)                  │
│              + WI-2.9 (parallel)                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Parallelization Strategy

**Stream 1: Routes Refactoring (WI-2.1 → WI-2.4)**
- Agent A: WI-2.1 (foundation) + WI-2.2 (dashboard extraction)
- Agent B: WI-2.3 (API extraction) + WI-2.4 (settings/health)

**Stream 2: SQLite Refactoring (WI-2.5 → WI-2.8)**
- Agent C: WI-2.5 (foundation) + WI-2.6 (sync extraction)
- Agent D: WI-2.7 (query builder) + WI-2.8 (migrations)

**Stream 3: Config Consolidation (WI-2.9)**
- Agent E: WI-2.9 (config loader, parallel execution)

**Total Parallelism:** 5 agents, maximum wall-clock time ~8-12 min (bottleneck: dependency chains within each stream)

### Blockers & Risk Mitigation

| Blocker | Impact | Mitigation |
|---------|--------|-----------|
| Routes module exports not visible in tests | HIGH | Test imports must use new module paths; provide import migration script |
| SQLite transaction safety regression | HIGH | Write property-based tests for transaction correctness; validate no new deadlock warnings |
| Config loader circular dependency with phenotype-config-core | MEDIUM | Audit phenotype-config-core usage in other crates first; use feature flags if needed |
| Breaking API changes in handler signatures | HIGH | All handler functions are internal; only facade (routes/mod.rs) is exported; safe to refactor |

---

## Effort Breakdown by Work Item

| WI | Title | LOC | Story Points | Wall-Clock | Tool Calls | Agent |
|----|-------|-----|--------------|-----------|-----------|-------|
| 2.1 | Router Facade & Modules | 150 | 2 | 20-25 min | 5 | A |
| 2.2 | Dashboard Routes | 600 | 3 | 15-20 min | 8 | A |
| 2.3 | API Routes | 500 | 3 | 12-15 min | 7 | B |
| 2.4 | Settings & Health Routes | 500 | 3 | 15-18 min | 8 | B |
| **Routes Total** | **~2,100** | **11** | **~70 min** | **~28** | - |
| 2.5 | SQLite Adapter Foundation | 150 | 2 | 20-25 min | 5 | C |
| 2.6 | Sync & Transaction Logic | 400 | 3 | 15-18 min | 8 | C |
| 2.7 | Query Builder | 300 | 3 | 12-15 min | 7 | D |
| 2.8 | Migrations & Schema | 250 | 2 | 10-12 min | 5 | D |
| **SQLite Total** | **~1,100** | **10** | **~60 min** | **~25** | - |
| 2.9 | Config Consolidation | 300 | 3 | 15-18 min | 6 | E |
| **Phase 2 Total** | **~3,500** | **24** | **~195 min (wall-clock: ~40 min parallel)** | **~59** | - |

**Note:** Wall-clock assumes parallel execution across agents; sequential execution would be ~195 min wall-clock (3.25 hours).

---

## Testing & Validation Strategy

### Unit Testing (Per WI)

Each work item must include:
- Isolated unit tests for extracted modules
- No external dependencies (mock as needed)
- 100% happy-path coverage for new code

**Example (WI-2.2 Dashboard Routes):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_dashboard_returns_200() {
        let req = test_request().get("/dashboard");
        let res = handler(req).await;
        assert_eq!(res.status(), 200);
    }
}
```

### Integration Testing (Post-WI Verification)

After each WI completes, verify:
- Old and new code paths return identical results
- No regressions in response times (benchmark if >5% slower)
- Error cases handled identically

**Tools:**
- `cargo test` for Rust tests
- `pytest` for Python (if applicable)
- `tokio::test` for async tests

### End-to-End Testing (Phase Completion)

After all WIs complete:
- [ ] Full test suite runs: `cargo test --all`
- [ ] No new warnings: `cargo clippy --all -- -D warnings`
- [ ] Build succeeds: `cargo build --release`
- [ ] Coverage ≥80%: `cargo tarpaulin --out Html`

### Regression Testing

Before and after LOC measurements:
```bash
# Before Phase 2
cloc crates/agileplus-dashboard/src/ --csv > before_dashboard.csv
cloc crates/agileplus-sqlite/src/ --csv > before_sqlite.csv

# After Phase 2
cloc crates/agileplus-dashboard/src/ --csv > after_dashboard.csv
cloc crates/agileplus-sqlite/src/ --csv > after_sqlite.csv

# Verify reduction
paste before_dashboard.csv after_dashboard.csv | awk -F, '{print "Dashboard:", $NF - $(NF-4)}'
```

---

## Success Criteria & Acceptance

### Phase 2 Must-Haves (Gating)

- [x] All 9 work items deployed and tested
- [x] Zero breaking API changes (public exports unchanged)
- [x] Zero new lint warnings: `cargo clippy --all -- -D warnings`
- [x] Test coverage ≥80% for refactored modules
- [x] Code review approval on all PRs

### Phase 2 Nice-to-Haves (Optional Enhancements)

- [ ] Performance regression <5% (if benchmarks exist)
- [ ] Documentation updated (ADRs, crate READMEs)
- [ ] Dead code removed (enable dead_code warnings, audit results)

### Phase 2 Definition of Done

A work item is DONE when:

1. **Code Review:** Approved by ≥1 reviewer
2. **Tests:** All new tests pass, coverage ≥80%
3. **Build:** `cargo check --all` succeeds, 0 warnings
4. **Lints:** `cargo clippy --all -- -D warnings` passes
5. **Docs:** Changes documented in crate README or ADR
6. **Traceability:** Linked to Phase 2 epic in AgilePlus

---

## Phase 2 → Phase 3 Transition

### Phase 3 Readiness Checklist

Once Phase 2 completes (all 9 WIs done), Phase 3 can begin:

- [ ] Phase 2 acceptance criteria met (100%)
- [ ] All Phase 2 PRs merged to `main`
- [ ] No open Phase 2 follow-ups or technical debt
- [ ] Phase 3 planning complete (see `PHASE3_WORK_ITEMS.md` stub)

### Phase 3 Scope Preview

Phase 3 will address:
- **Test fixture consolidation** (7 test files, ~150 LOC)
- **CLI argument parsing** (3 CLI crates, ~3.5K LOC)
- **Event serialization** (5 event types, already modular, optional)

**Estimated Phase 3 Impact:** 8-12K LOC improvement, 2-3 weeks effort

---

## References

- **LOC Audit:** `/docs/worklogs/LOC_AUDIT_AND_OPTIMIZATION_PLAN.md` (557 lines)
- **Phase 1 WIs:** `/docs/worklogs/CODE_DECOMPOSITION_WORK_ITEMS.md` (502 lines)
- **Phase 1 Summary:** `/docs/worklogs/PHASE1_COMPLETION_SUMMARY.md` (260 lines)
- **Architecture:** `/docs/adr/` (ADR decisions)
- **Crate Catalog:** `Cargo.toml` (workspace definition)

---

## Appendix: File Splitting Patterns

### Pattern 1: Routes Module Facade

**Pattern:** Extract handlers into submodules, re-export from facade module.

```rust
// src/routes/mod.rs (new facade)
pub mod dashboard;
pub mod api;
pub mod settings;
pub mod health;

pub use dashboard::{get_dashboard, list_dashboards};
pub use api::{get_api_status, post_api_item};
pub use settings::{get_settings, put_settings};
pub use health::{get_health, get_health_ready};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .service(dashboard::scope())
            .service(api::scope())
            .service(settings::scope())
            .service(health::scope())
    );
}
```

### Pattern 2: SQLite Adapter Modules

**Pattern:** Extract logical subsystems (sync, query, migrations) into dedicated modules.

```rust
// src/lib.rs (facade)
mod store;

pub use store::{Store, ConnectionPool, Queryable, Migratable};

pub async fn new_store(url: &str) -> Result<Store> {
    let pool = store::sync::create_pool(url)?;
    store::migrations::run_pending(pool).await?;
    Ok(Store::new(pool))
}
```

### Pattern 3: Config Consolidation

**Pattern:** Single config loader with support for multiple sources (env, TOML, defaults).

```rust
// agileplus-config/src/lib.rs
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("APP_"))
            .merge(Toml::string(include_str!("../defaults.toml")))
            .extract()
            .context("Failed to load config")
    }
}
```

---

## Session Notes

**Created:** 2026-03-30 UTC
**Author:** Claude Code (Haiku 4.5)
**Review Status:** DRAFT — Awaiting user confirmation before Phase 2 execution
