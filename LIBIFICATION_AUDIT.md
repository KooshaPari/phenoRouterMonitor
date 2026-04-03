# Libification Audit Report — Non-Excluded Repos

**Audit Date:** 2026-04-03  
**Scope:** Repos NOT in {Helios[lab,cli,app], portage, thegent, agileplus, tracera, agentapi++, cliproxyapi++}

---

## Executive Summary

The audit identified **significant duplication** across non-excluded repos with ~850+ LOC of duplicated error handling, 186 LOC of retry logic, and cross-language model duplication. The good news: **30 reusable crates already exist** in the `crates/` workspace as a model.

### Priority Recommendations

| Priority | Action | Est. Savings |
|----------|--------|-------------|
| **P1** | Promote `phenotype-error-core` adoption | 400-500 LOC |
| **P1** | Adopt `phenotype-retry` across all crates | 140 LOC |
| **P2** | Extract distributed auth logic → `phenotype-auth` | 600 LOC |
| **P2** | Create `serde-adapters` lib for serialization | 273 LOC |
| **P3** | Fix `config-core` edition mismatch | 350 LOC |

---

## 1. Repos Already Well-Libified

### ✅ crates/ (30 Crates — MODEL FOR LIBIFICATION)

**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/crates`

This workspace is **the benchmark** — 30 independently consumable libraries following ADR-001 (no cross-crate source dependencies):

| Crate | Purpose | Status |
|-------|---------|--------|
| `phenotype-event-sourcing` | Append-only event store + SHA-256 hash chains | ✅ |
| `phenotype-cache-adapter` | Two-tier LRU + DashMap cache w/ TTL | ✅ |
| `phenotype-policy-engine` | Rule-based policy evaluation (TOML) | ✅ |
| `phenotype-state-machine` | Generic FSM w/ transition guards | ✅ |
| `phenotype-contracts` | Shared traits and types | ✅ |
| `phenotype-error-core` | Canonical error types | ⚠️ UNUSED |
| `phenotype-retry` | Retry pattern (329 LOC) | ⚠️ UNDERUTILIZED |
| `phenotype-git-core` | Git operations via gix | ✅ |
| `phenotype-http-client-core` | HTTP transport + auth | ✅ |
| `phenotype-cost-core` | Token counting, provider pricing | ✅ |
| `phenotype-config-core` | Configuration management | ⚠️ EDITION MISMATCH |
| `phenotype-string` | Case conversion, slugify, inflection | ✅ |

---

## 2. High-Priority Libification Candidates

### 2.1 phenoSDK — HIGH PRIORITY

**Location:** `python/phenosdk/`

**Structure:**
```
phenosdk/
├── pheno-core/      # Core abstractions & FastMCP wrapper
├── pheno-mcp/       # MCP transport & tool registry ← HIGHEST FAN-IN
├── pheno-shared/    # Shared Pydantic models
└── pheno-plugins/   # Dynamic tool collections
```

**Duplicate with:**
- `agileplus-mcp` — MCP structure duplication
- `agileplus-shared` — Shared model duplication

**Extraction Candidates:**
1. `pheno-mcp` base package → standalone library (P1)
2. `pheno-shared` → sync with Rust via buf (P2)
3. Cross-language schemas: `EventEnvelope`, `AuditEntry` duplicated (135 LOC across Rust/Python/Go)

---

### 2.2 Distributed Auth Logic — HIGH PRIORITY

**Status:** NOT a standalone repo — functionality distributed across 4+ implementations

| Implementation | Location | LOC |
|----------------|----------|-----|
| AgilePlus | `agileplus-auth/` | 450 |
| thegent | `thegent-auth/` | 280 |
| heliosCLI | `codex-rs/core/auth.rs` | 320 |
| pheno-cli | `python/pheno-cli/auth.py` | 95 |
| HTTP Client | `crates/phenotype-http-client-core/src/auth.rs` | ~60 |

**Extraction Candidate:** `crates/phenotype-auth`
```rust
// Target structure
crates/phenotype-auth/src/lib.rs
├── pub mod jwt;           // JwtValidator, JwtClaims
├── pub mod session;       // SessionManager, Session
├── pub mod middleware;    // auth_middleware
└── pub mod permissions;
```

**Estimated Savings:** ~600 LOC

---

### 2.3 Distributed Git Operations — HIGH PRIORITY

**Status:** 6+ implementations across codebase

| Location | Operations | LOC |
|----------|-----------|-----|
| thegent-git | clone, checkout, commit, push, fetch | 709 |
| thegent-shims | git_checkout wrapper | 85 |
| thegent-hooks | git operations | 156 |
| agileplus-sync | git sync | 72 |
| heliosCLI | git via libgit2 | 95 |

**Status:** `phenotype-git-core` already exists (uses gix/gitoxide)

**Action:** Consolidate all 6+ implementations to use `phenotype-git-core`

---

## 3. Duplicate Code Patterns — Detailed

### 3.1 Error Handling — CRITICAL (~850 LOC duplicated)

**15+ independent error enums** with duplicated variants:

| Crate | Error Type | Duplicated Variants |
|-------|------------|---------------------|
| `agileplus-api` | `ApiError` | NotFound, BadRequest, Internal |
| `agileplus-domain` | `DomainError` | NotFound, Conflict, ValidationFailed |
| `agileplus-p2p` | `PeerDiscoveryError` | Nats, Serialization, NotFound |
| `agileplus-cache` | `CacheError` | Serialization, Redis, NotFound |
| `heliosCLI` | `CodexErr` | NotFound, BadRequest |
| `thegent-memory` | `Error` | ConnectionFailed, Timeout |

**Shared Variants (8+ crates):**
- `NotFound(String)` — 8+ crates
- `SerializationError` — 7+ crates
- `StorageError` — 5+ crates
- `Timeout` — 4+ crates
- `ValidationError` — 5+ crates

**Library Available:** `phenotype-error-core` — **UNUSED**

**Action:** Migrate all crates to `phenotype-error-core`, derive `From<...>` conversions automatically

---

### 3.2 Retry/Resilience — HIGH (~186 LOC)

**4 independent implementations:**

| Location | Algorithm | Jitter | LOC |
|----------|-----------|--------|-----|
| `agileplus-api/src/http/retry.rs` | exp(2^n) | ✅ | 44 |
| `agileplus-redis/src/retry.rs` | Linear | ❌ | 38 |
| `heliosCLI/core/src/http/retry.rs` | exp(2^n) | ✅ | 42 |
| `phenotype-event-sourcing/src/retry.rs` | exp(2^n)+cap | ❌ | 62 |

**Library Available:** `phenotype-retry` — **UNDERUTILIZED**

```rust
// Library provides robust builder pattern
use phenotype_retry::{RetryBuilder, Jitter};

