# AgilePlus Specification Outlines for New Forks

**Date**: 2026-03-30 | **Status**: Prepared (Not Yet Implemented)

---

## 1. forgecode-fork: Custom Providers + Subagents Capability

### Epic Overview

**Epic ID**: `eco-fork-001`  
**Title**: Custom Providers & Subagent Management Infrastructure  
**Owner**: TBD  
**Timeline**: Phase 1 (Weeks 1-3), Phase 2 (Weeks 4-6)  
**Effort**: 18-24 parallel subagents across 6 work packages  
**Dependencies**: phenotype-infrakit (v0.2.0+), AgilePlus core

#### Description

forgecode-fork extends AgilePlus with a **provider abstraction layer** and **subagent dispatch system**. This enables:
- Dynamic provider registration (LLM, code generation, refactoring, documentation)
- Subagent spawning from spec work packages
- Provider capability discovery and routing
- Multi-model LLM strategies (fallback, ensemble, streaming)
- Execution tracing and provider performance metrics

**Vision**: Transform AgilePlus from a spec-driven engine into a **composable agent orchestration platform** where each provider (Claude, Grok, local models) and each agent type (implement, review, validate, test) can be independently registered, versioned, and routed.

#### Success Criteria

| Criterion | Description | Acceptance |
|-----------|-------------|-----------|
| **Provider Registry** | Pluggable provider interface; ≥3 implementations (Claude, local, streaming) | `cargo test -p forgecode-providers` all pass; 0 clippy warnings |
| **Subagent Spawning** | Spawn agents from CLI; track via agileplus-sqlite; cascade to MCP tools | `agileplus spawn-agent WP01 --provider claude --model opus --fallback grok` works end-to-end |
| **Capability Discovery** | Introspect provider capabilities (models, cost, latency, features) | `agileplus provider list` + `agileplus provider show claude` render JSON schema |
| **Execution Tracing** | Hash-chained audit trail of all provider calls (who, when, cost, token count) | `agileplus audit provider-calls --provider claude` shows all calls; verifiable hash chain |
| **Fallback & Routing** | Route work to providers based on cost/capability/availability | WP validation automatically retries on provider failure; cost optimization via routing rules |
| **Test Coverage** | Unit tests for each provider; integration tests for orchestration | `cargo test --all --workspace` ≥90% coverage on provider crates |

#### Key Deliverables

1. **forgecode-providers** crate: Provider trait, registry, three reference implementations
2. **agileplus-provider-cli** integration: New `provider` and `spawn-agent` subcommands
3. **forgecode-agent-dispatch** service: gRPC server for spawning subagents
4. **Provider audit trail** in SQLite with hash-chain verification
5. **Documentation**: Provider development guide + example custom provider

#### Non-Goals

- Cloud-based provider orchestration (local-only)
- Provider quota management (cost budgeting)
- Streaming response UI in dashboard (log aggregation only for Phase 1)
- Multi-tenant isolation

---

### Work Packages

#### WP01: Provider Trait & Registry Foundation

**Lane**: Planned | **Phase**: 1 | **Depends On**: None  
**Estimated LOC**: 400 | **Estimated Tool Calls**: 12-15  
**FRs**: FR-PROV01, FR-PROV02

**Objectives**:
- Define `Provider` trait with `invoke()`, `stream()`, `capabilities()` methods
- Implement `ProviderRegistry` (singleton + thread-safe registration)
- Add SQLite schema for provider metadata and call audit
- Write unit tests + trait documentation

**Subtasks**:
- T001: Create `forgecode-providers/src/lib.rs` with `Provider` trait skeleton
- T002: Implement `ProviderRegistry::register()` and `::get()`
- T003: Add SQLite `provider_metadata` and `provider_calls` tables
- T004: Write trait documentation + example provider stub
- T005: Unit tests for registry operations

**Acceptance Criteria**:
- `cargo check -p forgecode-providers` zero errors
- `cargo test -p forgecode-providers` all pass
- Registry supports ≥5 concurrent provider registrations
- Hash-chain audit record persists to SQLite

---

#### WP02: Claude Provider Implementation

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP01  
**Estimated LOC**: 350 | **Estimated Tool Calls**: 10-12  
**FRs**: FR-PROV03, FR-PROV04

**Objectives**:
- Implement `ClaudeProvider` adapter for anthropic-sdk
- Support multi-model routing (Opus, Sonnet, Haiku)
- Implement fallback to Sonnet on Opus failure
- Add streaming and non-streaming modes
- Trace all calls to audit table with token counts

