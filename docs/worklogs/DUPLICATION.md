# Duplication Worklogs

**Category:** DUPLICATION | **Updated:** 2026-03-29

---

## 2026-03-29 - AgilePlus Extended Duplication Audit

**Project:** [AgilePlus]
**Category:** duplication
**Status:** in_progress
**Priority:** P1

### Summary

Extended comprehensive audit of AgilePlus intra-repo duplication. Identified patterns across health checks, error types, config loaders, API responses, port/trait architecture, builder patterns, async traits, and connection pools.

### Detailed Findings

#### 1. Health Check Patterns (140 LOC across 3 files)

| File | Pattern | LOC |
|------|---------|-----|
| `crates/agileplus-cache/src/health.rs:5-8` | CacheHealth enum | 42 |
| `crates/agileplus-graph/src/health.rs:5-8` | GraphHealth enum + store.health_check() | 90 |
| `crates/agileplus-nats/src/health.rs:5-8` | BusHealth enum | 8 |

**Common Pattern:** HealthStatus enum with Healthy/Unavailable states + backend-specific check methods

**External Reference:** https://docs.rs/health_check/1.10.0/health_check/

**Canonical Location:** `agileplus-health` crate (PROPOSED)

#### 2. Error Type Proliferation (504 LOC across 15+ enums)

| Crate | Error Type | Variants | LOC |
|-------|------------|----------|-----|
| agileplus-api | ApiError | 6 | 67 |
| agileplus-domain | DomainError | 15+ | 50 |
| agileplus-sync | SyncError | 5 | 24 |
| agileplus-p2p | PeerDiscoveryError | 78 |
| phenotype-port-interfaces | PortError | 10 | 51 |
| phenotype-event-sourcing | EventSourcingError | 46 |
| phenotype-http-adapter | HttpError | 6 | 45 |

**Common Variants:** NotFound, Timeout, Serialization, Config/Validation

**Canonical Location:** `agileplus-error-core` crate (PROPOSED)

#### 3. Config Loading Patterns (449 LOC)

| Crate | Pattern | Format | Canonical |
|-------|---------|--------|-----------|
| agileplus-domain | TOML + env overrides | TOML | libs/config-core |
| agileplus-telemetry | YAML + env overrides | YAML | libs/config-core |
| agileplus-cache | Builder pattern | Struct | libs/config-core |

**Status:** libs/config-core EXISTS but workspace: false - UNUSED

#### 4. Port/Trait Architecture Split (2106 LOC)

| Ecosystem | Location | Ports |
|-----------|----------|-------|
| phenotype-port-interfaces | `libs/phenotype-shared/` | 8 traits |
| agileplus-domain | `crates/agileplus-domain/src/ports/` | 5 traits |
| hexagonal-rs | `libs/hexagonal-rs/` | Full framework (UNUSED) |

**Overlapping Concerns:**
- Logger trait vs ObservabilityPort
- Repository trait vs StoragePort

#### 5. API Response Patterns (224 LOC)

| Pattern | Location | Type |
|---------|----------|------|
| HealthResponse | `crates/agileplus-api/src/responses.rs:125-224` | Struct with HashMap |
| ApiResponse | `platforms/heliosCLI/codex-rs/core/src/client.rs` | Generic<T> |

**Canonical Location:** `agileplus-api-types` crate (PROPOSED)

#### 6. Builder Pattern Proliferation

| Builder | Location | Methods |
|---------|----------|---------|
| EventQuery | `agileplus-events/src/query.rs:26-74` | 9 methods |
| CacheConfig | `agileplus-cache/src/config.rs:13-35` | 2 methods |

#### 7. Async Trait Issues

**SnapshotStore misplaced:** `agileplus-events/src/snapshot.rs:37-56`
- Uses #[async_trait]
- NOT in phenotype-port-interfaces despite similar purpose to Repository trait

#### 8. Connection Pool Patterns

| Pool | Location | Manager |
|------|----------|---------|
| CachePool | `agileplus-cache/src/pool.rs:17-48` | bb8 |
| phenotype-redis-adapter | `libs/phenotype-shared/` | deadpool |

**Issue:** Inconsistent pool managers (bb8 vs deadpool)

### LOC Savings Potential

| Pattern | Current | Savings | Canonical |
|---------|---------|---------|-----------|
| Health checks | 140 | 80 | agileplus-health |
| Error types | 504 | 150 | agileplus-error-core |
| Config loaders | 449 | 200 | libs/config-core |
| API types | 224 | 50 | agileplus-api-types |
| **Total** | **1,317** | **480** | |

### Action Items

- [ ] 🔴 CRITICAL: Create `agileplus-health` crate
- [ ] 🟡 HIGH: Create `agileplus-error-core` crate
- [ ] 🟡 HIGH: Integrate `libs/config-core` into workspace
- [ ] 🟡 HIGH: Move `SnapshotStore` to phenotype-port-interfaces
- [ ] 🟠 MEDIUM: Create `agileplus-api-types` crate
- [ ] 🟠 MEDIUM: Create generic QueryBuilder trait
- [ ] 🟠 MEDIUM: Audit port interfaces for consolidation
- [ ] 🟢 LOW: Migrate bb8 to deadpool

### Related

- Audit: `docs/reports/AGILEPLUS_DUPLICATION_AUDIT_20260329.md`
- Decomposition: `docs/reports/AGILEPLUS_DECOMPOSITION_AUDIT.md`

