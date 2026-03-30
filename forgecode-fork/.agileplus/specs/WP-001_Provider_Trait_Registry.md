# WP-001: Provider Trait & Registry Foundation

**Work Package ID**: WP-001
**Epic**: eco-fork-001 (Custom Providers & Subagent Management)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Define the core `Provider` trait with pluggable architecture and implement the thread-safe `ProviderRegistry` for registering and managing multiple LLM providers (Claude, local, streaming).

## Description

This WP establishes the foundational provider abstraction layer that all subsequent provider implementations (Claude, Local, Streaming) depend on. The registry enables dynamic provider registration, capability introspection, and audit trail recording.

---

## Objectives

- Define `Provider` trait with `invoke()`, `stream()`, and `capabilities()` methods
- Implement `ProviderRegistry` (singleton, thread-safe registration)
- Add SQLite schema for provider metadata and call audit
- Create comprehensive trait documentation with examples
- Write unit tests achieving 90%+ code coverage

---

## Acceptance Criteria

1. **Trait Definition**:
   - Provider trait compiles with 0 errors, 0 warnings
   - Trait includes methods: `invoke()`, `stream()`, `capabilities()`
   - All methods documented with rustdoc examples

2. **Registry Implementation**:
   - Registry supports ≥5 concurrent provider registrations
   - Thread-safe via Arc<DashMap> or RwLock
   - Supports provider lookup by name with O(1) performance

3. **SQLite Integration**:
   - Schema created: `provider_metadata` and `provider_calls` tables
   - Hash-chain audit record persists correctly
   - Migration script provided

4. **Testing**:
   - `cargo test -p forgecode-providers` all pass
   - Coverage ≥90% for trait and registry modules
   - No clippy warnings

5. **Documentation**:
   - Trait-level rustdoc with examples
   - README with provider development guide
   - Schema diagram in docs

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| forgecode-providers crate | New crate with trait and registry | Builds cleanly |
| Provider trait | Core abstraction with 3 methods | Documented with examples |
| ProviderRegistry | Thread-safe registry implementation | Tests pass, O(1) lookup |
| SQLite schema | provider_metadata and provider_calls tables | Migration script included |
| Unit tests | Registry and trait tests | ≥90% coverage |
| Documentation | Rustdoc and provider guide | All public APIs documented |

---

## Dependencies

**Depends On**: None (foundational)

**Blocks**:
- WP-002 (Claude Provider Implementation)
- WP-003 (Local Provider + Ollama)
- WP-004 (Subagent Spawning)

---

## Effort Estimate

- **Estimated LOC**: 400
- **Estimated Tool Calls**: 12-15
- **Estimated Duration**: 3-4 days (1 agent, full-time)
- **Parallel Work**: Can be completed independently

---

## Technical Details

### Key Components

```rust
pub trait Provider: Send + Sync {
    fn invoke(&self, prompt: String) -> Result<String>;
    fn stream(&self, prompt: String) -> Result<ProviderStream>;
    fn capabilities(&self) -> ProviderCapabilities;
}

pub struct ProviderRegistry {
    providers: Arc<DashMap<String, Arc<dyn Provider>>>,
}
```

### SQLite Schema

```sql
CREATE TABLE provider_metadata (
    provider_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT,
    capability_class TEXT,
    created_at TIMESTAMP
);

CREATE TABLE provider_calls (
    call_id INTEGER PRIMARY KEY,
    provider_id TEXT,
    timestamp TIMESTAMP,
    input_tokens INTEGER,
    output_tokens INTEGER,
    latency_ms INTEGER,
    status TEXT,
    hash_chain_prev TEXT,
    hash_chain_curr TEXT
);
```

### Integration Points

- **agileplus-cli**: Provider commands will depend on registry
- **agileplus-sqlite**: Schema migrations
- **phenotype-contracts**: Shared error types and traits

---

## Subtasks

- [ ] T001: Create `forgecode-providers/src/lib.rs` with `Provider` trait skeleton
- [ ] T002: Implement `ProviderRegistry::register()` and `::get()`
- [ ] T003: Add SQLite `provider_metadata` and `provider_calls` tables
- [ ] T004: Write trait documentation + example provider stub
- [ ] T005: Unit tests for registry operations (register, get, concurrent access)

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| Build Success | 0 errors, 0 warnings | `cargo check -p forgecode-providers` |
| Test Coverage | ≥90% | `cargo tarpaulin -p forgecode-providers` |
| Registry Concurrency | ≥5 concurrent registrations | Stress test passes |
| Lookup Performance | O(1) | Benchmark test |
| Documentation | 100% of public APIs | `cargo doc --open` |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Thread-safety issues | Medium | High | Use well-tested Arc<DashMap> or RwLock |
| SQLite schema conflicts | Low | Medium | Run migrations in tx, rollback on error |
| Trait design changes | Medium | High | Use comprehensive trait tests early |

---

## Traceability

**Functional Requirements**:
- FR-PROV01: Provider trait with pluggable architecture
- FR-PROV02: Thread-safe registry for provider registration

**Related Documents**:
- agileplus_spec_outlines.md (Lines 59-83)
- phenotype-infrakit docs (error handling, async patterns)

---

## Notes

- Consider using `async-trait` for async provider methods in future
- Hash-chain verification deferred to WP with audit API
- Local storage registry (Phase 1); distributed registry deferred to Phase 2

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