**Subtasks**:
- T006: Create `forgecode-providers/src/claude.rs` with `ClaudeProvider` struct
- T007: Implement `Provider::invoke()` with error handling
- T008: Add streaming support via `Provider::stream()` (returns AsyncIterator)
- T009: Wire token counting and audit trail
- T010: Write integration tests with mock Claude API
- T011: Document model selection strategy and fallback behavior

**Acceptance Criteria**:
- `agileplus invoke --provider claude --model opus --prompt "test"` succeeds
- Fallback to Sonnet on network error
- Audit table records provider calls with token counts
- Stream mode works for long-form outputs

---

#### WP03: Local Provider + Ollama Integration

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP01  
**Estimated LOC**: 280 | **Estimated Tool Calls**: 8-10  
**FRs**: FR-PROV05, FR-PROV06

**Objectives**:
- Implement `LocalProvider` for Ollama-compatible inference servers
- Support dynamic model discovery from Ollama API
- Add health checking and graceful degradation
- Implement cost model (free / zero tokens for local)

**Subtasks**:
- T012: Create `forgecode-providers/src/local.rs` with `LocalProvider`
- T013: Implement Ollama API client with model discovery
- T014: Add health check endpoint (`/health`)
- T015: Implement graceful degradation (continue with warning if unavailable)
- T016: Unit tests with mock Ollama server
- T017: Cost model verification (local = free)

**Acceptance Criteria**:
- `agileplus invoke --provider local --model llama2 --prompt "test"` succeeds
- Health check detects unavailable Ollama and warns user
- Audit table shows zero cost for local invocations
- Model discovery works for dynamic Ollama model lists

---

#### WP04: Provider-Aware Subagent Spawning

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP02, WP03  
**Estimated LOC**: 420 | **Estimated Tool Calls**: 14-16  
**FRs**: FR-PROV07, FR-SPAWN01

**Objectives**:
- Add `spawn-agent` CLI subcommand to choose provider + model + WP
- Persist agent spawn records to SQLite
- Implement agent lifecycle tracking (spawned, running, completed, failed)
- Wire into gRPC agent dispatch service

**Subtasks**:
- T018: Create `agileplus-cli/src/commands/spawn_agent.rs`
- T019: Add `SpawnRequest` message to agileplus-proto
- T020: Create `agent_spawns` SQLite table with lifecycle states
- T021: Implement spawn logic: resolve WP → create Agent record → invoke dispatch gRPC
- T022: Add agent status tracking in CLI (`agileplus agent status <agent_id>`)
- T023: Integration test: spawn, track, verify audit trail
- T024: Error handling for invalid providers, missing WPs, resource limits

**Acceptance Criteria**:
- `agileplus spawn-agent WP01 --provider claude --model opus` creates agent record
- Agent status visible in `agileplus agent list`
- Audit trail shows agent lifecycle (spawned → completed)
- Provider fallback works if Opus unavailable (retries with Sonnet)

---

#### WP05: Capability Discovery & Routing

**Lane**: Planned | **Phase**: 2 | **Depends On**: WP02, WP03, WP04  
**Estimated LOC**: 380 | **Estimated Tool Calls**: 11-13  
**FRs**: FR-PROV08, FR-PROV09

**Objectives**:
- Expose provider `capabilities()` method (models, max tokens, latency, cost/1k)
- Add `provider` CLI subcommand for discovery
- Implement smart routing based on WP requirements (complexity, cost, latency)
- Store capability metadata in SQLite for historical analysis

**Subtasks**:
- T025: Add `ProviderCapabilities` struct with serializable metadata
- T026: Implement `capabilities()` for Claude, Local, Streaming providers
- T027: Create `agileplus provider list` and `agileplus provider show <name>`
- T028: Add routing logic (WP complexity → provider recommendation)
- T029: Persist provider capabilities snapshot to SQLite for auditing
- T030: Unit tests for capability matching and routing heuristics

**Acceptance Criteria**:
- `agileplus provider list` shows all registered providers
- `agileplus provider show claude` renders JSON with models, costs, latencies
- WP routing recommends Opus for complex tasks, Haiku for simple, Local for offline
- Capability metadata persisted and queryable via SQL

---

#### WP06: Provider Performance Metrics & Feedback Loop

**Lane**: Planned | **Phase**: 2 | **Depends On**: WP04, WP05  
**Estimated LOC**: 340 | **Estimated Tool Calls**: 10-12  
**FRs**: FR-PROV10, FR-PROV11