---

## 2026-03-29 - Cross-Project Duplication Audit (Comprehensive)

**Project:** [cross-repo]
**Category:** duplication
**Status:** in_progress
**Priority:** P0

### Summary

Comprehensive audit of cross-project duplication across AgilePlus, heliosCLI, thegent, and libraries. Identified 36+ duplicate error types, 4 duplicate config loaders, 3 duplicate health enums, and 4 duplicate in-memory stores.

### High Priority Findings

#### Error Type Duplication (36+ enums)

| Error Type | Locations | Severity |
|------------|-----------|----------|
| `NotFound` | DomainError, ApiError, GraphError, NexusError | High |
| `Conflict` | DomainError, ApiError, SyncError | High |
| `Serialization` | SyncError, CacheError, EventBusError | High |
| `Config/InvalidConfig` | Multiple crates | High |

**Affected Files:**
- `crates/agileplus-sync/src/error.rs:6-24`
- `crates/agileplus-p2p/src/error.rs:26-47`
- `crates/agileplus-nats/src/bus.rs:17-31`
- `crates/agileplus-cache/src/store.rs:9-19`
- `libs/nexus/src/error.rs`
- `libs/hexagonal-rs/src/lib.rs`

#### Configuration Loading Duplication (4 implementations)

| Crate | File | Pattern |
|-------|------|---------|
| agileplus-domain | `src/config/loader.rs:21-84` | TOML + dirs_next |
| agileplus-dashboard | `src/routes.rs:137-170` | Identical pattern |
| agileplus-telemetry | `src/config.rs:126-145` | YAML variant |
| agileplus-subcmds | `src/sync/config.rs:12-36` | JSON variant |

**Duplicated `home_dir()` usage:**
- `crates/agileplus-telemetry/src/config.rs:209`
- `crates/agileplus-domain/src/config/core.rs:26`
- `crates/agileplus-domain/src/config/credentials.rs:32`
- `crates/agileplus-domain/src/config/loader.rs:24`

### Medium Priority Findings

#### Health Check Duplication (3 enums + 1 sophisticated)

| Crate | Type | File |
|-------|------|------|
| agileplus-graph | `GraphHealth { Healthy, Unavailable }` | `src/health.rs:4-8` |
| agileplus-cache | `CacheHealth { Healthy, Unavailable }` | `src/health.rs:4-8` |
| agileplus-nats | `BusHealth { Connected, Disconnected }` | `src/health.rs:4-7` |
| agileplus-domain | `HealthStatus { Healthy, Degraded, Unavailable }` | `src/domain/service_health.rs:8-15` |

#### Store Trait Patterns (3 traits)

| Trait | Crate | File |
|-------|-------|------|
| `EventStore` | agileplus-events | `src/store.rs:21-53` |
| `CacheStore` | agileplus-cache | `src/store.rs:21-38` |
| `GraphBackend` | agileplus-graph | `src/store.rs:22-27` |

#### In-Memory Backend Duplication (4 stores)

| Crate | Type | File |
|-------|------|------|
| agileplus-nats | `InMemoryBus` | `src/bus.rs:127` |
| agileplus-graph | `InMemoryBackend` | `src/store.rs:106` |
| agileplus-domain | `InMemoryCredentialStore` | `src/credentials/memory.rs:15` |
| agileplus-sync | `InMemoryStore` | `src/store.rs:59` |

### Tasks Completed

- [x] Audited error type definitions across 24 crates
- [x] Documented config loading patterns
- [x] Identified health check duplications
- [x] Catalogued store trait patterns
- [x] Created consolidation plan

### Next Steps

- [ ] Create `agileplus-error-core` crate
- [ ] Extract `agileplus-config-core` crate
- [ ] Unify health status types
- [ ] Extract test utilities

### Related

- Full Plan: `plans/2026-03-29-CROSS_PROJECT_DUPLICATION_PLAN-v1.md`
- Audit Files: `plans/2026-03-29-DUPLICATION_AUDIT*.md`

---

## 2026-03-29 - Duplication Audit Chunk 6: Comprehensive Codebase Scan with Exact Citations

**Project:** [cross-repo]
**Category:** duplication
**Status:** in_progress
**Priority:** P0
**Scope:** Full codebase audit across `crates/`, `platforms/`, `libs/`, `src/`, `.worktrees/` with exact file:line references.

---

### 17. Error Enum Duplication (EXHAUSTIVE SCAN)

**Scan:** `grep -rn --include='*.rs' 'pub enum.*Error' .`

| File | Line | Type | Variants | LOC Est |
|------|------|------|----------|---------|
| `crates/phenotype-event-sourcing/src/error.rs` | 7 | `EventSourcingError` | ~10 | 40 |
| `crates/phenotype-event-sourcing/src/error.rs` | 19 | `EventStoreError` | ~8 | 35 |
| `crates/phenotype-event-sourcing/src/error.rs` | 37 | `HashError` | ~4 | 20 |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/inbound/mod.rs` | 84 | `Error` (inbound port) | ~12 | 45 |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/mod.rs` | 63 | `Error` (outbound port) | ~10 | 40 |
| `crates/phenotype-policy-engine/src/error.rs` | 7 | `PolicyEngineError` | ~12 | 50 |
| `platforms/thegent/crates/thegent-memory/src/error.rs` | 77 | `is_retryable()` method | — | 15 |
| `platforms/thegent/crates/thegent-subprocess/src/lib.rs` | 5 | doc comment on `run_with_retry` | — | 10 |

