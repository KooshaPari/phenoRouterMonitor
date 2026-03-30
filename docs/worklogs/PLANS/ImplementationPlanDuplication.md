# Duplication Audit Implementation Plan

**Generated:** 2026-03-29
**Status:** Ready for Execution
**Priority:** P0

---

## Executive Summary

This plan addresses the critical findings from the comprehensive duplication audit:
- **1,650+ LOC** of unused library code
- **~189 LOC** of duplicated error types (12 types)
- **2,106 LOC** of port/trait architecture split
- **Savings potential: 2,800+ LOC** through consolidation

---

## Phase 1: Library Migration (Week 1)

### 1.1 Migrate `libs/` to Edition 2024

| Task ID | Library | Action | Estimated Time |
|---------|---------|--------|----------------|
| LIB-001 | `libs/logger/` | Migrate edition + integrate | 2 hours |
| LIB-002 | `libs/metrics/` | Migrate edition + integrate | 2 hours |
| LIB-003 | `libs/tracing/` | Migrate edition + integrate | 2 hours |
| LIB-004 | `libs/hexagonal-rs/` | Migrate edition + integrate | 4 hours |
| LIB-005 | `libs/hexkit/` | Deprecate (duplicate) | 1 hour |
| LIB-006 | `libs/cli-framework/` | Migrate edition + integrate | 3 hours |
| LIB-007 | `libs/config-core/` | Migrate edition + integrate | 4 hours |
| LIB-008 | `libs/cipher/` | Archive (unused) | 30 minutes |
| LIB-009 | `libs/gauge/` | Archive (unused) | 30 minutes |
| LIB-010 | `libs/nexus/` | Archive (unused) | 30 minutes |
| LIB-011 | `libs/xdd-lib-rs/` | Archive (unused) | 30 minutes |

**Total:** ~20 hours

### 1.2 Verification

```bash
cargo check --workspace
cargo test --workspace
```

---

## Phase 2: Error Core Creation (Week 2)

### 2.1 Create `libs/error-core/`

**Estimated Time:** 3 days

#### Step 1: Create crate structure (4 hours)

```bash
cargo new --lib libs/error-core
cd libs/error-core
# Add dependencies: thiserror, serde
```

#### Step 2: Define canonical error types (8 hours)

```rust
// src/storage.rs
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Connection failed: {0}")]
    Connection(String),
}

// src/serialization.rs
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

// src/sync.rs
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Store error: {0}")]
    Store(String),

    #[error("Serialization: {0}")]
    Serialization(#[from] SerializationError),

    #[error("Conflict: {0}")]
    Conflict(String),
}
```

#### Step 3: Define domain errors (4 hours)

```rust
// src/domain.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid transition: {0}")]
    InvalidTransition(String),

    #[error("Validation: {0}")]
    Validation(String),

    #[error("Internal: {0}")]
    Internal(String),
}
```

#### Step 4: Migrate crates (8 hours)

Per crate migration:
1. Add `error-core` dependency
2. Replace local error variants with imports
3. Add `#[from]` derives
4. Update tests

### 2.2 Migration Sequence

| Order | Crate | Dependencies |
|-------|-------|--------------|
| 1 | `agileplus-domain` | No dependencies |
| 2 | `agileplus-sync` | Domain |
| 3 | `agileplus-events` | Domain |
| 4 | `agileplus-cache` | Domain |
| 5 | `agileplus-graph` | Domain |
| 6 | `agileplus-p2p` | Domain, Sync, Events |
| 7 | `agileplus-api` | Domain, P2P |
| 8 | `agileplus-telemetry` | Domain |

---

## Phase 3: Port/Trait Consolidation (Week 3)

### 3.1 Integrate hexagonal-rs

**Estimated Time:** 5 days

1. Migrate `libs/hexagonal-rs` to edition 2024
2. Add to workspace
3. Create `Repository<E>` trait
4. Migrate store traits

### 3.2 Consolidate Port Traits

| Trait | Current | Target |
|-------|---------|--------|
| Repository | phenotype-port-interfaces + hexagonal-rs | hexagonal-rs |
| StoragePort | agileplus-domain | hexagonal-rs Repository |
| Logger | phenotype-port-interfaces | agileplus-telemetry |
| ObservabilityPort | agileplus-domain | agileplus-telemetry |

---

## Phase 4: HTTP Client Consolidation (Week 4)

### 4.1 Create `libs/http-client/`

**Estimated Time:** 3 days

```rust
pub struct HttpClient {
    client: reqwest::Client,
    base_url: Url,
    auth: Option<Auth>,
    retry: RetryConfig,
}

impl HttpClient {
    pub fn new(base_url: Url) -> Self { ... }

    pub fn with_auth(self, auth: Auth) -> Self { ... }

    pub fn with_retry(self, config: RetryConfig) -> Self { ... }

    pub async fn get(&self, path: &str) -> Result<Response> { ... }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> { ... }
}
```

### 4.2 Migrate existing clients

| Project | Files to Update |
|---------|----------------|
| agileplus-plane | `src/client/*.rs` |
| agileplus-github | `src/client/*.rs` |
| agileplus-telemetry | `src/http.rs` |
| agileplus-agents | `src/adapters/*.rs` |

---

## Phase 5: Config Consolidation (Week 5)

### 5.1 Evaluate config-rs

**Decision Criteria:**
- Does it support TOML, YAML, JSON?
- Does it support env var overrides?
- Does it support validation?
- Is it well-maintained?