**Objectives**:
- Collect provider performance metrics (latency, token count, error rate)
- Aggregate metrics by model/provider
- Feedback loop: auto-tune routing based on performance history
- Expose metrics via CLI and HTTP API

**Subtasks**:
- T031: Create `provider_metrics` and `provider_performance_log` SQLite tables
- T032: Implement metrics collection in Provider trait (record latency, tokens)
- T033: Create `ProviderMetrics` aggregation struct
- T034: Add `agileplus provider metrics <provider>` CLI command
- T035: Implement feedback loop: read historical metrics → adjust routing weights
- T036: HTTP API endpoint `/metrics/providers` returning JSON
- T037: Unit tests for metric aggregation and feedback

**Acceptance Criteria**:
- Metrics recorded for every provider call
- `agileplus provider metrics claude` shows avg latency, token count, error rate
- Routing automatically adjusts if provider latency degrades
- Historical metrics queryable for analysis and optimization

---

### Functional Requirements (FRs)

| ID | Description | WP |
|----|-----------|----|
| FR-PROV01 | Provider trait with pluggable architecture | WP01 |
| FR-PROV02 | Thread-safe registry for provider registration | WP01 |
| FR-PROV03 | Claude provider with multi-model support | WP02 |
| FR-PROV04 | Provider fallback and error handling | WP02 |
| FR-PROV05 | Local provider for Ollama-compatible inference | WP03 |
| FR-PROV06 | Provider health checking and graceful degradation | WP03 |
| FR-PROV07 | Subagent spawning with provider selection | WP04 |
| FR-SPAWN01 | Agent lifecycle tracking (spawned, running, completed) | WP04 |
| FR-PROV08 | Provider capability discovery and introspection | WP05 |
| FR-PROV09 | Smart routing based on WP requirements | WP05 |
| FR-PROV10 | Provider performance metrics collection | WP06 |
| FR-PROV11 | Feedback-driven routing optimization | WP06 |

---

## 2. phenotype-router-monitor: Consolidated API Monitoring & Routing

### Epic Overview

**Epic ID**: `eco-fork-002`  
**Title**: Consolidated API Monitoring, Routing & Load Balancing  
**Owner**: TBD  
**Timeline**: Phase 1 (Weeks 1-2), Phase 2 (Weeks 3-4)  
**Effort**: 15-18 parallel subagents across 5 work packages  
**Dependencies**: phenotype-infrakit (v0.2.0+), phenotype-health, phenotype-contracts

#### Description

phenotype-router-monitor is a **unified API gateway** and **observability platform** for the Phenotype ecosystem. It consolidates:
- Request routing (path-based, load balancing, circuit breakers)
- Health monitoring (passive + active checks; aggregate status)
- Rate limiting (per-endpoint, per-client, adaptive backoff)
- Metrics aggregation (latency, error rate, throughput by service)
- Traffic replay and shadow routing for A/B testing

**Vision**: Single entry point for all Phenotype services with transparent observability, failure recovery, and gradual rollout support.

#### Success Criteria

| Criterion | Description | Acceptance |
|-----------|-------------|-----------|
| **Request Routing** | Route traffic by service name, path pattern, header-based | `curl http://router:8080/agileplus/spec` routes to AgilePlus backend |
| **Health Aggregation** | Poll ≥5 backend services; compute aggregate status | `GET /health` returns aggregated status; individual service health queryable |
| **Load Balancing** | Distribute traffic across ≥2 backend instances per service | `curl http://router:8080/heliosapp/*` load-balances across heliosapp-1, heliosapp-2 |
| **Circuit Breaker** | Fail-fast on repeated backend failures; auto-recovery | Backend down → router returns 503 after 5 failures; recovers after 30s |
| **Rate Limiting** | Per-endpoint rate limits; graceful rejection at capacity | Endpoint limited to 100 req/sec; excess requests get 429 + Retry-After header |
| **Metrics Export** | Prometheus-compatible metrics; ≥10 key metrics | `GET /metrics` exports latency percentiles, error rates, throughput |
| **Test Coverage** | Integration tests for all routing paths + failure scenarios | `cargo test -p phenotype-router-monitor` ≥85% coverage |

#### Key Deliverables

1. **phenotype-router-core** crate: Routing engine, middleware, configuration
2. **phenotype-router-health** crate: Health checker aggregation
3. **phenotype-router-limiter** crate: Rate limiting + circuit breaker
4. **phenotype-router-metrics** crate: Prometheus exporter
5. **phenotype-router-cli** integration: Route management + status commands
6. **Documentation**: Router setup guide, configuration reference, troubleshooting

