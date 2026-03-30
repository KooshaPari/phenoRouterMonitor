# Cross-Repository Extraction Targets
## New Phenotype Shared Crates for Duplication Consolidation

**Date**: 2026-03-30  
**Scope**: Shared library extraction plan for phenotype-infrakit and heliosCLI  
**Total Consolidation Opportunity**: 550-750 LOC savings across 3 phases  

---

## Target 1: phenotype-error-macros (NEW)

### Overview

**Purpose**: Provide reusable derive macros for common error handling patterns  
**Status**: NEW (to be created)  
**Location**: `crates/phenotype-error-macros/`  
**Proposed Size**: ~150 LOC (macro implementation) + 100 LOC (tests)  

### Problem Statement

Currently, **20+ error handling patterns are repeated** across 14 files:
- Context-preserving `#[error("{context}: {source}")]` pattern (200 LOC)
- Manual `From<T>` conversions for io, json, toml errors (100 LOC)
- Error message templates and variant structs (100 LOC)

**Root Cause**: No macro support; each crate manually implements thiserror boilerplate.

### Solution Design

Create a **proc-macro crate** that generates common error patterns:

```rust
// Usage in any crate:
#[derive(ContextError)]
pub enum MyError {
    #[error_with_context]
    Io(std::io::Error),
    
    #[error_with_context]
    Json(serde_json::Error),
    
    #[error("validation failed: {0}")]
    Validation(String),
}

// Generates:
// pub enum MyError {
//     Io {
//         context: &'static str,
//         #[source]
//         source: std::io::Error,
//     },
//     Json {
//         context: &'static str,
//         #[source]
//         source: serde_json::Error,
//     },
//     Validation(String),
// }
//
// impl From<std::io::Error> for MyError { ... }
// impl From<serde_json::Error> for MyError { ... }
```

### Deliverables

1. **Macro Crate**: `crates/phenotype-error-macros/`
   - `lib.rs`: Core macro definitions (150 LOC)
   - `macros/context_error.rs`: ContextError derive macro (100 LOC)
   - `macros/converters.rs`: Auto From<T> generation (80 LOC)
   - Tests: Compile and runtime tests (120 LOC)
   - Documentation: Examples and patterns (80 LOC)

2. **Enhanced phenotype-error-core**
   - Re-export macros from phenotype-error-macros
   - Consolidate canonical error types

3. **Migration Guide**
   - Pattern matching: convert 14 error.rs files to use macros
   - Backward compatibility: old patterns still work

### Scope of Changes

**Affected Crates** (Phase 1):

phenotype-infrakit:
- phenotype-contracts/src/error.rs (convert to ContextError)
- phenotype-event-sourcing/src/error.rs (convert to ContextError)
- phenotype-http-client-core/src/error.rs (convert to ContextError)
- phenotype-retry/src/error.rs (convert to ContextError)
- phenotype-policy-engine/src/error.rs (convert to ContextError)

heliosCLI:
- codex-rs/core/src/error.rs (convert key variants)
- codex-rs/rmcp-client/src/error.rs (convert to ContextError)
- codex-rs/config/src/error.rs (convert to ContextError)

**LOC Reduction**: 200-250 LOC (eliminating boilerplate)

### Dependencies

```toml
[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
thiserror = "1.0"

[dev-dependencies]
trybuild = "1.0"
```

### Timeline & Effort

**Duration**: 2 days  
**Effort**: ~40 tool calls  
**Milestones**:
- Day 1: Implement ContextError macro, basic From conversion
- Day 1.5: Testing and edge cases (error message templates, multiple context levels)
- Day 2: Integration with phenotype-error-core, migration examples
- Day 2: PR review and documentation

### Success Criteria

- ✓ ContextError macro compiles and generates correct code
- ✓ From<T> conversions auto-generate correctly
- ✓ Error messages preserve original context patterns
- ✓ All 14 affected error.rs files migrate successfully
- ✓ Zero regressions in existing error handling
- ✓ Documentation and examples complete

### Rollback Plan

If macro implementation fails:
- Keep manual patterns in phenotype-error-core
- Use macro crate as optional feature (not required)
- Document workaround patterns