### 5.2 Integrate config-core

If `config-rs` is not adopted:

1. Migrate `libs/config-core` to edition 2024
2. Implement TOML loader
3. Implement YAML loader
4. Implement JSON loader
5. Migrate existing config loaders

---

## External Package Evaluation

### config-rs

**URL:** https://crates.io/crates/config
**Downloads:** 40M+
**Purpose:** Configuration management

**Pros:**
- Multi-source (env, files, args)
- Nested configuration
- Schema validation
- Well-maintained

**Cons:**
- Additional dependency
- May not fit all use cases

**Recommendation:** Evaluate for Week 5

### eventually

**URL:** https://crates.io/crates/eventually
**Stars:** ~500
**Purpose:** Event sourcing framework

**Pros:**
- Standardized Aggregate/Repository traits
- EventStore pattern
- Command handling

**Cons:**
- Learning curve
- May not fit all ES patterns

**Recommendation:** Evaluate for Week 6

### anthropic

**URL:** https://crates.io/crates/anthropic
**Purpose:** Claude SDK

**Pros:**
- First-class Claude support
- Token counting
- Tool use

**Cons:**
- New crate, may be unstable

**Recommendation:** Evaluate for Week 6

---

## Task Checklist

### Phase 1: Library Migration

- [ ] LIB-001: Migrate `libs/logger/` to edition 2024
- [ ] LIB-002: Migrate `libs/metrics/` to edition 2024
- [ ] LIB-003: Migrate `libs/tracing/` to edition 2024
- [ ] LIB-004: Migrate `libs/hexagonal-rs/` to edition 2024
- [ ] LIB-005: Deprecate `libs/hexkit/`
- [ ] LIB-006: Migrate `libs/cli-framework/` to edition 2024
- [ ] LIB-007: Migrate `libs/config-core/` to edition 2024
- [ ] LIB-008: Archive `libs/cipher/`
- [ ] LIB-009: Archive `libs/gauge/`
- [ ] LIB-010: Archive `libs/nexus/`
- [ ] LIB-011: Archive `libs/xdd-lib-rs/`
- [ ] Verify: `cargo check --workspace`
- [ ] Verify: `cargo test --workspace`

### Phase 2: Error Core

- [ ] ERR-001: Create `libs/error-core/` crate
- [ ] ERR-002: Define `StorageError`
- [ ] ERR-003: Define `SerializationError`
- [ ] ERR-004: Define `SyncError`
- [ ] ERR-005: Define `DomainError`
- [ ] ERR-006: Define `ApiError`
- [ ] ERR-007: Migrate agileplus-domain
- [ ] ERR-008: Migrate agileplus-sync
- [ ] ERR-009: Migrate agileplus-events
- [ ] ERR-010: Migrate agileplus-cache
- [ ] ERR-011: Migrate agileplus-graph
- [ ] ERR-012: Migrate agileplus-p2p
- [ ] ERR-013: Migrate agileplus-api
- [ ] Verify: All tests pass

### Phase 3: Port/Trait

- [ ] PORT-001: Migrate hexagonal-rs to edition 2024
- [ ] PORT-002: Create `Repository<E>` trait
- [ ] PORT-003: Consolidate Repository ↔ StoragePort
- [ ] PORT-004: Consolidate Logger ↔ ObservabilityPort
- [ ] Verify: All trait implementations work

### Phase 4: HTTP Client

- [ ] HTTP-001: Create `libs/http-client/`
- [ ] HTTP-002: Add retry configuration
- [ ] HTTP-003: Add authentication
- [ ] HTTP-004: Migrate agileplus-plane
- [ ] HTTP-005: Migrate agileplus-github
- [ ] HTTP-006: Migrate agileplus-telemetry
- [ ] HTTP-007: Migrate agileplus-agents
- [ ] Verify: All HTTP calls work

### Phase 5: Config

- [ ] CONFIG-001: Evaluate config-rs
- [ ] CONFIG-002: Migrate config-core if needed
- [ ] CONFIG-003: Migrate TOML loader
- [ ] CONFIG-004: Migrate YAML loader
- [ ] CONFIG-005: Migrate JSON loader
- [ ] Verify: Config loading works

---

## Effort Summary

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1 | 13 tasks | 20 hours |
| Phase 2 | 14 tasks | 3 days |
| Phase 3 | 5 tasks | 5 days |
| Phase 4 | 8 tasks | 3 days |
| Phase 5 | 6 tasks | 2 days |
| **Total** | **46 tasks** | **~3 weeks** |

---

## Success Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| Unused libraries | 11 | 0 | 0 |
| Error type LOC | ~600 | 200 | 150 |
| Config LOC | 500 | 150 | 100 |
| HTTP client LOC | 300 | 100 | 50 |
| Port/trait LOC | 2,106 | 1,000 | 500 |
| Total savings | - | - | 2,800 LOC |

---

## Related Documentation

- `MasterDuplicationAudit20260329.md` (parent dir) — complete audit findings; `PLANS/MasterDuplicationAudit.md` — working copy
- `PLANS/ERROR_CORE_EXTRACTION.md` - Detailed error extraction plan
- `PLANS/CONFIG_CORE_ACTIVATION.md` - Config integration plan
- `PLANS/EDITION_MIGRATION.md` - Edition migration guide
- `WorkLog.md` - Wave entries and progress

---

*Plan generated by FORGE (2026-03-29)*