#### Non-Goals

- Geographic routing / multi-region failover (single datacenter)
- mTLS authentication (relies on network-layer security)
- Response body transformation (transparent proxy only)
- WebSocket routing (HTTP/REST only)

---

### Work Packages

#### WP01: Router Core & Routing Engine

**Lane**: Planned | **Phase**: 1 | **Depends On**: None  
**Estimated LOC**: 450 | **Estimated Tool Calls**: 13-15  
**FRs**: FR-ROUTE01, FR-ROUTE02

**Objectives**:
- Implement core router using axum (async HTTP)
- Define `Route` configuration (service, path pattern, backend pool)
- Implement path-based routing with regex/wildcard matching
- Add request forwarding with header preservation
- Wire configuration loading from TOML

**Subtasks**:
- T001: Create `phenotype-router-core/src/lib.rs` with `Router` struct
- T002: Implement path pattern matching (regex + wildcard)
- T003: Create `Backend` pool with round-robin load balancing
- T004: Implement request forwarding with headers, query params
- T005: Add TOML configuration schema and loader
- T006: Create axum route handlers
- T007: Unit tests for routing logic and pattern matching
- T008: Integration test with 2 mock backends

**Acceptance Criteria**:
- `cargo check -p phenotype-router-core` zero errors
- Routing table loads from TOML config
- Request forwarded to correct backend; headers preserved
- Round-robin distributes requests across backends
- Configuration validation prevents invalid routes

---

#### WP02: Health Checking & Aggregation

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP01  
**Estimated LOC**: 380 | **Estimated Tool Calls**: 11-13  
**FRs**: FR-ROUTE03, FR-ROUTE04

**Objectives**:
- Implement `HealthChecker` (active HTTP probes + passive observation)
- Aggregate status across all backends
- Support custom health endpoints (`/health`, `/status`, custom paths)
- Implement exponential backoff for failing endpoints
- Expose aggregated status via HTTP endpoint

**Subtasks**:
- T009: Create `phenotype-router-health/src/lib.rs` with `HealthChecker` trait
- T010: Implement HTTP health probe (GET + custom status code thresholds)
- T011: Create `HealthAggregator` to track all backend states
- T012: Add passive observation (mark unhealthy on request failures)
- T013: Implement backoff strategy (exponential retry after N failures)
- T014: HTTP endpoint `GET /health` returning JSON with all backend states
- T015: Unit tests for health state transitions
- T016: Integration test: mark backend down, verify aggregator reflects it

**Acceptance Criteria**:
- `GET /health` returns JSON with ≥5 backend health states
- Aggregator marks backend unhealthy after 5 consecutive failures
- Unhealthy backend excluded from routing until next backoff retry
- Health status accurate within 5s of state change

---

#### WP03: Rate Limiting & Circuit Breaker

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP01  
**Estimated LOC**: 360 | **Estimated Tool Calls**: 10-12  
**FRs**: FR-ROUTE05, FR-ROUTE06

**Objectives**:
- Implement token-bucket rate limiter (per-endpoint)
- Add circuit breaker with half-open state for recovery
- Support adaptive backoff (increase limit as recovery succeeds)
- Return 429 + Retry-After header when rate limit exceeded
- Store limiter state in memory (no distributed coordination for Phase 1)

**Subtasks**:
- T017: Create `phenotype-router-limiter/src/lib.rs` with `RateLimiter` trait
- T018: Implement token-bucket algorithm with atomic operations
- T019: Create `CircuitBreaker` with states (closed, open, half-open)
- T020: Implement backoff logic (half-open retries every 30s)
- T021: Add rate limit headers to responses (X-RateLimit-Limit, X-RateLimit-Remaining, Retry-After)
- T022: TOML configuration for rate limits per endpoint
- T023: Unit tests for token bucket + circuit breaker state machines
- T024: Integration test: exceed rate limit, verify 429 response

**Acceptance Criteria**:
- Endpoint with limit=100 req/sec correctly rejects 101st request (429)
- Circuit breaker opens after 10 consecutive failures
- Half-open state allows 1 probe request every 30s
- Circuit breaker closes when probe succeeds
- Retry-After header set to backoff time on 429

---

#### WP04: Metrics Collection & Prometheus Export

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP01  
**Estimated LOC**: 340 | **Estimated Tool Calls**: 10-12  
**FRs**: FR-ROUTE07, FR-ROUTE08