RetryBuilder::new()
    .max_attempts(3)
    .base_delay(Duration::from_millis(100))
    .max_delay(Duration::from_secs(30))
    .with_jitter(Jitter::Full)
    .build()
```

**Action:** Audit which crates still use hand-rolled retry → migrate to `phenotype-retry`

---

### 3.3 Serialization — HIGH (~353 LOC)

**Hotspots:**

1. **Encrypted field serialization** — 3 crates with identical patterns (~90 LOC)
   - `agileplus-domain/src/credentials/mod.rs`
   - `agileplus-api/src/models/secret.rs`
   - `heliosCLI/core/src/secret.rs`

2. **MessagePack** — 3 NATS-related crates (~80 LOC)

3. **Cross-language models** — Same models in Rust/Python/Go:
   - `EventEnvelope` — 135 LOC (45 Rust + 38 Python + 52 Go)
   - `AuditEntry` — 58 LOC
   - `ToolCall` — 47 LOC
   - `AgentMessage` — 65 LOC

**Action:** Create `libs/serde-adapters` with reusable modules; implement Protobuf schemas for cross-language models

---

### 3.4 Configuration Loading — MEDIUM (~500 LOC)

| Location | Format | Pattern |
|----------|--------|---------|
| `agileplus-domain/src/config/loader.rs` | TOML | env overrides |
| `agileplus-telemetry/src/config.rs` | YAML | env overrides |
| `vibe-kanban/backend/src/models/config.rs` | JSON | defaults merge |

**Library Available:** `phenotype-config-core` — **EDITION MISMATCH (2021 vs 2024)**

**Action:** Fix edition compatibility, promote adoption

---

## 4. Repos NOT Found (Functionality Distributed)

These requested repos do NOT exist as standalone directories — functionality is distributed:

| Requested | Actual Location |
|-----------|-----------------|
| Authvault | agileplus-auth, thegent-auth, heliosCLI |
| Cmdra | heliosCLI, crates/phenotype-cli |
| Dino | Pattern extraction throughout |
| Planify | python/pheno-mcp |
| Tokn | phenotype-http-client-core |
| Kogito | Pattern extraction throughout |
| phenotype-cli-extensions | Various projects |

---

## 5. Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks)

1. **Migrate to phenotype-retry** — ~140 LOC savings
   - [ ] Audit all crates using hand-rolled retry
   - [ ] Replace with `phenotype-retry` builder
   - [ ] Update docs

2. **Promote phenotype-error-core** — ~400 LOC savings
   - [ ] Add missing variants (ConnectionFailed, Timeout)
   - [ ] Add derive macro for `From<...>` conversions
   - [ ] Migrate top 5 error-heavy crates

### Phase 2: Core Extractions (2-4 weeks)

3. **Extract phenotype-auth** — ~600 LOC savings
   - [ ] Define `Authenticator` trait
   - [ ] Implement JWT validator
   - [ ] Add session management
   - [ ] Create middleware helpers

4. **Fix config-core** — ~350 LOC savings
   - [ ] Resolve edition mismatch (2021 → 2024)
   - [ ] Add env var override support
   - [ ] Promote adoption

### Phase 3: Cross-Cutting (4-6 weeks)

5. **Create serde-adapters** — ~273 LOC savings
   - [ ] Encrypted field serialization helper
   - [ ] MessagePack adapter for NATS
   - [ ] Versioned serialization trait

6. **Protobuf schemas** — Eliminate cross-language duplication
   - [ ] Define `EventEnvelope` schema
   - [ ] Define `AuditEntry` schema
   - [ ] Generate Rust/Python/Go bindings

---

## 6. Metrics Summary

| Metric | Current | Target |
|--------|---------|--------|
| Duplicate error handling LOC | ~850 | ~0 |
| Duplicate retry logic LOC | ~186 | ~0 |
| Unused libraries | 2 | 0 |
| Cross-language model LOC | ~135 | ~50 (protobuf) |
| Config loading implementations | 3 | 1 |

---

## Appendix: Key File Locations

| Library | Path |
|---------|------|
| phenotype-error-core | `crates/phenotype-error-core/src/lib.rs` |
| phenotype-retry | `crates/phenotype-retry/src/builder.rs` |
| phenotype-config-core | `crates/phenotype-config-core/src/lib.rs` |
| phenotype-git-core | `crates/phenotype-git-core/src/lib.rs` |
| phenotype-string | `crates/phenotype-string/src/lib.rs` |
| pheno-mcp | `python/phenosdk/src/pheno/mcp/` |