---

## Target 2: phenotype-config-loader (EXPAND)

### Overview

**Purpose**: Unified configuration loading and merging framework  
**Status**: EXISTING (skeleton at 60 LOC)  
**Location**: `crates/phenotype-config-loader/`  
**Current Size**: 60 LOC  
**Proposed Size**: 260+ LOC (200 LOC enhancement)  

### Problem Statement

Configuration loading logic is **scattered and inconsistently implemented**:
- Figment setup patterns repeated 4+ times (100 LOC duplication)
- TOML merging logic duplicated in 2 places (60 LOC duplication)
- Error wrapping patterns repeated in 5 ConfigError enums (80 LOC duplication)
- Environment variable loading inconsistent (80 LOC duplication)

**Root Cause**: No unified config abstraction; each crate designs its own loader.

### Solution Design

Expand `phenotype-config-loader` with reusable builders and utilities:

```rust
// Usage in any crate:
let config = FigmentBuilder::new("MY_APP_")
    .with_toml_file("/etc/app.toml")?
    .with_env_vars()?
    .with_defaults(default_config)?
    .validate::<MyConfig>()?
    .build();

// Advanced: Custom merge strategy
let config = FigmentBuilder::new("MY_APP_")
    .with_toml_file("/etc/app.toml")?
    .with_override_file("/etc/app.local.toml", OverrideStrategy::DeepMerge)?
    .extract::<MyConfig>()?;
```

### Deliverables

1. **FigmentBuilder** (80 LOC)
   - Consistent figment setup pattern
   - Fluent API for configuration
   - Support for TOML, env vars, defaults, overrides

2. **ConfigMerger** (60 LOC)
   - TOML value merging logic
   - Merge strategies (shallow, deep, override)
   - Conflict resolution

3. **ConfigValidator** (80 LOC)
   - Schema validation framework
   - Custom validators
   - Error context preservation

4. **EnvLoader** (40 LOC)
   - Environment variable loading
   - Prefix handling
   - Type conversion

5. **Error Consolidation** (50 LOC)
   - ConfigError enum (io, json, toml, validation variants)
   - From conversions using phenotype-error-macros
   - Context helpers

6. **Tests & Documentation** (100 LOC)
   - Unit tests for each component
   - Integration tests
   - Examples and patterns

### Scope of Changes

**Enhanced Crates** (Phase 2):

phenotype-infrakit:
- phenotype-config-loader: Add FigmentBuilder, ConfigMerger, ConfigValidator
- phenotype-policy-engine/src/loader.rs: Use FigmentBuilder
- phenotype-telemetry/src/registry.rs: Use FigmentBuilder
- phenotype-config-core: Export consolidated ConfigError

heliosCLI (migration path, not Phase 2):
- codex-rs/core/src/config/service.rs: Use ConfigError from phenotype-config-loader
- codex-rs/config/src/loader.rs: Use FigmentBuilder
- codex-rs/config/src/validation.rs: Use ConfigValidator

**LOC Reduction**: 150-200 LOC (eliminating duplication and boilerplate)

### Dependencies

```toml
[dependencies]
figment = { version = "0.10", features = ["env", "toml"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
toml_edit = "0.22"
thiserror = "1.0"
phenotype-error-core = { path = "../phenotype-error-core" }
phenotype-error-macros = { path = "../phenotype-error-macros" }

[dev-dependencies]
tempfile = "3.0"
```

### Timeline & Effort

**Duration**: 3 days  
**Effort**: ~50 tool calls  
**Milestones**:
- Day 1: Implement FigmentBuilder and ConfigMerger
- Day 1.5: Implement ConfigValidator and EnvLoader
- Day 2: Error consolidation and integration with phenotype-error-macros
- Day 2.5: Testing, examples, and documentation
- Day 3: Migration of phenotype crates, PRs and review

### Success Criteria

- ✓ FigmentBuilder provides fluent API for config loading
- ✓ ConfigMerger handles TOML value merging correctly
- ✓ ConfigValidator consolidates validation logic
- ✓ Error handling uses consolidated ConfigError
- ✓ All phenotype crates migrate to use phenotype-config-loader
- ✓ heliosCLI migration path documented
- ✓ Zero regressions in existing configuration handling