**Objectives**:
- Collect request latency, response status codes, error rates per endpoint
- Aggregate metrics by service, path, status
- Export Prometheus-compatible metrics
- Expose HTTP endpoint `/metrics`

**Subtasks**:
- T025: Create `phenotype-router-metrics/src/lib.rs` with metric collectors
- T026: Implement histogram for request latency (p50, p95, p99)
- T027: Implement counters for requests by status code
- T028: Implement gauge for in-flight requests per service
- T029: Prometheus exporter format + `/metrics` endpoint
- T030: Integrate metrics middleware into router
- T031: Unit tests for metric collection accuracy
- T032: Verify Prometheus scrape format compliance

**Acceptance Criteria**:
- `/metrics` endpoint returns Prometheus text format
- Latency histogram shows p50, p95, p99 buckets
- Status code counters track 2xx, 4xx, 5xx separately
- In-flight gauge reflects current request count

---

#### WP05: Router CLI & Status Dashboard

**Lane**: Planned | **Phase**: 2 | **Depends On**: WP01-WP04  
**Estimated LOC**: 320 | **Estimated Tool Calls**: 9-11  
**FRs**: FR-ROUTE09, FR-ROUTE10

**Objectives**:
- Add `router` CLI subcommand to AgilePlus
- Show status of all routes, backends, health
- List aggregated metrics
- Enable/disable routes without restarting
- Add status dashboard (TUI or HTTP)

**Subtasks**:
- T033: Create `agileplus-cli/src/commands/router.rs`
- T034: Implement `router status` command (shows all routes + backend states)
- T035: Implement `router backends <route>` (list backends + health)
- T036: Implement `router metrics <route>` (show latency, error rate)
- T037: Implement `router disable <route>` / `router enable <route>`
- T038: HTTP API endpoints for router admin
- T039: TUI dashboard showing router status (ratatui)
- T040: Integration test: verify CLI shows accurate metrics

**Acceptance Criteria**:
- `agileplus router status` renders table with all routes
- `agileplus router backends agileplus` shows health of each backend
- `agileplus router metrics agileplus` shows latency percentiles
- Can disable/enable route via CLI; traffic is immediately routed/not-routed

---

### Functional Requirements (FRs)

| ID | Description | WP |
|----|-----------|----|
| FR-ROUTE01 | Path-based request routing with regex/wildcard matching | WP01 |
| FR-ROUTE02 | Round-robin load balancing across backend pool | WP01 |
| FR-ROUTE03 | Active health checking via HTTP probes | WP02 |
| FR-ROUTE04 | Health aggregation across all backends | WP02 |
| FR-ROUTE05 | Token-bucket rate limiting per endpoint | WP03 |
| FR-ROUTE06 | Circuit breaker with exponential backoff | WP03 |
| FR-ROUTE07 | Request latency histogram collection | WP04 |
| FR-ROUTE08 | Prometheus-compatible metrics export | WP04 |
| FR-ROUTE09 | CLI commands for route and backend management | WP05 |
| FR-ROUTE10 | Status dashboard (TUI) for router monitoring | WP05 |

---

## 3. bifrost-routing: LLM Routing Infrastructure

### Epic Overview

**Epic ID**: `eco-fork-003`  
**Title**: LLM Routing Infrastructure with Model Selection & Cost Optimization  
**Owner**: TBD  
**Timeline**: Phase 1 (Weeks 1-2), Phase 2 (Weeks 3-4)  
**Effort**: 16-20 parallel subagents across 6 work packages  
**Dependencies**: forgecode-fork (providers), phenotype-infrakit (v0.2.0+)

#### Description

bifrost-routing is a **specialized router** for LLM inference requests with intelligent model selection, cost optimization, and latency tradeoffs. It builds on forgecode-fork providers and phenotype-router-monitor infrastructure to:
- Route inference requests by workload type (coding, analysis, writing, retrieval)
- Select optimal model based on cost/latency/quality SLAs
- Implement token-aware routing (batch small requests to Haiku, large to Opus)
- Support A/B testing and shadow routing for model evaluations
- Track cost per request and enforce budget caps

**Vision**: Transform LLM selection from manual ("use Opus for everything") into data-driven ("route to Haiku/Sonnet/Opus based on historical performance and cost")

#### Success Criteria

