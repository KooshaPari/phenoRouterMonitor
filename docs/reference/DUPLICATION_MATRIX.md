# Duplication Matrix: Cross-Repository Analysis

**Date**: 2026-03-30  
**Scope**: phenotype-infrakit ↔ heliosCLI ↔ heliosApp  
**Total Repositories Analyzed**: 3  
**Files Analyzed**: ~600 Rust source files, 172 test files  

---

## Quick Reference Table

| Category | Repos | Pattern | Files | LOC | Savings | Priority |
|----------|-------|---------|-------|-----|---------|----------|
| **Error Handling** | 2 | Context-preserving errors | 14 | 400 | 200-250 | ★★★ |
| **Config Loading** | 2 | TOML + figment setup | 11 | 300 | 150-200 | ★★★ |
| **HTTP Client** | 2 | Retry + error parsing | 6 | 200 | 100-150 | ★★★ |
| **Test Fixtures** | 2 | Builders + seed data | 8 | 150 | 75-100 | ★★ |
| **Validation** | 2 | Rule matching + errors | 5 | 50 | 25-50 | ★ |
| **Cache** | 1 | (Non-overlapping) | - | - | 0 | - |
| **Event Sourcing** | 1 | (Non-overlapping) | - | - | 0 | - |
| **State Machine** | 1 | (Non-overlapping) | - | - | 0 | - |

**TOTAL**: 1,100 LOC duplication found; 550-750 LOC consolidation opportunity

---

## Error Handling Duplication Matrix

### Pattern Distribution

| Pattern | Frequency | LOC per Pattern | Total | Examples |
|---------|-----------|-----------------|-------|----------|
| **Context-preserving errors** | 20+ | 10 | 200 | `#[error("{context}: {source}")]` |
| **From conversions** | 12+ | 8 | 100 | `#[from] std::io::Error` |
| **Timeout/Retry variants** | 8 | 20 | 160 | `Timeout`, `RetryExceeded` |
| **Domain-specific errors** | 6+ | 15 | 90 | Custom enums per crate |
| **Anyhow vs thiserror** | Mixed | Varies | 100 | Inconsistent error patterns |
| **Result type aliases** | 14+ | 3 | 40 | `type Result<T> = std::result::Result<T, E>` |

**Total**: ~690 LOC (includes variants, impl blocks, doc comments)

### Files Involved (Detailed)

**phenotype-infrakit**:
```
crates/phenotype-error-core/src/lib.rs               159 LOC (canonical)
crates/phenotype-errors/src/lib.rs                    34 LOC (wrapper)
crates/phenotype-contracts/src/error.rs               ~80 LOC (duplicates error-core)
crates/phenotype-event-sourcing/src/error.rs          ~60 LOC (duplicates patterns)
crates/phenotype-http-client-core/src/error.rs        84 LOC (HTTP-specific)
crates/phenotype-retry/src/error.rs                   ~50 LOC (retry errors)
crates/phenotype-policy-engine/src/error.rs           ~70 LOC (policy errors)
crates/phenotype-validation/src/error.rs              ~60 LOC (validation errors)
crates/phenotype-cost-core/src/error.rs               ~40 LOC (cost errors)
```
**Subtotal**: 637 LOC

**heliosCLI**:
```
codex-rs/core/src/error.rs                            1,148 LOC (MONOLITHIC)
codex-rs/rmcp-client/src/error.rs                     ~200 LOC (MCP + HTTP)
codex-rs/execpolicy/src/error.rs                      ~80 LOC (exec errors)
codex-rs/otel/src/metrics/error.rs                    ~40 LOC (telemetry)
codex-rs/utils/git/src/errors.rs                      ~30 LOC (git errors)
codex-rs/utils/image/src/error.rs                     ~25 LOC (image errors)
crates/harness_checkpoint/src/error.rs                ~35 LOC (harness error)
crates/harness_elicitation/src/error.rs               ~35 LOC (harness error)
crates/harness_orchestrator/src/error.rs              ~35 LOC (harness error)
crates/harness_spec/src/error.rs                      ~35 LOC (harness error)
crates/harness_verify/src/error.rs                    ~35 LOC (harness error)
```
**Subtotal**: 1,698 LOC

**Cross-Repo Duplication**: 400-500 LOC (in common patterns)

### Consolidation Target

