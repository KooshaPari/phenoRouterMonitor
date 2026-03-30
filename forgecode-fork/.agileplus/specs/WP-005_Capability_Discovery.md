# WP-005: Capability Discovery & Routing

**Work Package ID**: WP-005
**Epic**: eco-fork-001 (Custom Providers & Subagent Management)
**Phase**: 2
**Status**: Pending
**Priority**: High
**Created**: 2026-03-30

---

## Overview

Implement provider capability discovery and smart routing based on work package complexity, cost, and latency requirements.

## Description

Expose provider capabilities (models, max tokens, latency SLA, cost/1k tokens) via CLI and API. Implement intelligent routing that recommends optimal provider based on work package requirements.

---

## Objectives

- Expose provider `capabilities()` method (models, costs, latencies)
- Create `agileplus provider` CLI subcommand for discovery
- Implement smart routing based on work package complexity assessment
- Store capability metadata in SQLite for historical analysis and trends

---

## Acceptance Criteria

1. **Capability Discovery**:
   - `agileplus provider list` shows all registered providers
   - `agileplus provider show <name>` renders JSON with models, costs, latencies
   - Capability data includes: model list, max tokens, latency p95, cost/1M

2. **Smart Routing**:
   - WP complexity classification (simple, medium, complex)
   - Routing logic: simple → Haiku, medium → Sonnet, complex → Opus
   - Cost optimization: cheaper model suggested for simple tasks

3. **Metadata Storage**:
   - Capability snapshots persisted to SQLite
   - Historical analysis possible (cost trends over time)
   - Queryable via SQL

4. **Testing**:
   - `cargo test -p agileplus-cli` all pass
   - Routing heuristic tests (all complexity levels)
   - Metadata persistence tests

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| ProviderCapabilities struct | Serializable metadata | Defines all fields |
| CLI commands | provider list/show | JSON output correct |
| Routing logic | Complexity → provider mapping | All cases covered |
| SQLite schema | provider_capabilities table | Queries work |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-002 (Claude Provider)
- WP-003 (Local Provider)
- WP-004 (Subagent Spawning)

**Blocks**:
- WP-06 (Performance Metrics)

---

## Effort Estimate

- **Estimated LOC**: 380
- **Estimated Tool Calls**: 11-13
- **Estimated Duration**: 3-4 days

---

## Technical Details

### Capability Discovery

```rust
pub struct ProviderCapabilities {
    pub provider_name: String,
    pub models: Vec<ModelCapability>,
    pub max_tokens: u32,
    pub latency_p95_ms: u32,
    pub cost_per_1m_tokens: f64,
    pub supports_streaming: bool,
}

pub struct ModelCapability {
    pub name: String,
    pub context_window: u32,
    pub cost_input_per_1m: f64,
    pub cost_output_per_1m: f64,
    pub latency_p95_ms: u32,
}
```

### CLI Usage

```bash
# List all providers
agileplus provider list
# Output:
# ┌──────────┬────────────┬────────────────────────┐
# │ Provider │ Models     │ Latency (p95)          │
# ├──────────┼────────────┼────────────────────────┤
# │ claude   │ opus, sonnet, haiku │ 500ms          │
# │ local    │ llama2     │ 1000ms                 │
# └──────────┴────────────┴────────────────────────┘

# Show provider details
agileplus provider show claude
# Output: JSON with full capabilities

# Get routing recommendation
agileplus provider recommend --complexity medium
# Output: "claude-sonnet (cost: $3/1M tokens)"
```

### SQLite Schema

```sql
CREATE TABLE provider_capabilities (
    provider_id TEXT,
    snapshot_date TIMESTAMP,
    models_json TEXT,
    max_tokens INTEGER,
    latency_p95_ms INTEGER,
    cost_per_1m_tokens REAL,
    PRIMARY KEY (provider_id, snapshot_date)
);
```

---

## Subtasks

- [ ] T025: Add `ProviderCapabilities` struct with serializable metadata
- [ ] T026: Implement `capabilities()` for Claude, Local providers
- [ ] T027: Create `agileplus provider list` and `agileplus provider show <name>`
- [ ] T028: Add routing logic (WP complexity → provider recommendation)
- [ ] T029: Persist provider capabilities snapshot to SQLite
- [ ] T030: Unit tests for capability matching and routing heuristics

---

## Routing Heuristic

| WP Complexity | Recommended Provider | Rationale |
|---------------|---------------------|-----------|
| Simple (tokens <500) | Haiku | Low cost, fast enough |
| Medium (tokens 500-2K) | Sonnet | Good balance |
| Complex (tokens >2K) | Opus | Best quality, higher cost acceptable |
| Analysis task | Opus | Reasoning capability |
| Code generation | Sonnet | Good balance for coding |

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| Provider List | All shown | `agileplus provider list` completeness |
| Routing Accuracy | >90% | Recommendations match manual selection |
| Capability Accuracy | ±5% | Latency estimates accurate |
| Metadata Freshness | <1h | Snapshots recent |
| Test Coverage | ≥85% | `cargo tarpaulin` |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Provider changes | Medium | Low | Refresh on each discovery call |
| Routing mismatches | Medium | Low | Monitor actual vs recommended |
| Cost metadata stale | Low | Medium | Update metadata periodically |

---

## Traceability

**Functional Requirements**:
- FR-PROV08: Provider capability discovery and introspection
- FR-PROV09: Smart routing based on WP requirements

---

## Notes

- Cost data sourced from provider metadata (hard-coded for Phase 1)
- Future: Real-time cost updates from provider APIs
- Routing weights configurable via TOML

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