| Criterion | Description | Acceptance |
|-----------|-------------|-----------|
| **Workload Routing** | Route by inferred workload type (code, analysis, write) | Request classifier labels each request; router selects model based on label |
| **Token-Aware Routing** | Route to Haiku if <500 tokens, Sonnet if <2K, Opus otherwise | Router inspects prompt length; selects model; cost tracking accurate |
| **Cost Optimization** | Reduce costs by 30-40% via smart routing | Historical data shows 35% cost reduction vs baseline (all Opus) |
| **SLA Enforcement** | Latency SLA per workload; fail-over if SLA breached | If Sonnet exceeds 2s latency, retry with Opus; track SLA breaches |
| **A/B Testing** | Shadow route to new model; collect metrics without routing traffic | `bifrost shadow <request> --model sonnet` logs metrics without changing routing |
| **Budget Enforcement** | Enforce per-user, per-project cost caps | User exceeding monthly budget gets 429 + clear error message |
| **Test Coverage** | Unit + integration tests for routing logic | `cargo test -p bifrost-routing` ≥90% coverage |

#### Key Deliverables

1. **bifrost-routing-core** crate: Routing engine, request classifier, SLA enforcement
2. **bifrost-routing-models** crate: LLM model definitions, cost/latency metadata
3. **bifrost-routing-analytics** crate: Cost tracking, A/B metrics, performance analysis
4. **bifrost-routing-cli** integration: Routing management, SLA tuning commands
5. **Documentation**: Model selection guide, cost optimization strategies, SLA tuning

#### Non-Goals

- Dynamic model fine-tuning based on feedback
- Multi-provider LLM routing (Claude-only for Phase 1)
- User-level SLA customization (global SLAs only for Phase 1)
- Cost prediction models (historical tracking only)

---

### Work Packages

#### WP01: Request Classifier & Workload Inference

**Lane**: Planned | **Phase**: 1 | **Depends On**: None  
**Estimated LOC**: 380 | **Estimated Tool Calls**: 11-13  
**FRs**: FR-BIFROST01, FR-BIFROST02

**Objectives**:
- Implement `RequestClassifier` to infer workload type from request content
- Support 4 workload types: code, analysis, writing, retrieval
- Implement heuristic-based classification (keyword matching, length, complexity)
- Add SQLite schema for classification history
- Expose `/classify` API endpoint

**Subtasks**:
- T001: Create `bifrost-routing-models/src/workload.rs` with `WorkloadType` enum
- T002: Create `bifrost-routing-core/src/classifier.rs` with classification rules
- T003: Implement rule engine (keyword weights, prompt length factor, complexity heuristic)
- T004: Add SQLite schema for classification audit trail
- T005: Create HTTP endpoint POST `/classify` accepting JSON request
- T006: Unit tests for classifier accuracy on mock prompts
- T007: Edge cases: empty prompt, very long prompt, mixed workload

**Acceptance Criteria**:
- Classifier correctly labels "write a function" as code, "analyze sales data" as analysis
- Classification stored in SQLite for analysis
- `/classify` endpoint returns JSON with workload label + confidence score
- Classification latency <10ms

---

#### WP02: LLM Model Registry & Capability Metadata

**Lane**: Planned | **Phase**: 1 | **Depends On**: None  
**Estimated LOC**: 300 | **Estimated Tool Calls**: 8-10  
**FRs**: FR-BIFROST03, FR-BIFROST04

**Objectives**:
- Create model registry with Claude models (Opus, Sonnet, Haiku)
- Store cost/token, latency SLA, accuracy tier, training data cutoff
- Support adding custom models (local, other providers for Phase 2)
- Expose model discovery API

**Subtasks**:
- T008: Create `bifrost-routing-models/src/registry.rs` with `ModelRegistry`
- T009: Define `ModelSpec` struct with cost, latency, accuracy metadata
- T010: Hard-code Claude models (Opus: $15/1M input, Sonnet: $3/1M, Haiku: $0.80/1M)
- T011: Add SQLite schema for model metadata versioning
- T012: HTTP endpoint GET `/models` returning all models + specs
- T013: Unit tests for model selection logic
- T014: Edge case: unsupported model name

**Acceptance Criteria**:
- Registry stores ≥3 Claude models with accurate cost/latency metadata
- `/models` returns JSON list with all models
- Model lookup by name succeeds; invalid names return error
- Metadata versioned in SQLite for historical cost tracking

---

#### WP03: Token-Aware & Workload-Based Model Selection

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP01, WP02  
**Estimated LOC**: 400 | **Estimated Tool Calls**: 12-14  
**FRs**: FR-BIFROST05, FR-BIFROST06