### Integration with phenotype-error-macros

This crate **depends on** phenotype-error-macros:
- ConfigError uses `#[derive(ContextError)]`
- Reduces boilerplate in error definitions

---

## Target 3: phenotype-test-fixtures (NEW)

### Overview

**Purpose**: Shared test builders, mocks, and seed data  
**Status**: NEW (to be created)  
**Location**: `crates/phenotype-test-fixtures/`  
**Proposed Size**: ~200 LOC (core) + 150 LOC (tests)  

### Problem Statement

Test infrastructure is **scattered** with minimal sharing:
- Mock response builders repeated in 5+ test files (100 LOC duplication)
- Test client builders duplicated in 6 crates (72 LOC duplication)
- Seed data generators created independently in 4+ places (100 LOC duplication)
- Custom assertion helpers reinvented multiple times (45 LOC duplication)

**Root Cause**: No centralized test fixture library; each crate defines its own.

### Solution Design

Create **reusable test builders and fixtures**:

```rust
// Usage in tests:
#[test]
fn test_config_loading() {
    let config = ConfigBuilder::new()
        .with_database_url("sqlite:///:memory:")
        .with_log_level("debug")
        .build();
    assert_config(&config, |c| {
        c.log_level == "debug" && c.database_url.contains("sqlite")
    });
}

#[test]
fn test_http_client() {
    let response = HttpResponseBuilder::success()
        .with_body(r#"{"status": "ok"}"#)
        .build();
    
    assert_http_success(&response);
    assert_json_body(&response, |body| {
        body["status"] == "ok"
    });
}

#[test]
fn test_error_handling() {
    let error = MyError::io_context("reading config", io_err);
    assert_error_message(&error, |msg| msg.contains("reading config"));
}
```

### Deliverables

1. **Builders** (80 LOC)
   - `HttpResponseBuilder`: Mock HTTP responses
   - `ConfigBuilder`: Test configuration objects
   - `ErrorBuilder`: Custom error instances
   - `RequestBuilder`: HTTP request mocks

2. **Seed Data** (60 LOC)
   - `SeedData` trait for domain objects
   - Parametrized dummy data generators
   - Default fixtures for common types

3. **Assertions** (50 LOC)
   - `assert_http_success()`: HTTP response assertions
   - `assert_json_body()`: JSON parsing + assertions
   - `assert_error_message()`: Error message matching
   - Custom matchers for domain types

4. **Snapshots & Fixtures** (40 LOC)
   - Response snapshot templates
   - Error message templates
   - Configuration presets

5. **Tests & Documentation** (100 LOC)
   - Unit tests for builders
   - Example usage patterns
   - Best practices guide

### Scope of Changes

**Affected Crates** (Phase 3, optional):

phenotype-infrakit:
- crates/phenotype-test-infra: Move common builders here
- All test files: Use ConfigBuilder, HttpResponseBuilder

heliosCLI:
- codex-rs/app-server-protocol/src/schema_fixtures.rs: Use SeedData
- codex-rs/core/tests/common/lib.rs: Use HttpResponseBuilder
- All test suites: Use phenotype-test-fixtures builders

**LOC Reduction**: 100-150 LOC (through builder reuse)

### Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
# None required (used in dev only)
```

### Timeline & Effort

**Duration**: 2 days  
**Effort**: ~35 tool calls  
**Milestones**:
- Day 1: Implement builders (ConfigBuilder, HttpResponseBuilder, ErrorBuilder)
- Day 1.5: Implement assertions and snapshots
- Day 2: SeedData generators and documentation
- Day 2: Migration of test suites, examples, review

### Success Criteria

- ✓ Builders provide fluent API for test object construction
- ✓ Assertions simplify test code and improve readability
- ✓ SeedData generators eliminate duplicate fixture creation
- ✓ Test code reduction: 100+ LOC eliminated across both repos
- ✓ No regressions in existing tests
- ✓ Documentation and examples complete

### Priority

**MEDIUM** (optional for Phase 3; lower priority than Phases 1-2)

Recommendation: Execute Phases 1-2 immediately; defer Phase 3 to Q2 when test patterns stabilize.

---

## Target 4: phenotype-http-client-adapters (SPLIT)

### Overview

**Purpose**: HTTP client implementations and adapter framework  
**Status**: EXISTING (monolithic at 705 LOC)  
**Location**: `crates/phenotype-http-client-core/` (split into adapters)  
**Current Approach**: Single monolithic crate  
**Proposed Approach**: Core trait + separate adapters  

### Problem Statement

`phenotype-http-client-core` is **monolithic** (705 LOC) with **mixed concerns**:
- Core HTTP trait (50 LOC) — reusable
- Reqwest sync adapter (347 LOC) — specific to sync
- Auth middleware (120 LOC) — specific to basic auth
- Retry logic (107 LOC) — reusable
- HTTP errors (84 LOC) — reusable

heliosCLI **cannot easily reuse** because:
- Sync focus (phenotype) vs async/await (heliosCLI)
- No async adapter included
- MCP + OAuth are domain-specific (should stay in heliosCLI)

### Solution Design

**Split into focused crates**:

```
phenotype-http-client-core (300 LOC)
├── HttpTransport trait (core)
├── HttpResponse type + helpers
├── TransportError enum
└── Retry logic (composable)

phenotype-http-client-adapters/
├── phenotype-http-client-reqwest-sync (200 LOC)
│   └── Reqwest synchronous adapter + builder
└── phenotype-http-client-tokio-async (150 LOC, Phase 4)
    └── Tokio async adapter + builder

heliosCLI (stays domain-specific):
├── codex-rs/rmcp-client/ (MCP + OAuth)
└── codex-rs/network-proxy/ (MITM proxy, streaming)
```

### Deliverables

1. **phenotype-http-client-core** (refactored, 250 LOC)
   - HttpTransport trait definition
   - HttpResponse + HttpRequest types
   - TransportError + Result<T>
   - Retry middleware (composable)
   - Status code helpers

2. **phenotype-http-client-reqwest-sync** (NEW, 200 LOC)
   - Reqwest synchronous implementation
   - Builder pattern for client creation
   - Auth middleware (basic, bearer)
   - Error mapping
   - Timeout + retry integration

3. **phenotype-http-client-tokio-async** (NEW, 150 LOC, Phase 4)
   - Tokio async implementation
   - Builder pattern for async client
   - Streaming support
   - Error mapping
   - Timeout + retry integration

4. **Tests & Documentation** (100 LOC)
   - Unit tests for each adapter
   - Integration tests
   - Examples: sync + async usage

### Scope of Changes

**Phase 2 (core only)**:
- Refactor phenotype-http-client-core (reduce from 705 to 300 LOC)
- Create phenotype-http-client-reqwest-sync (200 LOC)
- Migrate phenotype crates to use sync adapter

**Phase 4 (async, optional)**:
- Create phenotype-http-client-tokio-async (150 LOC)
- Enable heliosCLI to adopt phenotype crates for base HTTP support

**Migration Path for heliosCLI**:
- Adopt phenotype-http-client-core as base trait
- Build MCP + OAuth layers on top
- Keep domain-specific network-proxy separate

### Dependencies

**phenotype-http-client-core**:
```toml
[dependencies]
thiserror = "1.0"

[dev-dependencies]
# None required
```

**phenotype-http-client-reqwest-sync**:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["rt"] }
thiserror = "1.0"
phenotype-http-client-core = { path = "../phenotype-http-client-core" }

[dev-dependencies]
tokio = { version = "1.0", features = ["macros", "rt"] }
```