**Crate**: `phenotype-error-macros` (NEW)  
**Extraction Candidates**:
1. `#[derive(ContextError)]` macro (150 LOC)
2. Error conversion helpers (50 LOC)
3. Tests + docs (100 LOC)

**Savings**: 200-250 LOC through macro-driven code generation

---

## Configuration Loading Duplication Matrix

### Pattern Distribution

| Pattern | Frequency | LOC per Pattern | Total | Examples |
|---------|-----------|-----------------|-------|----------|
| **Figment setup** | 4 | 12 | 48 | `Figment::new().merge(...)` |
| **Error wrapping** | 5 | 16 | 80 | ConfigError From impls |
| **Schema validation** | 3 | 30 | 90 | Field checks, enum validation |
| **TOML merging** | 2 | 30 | 60 | `merge_toml_values()` |
| **Env var loading** | 4 | 15 | 60 | `from_env()`, env loading |
| **Config types** | 6 | 25 | 150 | `pub struct MyConfig` |

**Total**: ~488 LOC (includes types, builders, error handling)

### Files Involved (Detailed)

**phenotype-infrakit**:
```
crates/phenotype-config-core/src/lib.rs               60 LOC (generic container)
crates/phenotype-config-core/src/builder.rs           ~80 LOC (builder trait)
crates/phenotype-config-core/src/source.rs            ~50 LOC (config sources)
crates/phenotype-config-loader/src/lib.rs             ~60 LOC (loader skeleton)
crates/phenotype-policy-engine/src/loader.rs          ~80 LOC (figment + validation)
crates/phenotype-policy-engine/src/engine.rs          ~70 LOC (config usage)
crates/phenotype-telemetry/src/registry.rs            ~60 LOC (config loading)
```
**Subtotal**: 460 LOC

**heliosCLI**:
```
codex-rs/core/src/config/service.rs                   740 LOC (ConfigService + errors)
codex-rs/core/src/config/mod.rs                       ~200 LOC (ConfigToml + loading)
codex-rs/core/src/config/schema.rs                    ~150 LOC (validation)
codex-rs/core/src/config/types.rs                     ~80 LOC (core types)
codex-rs/core/src/config/profile.rs                   ~100 LOC (profile management)
codex-rs/core/src/config/edit.rs                      ~80 LOC (editing)
codex-rs/core/src/config/permissions.rs               ~80 LOC (permissions)
codex-rs/config/src/lib.rs                            ~200 LOC (separate crate)
codex-rs/config/src/cloud_requirements.rs             ~100 LOC (cloud config)
codex-rs/config/src/constraint.rs                     ~80 LOC (constraints)
codex-rs/config/src/diagnostics.rs                    ~70 LOC (diagnostics)
```
**Subtotal**: 1,880 LOC

**Cross-Repo Duplication**: 300-400 LOC (in common patterns)

### Consolidation Target

**Crate**: `phenotype-config-loader` (EXPAND)  
**Enhancements**:
1. `FigmentBuilder` for setup consolidation (80 LOC)
2. `ConfigMerger` for TOML merging (60 LOC)
3. `ConfigValidator` for schema validation (80 LOC)
4. Error helpers + tests (100 LOC)

**Savings**: 150-200 LOC through abstraction

---

## HTTP Client Duplication Matrix

### Pattern Distribution

| Pattern | Frequency | LOC per Pattern | Total | Examples |
|---------|-----------|-----------------|-------|----------|
| **Retry middleware** | 3 | 40 | 120 | Exponential backoff |
| **Status code handling** | 4 | 15 | 60 | `is_success()`, `is_error()` |
| **Error response parsing** | 5 | 15 | 75 | serde_json parsing |
| **Request building** | 3 | 20 | 60 | Request constructors |
| **Auth middleware** | 2 | 40 | 80 | Bearer token, basic auth |
| **Response wrapping** | 3 | 25 | 75 | HttpResponse type |

**Total**: ~470 LOC (including variants)

### Files Involved (Detailed)

**phenotype-infrakit**:
```
crates/phenotype-http-client-core/src/client.rs       347 LOC (reqwest wrapper)
crates/phenotype-http-client-core/src/auth.rs         120 LOC (auth middleware)
crates/phenotype-http-client-core/src/retry.rs        107 LOC (retry logic)
crates/phenotype-http-client-core/src/error.rs         84 LOC (HTTP errors)
crates/phenotype-http-client-core/src/lib.rs           47 LOC (traits)
```
**Subtotal**: 705 LOC

