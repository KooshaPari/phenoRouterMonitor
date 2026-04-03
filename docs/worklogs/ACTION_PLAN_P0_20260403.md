# Action Plan: P0 Immediate Actions

**Project:** [cross-repo]
**Status:** action_plan
**Priority:** P0

---

## Overview

Based on the deep audit findings, here are the specific tasks to execute immediately for maximum LOC savings with minimal effort.

---

## P0-1: Fix Edition Mismatch (config-core & hexagonal-rs)

### Context
Two complete libraries exist but are unused due to edition 2021 vs 2024 mismatch:
- `libs/config-core/` - Complete config loading
- `libs/hexagonal-rs/` - Generic Repository patterns

### Action
1. Check current edition in both libraries' Cargo.toml
2. Update to edition = "2024"
3. Verify compilation with `cargo check`
4. Create PR to integrate into TOML/YAML locations

### Estimated Impact
- **LOC Savings**: ~800 (config) + ~1,100 (repository) = ~1,900 LOC

---

## P0-2: Consolidate Error Handling

### Context
Two competing error crates with identical functionality:
- `phenotype-errors` - Active (used by test-infra, telemetry)
- `phenotype-error-core` - UNUSED (0 dependents)

### Action
1. Audit `phenotype-error-core` variants not in `phenotype-errors`
2. Add missing variants to `phenotype-errors`
3. Add deprecation notice to `phenotype-error-core`
4. Add `#[from]` attributes to reduce boilerplate

### Estimated Impact
- **LOC Savings**: ~750-900 LOC (error definitions + From impls)

---

## P0-3: Fix APIClient Performance Anti-Pattern

### Context
`platforms/thegent/src/thegent/api_helpers/__init__.py` creates new HTTP client per request - defeats connection pooling.

### Action
1. Find APIClient class definition
2. Refactor to use shared client with connection pool
3. Verify no regression in tests

### Estimated Impact
- **Performance**: Significant improvement in HTTP request latency

---

## P1-1: Activate phenotype-test-infra

### Context
`crates/phenotype-test-infra/` exists with TempDir, TestFixture, MockClock, capture_logs but projects create their own fixtures.

### Action
1. Audit what's in phenotype-test-infra
2. Add missing features (if any)
3. Add integration docs for consuming crates
4. Create migration guide for existing InMemory implementations

### Estimated Impact
- **LOC Savings**: ~250 LOC (test fixtures consolidation)

---

## P1-2: Adopt phenotype-port-traits Repository

### Context
Generic `Repository<E, I>` trait exists in `phenotype-port-traits` but not adopted - each domain crate defines its own.

### Action
1. Audit existing domain repository traits (StoragePort, EventStore, etc.)
2. Create migration path to generic Repository
3. Update dependent crates

### Estimated Impact
- **LOC Savings**: ~200+ LOC (trait definitions)

---

## P1-3: Migrate portage Config to pydantic-settings

### Context
pydantic-settings is in dependencies but NOT used - code uses manual `os.environ.get()` throughout.

### Action
1. Create `src/harbor/config/settings.py` with BaseSettings
2. Replace all `os.environ.get()` calls with settings class fields
3. Remove manual `load_dotenv()` calls

### Estimated Impact
- **LOC Savings**: ~150 LOC (config loading)
- **Type Safety**: Runtime validation of config values

---

## P2-1: Update litellm Version

### Context
Pinned to 1.81.1 while litellm 2.x available. Exclusions for 1.82.7/1.82.8 were security response.

### Action
1. Test with latest litellm 1.x or 2.x
2. Remove security exclusions if safe
3. Update pyproject.toml

### Estimated Impact
- **Security**: Patches to latest version

---

## P2-2: Create libs/http-client

### Context
Multiple HTTP wrapper implementations with overlapping functionality:
- FastHTTPClient (thegent)
- HTTPConnectionPool (heliosCLI)
- HTTPClient (thegent)
- APIClient (thegent)

### Action
1. Design unified wrapper with connection pooling
2. Implement in single location
3. Migrate consumers

### Estimated Impact
- **LOC Savings**: ~200 LOC

---

## P2-3: Activate phenotype-logging

### Context
`crates/phenotype-logging/` exists with init_logging, env support, JSON/Compact/Pretty formats but inconsistent adoption.

### Action
1. Add to workspace dependencies
2. Document as required in architecture standards
3. Migrate existing tracing setup

### Estimated Impact
- **LOC Savings**: ~300 LOC

---

## Implementation Sequence

```
Week 1:
├── P0-1: Fix edition mismatch (config-core)
├── P0-1: Fix edition mismatch (hexagonal-rs)
└── P0-2: Consolidate error handling

Week 2:
├── P0-3: Fix APIClient performance
├── P1-1: Activate phenotype-test-infra
└── P1-3: Migrate portage config

Week 3:
├── P1-2: Adopt phenotype-port-traits
├── P2-1: Update litellm
└── P2-2: Create http-client

Week 4:
└── P2-3: Activate phenotype-logging
```

---

## Success Metrics

- **Total LOC Reduced**: 4,850+ LOC
- **Library Utilization**: 5+ previously unused libraries activated
- **Performance**: HTTP connection pooling improvements
- **Type Safety**: Config validation at runtime