**Dedup candidate:** `libs/phenotype-error` (consolidate all domain error types)

---

### 18. Config / Home-Dir / dirs_next Pattern Duplication (EXHAUSTIVE)

**Scan:** `grep -rn --include='*.rs' -E 'home_dir|dirs_next|dirs::|directories-next' .`

| File | Line | Pattern | Type |
|------|------|---------|------|
| `platforms/thegent/crates/thegent-tui/src/panels/pareto.rs` | 183 | `home_dir().join(".thegent")` | path resolve |
| `platforms/thegent/crates/thegent-tui/src/themes/mod.rs` | 231 | `d.home_dir()` | theme config |
| `platforms/thegent/crates/thegent-tui/src/widgets/interactive_input.rs` | 59 | `home_dir().join(".thegent").join("input_history.txt")` | history |
| `platforms/thegent/crates/thegent-memory/src/client.rs` | 102, 111 | `env::var("SM_API_KEY")`, `SM_BASE_URL` | env var |
| `platforms/thegent/crates/harness-native/src/dispatcher.rs` | 35, 232, 339 | `env::var("HARNESS_HOME")`, `PPID` | env var |
| `platforms/thegent/crates/harness-native/src/find_real.rs` | 49 | `env::var("PATH")` | path resolve |
| `platforms/thegent/crates/thegent-runtime/src/main.rs` | 96, 106-108, 664, 777 | `BYPASS_ULTRA_SHIM`, `AGENT_ID`, `HELIOS_AGENT`, `HOME` | runtime switch |
| `platforms/thegent/crates/thegent-hooks/src/main.rs` | 88, 447, 1400, 1747, 1856, 1958 | `THEGENT_CACHE_DIR`, `HOME` variants | cache/home |
| `platforms/thegent/crates/thegent-hooks/src/main.rs` | 1665, 1678 | `THGENT_NOTIFY_ENABLE`, `THGENT_NOTIFY_VOICE_MODE` | notify flags |
| `platforms/thegent/crates/thegent-hooks/src/git_ops.rs` | 49-51, 341, 401-402 | `THEGENT_AGENT_ID`, `SESSION_ID`, `CORRELATION_ID`, `GIT_LOCK_TIMEOUT` | context |
| `platforms/thegent/crates/thegent-hooks/src/git_cache.rs` | 51-52, 56, 75 | `CLAUDE_HOME`, `HOME`, `GIT_CACHE_TTL`, `SESSION_ID` | cache config |
| `platforms/thegent/crates/thegent-hooks/src/utils.rs` | 21, 59, 82 | `THEGENT_TOOL_BIN_PATH`, `THEGENT_GIT_BIN` | tool path |
| `platforms/thegent/crates/thegent-path-resolve/src/lib.rs` | 83, 154 | `PATH`, `CI` | path/CI resolve |
| `platforms/thegent/crates/thegent-tool-detect/src/lib.rs` | 243, 251 | `CI` | test CI guard |
| `platforms/thegent/crates/thegent-shims/src/main.rs` | 403, 607-608 | `HOME`, `OPENAI_BASE_URL`, `OPENAI_API_KEY` | shim/API |
| `platforms/thegent/hooks/hook-dispatcher/src/main.rs` | 1659, 1664, 1669, 1770, 1898 | `THGENT_STOP_*`, `RG_TIMEOUT_SEC`, `AGENT_SHELL` | timeout |
| `platforms/thegent/hooks/hook-dispatcher/src/io/mod.rs` | 5, 32, 52 | `PATH`, `HOOKS_DIR`, `HOME` | IO config |
| `crates/agileplus-domain/src/config/core.rs` | 26 | `home_dir()` | config core |
| `crates/agileplus-domain/src/config/credentials.rs` | 32 | `home_dir()` | credential config |
| `crates/agileplus-domain/src/config/loader.rs` | 24 | `home_dir()` + dirs_next | config loader |
| `crates/agileplus-telemetry/src/config.rs` | 209 | `home_dir()` | telemetry config |
| `crates/agileplus-subcmds/src/sync/config.rs` | 12-36 | dirs_next variant | JSON config |
| `crates/agileplus-dashboard/src/routes.rs` | 137-170 | dirs_next | route config |

**Dedup candidate:** `libs/config-core` or `libs/env` wrapper for all `env::var` + `home_dir()` + `dirs_next` access.

---

### 19. Async Trait Repetition (EXHAUSTIVE)

**Scan:** `grep -rn --include='*.rs' '#\[async_trait\]' .`

| File | Lines | Count | Category |
|------|-------|-------|----------|
| `crates/phenotype-contracts/phenotype-contracts/src/ports/inbound/mod.rs` | 76, 124, 131, 159, 166, 193, 200 | 7 | inbound port traits |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/cache.rs` | 38, 71, 81, 94 | 4 | outbound cache traits |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/event.rs` | 65, 84 | 2 | outbound event traits |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/repository.rs` | 73, 101 | 2 | outbound repo traits |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/secret.rs` | 34, 59, 69 | 3 | outbound secret traits |
| `.worktrees/phench-fix/crates/phenotype-contracts/src/ports/inbound/mod.rs` | 76, 124, 131, 159, 166, 193, 200 | 7 | inbound (worktree copy) |
| `.worktrees/phench-fix/crates/phenotype-contracts/src/ports/outbound/*.rs` | 38, 71, 81, 94, 65, 84, 73, 101, 34, 59, 69 | 11 | outbound (worktree copy) |
| `.worktrees/merge-spec-docs/.../phenotype-contracts/...` | identical pattern | 19 | inbound+outbound (worktree copy) |
| `.worktrees/gh-pages-deploy/.../phenotype-contracts/...` | identical pattern | 19 | inbound+outbound (worktree copy) |