**Objectives**:
- Implement `ModelSelector` to choose model based on prompt length + workload type
- Define routing rules (code+short → Haiku, analysis+long → Opus, etc.)
- Implement cost-aware fallback (if cost SLA exceeded, degrade gracefully)
- Add TOML configuration for routing rules

**Subtasks**:
- T015: Create `bifrost-routing-core/src/selector.rs` with `ModelSelector`
- T016: Implement token counting from prompt (approximate via length)
- T017: Define routing rules matrix (workload × token-range → model)
- T018: Implement cost heuristic (cost per query < user budget)
- T019: Add TOML config schema for routing rules
- T020: Create HTTP endpoint POST `/select-model` accepting request
- T021: Unit tests for selection logic (all combinations of workload + token range)
- T022: Integration test: verify selected model matches expected based on inputs

**Acceptance Criteria**:
- Token-aware: <500 tokens → Haiku, 500-2000 → Sonnet, >2000 → Opus
- Workload-aware: code → prefer Haiku/Sonnet; analysis → prefer Opus
- Cost-aware: if cost exceeds cap, use cheaper model or return error
- `/select-model` returns model name + estimated cost

---

#### WP04: SLA Enforcement & Latency Timeout

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP03  
**Estimated LOC**: 320 | **Estimated Tool Calls**: 9-11  
**FRs**: FR-BIFROST07, FR-BIFROST08

**Objectives**:
- Define SLA per workload (code: <5s, analysis: <10s, writing: <15s)
- Monitor request latency against SLA
- Implement failover (if SLA breached, retry with faster model or timeout)
- Track SLA breach rate per workload

**Subtasks**:
- T023: Create `bifrost-routing-core/src/sla.rs` with `SLAPolicy`
- T024: Define SLA matrix in TOML config (workload → max latency)
- T025: Implement SLA monitoring (record start time → compare to SLA at end)
- T026: Implement failover logic (latency exceeds SLA → try Haiku, then Sonnet)
- T027: Add SQLite schema for SLA tracking (query time, SLA limit, breached)
- T028: Create HTTP endpoint POST `/invoke-with-sla` enforcing SLA
- T029: Unit tests for SLA evaluation and failover
- T030: Integration test: verify failover triggers on SLA breach

**Acceptance Criteria**:
- SLA enforced per request (measure latency, compare to SLA)
- On SLA breach, failover to faster model automatically
- `/invoke-with-sla` succeeds if SLA met; fails gracefully if SLA exceeded
- SLA breach tracked in SQLite for reporting

---

#### WP05: Cost Tracking & Budget Enforcement

**Lane**: Planned | **Phase**: 1 | **Depends On**: WP02, WP03, WP04  
**Estimated LOC**: 360 | **Estimated Tool Calls**: 10-12  
**FRs**: FR-BIFROST09, FR-BIFROST10

**Objectives**:
- Track cost per LLM request (input tokens × model cost)
- Implement user/project budget caps (monthly + daily)
- Enforce budget (return 429 when exceeded)
- Expose cost tracking via API and CLI

**Subtasks**:
- T031: Create `bifrost-routing-analytics/src/cost_tracker.rs`
- T032: Implement cost calculation (prompt tokens × model cost/1M)
- T033: Add SQLite schema for cost ledger (user, project, cost, timestamp)
- T034: Implement budget enforcement (sum last 30 days costs; compare to budget)
- T035: Create HTTP endpoint POST `/budget/check` returning remaining budget
- T036: Create CLI command `bifrost budget set <user> <monthly_limit>`
- T037: Create CLI command `bifrost budget show <user>` showing usage + remaining
- T038: Unit tests for cost calculation and budget enforcement
- T039: Integration test: exceed budget, verify 429 response

**Acceptance Criteria**:
- Cost accurate: 1000 tokens @ Haiku ($0.80/1M) = $0.0008 per request
- Budget enforced: user with $10/month cap exceeding usage gets 429
- `/budget/check` returns remaining budget in dollars
- Cost ledger auditable and queryable per user/project/time

---

#### WP06: A/B Testing & Shadow Routing

**Lane**: Planned | **Phase**: 2 | **Depends On**: WP03, WP05  
**Estimated LOC**: 340 | **Estimated Tool Calls**: 10-12  
**FRs**: FR-BIFROST11, FR-BIFROST12

**Objectives**:
- Shadow route requests to alternate model for evaluation
- Collect metrics (latency, quality, cost) without affecting user traffic
- Support A/B experiment configuration (e.g., "shadow 10% to Sonnet")
- Expose shadow routing results via API and analytics dashboard

