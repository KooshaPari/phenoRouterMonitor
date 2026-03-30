# Consolidation Roadmap: Routing & Cost Core Extraction

**Document Type:** Implementation Roadmap
**Date:** 2026-03-30
**Status:** Ready for Phase 1 Planning
**Effort Estimate:** 14-20 weeks, 5-8 parallel agents

---

## Overview

Extract Phenotype's 43-router ecosystem and cost tracking logic into **4 shareable Rust crates** that can be consumed by:
- **VibeProxy** (if macOS fork is activated in Phase 3+)
- **Phenotype ecosystem** (thegent, heliosCLI, etc.)
- **External harnesses** (Cursor, Roo Code, Kilo Code)
- **OSS projects** via crates.io

This enables **fast iteration** on routing algorithms and cost models without cross-project coordination bottlenecks.

---

## Phase 1: Extract Shared Libraries (4-6 weeks)

### Crate 1: `phenotype-cost-core` (3 weeks, 1-2 agents)

**Purpose:** Generic cost calculation + token pricing database

**Components:**

1. **Token Counter Trait** (Rust trait + PyO3 binding)
   ```rust
   pub trait TokenCounter {
       fn count_input(&self, text: &str) -> usize;
       fn count_output(&self, text: &str) -> usize;
   }
   ```
   - Implementations: OpenAI (GPT-3.5, GPT-4), Anthropic (Claude variants), Google (Gemini), etc.
   - ~800 LOC

2. **Provider Pricing Registry**
   ```rust
   pub struct ModelPrice {
       provider: String,
       model: String,
       input_usd_per_1k: f64,
       output_usd_per_1k: f64,
       region: Option<String>, // "us-east", etc.
   }
   ```
   - Extracted from: `thegent/utils/routing_impl/cost_calculator.py` + model_metadata.py
   - Versioning: Semantic (major = new provider, minor = new models, patch = price updates)
   - ~2,000 LOC (data structures + registry)

3. **Cost Calculator**
   ```rust
   pub fn calculate_cost(
       model: &str,
       input_tokens: usize,
       output_tokens: usize,
   ) -> Result<f64, CostError>
   ```
   - Extract from: `cost_calculator.py` (5.2K LOC)
   - ~500 LOC

4. **Budget Tracker**
   ```rust
   pub struct Budget {
       limit_usd: f64,
       current_spent: f64,
       alerts: Vec<Alert>, // @50%, @75%, @90%
   }
   ```
   - Extract from: `budget.py` (8.1K LOC)
   - ~1,200 LOC

**Deliverables:**
- `phenotype-cost-core` crate with examples
- Python bindings via PyO3 (for VibeProxy Python integration)
- Tests: 100+ property-based tests for price accuracy
- Documentation: Versioning policy, model update procedure
- CI/CD: Automated pricing validation against public APIs

**Blockers:** None

**Metrics:**
- Input tokens ±5% accuracy vs official APIs
- Budget enforcement latency <10ms
- Zero runtime panics

---

### Crate 2: `phenotype-models` (2-3 weeks, 1-2 agents)

**Purpose:** Unified model registry + metadata

**Components:**

1. **Model Registry Structure**
   ```rust
   pub struct ModelMetadata {
       id: String, // "gpt-4-turbo", "claude-3-opus", etc.
       provider: Provider,
       context_window: usize,
       supports_vision: bool,
       supports_streaming: bool,
       max_output_tokens: Option<usize>,
       capabilities: Vec<String>, // ["code", "reasoning", "vision"]
       status: ModelStatus, // active, deprecated, preview
       released: Date,
       deprecated_date: Option<Date>,
   }
   ```
   - Extract from: `model_metadata.py` (14.3K LOC)
   - ~1,500 LOC

2. **Provider Endpoints**
   ```rust
   pub struct ProviderConfig {
       name: String, // "openai", "anthropic", etc.
       base_url: String,
       auth_type: AuthType, // api_key, oauth, etc.
       rate_limits: RateLimitConfig,
       supported_models: Vec<String>,
   }
   ```
   - ~800 LOC