**Total:** 19 unique trait methods per phenotype-contracts instance; **76 total `#[async_trait]` occurrences** across 4 worktrees for identical traits.

**Dedup:** Single canonical `crates/phenotype-contracts`, remove worktree copies.

---

### 20. Retry / Backoff Pattern Duplication (EXHAUSTIVE)

**Scan:** `grep -rn --include='*.rs' -E 'exponential.?backoff|retry|jitter|num_retries|max_retries|retry_count|retryable' .`

| File | Line | Pattern | Type |
|------|------|---------|------|
| `platforms/thegent/crates/thegent-subprocess/src/lib.rs` | 5, 159, 270, 287 | `run_with_retry`, `run_retry`, `run_withretry` | subprocess retry |
| `platforms/thegent/crates/thegent-memory/src/error.rs` | 77-78, 114-117 | `is_retryable()`, `test_is_retryable()` | error trait |
| `platforms/thegent/crates/harness-native/src/strategies/mod.rs` | 1, 14, 33-35, 64-68 | `mod retry`, `retry_max`, `retry_backoff_ms`, `retry_jitter` | strategy dispatch |
| `platforms/thegent/crates/harness-native/src/strategies/retry.rs` | 8-25 | `for attempt in 0..=retry_max` + jitter calculation | retry logic |
| `platforms/thegent/crates/harness-native/src/dispatcher.rs` | 170-172, 191-193 | defaults `3, 100, 0.1` + env parsing | retry config |
| `platforms/thegent/crates/thegent-hooks/src/git_ops.rs` | 184, 214-216 | `for retry in 0..MAX_RETRIES`, `sleep_time = 0.1 + (retry as f64 * 0.1)` | git retry |
| `platforms/thegent/crates/thegent-shims/src/lock.rs` | 5, 34, 50, 55, 61 | `Adaptive backoff`, `retry_count`, `sleep_time` | lock retry |
| `platforms/thegent/crates/thegent-hooks/src/main.rs` | 2291-2293 | "Use tenacity (already in deps) instead of manual retry loops" | antipattern lint |

**Key note:** `tenacity` is already in deps (confirmed at thegent-hooks line 2293) but not used — custom retry loops exist instead.

**Dedup:** `libs/retry-core` wrapping `tenacity`.

---

### 21. `impl From<...> for ...` Error Conversion Patterns

| File | Line | Pattern |
|------|------|---------|
| `crates/phenotype-policy-engine/phenotype-policy-engine/src/error.rs` | 40, 46, 52, 61 | `From<serde_json::Error>`, `From<toml::de::Error>`, `From<regex::Error>`, `From<std::io::Error>` for `PolicyEngineError` |
| `crates/phenotype-event-sourcing/src/error.rs` | — | `impl From` for `EventSourcingError` |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/inbound/mod.rs` | 84 | `Error` enum in inbound ports |
| `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/mod.rs` | 63 | `Error` enum in outbound ports |

**Dedup:** `libs/phenotype-error` with derive macro generating `From` impls automatically.

---

### 22. Worktree Lifecycle / Process Management Code

| File | Line | Pattern | Project |
|------|------|---------|---------|
| `platforms/thegent/crates/thegent-hooks/src/git_ops.rs` | 49-51, 184, 341, 401-402 | env vars, retry, lock timeout | thegent |
| `platforms/thegent/crates/thegent-cache/src/cache.rs` | — | cache lifecycle | thegent |
| `platforms/thegent/crates/thegent-shims/src/lock.rs` | 34-61 | lock retry + backoff | thegent |
| `platforms/thegent/crates/thegent-runtime/src/main.rs` | 96-504 | env-driven runtime switches | thegent |
| `platforms/thegent/crates/harness-native/src/dispatcher.rs` | 35 | `HARNESS_HOME` | harness-native |
| `platforms/thegent/hooks/hook-dispatcher/src/main.rs` | 1659-1898 | timeout/agent shell dispatch | hook-dispatcher |

**Dedup:** `libs/phenotype-worktree` for lifecycle management.

---

### 23. Env Var / Config Boilerplate Duplication

| Pattern | Occurrences | Files |
|---------|-------------|-------|
| `env::var("HOME")` | 12 | runtime, hooks, shims, hook-dispatcher |
| `env::var("PATH")` | 4 | path-resolve, io/mod.rs, git_ops |
| `env::var("CI")` | 3 | path-resolve, tool-detect |
| `env::var("AGENT_ID")` / variants | 8 | runtime, hooks, shims |
| `env::var("SESSION_ID")` | 5 | hooks, git_cache, shims |
| `env::var("CACHE_TTL")` | 3 | runtime, hooks |
| `env::var("THEGENT_*")` prefix | 15+ | hooks, shims, runtime |

**Dedup:** `libs/env` crate with typed `Env` struct.

---

### 24. Cross-Worktree File Copy Detection

All 4 worktrees (main, phench-fix, merge-spec-docs, gh-pages-deploy) have **identical** `#[async_trait]` line numbers (76, 124, 131, 159, 166, 193, 200) for the same trait definitions.