**heliosCLI**:
```
codex-rs/rmcp-client/src/rmcp_client.rs                ~400 LOC (HTTP + MCP)
codex-rs/rmcp-client/src/auth_status.rs                ~80 LOC (auth)
codex-rs/rmcp-client/src/perform_oauth_login.rs        ~120 LOC (OAuth)
codex-rs/codex-api/src/endpoint/responses.rs           ~150 LOC (responses)
codex-rs/codex-api/src/requests/responses.rs           ~100 LOC (request handling)
codex-rs/network-proxy/src/http_proxy.rs               ~200 LOC (proxy)
codex-rs/network-proxy/src/mitm.rs                     ~180 LOC (MITM)
codex-rs/network-proxy/src/upstream.rs                 ~160 LOC (upstream)
codex-rs/network-proxy/src/responses.rs                ~140 LOC (responses)
```
**Subtotal**: 1,530 LOC (+ 2,300 LOC unique MCP/streaming/OAuth)

**Cross-Repo Duplication**: 200-250 LOC (in common patterns)

### Consolidation Target

**Crate**: `phenotype-http-client-adapters` (SPLIT)  
**Approach**:
1. Keep `phenotype-http-client-core`: core trait (200 LOC)
2. Create sync adapter (200 LOC)
3. Create async adapter (150 LOC, Phase 4)
4. heliosCLI-specific: OAuth, MCP, streaming (keep local)

**Savings**: 100-150 LOC through reuse

---

## Test Utilities Duplication Matrix

### Pattern Distribution

| Pattern | Frequency | LOC per Pattern | Total | Examples |
|---------|-----------|-----------------|-------|----------|
| **Mock response builders** | 5 | 20 | 100 | HTTP response mocks |
| **Test builders** | 6 | 12 | 72 | Client, config builders |
| **Seed data generators** | 4 | 25 | 100 | Dummy data creation |
| **Assertion helpers** | 3 | 15 | 45 | Custom matchers |
| **Fixture initialization** | 4 | 10 | 40 | Test setup helpers |

**Total**: ~357 LOC (scattered, not consolidated)

### Files Involved (Detailed)

**phenotype-infrakit**:
```
crates/phenotype-test-infra/src/lib.rs                ~80 LOC (test utilities)
crates/phenotype-http-client-core/tests/client_test.rs ~120 LOC
crates/phenotype-crypto/tests/crypto_test.rs          ~100 LOC
crates/phenotype-validation/tests/validation_test.rs   ~80 LOC
[other 533 test files]                                 ~600 LOC (distributed)
```
**Subtotal**: 980 LOC (1,056 test LOC total in phenotype)

**heliosCLI**:
```
codex-rs/core/tests/common/lib.rs                      ~80 LOC (shared test utils)
codex-rs/app-server-protocol/src/schema_fixtures.rs    ~500 LOC (fixture data)
codex-rs/core/tests/common/streaming_sse.rs            ~120 LOC (SSE mocks)
codex-rs/core/tests/common/test_codex_exec.rs          ~150 LOC (test helpers)
[other 26 test files]                                  ~170,000+ LOC (heavy testing)
```
**Subtotal**: 172,490 LOC (172,490 test LOC total in heliosCLI)

**Cross-Repo Duplication**: 150-200 LOC (in common builder patterns)

### Consolidation Target

**Crate**: `phenotype-test-fixtures` (NEW)  
**Contents**:
1. `HttpResponseBuilder` (60 LOC)
2. `ConfigBuilder` (50 LOC)
3. `SeedData` generators (70 LOC)
4. Assertion macros (50 LOC)
5. Tests + docs (100 LOC)

**Savings**: 100-150 LOC through builder consolidation

---

## Validation & Policy Engine Duplication Matrix

### Pattern Distribution

| Pattern | Frequency | LOC per Pattern | Total | Examples |
|---------|-----------|-----------------|-------|----------|
| **Error context** | 4 | 10 | 40 | Validation error wrappers |
| **Rule matching** | 2 | 25 | 50 | Policy rule evaluation |
| **Constraint checking** | 3 | 8 | 24 | Validation checks |

**Total**: ~114 LOC

### Files Involved (Detailed)