**phenotype-http-client-tokio-async** (Phase 4):
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1.0", features = ["rt", "time"] }
bytes = "1.0"
futures = "0.3"
phenotype-http-client-core = { path = "../phenotype-http-client-core" }
```

### Timeline & Effort

**Phase 2** (Core + Sync Adapter):
- Duration: 3-4 days
- Effort: ~50 tool calls
- Deliverable: phenotype-http-client-core refactored + sync adapter

**Phase 4** (Async Adapter):
- Duration: 2-3 days
- Effort: ~30 tool calls
- Deliverable: async adapter + heliosCLI migration path documented

### Success Criteria

- ✓ Core HttpTransport trait is minimal and reusable
- ✓ Sync adapter provides full reqwest functionality
- ✓ Async adapter (Phase 4) provides tokio async support
- ✓ Retry logic is composable and works with both adapters
- ✓ All phenotype crates use adapters
- ✓ Zero regressions in existing HTTP handling
- ✓ heliosCLI migration path documented

### Priority

**MEDIUM** (Phase 2 high priority for refactoring, Phase 4 optional for Q3)

Recommendation: Execute Phase 2 refactoring after config consolidation; defer Phase 4 async adapter to Q3.

---

## Consolidation Candidates (Already Extracted)

These crates are **already well-extracted** and require **no major changes**:

### phenotype-error-core

**Status**: ✓ GOOD (canonical error types)  
**Size**: 159 LOC  
**Purpose**: Centralized error enum  
**Recommendation**: Keep as-is; enhance with phenotype-error-macros

### phenotype-config-core

**Status**: ✓ GOOD (generic container)  
**Size**: 60 LOC  
**Purpose**: Generic config value storage  
**Recommendation**: Keep as-is; expand phenotype-config-loader for loading logic

### phenotype-http-client-core

**Status**: ⚠ NEEDS REFACTORING (monolithic)  
**Size**: 705 LOC  
**Purpose**: HTTP client abstraction  
**Recommendation**: Split into core trait (250 LOC) + adapter implementations

---

## Cross-Repo Dependency Graph

```
PHASE 1 (Error Consolidation)
├── phenotype-error-macros (NEW)
│   └── phenotype-error-core (existing)
│       └── All error.rs files

PHASE 2 (Config Consolidation)
├── phenotype-config-loader (EXPAND)
│   ├── phenotype-error-macros (from Phase 1)
│   └── phenotype-config-core (existing)
│       └── All config.rs files
│
├── phenotype-http-client-adapters (SPLIT)
│   ├── phenotype-http-client-core (refactored)
│   │   └── phenotype-retry (uses core)
│   └── phenotype-http-client-reqwest-sync (NEW)
│       └── All HTTP client code

PHASE 3 (Test Consolidation, Optional)
└── phenotype-test-fixtures (NEW)
    └── All test suites

PHASE 4 (Async Support, Q3 2026)
└── phenotype-http-client-tokio-async (NEW)
    └── heliosCLI async support path
```

---

## Effort & Timeline Summary

| Crate | Status | Effort | Timeline | Savings | Phase |
|-------|--------|--------|----------|---------|-------|
| phenotype-error-macros | NEW | 40 calls | 2 days | 200-250 LOC | 1 |
| phenotype-config-loader | EXPAND | 50 calls | 3 days | 150-200 LOC | 2 |
| phenotype-http-client-adapters | SPLIT | 50 calls | 3-4 days | 100-150 LOC | 2 |
| phenotype-test-fixtures | NEW | 35 calls | 2 days | 100-150 LOC | 3 |
| phenotype-http-client-tokio-async | NEW | 30 calls | 2-3 days | 50-100 LOC | 4 |

**Total (Phases 1-3)**: 7-9 days, ~175 tool calls, 550-750 LOC savings  
**Optional Phase 4**: +2-3 days for async support (Q3 2026)

---

## Next Steps

1. **Approve Phases 1-2** for immediate execution (5-7 days)
2. **Create task tracking** for each crate and migration
3. **Spawn parallel agents** for Phase 1 macro implementation and Phase 2 config enhancement
4. **Review & merge** Phase 1 PRs before starting Phase 2
5. **Execute Phase 2** after Phase 1 completion
6. **Defer Phase 3** to Q2 when test patterns stabilize
7. **Plan Phase 4** for Q3 when heliosCLI async needs mature

**Recommendation**: Start with Phases 1-2 immediately; these are **high-confidence, low-risk changes** that will **eliminate 350-450 LOC of duplication** and **establish shared library patterns** for the entire ecosystem.