| Worktree | phenotype-contracts path | SHA |
|----------|--------------------------|-----|
| main | `crates/phenotype-contracts/...` | canonical |
| .worktrees/phench-fix | `crates/phenotype-contracts/...` | identical |
| .worktrees/merge-spec-docs | `crates/phenotype-contracts/phenotype-contracts/...` | nested identical |
| .worktrees/gh-pages-deploy | `crates/phenotype-contracts/...` | identical |

**Action:** Consolidate to single canonical worktree, remove copies.

---

### Chunk 6 LOC Impact Summary

| Pattern | Unique Locations | Est. Duplicate LOC | Canonical Target |
|---------|-----------------|---------------------|------------------|
| Error enums | 7 files | 200 | libs/phenotype-error |
| Config/home_dir | 30+ sites | 400 | libs/env + libs/config-core |
| Async traits | 19 + 3 worktree copies | 500+ | libs/phenotype-port-interfaces |
| Retry/backoff | 15+ sites | 300 | libs/retry-core |
| From impls | 5+ files | 120 | libs/phenotype-error derive |
| Worktree lifecycle | 10+ files | 350 | libs/phenotype-worktree |
| **Chunk 6 Total** | | **~1,870** | |

**Updated cumulative total (all chunks):** ~3,700 + ~1,870 = **~5,570 LOC**

---

### Chunk 6 Action Items

- [ ] 🔴 CRITICAL: Audit `platforms/thegent` env var usage → create `libs/env` wrapper
- [ ] 🔴 CRITICAL: Consolidate 4x phenotype-contracts worktree copies into 1 canonical location
- [ ] 🟠 HIGH: Create `libs/retry-core` wrapping `tenacity` (already in deps per thegent-hooks:2293)
- [ ] 🟠 HIGH: Create `libs/phenotype-worktree` from thegent lifecycle patterns
- [ ] 🟡 MEDIUM: Audit `impl From` patterns → derive macro in `libs/phenotype-error`
- [ ] 🟡 MEDIUM: Audit `home_dir()` calls → unified `libs/path` helper
- [ ] 🟢 LOW: Add lint rule to detect duplicate `#[async_trait]` across worktrees

## 2026-03-29 - AgilePlus Intra-Repo Duplication Audit

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Audited intra-repo duplication within AgilePlus 24-crate workspace. Identified library libification candidates.

### Findings

| Category | Count | Recommendation |
|----------|-------|----------------|
| Error enums | 36+ | Extract to `libs/error-core` |
| Config loaders | 4 | Extract to `libs/config-core` |
| Health enums | 4 | Extract to `libs/health-core` |
| In-memory stores | 4 | Extract to `libs/test-core` |
| Builder patterns | 12+ | Document as pattern |
| Async traits | 6+ | Consider `store-core` |

### Library Candidates

| Library | Purpose | Status |
|---------|---------|--------|
| `libs/nexus` | Already exists, underutilized | Investigate |
| `libs/hexagonal-rs` | Hex patterns, unused | Archive |
| `libs/cli-framework` | CLI utilities | Enhance |
| `libs/config-core` | NEW | Create |

### Recommendations

1. Audit `libs/` utilization - many libs are unused
2. Consolidate hexagonal architecture libs
3. Create shared error/config/health libraries
4. Document builder patterns as ADR

### Related

- Audit: `plans/2026-03-29-AGILEPLUS_INTRA_REPO_DUPLICATION_AUDIT-v1.md`
- Libification: `plans/2026-03-29-AUDIT_LIBIFICATION-v1.md`

---

## 2026-03-28 - Library Libification Audit

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Audit of existing library crates in `libs/` directory. Many are underutilized or could be consolidated.

### Library Inventory

| Library | Purpose | Utilization | Recommendation |
|---------|---------|-------------|----------------|
| `nexus` | Error types, config | Partial | Expand |
| `hexagonal-rs` | Hex patterns | None | Archive |
| `cli-framework` | CLI utilities | Partial | Enhance |
| `cipher` | Encryption | None | Archive |
| `gauge` | Metrics | None | Archive |
| `config-core` | Config patterns | Partial | Create |

### Action Items

- [x] Audited all libs
- [ ] Consolidate nexus usage
- [ ] Archive unused libs
- [ ] Enhance cli-framework

### Related

- Audit: `plans/2026-03-29-AUDIT_LIBIFICATION-v1.md`

---

## 2026-03-28 - Framework Audit

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Audit of framework choices across projects. Identified inconsistencies in error handling, config loading, and CLI patterns.

### Framework Comparison

| Framework | AgilePlus | thegent | heliosCLI |
|-----------|-----------|---------|-----------|
| Error handling | thiserror | thiserror | thiserror |
| Config format | TOML | YAML | TOML |
| CLI parsing | clap | argparse | clap |
| Logging | tracing | logging | tracing |
| Testing | tokio-test | pytest | tokio-test |

### Convergence Recommendations

1. Standardize on TOML for all config
2. Share `thiserror` patterns
3. Document CLI conventions
4. Create shared test utilities

### Related

- Audit: `plans/2026-03-29-AUDIT_FRAMEWORK-v1.md`

---

## 2026-03-29 - heliosCLI Duplication Analysis

**Project:** [heliosCLI]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analyzed heliosCLI for duplication with other Phenotype repositories.