3. **Model Alias Resolver**
   ```rust
   pub fn resolve_alias(alias: &str) -> Result<&ModelMetadata>
   // Handles: "gpt4" → "gpt-4-turbo", "claude" → "claude-3-opus", etc.
   ```
   - Extract from: `harness_model_mapping.py` (5.7K LOC)
   - ~600 LOC

4. **Semantic Versioning Policy**
   - MAJOR (0.X.0): Breaking changes (model removal, capability changes)
   - MINOR (X.Y.0): New models, new providers
   - PATCH (X.Y.Z): Price updates, bug fixes
   - Release triggered by: AgilePlus spec eco-MODELS-RELEASE
   - Example: 0.3.0 adds GPT-4o, deprecates text-davinci-003

**Deliverables:**
- `phenotype-models` crate with embedded JSON registry
- Model update CLI tool: `phenotype-models-update --add gpt-4o`
- Tests: Verify all models have pricing, endpoints, capabilities
- Validation: No duplicate aliases, no circular aliases
- Documentation: How to add new models

**Blockers:** Need to scrape/maintain official model lists

**Metrics:**
- Model lookup: <1µs (in-memory)
- Registry JSON <50KB gzipped
- 99.9% uptime of model availability

---

### Crate 3: `phenotype-observability` (1-2 weeks, 1 agent)

**Purpose:** Standardized observability types and metrics

**Components:**

1. **Header Types** (GW-* series from cliproxy_adapter.py)
   ```rust
   pub struct ObservabilityHeaders {
       event_id: String, // "tg-<8hex>" (GW-35)
       ttft_ms: Option<f64>, // Time-to-first-token (GW-38)
       fallback_step: u32, // 0 = primary, 1+ = fallback (GW-36)
       response_cost_usd: Option<f64>, // GW-48
       cache_status: Option<CacheStatus>, // HIT | MISS (GW-24)
       cache_ttl_seconds: Option<u32>,
   }
   ```
   - Extract from: `cliproxy_adapter.py` (lines 195-500)
   - ~800 LOC

2. **Metrics Definitions**
   ```rust
   pub enum Metric {
       TokensProcessed(usize),
       CostUSD(f64),
       LatencyMs(f64),
       CacheHit,
       CacheMiss,
       ErrorCount(ErrorType),
       FallbackActivations,
   }
   ```
   - ~400 LOC

3. **Tracing Context**
   ```rust
   pub struct TraceContext {
       trace_id: String,
       span_id: String,
       parent_span_id: Option<String>,
       metadata: Map<String, String>,
   }
   ```
   - ~300 LOC

**Deliverables:**
- `phenotype-observability` crate (serializable JSON/binary)
- Integration with `tracing` crate for Rust
- Python bindings via PyO3
- Tests: Round-trip serialization, header validation

**Blockers:** None

**Metrics:**
- Serialization overhead <1ms
- JSON size <500 bytes per request
- Zero allocations on hot path

---

### Crate 4: `phenotype-router-traits` (2-3 weeks, 1-2 agents)

**Purpose:** Pluggable router interface + common patterns

**Components:**

1. **Core Router Trait**
   ```rust
   #[async_trait]
   pub trait Router {
       async fn route(&self, request: &RoutingRequest) -> Result<RoutingDecision>;
   }

   pub struct RoutingRequest {
       model: String,
       input_tokens: usize,
       metadata: Map<String, String>, // tags, user_id, etc.
       budget_remaining: f64,
       preferences: UserPreferences,
   }

   pub struct RoutingDecision {
       provider: Provider,
       model: String,
       fallback_providers: Vec<Provider>,
       cost_estimate: f64,
       latency_estimate_ms: f64,
   }
   ```
   - ~600 LOC

2. **Error Types**
   ```rust
   pub enum RoutingError {
       NoBudget { limit: f64, required: f64 },
       NoAvailableProvider,
       ConfigError(String),
       RateLimited { reset_after_ms: u64 },
   }
   ```
   - ~200 LOC

3. **Common Implementations**
   ```rust
   // Adapters for existing 43 routers
   pub struct CostAwareRouter { ... }
   pub struct CircuitBreakerRouter { ... }
   pub struct ParsingRouter { ... }
   // (each wrapped to implement the trait)
   ```
   - ~2,000 LOC (adapters for 10-15 key routers)

