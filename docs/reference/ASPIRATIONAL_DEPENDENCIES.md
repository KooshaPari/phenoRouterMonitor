# Aspirational Dependencies — Phenotype Infrakit

**Date**: 2026-03-30
**Version**: 0.2.0
**Status**: Planning & Documentation

---

## Overview

This document tracks workspace dependencies that are **declared but not yet actively used**. These are "aspirational" in the sense that they represent intentional planned usage, not accidental inclusions.

Each entry includes:
- Dependency name and version
- Intended use case
- Priority level (High/Medium/Low)
- Target crate(s) for adoption
- Timeline estimate

---

## Aspirational Dependencies Register

### 1. `anyhow` (v1.0)

**Declared in**: `[workspace.dependencies]`

**Status**: Aspirational (not used by 7 core members)

**Intended Use**:
- Error context wrapping in application layers (vs library crates)
- Simplified error handling in binaries and integration tests
- Dev-dependencies for test fixtures and harnesses

**Target Crates**:
- `agileplus-cli` — Command-line error context
- `agileplus-dashboard` — Server error responses
- Integration test suites (dev-dependencies)

**Priority**: High

**Timeline**: Phase 2 (when integration patterns finalized)

**Notes**:
- `phenotype-error-core` and `phenotype-errors` provide structured errors for libraries
- `anyhow` bridges to application layers where error context is needed
- Consensus: Keep declared (active future use) rather than comment out

---

### 2. `moka` (v0.12, features: `["sync"]`)

**Declared in**: `[workspace.dependencies]`

**Status**: Aspirational (declared but not used; `lru` used in phenotype-cache-adapter instead)

**Intended Use**:
- Advanced distributed caching with TTL, expiry, and metrics
- Future replacement/upgrade from `lru` when performance requirements increase
- Multi-tier caching strategies (L1: LRU, L2: moka)

**Target Crates**:
- `phenotype-cache-adapter` — Upgrade tier 2 cache implementation
- `agileplus-dashboard` — Session caching with TTL

**Priority**: Medium

**Timeline**: Phase 3 (when cache optimization is prioritized)

**Notes**:
- Currently `phenotype-cache-adapter` uses `lru` + `dashmap` + `tokio`
- Moka provides better TTL semantics and metrics out-of-the-box
- Reference: Keep as workspace dependency to avoid add/remove churn

---

### 3. `reqwest` (v0.12)

**Declared in**: `[workspace.dependencies]`

**Status**: Aspirational (not used by core members; AgilePlus has own vendored variants)

**Intended Use**:
- HTTP client for external service integrations
- GitHub API interactions
- Slack/Jira webhook callbacks
- Third-party SAAS connectors

**Target Crates**:
- `agileplus-github` — GitHub API client (currently hand-rolled)
- `agileplus-integration-tests` — External service mocking/testing
- `phenotype-*` crates in Phase 2+ (service adapters)

**Priority**: High

**Timeline**: Phase 2 (service integration layer)

**Notes**:
- AgilePlus already has `reqwest` in some crate manifests (localized usage)
- Consolidate to workspace dependency when integration layer is standardized
- TLS stack: use `reqwest` features (`rustls`, `native-tls`) per deployment needs

---

### 4. `tracing` (v0.1)

**Declared in**: `[workspace.dependencies]`

**Status**: Aspirational (declared but zero usage in source code)

**Intended Use**:
- Distributed tracing instrumentation
- Span/event emission for observability
- Integration with Jaeger, OpenTelemetry, Honeycomb
- Debug logging in async contexts

**Target Crates**:
- `phenotype-telemetry` — Tracing adapter/bridge (currently empty stub)
- `phenotype-cache-adapter` — Request tracing (already declared locally)
- `agileplus-dashboard` — Request/response tracing
- All async services (Phase 2+)

**Priority**: High

**Timeline**: Phase 2-3 (observability layer)

**Notes**:
- `phenotype-telemetry` scaffold exists but is not populated
- Should export `tracing` macros + pre-configured spans
- Pair with `tracing-subscriber` for initialization (not yet declared)
- Reference: Keep declared as foundational for observability roadmap

---

## Summary Table

| Dependency | Version | Status | Priority | Timeline | Target Crate |
|-----------|---------|--------|----------|----------|--------------|
| `anyhow` | 1.0 | Aspirational | High | Phase 2 | agileplus-{cli,dashboard} |
| `moka` | 0.12 | Aspirational | Medium | Phase 3 | phenotype-cache-adapter |
| `reqwest` | 0.12 | Aspirational | High | Phase 2 | agileplus-github, tests |
| `tracing` | 0.1 | Aspirational | High | Phase 2-3 | phenotype-telemetry |

---

## Unused Dependencies (Candidates for Removal)

The following dependencies are declared but **not aspirational** — they may be removed if unused after 60 days:

| Dependency | Version | Usage Count | Recommendation | Notes |
|-----------|---------|-------------|-----------------|-------|
| `blake3` | 1.0 | 0 (in 7 core members) | Remove | Used only in orphaned crates (phenotype-event-sourcing, phenotype-crypto); already declared locally there |
| `once_cell` | 1.0 | 0 | Remove | No known usage pattern; could use `std::sync::OnceLock` (Rust 1.70+) instead |
| `parking_lot` | 0.12 | 0 | Remove | Unused; standard `std::sync::Mutex` adequate for current needs |
| `hex` | 0.4 | 0 | Remove | No usage in source; `sha2` already includes hex encoding if needed |

---

## Action Items

- [ ] **Week 2**: Verify each aspirational dependency with target crate owners
- [ ] **Phase 2 start**: Activate `anyhow`, `reqwest` migrations
- [ ] **Phase 2 start**: Populate `phenotype-telemetry` with tracing bridge
- [ ] **Phase 3 start**: Evaluate `moka` upgrade path for cache-adapter
- [ ] **After 60 days**: Remove genuinely unused deps (`blake3`, `once_cell`, `parking_lot`, `hex`)

---

**Generated**: 2026-03-30
**Maintained By**: Phenotype Team
**Related Docs**: `docs/audits/CARGO_MEMBERS_INVENTORY.md`, `Cargo.toml`