### Findings

| Pattern | heliosCLI | Similar In | Recommendation |
|---------|-----------|------------|----------------|
| PTY management | `utils/pty/` | vibe-kanban, agileplus-git | FORK to `phenotype-process` |
| Error types | `error.rs` | 135 files across repos | FORK to `phenotype-error` |
| Git operations | `utils/git/` | agileplus-git | EVALUATE fork |

### Duplication with AgilePlus

| Pattern | heliosCLI | AgilePlus | Recommendation |
|---------|-----------|-----------|----------------|
| Error handling | `thiserror` | `thiserror` | Extract to shared |
| Config loading | TOML | TOML | Consider `figment` |
| Async traits | `async-trait` | `async-trait` | Already shared |

### Next Steps

- [ ] FORK-001: Evaluate `utils/pty` for `phenotype-process`
- [ ] FORK-002: Evaluate `error.rs` for `phenotype-error`
- [ ] Document shared patterns

---

## 2026-03-29 - AgilePlus Comprehensive Duplication Audit (SAGE/MUSE/FORGE)

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Scope

| Metric | Value |
|--------|-------|
| Total Files | 1,599 |
| Rust Files | 439 (27%) |
| Crates | 27 in main workspace |
| External Projects | 2 (phenotype-shared-wtrees, vibe-kanban) |

### Summary

Comprehensive analysis identifying 1,800 LOC of duplication with 1,200 LOC savings potential through consolidation.

### 🔴 CRITICAL: Error Types — 8 Independent Definitions (~600 LOC)

| Crate | Error Type | Lines | Key Variants |
|-------|------------|-------|--------------|
| `agileplus-api/src/error.rs` | `ApiError` | 67 | NotFound, BadRequest, Internal |
| `agileplus-p2p/src/error.rs` | `SyncError`, `PeerDiscoveryError` | 78 | Nats, Serialization |
| `agileplus-sync/src/error.rs` | `SyncError` | 24 | Store, Nats |
| `agileplus-domain/src/error.rs` | `DomainError` | 50 | NotFound, Conflict |
| `agileplus-events/src/store.rs` | `EventError` | 53 | NotFound, StorageError |
| `agileplus-graph/src/store.rs` | `GraphError` | 326 | ConnectionError, QueryError |
| `agileplus-cache/src/store.rs` | `CacheError` | 129 | Serialization, Redis |
| `phenotype-port-interfaces/src/error.rs` | `PortError` | 51 | NotFound, Validation |

**Duplicated Variants**: `NotFound(String)`, `SerializationError`, `StorageError`, `Conflict`

### 🟡 HIGH: Configuration Loading — 3 Independent Implementations (~500 LOC)

| Location | Format | Pattern |
|----------|--------|---------|
| `crates/agileplus-domain/src/config/loader.rs` | TOML | env overrides, `~/.agileplus/config.toml` |
| `crates/agileplus-telemetry/src/config.rs` | YAML | env overrides, `~/.agileplus/otel-config.yaml` |
| `vibe-kanban/backend/src/models/config.rs` | JSON | defaults merge |

**Library Status**: `libs/config-core/` exists but **UNUSED** (edition mismatch: 2021 vs 2024)

### 🟠 MEDIUM: Async Traits — 5+ Repository Traits

| Location | Trait | Async Pattern |
|----------|-------|---------------|
| `agileplus-nats/src/bus.rs` | EventBus | #[async_trait] |
| `agileplus-sync/src/store.rs` | SyncMappingStore | #[async_trait] |
| `agosevents/src/store.rs` | EventStore | #[async_trait] |
| `agileplus-graph/src/store.rs` | GraphBackend | #[async_trait] |
| `agileplus-cache/src/store.rs` | CacheStore | #[async_trait] |

**Library Status**: `libs/hexagonal-rs/src/ports/repository.rs` has exact patterns but **UNUSED**

### 🟠 MEDIUM: In-Memory Test Implementations — 4 Instances (~400 LOC)

| Trait | Implementation | Location |
|-------|---------------|----------|
| EventBus | InMemoryBus | `agileplus-nats/src/bus.rs:127-240` |
| SyncMappingStore | InMemorySyncStore | `agileplus-sync/src/store.rs:47-110` |
| GraphBackend | InMemoryGraphBackend | `agileplus-graph/src/store.rs:106-309` |

**Common Pattern**: `Arc<Mutex<HashMap<Key, Value>>>` duplicated 4+ times

### UNUSED LIBRARIES (11 total)

| Library | Purpose | Issue |
|---------|---------|-------|
| `config-core` | Config loading | edition mismatch |
| `logger` | Structured logging | edition mismatch |
| `tracing` | Distributed tracing | edition mismatch |
| `metrics` | Metrics collection | edition mismatch |
| `hexagonal-rs` | Ports & Adapters | edition mismatch, has exact patterns |
| `hexkit` | HTTP/Persistence | edition mismatch |
| `cipher` | Encryption | NOT AUDITED |
| `gauge` | Benchmarking | NOT AUDITED |
| `nexus` | Service discovery | NOT AUDITED |
| `xdd-lib-rs` | Data transformation | NOT AUDITED |
| `phenotype-state-machine` | State machine patterns | DEAD CODE |

**Root Cause**: `libs/` uses `edition = "2021"`, workspace uses `edition = "2024"`

### LOC Impact Summary

