# Phenotype Ecosystem Consolidation Audit
Date: 2026-03-29
Last Updated: 2026-03-29 (AgilePlus Rust Monorepo Analysis)

---

## AgilePlus Rust Monorepo Audit (2026-03-29)

### Scope
- **27 crates** in main workspace
- **439 Rust files** (~27% of 1,599 total files)
- **2 external projects**: phenotype-shared-wtrees, vibe-kanban

### HIGH PRIORITY: Error Types — 8 Independent Definitions (~600 LOC)

| Crate | Error Type | Lines | Key Variants |
|-------|------------|-------|--------------|
| `agileplus-api/src/error.rs` | `ApiError` | 67 | NotFound, BadRequest, Internal |
| `agileplus-p2p/src/error.rs` | `SyncError`, `PeerDiscoveryError`, `ConnectionError` | 78 | Nats, Serialization |
| `agileplus-sync/src/error.rs` | `SyncError` | 24 | Store, Nats |
| `agileplus-domain/src/error.rs` | `DomainError` | 50 | NotFound, Conflict |
| `agileplus-events/src/store.rs` | `EventError` | 53 | NotFound, StorageError |
| `agileplus-graph/src/store.rs` | `GraphError` | 326 | ConnectionError, QueryError |
| `agileplus-cache/src/store.rs` | `CacheError` | 129 | Serialization, Redis |
| `phenotype-port-interfaces/src/error.rs` | `PortError` | 51 | NotFound, Validation |

**Duplicated Variants**: `NotFound(String)`, `SerializationError`, `StorageError`, `Conflict`

**Recommendation**: Create `libs/agileplus-error/` with composable error variants.

### HIGH PRIORITY: Configuration Loading — 3 Independent Implementations (~500 LOC)

| Location | Format | Pattern |
|----------|--------|---------|
| `crates/agileplus-domain/src/config/loader.rs` | TOML | env overrides, `~/.agileplus/config.toml` |
| `crates/agileplus-telemetry/src/config.rs` | YAML | env overrides, `~/.agileplus/otel-config.yaml` |
| `vibe-kanban/backend/src/models/config.rs` | JSON | defaults merge |

**Library Status**: `libs/config-core/` exists but **UNUSED** (edition mismatch: 2021 vs 2024)

### MEDIUM PRIORITY: Async Traits — 5+ Repository Traits

| Location | Trait | Async Pattern |
|----------|-------|---------------|
| `agileplus-nats/src/bus.rs` | EventBus | #[async_trait] |
| `agileplus-sync/src/store.rs` | SyncMappingStore | #[async_trait] |
| `agileplus-events/src/store.rs` | EventStore | #[async_trait] |
| `agileplus-graph/src/store.rs` | GraphBackend | #[async_trait] |
| `agileplus-cache/src/store.rs` | CacheStore | #[async_trait] |

**Library Status**: `libs/hexagonal-rs/src/ports/repository.rs` has exact patterns but **UNUSED**

### MEDIUM PRIORITY: In-Memory Test Implementations — 4 Instances (~400 LOC)

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

---

## P1 — Eliminate Entire Repos (merge into canonical)

1. **helix-logging + helix-tracing** → subsume into `tracely` → rename to `phenotype-observability`
2. **thegent-cache (FacetRs)** → moka already in phenotype-shared; expose `phenotype-shared::cache` module; archive repo
3. **phenotype-cipher** → merge into `phenotype-config/pheno-crypto`
4. **hexagon-go** → merge template into `template-lang-go`; keep `go-hex` as pure lib

## P2 — Add Shared Module, Remove Per-Repo Impls

5. Error types → `phenotype-shared/crates/phenotype-errors`
6. CLI kit (Rust) → merge clikit + apikit → `phenotype-shared/crates/phenotype-cli`
7. Go logging → standardize cliproxyapi-plusplus + KodeVibe-Go on slog stdlib
8. Python HTTP → standardize zen + portage on httpx

## P3 — Wrap 3rd-Party Libs

9. phenotype-vessel → wrap bollard crate (~50 LOC vs ~500)
10. phenotype-patch → wrap `similar` crate
11. phenotype-sentinel → wrap `governor` + `circuit-breaker`
12. pheno-crypto/phenotype-cipher → wrap RustCrypto chacha20poly1305 + age

## P4 — Version Bumps (DONE via PRs)

- serde_yaml 0.9 → serde-yaml-ng 0.10 (phenotype-gauge, phenotype-xdd-lib, tokenledger)
- thiserror 1.0 → 2 (phenotype-cipher, phenotype-forge, phenotype-nexus, phenotype-xdd-lib, clikit, thegent-plugin-host)
- tokio unpin from 1.35 → ^1 (thegent-plugin-host)
- logrus → slog stdlib (cliproxyapi-plusplus, KodeVibe-Go)
- phenotype-cli-core + hexagon-go Go 1.21 → 1.26
- KWatch bubbletea v0.25 → v1.3.x

## Duplication Matrix

| Pattern | Own impls | Canonical | Action |
|---------|-----------|-----------|--------|
| Rust logging/tracing | helix-logging, helix-tracing, tracely, AgilePlus | tracely → phenotype-observability | P1 |
| Go logging | logrus (2 repos) | slog stdlib | P4 done |
| Rust config | phenotype-config, AgilePlus creds | phenotype-config/pheno-crypto | P2 |
| Rust cache | thegent-cache, phenotype-shared inline | phenotype-shared/phenotype-cache | P1 |
| Rust crypto | phenotype-cipher, pheno-crypto | pheno-crypto (consolidated) | P1 |
| Go hex arch | go-hex, hexagon-go, template-lang-go | go-hex lib + template-lang-go | P1 |
| Rust errors | every repo | phenotype-errors (new) | P2 |
| Rust CLI kit | clikit, apikit, AgilePlus bin | phenotype-shared/phenotype-cli | P2 |
| Python CLI | thegent, phench, portage | phenotype-cli-py (new) | P2 |

## 3rd-Party Replacement Candidates

| Hand-rolled | OSS replacement | LOC saved | Priority |
|-------------|----------------|-----------|---------|
| phenotype-vessel | bollard v0.18 | ~500 | HIGH |
| phenotype-patch | similar v2.7 | ~300 | MED |
| phenotype-sentinel | governor + circuit-breaker | ~400 | MED |
| pheno-crypto AES | chacha20poly1305 + age | ~200 | HIGH (security) |
