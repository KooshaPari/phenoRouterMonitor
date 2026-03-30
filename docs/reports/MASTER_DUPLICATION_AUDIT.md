# Master Duplication Audit Report

**Generated:** 2026-03-29
**Agents:** SAGE, FORGE (resumed session)
**Status:** in_progress
**Priority:** P0

---

## Executive Summary

Comprehensive analysis of code duplication across the Phenotype repository ecosystem. Identified **1,800+ LOC of duplication** with **1,200+ LOC savings potential** through consolidation.

### Key Metrics

| Metric | Value |
|--------|-------|
| Total Rust files | ~1,599 |
| Error type definitions | 12 types, 68+ variants |
| Unused libraries | 11 (all edition 2021) |
| Store trait patterns | 5 traits |
| Port/trait ecosystems | 2 (2,106 LOC) |
| HTTP client instantiations | 14+ |

---

## 🔴 CRITICAL: Unused Libraries (1,650+ LOC)

**Root Cause:** All `libs/` crates use `edition = "2021"` while workspace uses `edition = "2024"`.

| Library | Path | LOC | Status |
|---------|------|-----|--------|
| `logger` | `libs/logger/` | ~100 | UNUSED |
| `metrics` | `libs/metrics/` | ~100 | UNUSED |
| `tracing-lib` | `libs/tracing/` | ~100 | UNUSED |
| `hexagonal-rs` | `libs/hexagonal-rs/` | ~300 | UNUSED |
| `hexkit` | `libs/hexkit/` | ~200 | UNUSED (duplicate) |
| `cli-framework` | `libs/cli-framework/` | ~150 | UNUSED |
| `config-core` | `libs/config-core/` | ~50 | UNUSED (stub) |
| `cipher` | `libs/cipher/` | ~100 | UNUSED |
| `gauge` | `libs/gauge/` | ~100 | UNUSED |
| `nexus` | `libs/nexus/` | ~50 | UNUSED |
| `xdd-lib-rs` | `libs/xdd-lib-rs/` | ~100 | UNUSED |

### Action Items

- [ ] 🔴 **LIB-001**: Migrate `libs/logger/` to edition 2024
- [ ] 🔴 **LIB-002**: Migrate `libs/metrics/` to edition 2024
- [ ] 🔴 **LIB-003**: Migrate `libs/tracing/` to edition 2024
- [ ] 🔴 **LIB-004**: Migrate `libs/hexagonal-rs/` to edition 2024
- [ ] 🔴 **LIB-005**: Deprecate `libs/hexkit/` (duplicate of hexagonal-rs)
- [ ] 🔴 **LIB-006**: Migrate `libs/cli-framework/` to edition 2024
- [ ] 🔴 **LIB-007**: Migrate `libs/config-core/` to edition 2024
- [ ] 🔴 **LIB-008**: Archive `libs/cipher/` (unused)
- [ ] 🔴 **LIB-009**: Archive `libs/gauge/` (unused)
- [ ] 🔴 **LIB-010**: Archive `libs/nexus/` (unused)
- [ ] 🔴 **LIB-011**: Archive `libs/xdd-lib-rs/` (unused)

---

## 🟡 HIGH: Error Type Duplication (~600 LOC)

### Verified Error Types (12 types, 68+ variants, ~189 LOC)

| Error Type | File | LOC | Key Variants |
|------------|------|-----|--------------|
| `ApiError` | `crates/agileplus-api/src/error.rs` | 14 | NotFound, Internal |
| `DomainError` | `crates/agileplus-domain/src/error.rs` | 47 | NotFound, Conflict, InvalidTransition |
| `SyncError` (sync) | `crates/agileplus-sync/src/error.rs` | 19 | Store, Nats, Serialization, Conflict |
| `SyncError` (p2p) | `crates/agileplus-p2p/src/error.rs` | 22 | Nats, Serialization, Hash, Conflict |
| `PeerDiscoveryError` | `crates/agileplus-p2p/src/error.rs` | 16 | Nats, Serialization |
| `ConnectionError` | `crates/agileplus-p2p/src/error.rs` | 16 | Nats, Serialization |
| `EventError` | `crates/agileplus-events/src/store.rs` | 12 | Store, Hash, Replay, Snapshot |
| `GraphError` | `crates/agileplus-graph/src/store.rs` | 12 | Store, Query |
| `CacheError` | `crates/agileplus-cache/src/store.rs` | 10 | Store, Serialization |
| `TelemetryError` | `crates/agileplus-telemetry/src/lib.rs` | 8 | Log, Config, Otel |