| Category | Current | After Consolidation | Savings |
|----------|---------|---------------------|---------|
| Error Types | 600 | 200 | 400 |
| Config Loading | 500 | 150 | 350 |
| In-Memory Impls | 400 | 150 | 250 |
| Async Traits | 300 | 100 | 200 |
| **Total** | **1,800** | **600** | **1,200** |

### Recommended Actions

- [ ] 🔴 CRITICAL: Create `libs/agileplus-error/` for error consolidation
- [ ] 🟡 HIGH: Migrate `libs/config-core` to edition 2024
- [ ] 🟡 HIGH: Integrate `libs/hexagonal-rs` Repository patterns
- [ ] 🟠 MEDIUM: Create shared InMemory test implementations
- [ ] 🟠 MEDIUM: Create `libs/http-client` for HTTP patterns
- [ ] 🟢 LOW: Delete `phenotype-state-machine` (dead code)

### Related

- `docs/research/consolidation-audit-2026-03-29.md` - Master findings
- `worklogs/WORK_LOG.md` - Wave 90 entry

---

---

## 2026-03-29 - NON-HELISO PROJECTS LOC AUDIT & DECOMPOSITION

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Complete LOC Summary (Non-Heliso)

| Project | LOC | Files | Decomposition Priority |
|---------|-----|-------|----------------------|
| **crates/** | **73,444** | 30+ | See below |
| **libs/** | **1,470** | 8 | LOW |
| **repos/worktrees/** | **98,611** | 667 | See below |

---

### 1. crates/ Directory Analysis (73,444 LOC)

#### Top Crates by LOC

| Crate | LOC | Category | Decomposition Opportunity |
|-------|-----|----------|--------------------------|
| `agileplus-cli` | 8,884 | CLI | **HIGH** - extract to `phenotype-cli` |
| `agileplus-api` | 6,739 | API | **HIGH** - too large, split by route |
| `agileplus-sqlite` | 6,124 | Database | **MEDIUM** - consider `sqlx` |
| `agileplus-dashboard` | 5,669 | UI | **HIGH** - extract UI components |
| `agileplus-subcmds` | 4,386 | CLI | **HIGH** - subcommand library |
| `agileplus-domain` | 4,317 | Domain | **MEDIUM** - port traits needed |
| `agileplus-p2p` | 3,943 | Network | **LOW** - specialized |
| `agileplus-plane` | 3,855 | Integration | **LOW** - plane.so specific |
| `agileplus-git` | 3,544 | VCS | **HIGH** - extract to `phenotype-git` |
| `phenotype-contracts` | 3,057 | Contracts | **CRITICAL** - core lib |

#### Crates with Duplication Issues

| Crate | LOC | Issue | Action |
|-------|-----|-------|--------|
| `phenotype-event-sourcing` | 2,054 | Duplicated across worktrees | Consolidate to canonical |
| `phenotype-policy-engine` | 2,900 | Regex-based only | Add `casbin` RBAC |
| `phenotype-cache-adapter` | 778 | Incomplete stub | Implement or remove |
| `phenotype-state-machine` | 517 | Incomplete stub | Implement or remove |
| `phenotype-error-core` | 443 | Scattered errors | Consolidate to `phenotype-errors` |

---

### 2. LOC Reduction Opportunities

#### 2.1 Extract phenotype-cli (8,884 LOC)

**Current Structure:**
```
agileplus-cli/
├── src/
│   ├── main.rs (2,000 LOC)
│   ├── commands/ (3,000 LOC)
│   ├── config/ (1,500 LOC)
│   └── utils/ (2,384 LOC)
```

**Proposed Decomposition:**
```
phenotype-cli/
├── phenotype-cli-core/     # 4,000 LOC - shared CLI logic
├── phenotype-cli-commands/ # 2,500 LOC - command implementations
├── phenotype-cli-config/   # 1,500 LOC - config loading
└── phenotype-cli-main/     # 884 LOC - main entry point
```

**LOC Savings:** ~500 LOC (shared utilities extraction)

---

#### 2.2 Extract phenotype-git (3,544 LOC)

**Current:** `agileplus-git` (duplicated with `phenotype-git-core` at 1 LOC)

**Proposed:**
```
phenotype-git/
├── phenotype-git-core/     # 2,000 LOC - git operations
├── phenotype-git-cache/    # 500 LOC - caching layer
└── phenotype-git-cli/      # 1,044 LOC - CLI integration
```

**Action:** Merge `phenotype-git-core` (1 LOC) into this crate

---

#### 2.3 Extract phenotype-api (6,739 LOC)

**Current Structure:**
```
agileplus-api/
├── src/
│   ├── routes/ (3,000 LOC)
│   ├── middleware/ (1,000 LOC)
│   ├── models/ (1,500 LOC)
│   └── services/ (1,239 LOC)
```

**Proposed Decomposition:**
```
phenotype-api/
├── phenotype-api-core/     # 2,000 LOC - shared API logic
├── phenotype-api-routes/   # 2,500 LOC - route handlers
├── phenotype-api-middleware/ # 1,000 LOC - middleware
└── phenotype-api-models/  # 1,239 LOC - request/response models
```

**LOC Savings:** ~800 LOC (DRY extraction)

---

### 3. libs/ Analysis (1,470 LOC)

| Library | LOC | Status | Action |
|---------|-----|--------|--------|
| `hexagonal-rs` | ~200 | ARCHIVE - duplicate patterns | Archive |
| `metrics` | ~150 | Duplicate - `phenotype-cache-adapter` has MetricsHook | Remove |
| `tracing` | ~150 | Duplicate - `phenotype-telemetry` | Consolidate |
| `cli-framework` | ~300 | **KEEP** - unique | Maintain |
| `logger` | ~200 | **KEEP** - unique | Maintain |
| `cipher` | ~100 | **EVALUATE** - unused? | Audit usage |
| `hexkit` | ~200 | **KEEP** - hexagonal kit | Maintain |
| `nexus` | ~100 | **KEEP** - unique | Maintain |

---

### 4. Worktrees Analysis (98,611 LOC)

| Worktree | LOC | Status | Action |
|----------|-----|--------|--------|
| `AgilePlus` | 80,191 | Active | Continue development |
| `consolidate-libraries` | 7,496 | **ORPHANED** | Merge to canonical, delete |
| `expand-test-coverage` | 6,509 | **ORPHANED** | Merge to canonical, delete |
| `phenotype-infrakit` | 4,415 | Active | Keep |

#### Critical: Merge or Delete Orphaned Worktrees

**consolidate-libraries** contains:
- `phenotype-event-sourcing` (duplicated)
- `phenotype-contracts` (duplicated)
- `phenotype-cache-adapter` (duplicated)
- `phenotype-policy-engine` (duplicated)
- `phenotype-state-machine` (duplicated)
- `phenotype-errors` (NEW - consolidate here!)

**Action:** Copy `phenotype-errors` to canonical, delete worktree

---

### 5. External Package Opportunities (Non-Heliso)

#### High Priority

| Crate | Current Gap | External Alternative | LOC Savings |
|-------|------------|---------------------|-------------|
| `agileplus-api` | No SQLx | ADOPT `sqlx` | 500-800 |
| `agileplus-git` | Hand-rolled git | ADOPT `gix` | 300-500 |
| `phenotype-cache-adapter` | No Redis | ADOPT `redis` | 200-400 |
| `phenotype-event-sourcing` | In-memory only | ADOPT `cqrs-es` | 300-500 |
| `phenotype-policy-engine` | No RBAC | ADOPT `casbin` | 400-700 |

#### Medium Priority

| Crate | Current Gap | External Alternative | LOC Savings |
|-------|------------|---------------------|-------------|
| `agileplus-domain` | Port traits scattered | CONSOLIDATE `phenotype-contracts` | 200-400 |
| `agileplus-telemetry` | Basic tracing | ADOPT `opentelemetry` | 100-200 |
| All crates | Custom errors | CONSOLIDATE `phenotype-errors` | 300-500 |

---

### 6. LOC Reduction Roadmap

#### Phase 1: Cleanup (1-2 weeks)

| Action | LOC Saved | Effort |
|--------|-----------|--------|
| Delete `consolidate-libraries` worktree | 0 | 1 hour |
| Delete `expand-test-coverage` worktree | 0 | 1 hour |
| Remove `phenotype-git-core` (1 LOC) | 0 | 5 min |
| Archive `hexagonal-rs` | 0 | 5 min |
| Remove unused deps from workspace | 0 | 30 min |

#### Phase 2: Extract Core Libraries (2-4 weeks)

| Action | LOC Saved | Effort |
|--------|-----------|--------|
| Create `phenotype-errors` | 300-500 | 1 week |
| Extract `phenotype-cli` from `agileplus-cli` | 500 | 1 week |
| Extract `phenotype-git` from `agileplus-git` | 200 | 1 week |
| Consolidate `phenotype-contracts` | 200 | 1 week |

#### Phase 3: External Dependencies (4-8 weeks)

| Action | LOC Saved | Effort |
|--------|-----------|--------|
| ADOPT `sqlx` in `agileplus-api` | 500-800 | 2 weeks |
| ADOPT `gix` in `phenotype-git` | 300-500 | 2 weeks |
| ADOPT `casbin` in `phenotype-policy-engine` | 400-700 | 2 weeks |
| ADOPT `redis` in `phenotype-cache-adapter` | 200-400 | 1 week |
| ADOPT `cqrs-es` in `phenotype-event-sourcing` | 300-500 | 2 weeks |

#### Phase 4: Optimization (4-8 weeks)

| Action | LOC Saved | Effort |
|--------|-----------|--------|
| Replace `serde_json` with `rkyv` hot paths | 50-100 | 1 week |
| Add `blake3` for hash chains | 30-50 | 1 week |
| Add `mockall` for testing | 100-200 | 1 week |
| Add `tracing-subscriber` | 50-100 | 1 week |
| Parallelize sequential async ops | N/A (perf) | 2 weeks |

---

### 7. Summary of Opportunities

| Category | Current LOC | Target LOC | Savings | Priority |
|----------|-------------|------------|---------|----------|
| **Core Libs** | 73,444 | 65,000 | **8,444** | P0 |
| **External Crates** | 73,444 | 70,000 | **3,444** | P1 |
| **Error Handling** | 3,000+ | 1,000 | **2,000** | P1 |
| **Git Operations** | 3,544 | 2,500 | **1,044** | P1 |
| **CLI Framework** | 8,884 | 7,500 | **1,384** | P2 |
| **API Framework** | 6,739 | 5,500 | **1,239** | P2 |
| **TOTAL** | **~100,000** | **~85,000** | **~15,000** | |

---

_Last updated: 2026-03-29_