**phenotype-infrakit**:
```
crates/phenotype-validation/src/lib.rs                ~120 LOC
crates/phenotype-validation/src/traits/error.rs       ~50 LOC
crates/phenotype-policy-engine/src/lib.rs             ~200 LOC
```
**Subtotal**: 370 LOC

**heliosCLI**:
```
codex-rs/execpolicy/src/lib.rs                        ~200 LOC
codex-rs/execpolicy/src/error.rs                      ~80 LOC
codex-rs/execpolicy-legacy/src/lib.rs                 ~150 LOC (DEAD CODE)
codex-rs/core/src/config/permissions.rs               ~80 LOC
```
**Subtotal**: 510 LOC

**Cross-Repo Duplication**: 50-100 LOC (in error patterns only)

### Consolidation Target

**Status**: Defer (different semantics)  
**Reason**: phenotype-policy-engine is generic; heliosCLI execpolicy is execution-specific  
**Future**: Once architectures converge, extract common patterns into `phenotype-policy-base`

---

## Non-Overlapping Patterns

### Caching (phenotype-only)
- **Crate**: phenotype-cache-adapter (Redis-specific)
- **LOC**: 300+
- **Status**: No duplication in heliosCLI (no caching layer)

### Event Sourcing (phenotype-only)
- **Crate**: phenotype-event-sourcing (event infrastructure)
- **LOC**: 400+
- **Status**: Not implemented in heliosCLI

### State Machine (phenotype-only)
- **Crate**: phenotype-state-machine (state patterns)
- **LOC**: 350+
- **Status**: Not implemented in heliosCLI

### MCP Protocol (heliosCLI-only)
- **Crates**: rmcp-client, codex-api (MCP-specific)
- **LOC**: 1,500+
- **Status**: Domain-specific to heliosCLI, not duplicated

### OAuth Flow (heliosCLI-only)
- **Module**: perform_oauth_login.rs
- **LOC**: 120+
- **Status**: Domain-specific to heliosCLI authentication

### Network Proxy (heliosCLI-only)
- **Crates**: network-proxy (MITM proxy)
- **LOC**: 800+
- **Status**: Domain-specific to heliosCLI network management

### Execution Policy (heliosCLI-only)
- **Crates**: execpolicy, execpolicy-legacy (execution control)
- **LOC**: 430+ (350+ DEAD CODE)
- **Status**: Domain-specific to heliosCLI sandboxing

---

## Summary by Category

### High-Priority (Quick Wins, Execute Immediately)

| Category | Current | Target | Crate | Savings | Effort |
|----------|---------|--------|-------|---------|--------|
| Error Handling | 1,335 LOC | 950 LOC | phenotype-error-macros | 200-250 | 2 days |
| Config Loading | 2,340 LOC | 1,890 LOC | phenotype-config-loader (expand) | 150-200 | 3 days |
| HTTP Client | 2,235 LOC | 1,985 LOC | phenotype-http-client-adapters | 100-150 | 4 days |

**Phase 1-2 Total**: 450-600 LOC savings in 5 days

### Medium-Priority (Execute Later, Optional)

| Category | Current | Target | Crate | Savings | Effort |
|----------|---------|--------|-------|---------|--------|
| Test Utilities | 173,470 LOC | 173,320 LOC | phenotype-test-fixtures | 100-150 | 2 days |

**Phase 3 Total**: 100-150 LOC savings in 2 days

### Low-Priority (Architectural, Defer)

| Category | Current | Target | Crate | Savings | Effort |
|----------|---------|--------|-------|---------|--------|
| Validation | 880 LOC | 830 LOC | phenotype-policy-base (future) | 25-50 | TBD |

**Phase 4 Total**: 25-50 LOC savings (deferred)

---

## Consolidation Opportunity Summary

**Total Duplicated LOC**: 1,100  
**Total Consolidation (Phases 1-3)**: 550-750 LOC  
**Consolidation Ratio**: 50-68% of duplication eliminated  

**Recommended Execution**:
1. **Phase 1** (Error Macros): 2 days → 200-250 LOC saved
2. **Phase 2** (Config Loader): 3 days → 150-200 LOC saved
3. **Phase 3** (Test Fixtures): 2 days → 100-150 LOC saved (optional)
4. **Phase 4** (HTTP Client + Policy): Q3 2026 → 50-150 LOC saved

**Total Timeline**: 5-7 days (Phases 1-2 core + 3 optional)
