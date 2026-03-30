# WP-001: Router Core & Routing Engine

**Work Package ID**: WP-001
**Epic**: eco-fork-002 (Consolidated API Monitoring & Routing)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Implement core router using axum (async HTTP framework) with path-based routing, backend pool management, and request forwarding.

## Description

Build the foundational routing engine supporting service discovery, path pattern matching, round-robin load balancing, and header preservation.

---

## Objectives

- Implement core router using axum
- Define `Route` configuration (service, path pattern, backend pool)
- Implement path-based routing with regex and wildcard matching
- Add request forwarding with header and query param preservation
- Wire configuration loading from TOML

---

## Acceptance Criteria

1. **Router Engine**:
   - `cargo check -p phenotype-router-core` zero errors
   - Axum server starts successfully
   - Routes TOML config loads without errors

2. **Routing**:
   - Requests routed to correct backend
   - Path patterns (regex, wildcard) work correctly
   - Query parameters and headers preserved

3. **Load Balancing**:
   - Round-robin distributes requests across backends
   - All backends receive roughly equal traffic

4. **Configuration**:
   - TOML schema validated
   - Invalid routes rejected with clear error
   - Hot reload support (future)

5. **Testing**:
   - `cargo test -p phenotype-router-core` all pass
   - Integration test with 2 mock backends

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| Router struct | Core routing engine | Trait impl, no warnings |
| Path matching | Regex + wildcard patterns | All cases work |
| Backend pool | Round-robin distribution | Equal traffic |
| Config loader | TOML schema + validation | Loads correctly |
| Request forwarding | Headers, query params preserved | Full fidelity |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**: None (foundational)

**Blocks**:
- WP-002 (Health Checking)
- WP-003 (Rate Limiting)
- WP-04 (Metrics)

---

## Effort Estimate

- **Estimated LOC**: 450
- **Estimated Tool Calls**: 13-15
- **Estimated Duration**: 4-5 days

---

## Technical Details

### Key Structures

```rust
pub struct Router {
    routes: Vec<RouteConfig>,
    client: reqwest::Client,
    backend_pools: DashMap<String, BackendPool>,
}

pub struct RouteConfig {
    service: String,
    path_pattern: String,
    backends: Vec<BackendAddress>,
    timeout_ms: u64,
}

pub struct BackendPool {
    backends: Vec<BackendAddress>,
    current_index: Arc<AtomicUsize>,
}
```

### Config Example

```toml
[[routes]]
service = "agileplus"
path_pattern = "^/agileplus/(.*)"
timeout_ms = 30000
backends = [
  "http://agileplus-1:3000",
  "http://agileplus-2:3000"
]

[[routes]]
service = "heliosapp"
path_pattern = "^/heliosapp/(.*)"
timeout_ms = 30000
backends = [
  "http://heliosapp-1:8080",
  "http://heliosapp-2:8080"
]
```

---

## Subtasks

- [ ] T001: Create `phenotype-router-core/src/lib.rs` with Router struct
- [ ] T002: Implement path pattern matching (regex + wildcard)
- [ ] T003: Create BackendPool with round-robin load balancing
- [ ] T004: Implement request forwarding with headers/query params
- [ ] T005: Add TOML configuration schema and loader
- [ ] T006: Create axum route handlers
- [ ] T007: Unit tests for routing logic
- [ ] T008: Integration test with 2 mock backends

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| Route Accuracy | 100% | Request routed to correct backend |
| Load Distribution | ±5% deviation | Round-robin test |
| Config Validation | 100% | Invalid config rejected |
| Header Preservation | 100% | All headers forwarded |
| Test Coverage | ≥85% | `cargo tarpaulin` |

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