4. **Router Composition** (composite pattern)
   ```rust
   pub struct CompositeRouter {
       routers: Vec<Box<dyn Router>>,
       strategy: CompositionStrategy, // first_success, weighted_vote, etc.
   }
   ```
   - ~800 LOC

**Deliverables:**
- `phenotype-router-traits` crate
- Adapter implementations for 10-15 key routers (cost-aware, circuit-breaker, pareto, etc.)
- Python bindings (PyO3)
- Examples: Building custom routers
- Tests: Mock routers, composition tests

**Blockers:** Requires stable interface agreement with LiteLLM router

**Metrics:**
- Router latency: <5ms for decision
- Memory footprint per router: <10MB
- No blocked threads (all async)

---

## Phase 2: Migrate VibeProxy (2-3 weeks) — *Conditional*

**Only activate if VibeProxy fork becomes active in Phase 3+**

### Tasks

1. **Replace cost calculations**
   - Use `phenotype-cost-core` instead of CLIProxyAPIPlus inline calculations
   - Estimated effort: 3 days

2. **Migrate model registry**
   - Use `phenotype-models` instead of embedded model list
   - Add model update workflow to VibeProxy CI
   - Estimated effort: 2 days

3. **Adopt observability headers**
   - Add `phenotype-observability` headers to VibeProxy responses
   - Integrate with macOS menu bar UI (display TTFT, cost, fallback count)
   - Estimated effort: 4 days

4. **Test compatibility**
   - Ensure VibeProxy + cliproxyapi-plusplus work with new libraries
   - Integration tests with Phenotype routing layer
   - Estimated effort: 3 days

**Deliverable:** VibeProxy PR with all 4 shared libraries as dependencies

---

## Phase 3: Consolidate Phenotype Routers (6-8 weeks)

### Tasks

1. **Create router adapter layer**
   - Implement `Router` trait for each of 43 routers
   - ~1,500 LOC of adapters
   - Estimated effort: 3 weeks, 2 agents

2. **Deprecate legacy inline implementations**
   - Mark `routing_impl/*.py` as deprecated (legacy)
   - Migrate all tests to use new trait-based routers
   - Estimated effort: 2 weeks, 1 agent

3. **Publish shared libraries to crates.io**
   - `phenotype-cost-core`, `phenotype-models`, `phenotype-observability`, `phenotype-router-traits`
   - CI/CD: Automated version bumps tied to releases
   - Estimated effort: 3 days, 1 agent

4. **Python bindings (PyO3)**
   - All 4 crates get `pyo3` bindings for Phenotype use
   - Generate stub files for IDE support
   - Estimated effort: 2 weeks, 1 agent

5. **Documentation**
   - Migration guide for custom routers
   - API reference (Rust + Python)
   - Examples: Building new router types
   - Estimated effort: 1 week, 1 agent

---

## Phase 4: Deprecation & Cleanup (2 weeks)

### Tasks

1. **Remove legacy code**
   - Delete Python implementations from `routing_impl/` (not used by default)
   - Archive to `.archive/routing_impl_legacy_py/`
   - Estimated effort: 2 days

2. **Final testing**
   - Integration tests: Phenotype + VibeProxy (if applicable) + external harnesses
   - Load testing: 1000 req/s with new routers
   - Estimated effort: 3 days

3. **Release notes**
   - Changelog entry: "Extracted routing + cost libraries"
   - Migration guide for users
   - Estimated effort: 2 days

---

## Effort Breakdown

| Phase | Tasks | Weeks | Agents | Output |
|-------|-------|-------|--------|--------|
| **Phase 1** | 4 crates | 4-6 | 5-8 | 4 published crates, 100+ tests |
| **Phase 2** | VibeProxy integration (conditional) | 2-3 | 2 | 1 PR to VibeProxy (if fork active) |
| **Phase 3** | Router consolidation | 6-8 | 4-5 | Phenotype using all 4 crates, crates.io published |
| **Phase 4** | Cleanup + documentation | 2 | 2 | Released version, migration guide |
| **TOTAL** | | **14-20** | 5-8 | **4 crates, 1 Phenotype release, optional VibeProxy PR** |