**Subtasks**:
- T040: Create `bifrost-routing-core/src/shadow.rs` with `ShadowRouter`
- T041: Implement shadow request logic (route to experiment model in background)
- T042: Add SQLite schema for shadow metrics (original model, shadow model, latency, cost)
- T043: Create HTTP endpoint POST `/invoke-with-shadow` accepting experiment config
- T044: Implement experiment sampling (e.g., shadow 10% of requests)
- T045: Create CLI command `bifrost shadow list` showing active experiments
- T046: Create CLI command `bifrost shadow results <experiment>` showing metrics
- T047: Unit tests for shadow routing logic and metric collection
- T048: Integration test: run shadow experiment, verify metrics collected

**Acceptance Criteria**:
- Shadow requests don't affect user response (background execution)
- Metrics collected: latency, cost, quality (success/failure)
- `/invoke-with-shadow` supports sampling percentage
- `bifrost shadow results` shows aggregated metrics (avg latency, cost delta)

---

### Functional Requirements (FRs)

| ID | Description | WP |
|----|-----------|----|
| FR-BIFROST01 | Request classifier inferring workload type | WP01 |
| FR-BIFROST02 | Classification audit trail in SQLite | WP01 |
| FR-BIFROST03 | LLM model registry with cost/latency metadata | WP02 |
| FR-BIFROST04 | Model discovery API and CLI | WP02 |
| FR-BIFROST05 | Token-aware model selection | WP03 |
| FR-BIFROST06 | Workload-based model routing | WP03 |
| FR-BIFROST07 | SLA enforcement per workload | WP04 |
| FR-BIFROST08 | Failover on SLA breach | WP04 |
| FR-BIFROST09 | Cost tracking per request | WP05 |
| FR-BIFROST10 | Budget enforcement and cap enforcement | WP05 |
| FR-BIFROST11 | A/B testing via shadow routing | WP06 |
| FR-BIFROST12 | Shadow routing metrics collection and analysis | WP06 |

---

## Summary & Execution Strategy

### Cross-Fork Dependencies

```
forgecode-fork (eco-fork-001)
  ↓ (provides Provider abstraction)
bifrost-routing (eco-fork-003)
  ↓ (routes to providers)
phenotype-router-monitor (eco-fork-002)
  ↓ (monitors bifrost + others)
AgilePlus (integrates all three)
```

### Parallel Execution

- **Phase 1A (Weeks 1-2)**: WP01 across all three forks (foundational work)
  - forgecode-fork: WP01 (Provider trait)
  - phenotype-router-monitor: WP01 (Routing engine)
  - bifrost-routing: WP01-WP02 (Classification + model registry)

- **Phase 1B (Weeks 2-3)**: Core implementations
  - forgecode-fork: WP02-WP04 (Provider impls, spawning, discovery)
  - phenotype-router-monitor: WP02-WP04 (Health, rate limiting, metrics)
  - bifrost-routing: WP03-WP05 (Selection, SLA, cost tracking)

- **Phase 2 (Weeks 4-6)**: Advanced features + integration
  - forgecode-fork: WP05-WP06 (Routing + feedback loop)
  - phenotype-router-monitor: WP05 (CLI + dashboard)
  - bifrost-routing: WP06 (A/B testing + shadow routing)

### Total Effort

- **Parallel Subagents per Phase**: 12-15 agents (Phase 1A), 18-20 agents (Phase 1B/2)
- **Wall-Clock Time**: ~4-6 weeks for all three forks (v0.1.0 release)
- **Tool Calls per Fork**: 72-99 (forgecode), 54-66 (phenotype-router), 72-90 (bifrost)
- **Total LOC**: ~4,800-5,200 across all three forks

### Acceptance & Quality Gates

- **Build**: `cargo build --all --release` zero warnings
- **Test**: `cargo test --workspace` ≥85% coverage per fork
- **Lint**: `cargo clippy --workspace -- -D warnings` zero warnings
- **Docs**: All public APIs documented; examples provided
- **Integration**: All FRs traced to ≥1 test; all tests passing

---

## Next Steps (When User Approves)

1. Create AgilePlus specs for each fork in `.agileplus/specs/`
2. Create work package directories with task templates
3. Activate 12-15 subagents per fork for Phase 1A
4. Track progress in AgilePlus dashboard
5. Weekly rollup reports for user review

---

**End of Specification Outlines**