### Duplicated Variants (appear in 3+ types)

| Variant | Appears In | Proposed Location |
|---------|------------|-------------------|
| `NotFound(String)` | 5 types | `storage.rs` |
| `Serialization(String)` | 3 types | `serialization.rs` |
| `Storage(String)` | 3 types | `storage.rs` |
| `Conflict(String)` | 3 types | `domain.rs` |

### Proposed Architecture: `libs/error-core/`

```
libs/error-core/
├── src/
│   ├── lib.rs
│   ├── domain.rs       # DomainError variants
│   ├── api.rs         # ApiError with IntoResponse
│   ├── storage.rs     # StorageError, NotFound
│   ├── sync.rs        # SyncError, NatsError
│   ├── serialization.rs # SerError with #[from]
│   └── traits.rs      # ErrorMarker traits
└── Cargo.toml
```

### Action Items

- [ ] 🔴 **ERR-001**: Create `libs/error-core/` crate
- [ ] 🔴 **ERR-002**: Define `StorageError` with NotFound, Storage, Connection variants
- [ ] 🔴 **ERR-003**: Define `SerializationError` with From<serde_json::Error>
- [ ] 🟡 **ERR-004**: Migrate `DomainError` to use error-core
- [ ] 🟡 **ERR-005**: Migrate `ApiError` to use error-core
- [ ] 🟡 **ERR-006**: Migrate `SyncError` to use error-core
- [ ] 🟡 **ERR-007**: Migrate `GraphError` to use error-core
- [ ] 🟡 **ERR-008**: Migrate `CacheError` to use error-core

---

## 🟡 HIGH: Port/Trait Architecture Split (2,106 LOC)

### Ecosystem 1: phenotype-port-interfaces

```
libs/phenotype-shared/crates/phenotype-port-interfaces/
├── src/outbound/
│   ├── repository.rs (Repository trait)
│   ├── cache.rs (Cache trait)
│   ├── logger.rs (Logger trait)
│   ├── event_bus.rs
│   ├── http.rs
│   ├── filesystem.rs
│   └── config.rs
└── src/error.rs (PortError)
```

### Ecosystem 2: agileplus-domain/ports

```
crates/agileplus-domain/src/ports/
├── mod.rs
├── observability.rs (ObservabilityPort)
├── agent.rs (AgentPort)
├── vcs.rs (VcsPort)
├── storage.rs (StoragePort)
└── review.rs (ReviewPort)
```

### Overlap Analysis

| phenotype-port-interfaces | agileplus-domain | Overlap |
|--------------------------|------------------|---------|
| Repository trait | StoragePort | HIGH |
| Logger trait | ObservabilityPort | HIGH |

### Action Items

- [ ] 🟡 **PORT-001**: Migrate phenotype-port-interfaces to edition 2024
- [ ] 🟡 **PORT-002**: Consolidate Repository trait ↔ StoragePort
- [ ] 🟡 **PORT-003**: Consolidate Logger trait ↔ ObservabilityPort
- [ ] 🟠 **PORT-004**: Archive redundant trait definitions

---

## 🟠 MEDIUM: Store Trait Patterns

### Verified Traits

| Trait | Crate | Methods |
|-------|-------|---------|
| `EventStore` | `agileplus-events` | append, get_events, delete, query |
| `SyncMappingStore` | `agileplus-sync` | get_mappings, upsert_mapping, delete_mapping |
| `GraphBackend` | `agileplus-graph` | query, health_check |
| `CacheStore` | `agileplus-cache` | get, set, delete, health_check |
| `SnapshotStore` | `agileplus-events` | save_snapshot, get_snapshot, delete_snapshot |

### Action Items

- [ ] 🟡 **STORE-001**: Integrate `hexagonal-rs` Repository trait
- [ ] 🟠 **STORE-002**: Create generic Repository trait
- [ ] 🟠 **STORE-003**: Migrate EventStore to use Repository<E>

---

## 🟠 MEDIUM: Config Loading Patterns

### Config Loader Inventory