---

## Success Criteria

### Phase 1
- [ ] All 4 crates compile without warnings
- [ ] 100+ property-based tests pass (pricing accuracy, token counting)
- [ ] PyO3 bindings work with Python 3.8+
- [ ] Documentation complete (README, examples, versioning policy)
- [ ] Published to crates.io with 1000+ downloads (first month)

### Phase 2
- [ ] VibeProxy uses all 4 crates (if fork activated)
- [ ] VibeProxy CI passes with new dependencies
- [ ] Integration tests: VibeProxy ↔ Phenotype routing layer

### Phase 3
- [ ] All 43 routers adapted to trait-based interface
- [ ] Phenotype tests pass (100% coverage of routing)
- [ ] Performance: router decision latency <5ms (p99)
- [ ] Legacy Python code archived, not deleted

### Phase 4
- [ ] No Python code in `routing_impl/*.py` (all legacy)
- [ ] Integration tests pass with new clean architecture
- [ ] Migration guide published
- [ ] Release notes: "Phenotype v0.4.0 — Extracted routing core"

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| **PyO3 binding performance** | Medium | Benchmark early (Phase 1, week 2); profile hot paths |
| **LiteLLM compatibility** | High | Coordinate with LiteLLM maintainers; test adapter before Phase 3 |
| **Model registry staleness** | Medium | Automate updates from APIs (OpenAI, Anthropic, Google); CI/CD verification |
| **VibeProxy divergence** | Low | If fork not activated by Q4 2026, defer Phase 2 indefinitely |
| **Breaking changes** | Low | Use trait versioning; minor version bumps for new routers |

---

## Dependencies & Blockers

| Item | Type | Resolution |
|------|------|-----------|
| **LiteLLM interface stability** | Blocker | Verify with LiteLLM maintainers that router interface is stable |
| **Model pricing accuracy** | Dependency | Automate price verification against official APIs (weekly) |
| **PyO3 maturity** | Dependency | PyO3 v0.21+ required; verify with Python 3.11 |
| **VibeProxy fork decision** | Conditional | Defer Phase 2 until fork is activated (decision point: Q4 2026) |

---

## Deliverables Checklist

### Phase 1
- [ ] `phenotype-cost-core` crate (Rust + PyO3)
- [ ] `phenotype-models` crate (with JSON registry)
- [ ] `phenotype-observability` crate
- [ ] `phenotype-router-traits` crate
- [ ] 4 crates on crates.io
- [ ] 100+ tests (unit, property-based, integration)
- [ ] Documentation (README, examples, API reference)
- [ ] Model update CLI tool

### Phase 2 (conditional)
- [ ] VibeProxy fork using all 4 crates
- [ ] VibeProxy PR with integration tests

### Phase 3
- [ ] All 43 routers adapted to trait-based
- [ ] Python bindings for all 4 crates
- [ ] Phenotype v0.4.0 release
- [ ] Legacy code archived

### Phase 4
- [ ] Migration guide published
- [ ] Release notes complete
- [ ] Zero legacy Python router code in active use

---

## Timeline Estimate

```
2026-04 (6 weeks):     Phase 1 — Extract 4 crates, 100+ tests, crates.io publish
2026-05-06 (4 weeks):  Phase 2 — VibeProxy integration (if activated)
2026-06-07 (8 weeks):  Phase 3 — Consolidate Phenotype routers, Python bindings
2026-08 (2 weeks):     Phase 4 — Cleanup, documentation, release
───────────────────────────────────────────────────────────
2026-09 (target):      Phenotype v0.4.0 released with routing core
```

---

## Next Steps

1. **Approve Phase 1 scope** in AgilePlus
2. **Create 4 specs** (eco-COST-CORE, eco-MODELS, eco-OBSERVABILITY, eco-ROUTER-TRAITS)
3. **Assign agents** to each crate (2 agents per crate for parallelism)
4. **Schedule kick-off**: 2026-04-01 (start Phase 1)
5. **Monitor progress** via AgilePlus worklog weekly

---

**Roadmap Owner:** Architecture Team
**Approval:** @user
**Status:** Ready for Phase 1 Planning
**Last Updated:** 2026-03-30