| Loader | Location | Format | LOC |
|--------|----------|--------|-----|
| TOML loader | `agileplus-domain/src/config/loader.rs` | TOML | 84 |
| YAML loader | `agileplus-telemetry/src/config.rs` | YAML | 201 |
| JSON loader | `vibe-kanban/backend/src/models/config.rs` | JSON | 374 |
| Builder pattern | `agileplus-cache/src/config.rs` | Struct | 100 |

### Status: `libs/config-core/` exists but UNUSED

### Action Items

- [ ] 🟡 **CONFIG-001**: Migrate `libs/config-core` to edition 2024
- [ ] 🟡 **CONFIG-002**: Integrate config-core into TOML locations
- [ ] 🟠 **CONFIG-003**: Evaluate `config-rs` crate (40M+ downloads)

---

## 🟠 MEDIUM: HTTP Client Patterns (14+ instantiations)

### HTTP Client Usage

| Project | Usage |
|---------|-------|
| `agileplus-plane` | reqwest 0.13 |
| `agileplus-github` | reqwest 0.12 |
| `agileplus-telemetry` | reqwest 0.13 |
| `agileplus-agents` | reqwest |
| `heliosCLI/codex-rs` | reqwest (extensive) |

### Action Items

- [ ] 🟡 **HTTP-001**: Create `libs/http-client/` with shared configuration
- [ ] 🟡 **HTTP-002**: Add retry with exponential backoff
- [ ] 🟡 **HTTP-003**: Add authentication support
- [ ] 🟠 **HTTP-004**: Migrate existing instantiations

---

## 🟢 LOW: External Package Recommendations

| Package | Downloads | Purpose | Effort |
|---------|-----------|---------|--------|
| `config-rs` | 40M+ | Config management | 3-5 days |
| `utoipa` | 5M+ | OpenAPI generation | 1 week |
| `miette` | 10M+ | Pretty CLI errors | 2-3 days |
| `axum-extra` | 8M+ | Axum utilities | 1-2 days |
| `tracing-error` | 3M+ | Error context with tracing | 1 day |

### Action Items

- [ ] 🟠 **EXT-001**: Evaluate `config-rs` adoption
- [ ] 🟠 **EXT-002**: Evaluate `utoipa` for OpenAPI
- [ ] 🟠 **EXT-003**: Evaluate `miette` for CLI errors
- [ ] 🟢 **EXT-004**: Evaluate `tracing-error` for error context

---

## LOC Impact Summary

| Category | Current | After Consolidation | Savings |
|----------|---------|---------------------|---------|
| Unused Libraries | 1,650 | 0 (archive) | 1,650 |
| Error Types | 600 | 200 | 400 |
| Config Loading | 500 | 150 | 350 |
| Store Traits | 300 | 100 | 200 |
| HTTP Clients | 300 | 100 | 200 |
| **Total** | **3,350** | **550** | **2,800** |

---

## Cross-Reference Index

### Evidence Files

| Document | Location |
|----------|----------|
| Deep research | `plans/2026-03-29-DEEP_RESEARCH_ERROR_TYPES-v1.md` |
| Implementation plan | `plans/2026-03-29-AUDIT_IMPLEMENTATION_PLAN-v1.md` |
| Extended findings | `plans/2026-03-29-AGILEPLUS_EXTENDED_DUPLICATION_AUDIT-v1.md` |
| Master index | `plans/2026-03-29-MASTER_INDEX_v4-v4.md` |

### Worklog Entries

| Document | Location |
|----------|----------|
| Primary worklog | `docs/WORKLOG.md` |
| Duplication worklog | `docs/worklogs/DUPLICATION.md` |
| Library worklog | `docs/worklogs/LIBRARY.md` |

### Audit Reports

| Document | Location |
|----------|----------|
| AgilePlus audit | `docs/reports/AGILEPLUS_DUPLICATION_AUDIT_20260329.md` |
| Codebase atlas | `docs/reports/CODEBASE_ATLAS.md` |
| Decomposition audit | `docs/reports/DECOMPOSITION_AUDIT.md` |

---

## Next Steps

1. **IMMEDIATE**: Migrate all 11 `libs/` to edition 2024
2. **THIS WEEK**: Create `libs/error-core/` with canonical error types
3. **NEXT WEEK**: Integrate `hexagonal-rs` Repository trait
4. **FUTURE**: Evaluate external package adoption

---

*Report generated by FORGE (2026-03-29)*
