# 2026 Tech Radar — Latest Versions & Emerging Alternatives

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0 (Critical for ecosystem planning)
**Research Date:** 2026-03-29
**Sources:** WebSearch of latest crate registries, 2026 blog posts, framework release notes

## Executive Summary

Comprehensive 2026 package research across Rust, Go, TypeScript, and Python ecosystems. This section captures the latest stable versions and emerging technologies that should be evaluated for Phenotype adoption. Total: 25+ new research entries with version specifics, GitHub stars, and actionable recommendations.

### Key Findings

- **Rust ORM landscape stabilized**: SeaORM 2.0 (January 2026) now production-ready as best async choice over sqlx for high-level queries
- **Go dependency injection matured**: Uber Fx (1.20+) preferred over manual wiring; slog stdlib required (logrus deprecated)
- **TypeScript ecosystem consolidating**: Drizzle 0.30+, tRPC 11.x, Zod 3.23 forming de facto standard stack
- **Python/FastAPI migration path**: Pydantic v2 (0.25+) now required; v1 deprecated; migrate from Django → FastAPI
- **AI/LLM integration standardized**: Claude Agent SDK 0.2+ (Node.js), 0.1.48 (Python); LangGraph 1.0+ stable (Oct 2025); CrewAI 0.47+ with A2A protocol
- **WASM/Extism production-ready**: Cross-language plugin architecture mature; Extism Go SDK rewritten with Wazero (2026)
- **Observability mature**: OpenTelemetry 0.22+ stable; opentelemetry-jaeger exporter deprecated in favor of OTLP (all backends)
- **Testing accelerated**: Vitest 3-5x faster than Jest; Playwright dominates E2E (overtook Cypress); cucumber + playwright standard for BDD

### Action Items (This Quarter)

1. Migrate Go services (clipproxyapi-plusplus, KodeVibe-Go) from logrus → slog
2. Adopt Sea-ORM 2.0 for new async Rust DB services
3. Replace figment stub with actual figment 0.10+ adoption in phenotype-config
4. Evaluate casbin-rs 2.20+ for phenotype-policy-engine v2
5. Standardize on OpenTelemetry OTLP (deprecate direct Jaeger exporter)

---

## Rust Crate Ecosystem — 2026 Latest Versions

### Web Frameworks & Async Runtime

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `axum` | 0.8.x (0.9 pending) | 17K | **ADOPT** | Stable, tokio-native, modular tower middleware, excellent error handling | 0 (optimal) |
| `tokio` | 1.x (stable 1.40+) | 27K | **ADOPT** - in use | Latest with improved cancellation and task abort patterns | 0 |
| `tower` | 0.4+ | 3K | **ADOPT** - in use | Middleware abstraction, composable layers, standardized trait | 0 |
| `hyper` | 1.x | 14K | **ADOPT** - foundation | HTTP/1.1 and HTTP/2, async, used by axum | 0 |
| `h2` | 0.4+ | 2K | **ADOPT** | HTTP/2 protocol implementation (tokio-native) | 0 |

**Impacted Repos:** phenotype-infrakit services (agent-api-plusplus, cliproxyapi-plusplus)
**Assessment:** Stack optimal; no changes needed. Already using all latest versions.

---

### Configuration Management

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `figment` | 0.10.8+ | 300 | **ADOPT** | Hierarchical config from multiple sources (TOML/YAML/JSON/ENV), merge strategies | 200-300 |
| `config-rs` | 0.14+ | 500 | **EVALUATE** | Lightweight alternative; similar feature set | - |
| `envy` | 0.4+ | 200 | **MONITOR** | ENV-based config only (narrower scope) | - |

**Impacted Repos:** phenotype-infrakit (phenotype-policy-engine, phenotype-config)
**Action:** Replace ad-hoc ENV/JSON parsing with figment; implement in phenotype-config wrapper for cross-project use

**Estimated Savings:** Replace 200-300 LOC of ENV parsing and merge logic across phenotype-config, AgilePlus settings, agent configuration

---

### Authorization & Policy Engines

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `casbin-rs` | 2.20.0+ | 2K | **WRAP** | Cross-language RBAC/ABAC (supported in Go, Java, Python, Node.js, Rust); production-grade | 400-600 |
| `oso` | 0.27+ | 1.5K | **EVALUATE** | Batteries-included authorization framework; Zanzibar-inspired | - |
| `cedar-policy-core` | 3.1+ | 800 | **EVALUATE** | AWS open-sourced; policy-as-code (CEDARL language) | - |
| `spicedb-client` | 0.1+ | 100 | **EVALUATE** | SpiceDB (Zanzibar paper implementation); fine-grained permissions | - |

**Impacted Repos:** phenotype-policy-engine (extend v2 with persistent policy store)
**Action:** Evaluate casbin-rs for cross-language policy engine; create phenotype-casbin-wrapper

**Estimated Savings:** Consolidate phenotype-policy-engine + role definitions across AgilePlus, agent RBAC → unified casbin store

---

### Error Handling & Diagnostics

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `miette` | 0.7.0+ | 2K | **ADOPT** | Fancy diagnostic reporting (enable fancy feature only in bin crates) | 100-200 |
| `anyhow` | 1.0+ | 5K | **ADOPT** - in use | General-purpose error type with context chaining; application default | 0 |
| `thiserror` | 2.0+ (from 1.0) | 3K | **ADOPT** | Custom error types with From/Display derives; library default | 0 |
| `snafu` | 0.8+ | 500 | **EVALUATE** | Context-focused error handling; alternative to miette | - |
| `eyre` | 0.6+ | 2K | **EVALUATE** | Hook-based error context; good for debugging workflows | - |

**Impacted Repos:** All Rust crates (especially phenotype-error, agent-api-plusplus)
**Action:** Ensure phenotype-error uses thiserror 2.x + miette 0.7 (fancy disabled in libs, enabled in bins)

**Estimated Savings:** 100-200 LOC of custom error formatting; unified diagnostics across all services

---

### ORM & Database Access

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `sea-orm` | 2.0+ (Jan 2026) | 8K | **ADOPT** | Async-first ActiveRecord ORM, entity generation, migration CLI, production-ready | 400-600 |
| `sqlx` | 0.8.6+ | 12K | **ADOPT** | Compile-time SQL verification, built-in connection pooling, migrations included | 200-300 |
| `diesel` | 2.3.6+ (Jan 2026) | 12K | **MONITOR** | Sync-first (async via diesel-async 0.5+), strongest compile-time guarantees | - |
| `tokio-postgres` | 0.7+ | 1.5K | **EVALUATE** | Low-level control, query pipelining (sqlx/sea-orm don't support) | - |
| `rusqlite` | 0.32+ | 1.5K | **MONITOR** - in use | SQLite only, good for local caches | - |

**Impacted Repos:** phenotype-infrakit, AgilePlus (potential FastAPI/SQLx migration path)
**Action:** For new async Rust services with complex ORM queries, prefer sea-orm 2.0 over sqlx; for simple queries, sqlx sufficient

**Estimated Savings:** 400-600 LOC if migrating from Django ORM (AgilePlus) to async SQLx/sea-orm

---

### Logging, Tracing & Observability

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `tracing` | 0.27+ | 2K | **ADOPT** - in use | Spans, structured fields, async-aware, 0-cost when disabled | 0 |
| `tracing-subscriber` | 0.3.18+ | 1.5K | **ADOPT** - in use | Composable layer system (JSON, fmt, OpenTelemetry), environment filtering | 0 |
| `opentelemetry` | 0.22+ | 1K | **ADOPT** | Metrics (stable), Logs (stable), Traces (beta); OTLP protocol | 100 |
| `opentelemetry-otlp` | 0.15+ | - | **ADOPT** | Unified OTLP exporter (Jaeger, Prometheus, vendors, Grafana Tempo) | 0 |
| `opentelemetry-jaeger` | 0.21 | - | **DEPRECATED** | Exporter deprecated; migrate to opentelemetry-otlp | - |
| `tokio-console` | 0.2+ | 500 | **ADOPT** | Runtime task debugging (TUI), task wakeup inspection, async debugging | 0 |
| `tracing-flame` | 0.2+ | 300 | **EVALUATE** | Flame graphs from tracing spans (complement to cargo-flamegraph) | - |

**Impacted Repos:** phenotype-observability, all services
**Action:** Update phenotype-observability to use opentelemetry-otlp (remove direct jaeger exporter); migrate tracing-subscriber config

**Estimated Savings:** 100 LOC; unified backend flexibility (can switch Jaeger → Grafana Tempo without code changes)

---

### Container Management & Orchestration

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `bollard` | 0.18+ | 700 | **WRAP** | Docker daemon API, async/await, Windows Named Pipes, connection pooling | 500+ |
| `bollard-containerd` | 0.1+ | 100 | **EVALUATE** | containerd alternative (less mature) | - |
| `docker-api` | 0.15+ | 200 | **EVALUATE** | Alternative Docker client library | - |

**Impacted Repos:** phenotype-vessel (currently hand-rolled container management ~500 LOC)
**Action:** Replace phenotype-vessel with bollard wrapper in phenotype-shared/crates/phenotype-vessel

**Estimated Savings:** 500+ LOC of custom Docker API wrapper; reuse across heliosCLI, AgilePlus container features

---

### Concurrency, Synchronization & Data Structures

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `parking_lot` | 0.12+ | 2K | **ADOPT** | Faster Mutex/RwLock (1.5-3x vs std), fair locks, no panics | 0 |
| `dashmap` | 5.5+ | 1.5K | **EVALUATE** | Concurrent HashMap (lock-free reads), good for cache workloads | 100 |
| `flume` | 0.11+ | 700 | **EVALUATE** | High-throughput multi-producer channels (outperforms crossbeam for some patterns) | 50 |
| `crossbeam-channel` | 0.5+ | 1K | **MONITOR** | Standard async channel library | - |
| `arc-swap` | 1.6+ | 500 | **MONITOR** | Atomic swap for RwLock<Arc<T>> patterns | - |

**Impacted Repos:** phenotype-cache-adapter, event-sourcing crates (read-heavy workloads)
**Action:** Benchmark dashmap for phenotype-cache-adapter; consider for skill registry (read-heavy)

---

### Testing & Benchmarking

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `cargo-nextest` | 0.9+ | 2K | **ADOPT** | Parallel test runner (3-5x faster), partitioned execution | 0 |
| `criterion` | 0.5+ | 2K | **ADOPT** | Rigorous microbenchmarking with statistical analysis | 0 |
| `proptest` | 1.4+ | 1.5K | **ADOPT** | Property-based testing (complement to unit tests) | 0 |
| `quickcheck` | 1.0+ | 1K | **MONITOR** | Alternative to proptest (smaller ecosystem) | - |
| `rstest` | 0.18+ | 800 | **ADOPT** | Parameterized testing (pytest-like fixtures for Rust) | 0 |

**Impacted Repos:** All test suites
**Action:** Migrate CI to cargo-nextest; add proptest for state machine (phenotype-state-machine) and policy engine (phenotype-policy-engine) property tests

---

## Go Ecosystem — 2026 Latest Versions

### Structured Logging

| Package | Version | Recommendation | Rationale | Impacted Repos |
|---------|---------|----------------|-----------|----------------|
| `slog` (stdlib) | 1.21+ | **ADOPT** | Native structured logging (Go 1.21+), JSON output, handlers | clipproxyapi-plusplus, KodeVibe-Go |
| `logrus` | deprecated | **REMOVE** | Replaced by slog; security issues, unmaintained | clipproxyapi-plusplus (2 uses), KodeVibe-Go |
| `zap` | 1.26+ | **MONITOR** | High-performance alternative; heavier than slog | - |

**Action:** Migrate clipproxyapi-plusplus + KodeVibe-Go from logrus → slog (P4 task, high priority)

**Estimated Savings:** 100-150 LOC of custom logging setup; use slog.JSONHandler for structured output

---

### Dependency Injection

| Package | Version | Recommendation | Rationale |
|---------|---------|----------------|-----------|
| `uber/fx` | 1.20+ | **ADOPT** | DI + application lifecycle management, cleaner than manual wiring, structured |
| `wire` | 0.12.1 or 0.3.0 | **MONITOR** | Compile-time DI (conflicting version data in 2026 sources) |
| Interface-based | stdlib | **ADOPT** | Go 1.21+ struct composition + interface contracts |

**Impacted Repos:** Future Go services in phenotype ecosystem
**Action:** Use Uber Fx for new Go services requiring DI; wire for code-generation DI if needed

---

### ORM & Database

| Package | Version | Recommendation | Rationale |
|---------|---------|----------------|-----------|
| `ent` | 0.14+ | **ADOPT** | Entity-driven ORM, schema-first, code generation, graph queries |
| `sqlc` | 1.30+ | **ADOPT** | Type-safe SQL from raw queries, no runtime reflection |
| `gorm` | 1.25+ | **MONITOR** | Most popular Go ORM; older patterns, magic methods |
| `migrate` | 4.17+ | **ADOPT** | Database migration tool (Flyway equivalent) |

**Action:** Prefer ent for entity-rich services; sqlc for query-heavy services

---

### gRPC & RPC

| Package | Version | Recommendation | Rationale |
|---------|---------|----------------|-----------|
| `connectrpc/connect-go` | 1.16+ | **ADOPT** | gRPC-compatible, HTTP/1.1 support, curl-able endpoints, browser support |
| `grpc/grpc-go` | 1.65+ | **ADOPT** | Standard gRPC (HTTP/2 only), mature ecosystem |
| `twirp` | 8.1+ | **EVALUATE** | Alternative RPC framework (Twitch-made, simpler than gRPC) |

**Impacted Repos:** Agent communication (heliosCLI ↔ AgilePlus, agent-api-plusplus ↔ cliproxyapi-plusplus)
**Action:** Evaluate ConnectRPC for cross-language agent communication (curl-able gRPC)

---

### Observability

| Package | Version | Recommendation | Rationale |
|---------|---------|----------------|-----------|
| `opentelemetry-go` | 1.26+ | **ADOPT** | Distributed tracing (v1 stable), metrics, logs |
| `go-otelgrpc` | 0.51+ | **ADOPT** | Auto-instrumentation for gRPC services |
| `go-otelhttp` | 0.51+ | **ADOPT** | Auto-instrumentation for HTTP services |

**Action:** Add OpenTelemetry to clipproxyapi-plusplus + KodeVibe-Go; migrate from custom telemetry

---

## TypeScript/Node.js Ecosystem — 2026 Latest Versions

### Web Frameworks & API

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `hono` | 4.x+ | 18K | **ADOPT** | Universal runtime (edge, serverless, Node.js), lightweight, Zod integration | 100-150 |
| `express` | 4.18+ | 65K | **MAINTAIN** | Stable, heavy; use for legacy (AgilePlus dashboard currently) | - |
| `fastify` | 4.26+ | 32K | **EVALUATE** | High-performance, JSON schema validation built-in | - |
| `trpc` | 11.x+ | 18K | **ADOPT** | Type-safe RPC (end-to-end inference), automatic validation | 200-300 |
| `encore` | 1.18+ | 5K | **EVALUATE** | Full-stack TypeScript framework, infra as code | - |

**Impacted Repos:** AgilePlus web dashboard, thegent dashboard backend
**Action:** Consider Hono for new serverless agent APIs; tRPC for type-safe dashboard ↔ backend RPC

**Estimated Savings:** 200-300 LOC with tRPC (no manual API route definitions + validation schemas)

---

### ORMs & Database

| Package | Version | GitHub Stars | Recommendation | Rationale | LOC Savings |
|---------|---------|--------------|----------------|-----------|-------------|
| `drizzle-orm` | 0.30+ | 26K | **ADOPT** | Lightweight SQL-first ORM, Zod schema generation, type-safe queries | 300-400 |
| `prisma` | 5.x+ | 39K | **MONITOR** | Full-featured, heavier (code generation), good for rapid dev | - |
| `typeorm` | 0.3+ | 34K | **MONITOR** | Decorator-based ORM, complex, good for large enterprises | - |
| `mikro-orm` | 6.x+ | 8K | **EVALUATE** | Convention-based, good middle ground | - |

**Impacted Repos:** AgilePlus web backend (currently Sequelize), thegent backend
**Action:** If migrating from Sequelize/TypeORM, prefer Drizzle for lightweight + type safety + Zod integration

**Estimated Savings:** 300-400 LOC (schema definition, query building, validation)

---

### Validation & Schema

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `zod` | 3.23+ | 30K | **ADOPT** | Runtime schema validation + TypeScript inference, best DX |
| `valibot` | 0.30+ | 6K | **EVALUATE** | Lighter alternative to zod (modular, tree-shakeable) |
| `effect-schema` | bundled with effect-ts | 7K | **EVALUATE** | Functional approach, composable validators |
| `ajv` | 8.12+ | 13K | **ADOPT** | JSON Schema validator (production-grade, fast) |

**Impacted Repos:** AgilePlus API validation, agent command specs
**Action:** Use Zod for runtime validation + TypeScript types in API handlers

---

### RPC & API

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `trpc` | 11.x+ | 18K | **ADOPT** | Typesafe RPC (end-to-end + middleware), automatic validation |
| `graphql` | 16.8+ | 20K | **MONITOR** | If complex query patterns needed (dashboard filters, nested queries) |
| `connectrpc/connect-web` | 1.5+ | 3K | **EVALUATE** | ConnectRPC browser client (gRPC-web alternative) |

**Impacted Repos:** AgilePlus dashboard ↔ backend, agent command dispatch
**Action:** Use tRPC for type-safe agent command dispatch (AgilePlus dashboard → backend)

---

### Testing

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `vitest` | 1.2+ | 12K | **ADOPT** | 3-5x faster than Jest, Jest-compatible API, native ESM |
| `playwright` | 1.40+ | 65K | **ADOPT** | E2E testing (multi-browser, debugging tools), overtook Cypress |
| `cucumber` | 9.6+ | 2.5K | **ADOPT** | BDD testing (with vitest + playwright) |
| `@testing-library/react` | 14.0+ | 14K | **ADOPT** | Component testing (user-centric) |

**Impacted Repos:** AgilePlus web, thegent dashboard
**Action:** Migrate from Jest → Vitest for 3-5x speed; add Playwright E2E + Cucumber BDD

---

### Effect-ts (Functional Programming)

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `effect` | 3.x+ | 7K | **EVALUATE** | Functional error handling + DI (advanced); good for type-safe error tracking |
| `effect-schema` | bundled | - | **EVALUATE** | Schema validation with functional composition |
| `effect-platform` | bundled | - | **MONITOR** | Platform abstraction (Node.js, Bun, Deno) |

**Action:** Monitor for adoption; good alternative to anyhow-style error handling for type-safe code

---

## Python Ecosystem — 2026 Latest Versions

### Web Frameworks & API

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `fastapi` | 0.110+ | 74K | **ADOPT** | Modern async Python web framework (Pydantic v2 first-class) |
| `django` | 5.1+ | 76K | **MAINTAIN** | Legacy (AgilePlus); deprecate for new services |
| `starlette` | 0.36+ | 10K | **ADOPT** | ASGI foundation for FastAPI, lightweight |
| `httpx` | 0.26+ | 13K | **ADOPT** | Async HTTP client (replaces requests), type-annotated |

**Impacted Repos:** AgilePlus (FastAPI migration path), heliosApp, heliosCLI
**Action:** Plan FastAPI migration for AgilePlus v2; standardize on httpx + Pydantic v2

---

### Data Validation

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `pydantic` | 2.6+ | 20K | **ADOPT** | Runtime validation + TypeScript-like types; **v1 deprecated** |
| `attrs` | 23.2+ | 5K | **EVALUATE** | Simpler alternative to Pydantic (fewer features) |
| `msgspec` | 0.18+ | 700 | **EVALUATE** | Fast serialization + validation (performance-focused) |

**Impacted Repos:** heliosApp, heliosCLI, AgilePlus settings
**Action:** Migrate any Pydantic v1 code to v2 (breaking changes but required by FastAPI 0.110+)

---

### Workflow Orchestration

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `temporalio` | 1.3+ | 2K | **WRAP** | Temporal Python SDK (durable execution, long-running workflows) |
| `prefect` | 3.0+ | 15K | **EVALUATE** | Flow-based orchestration (newer: Prefect Cloud 3.0 with AI features) |
| `dagster` | 1.6+ | 11K | **EVALUATE** | Asset-driven orchestration (good for data pipelines) |
| `airflow` | 2.8+ | 36K | **MONITOR** | Legacy (complex); maintain existing pipelines only |

**Impacted Repos:** AgilePlus workflow engine (if adding; use Temporal SDK)
**Action:** Consider Temporal SDK for agent workflow orchestration (durable tasks, retries)

---

### Event Sourcing & Messaging

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `eventsourcing` | 5.0+ | 1K | **WRAP** | Python event sourcing library (production-ready) |
| `redis` | 5.0+ | 10K | **ADOPT** - in use | Redis client library |
| `kombu` | 5.3+ | 1.5K | **EVALUATE** | Message broker abstraction (Celery-compatible) |
| `pydantic-eventsourcing` | 0.1+ | 50 | **EVALUATE** | Pydantic + eventsourcing integration |

**Impacted Repos:** heliosApp event handling, AgilePlus event log
**Action:** Consider eventsourcing for AgilePlus event-driven system; ensure Pydantic v2 compatibility

---

### Logging & Observability

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `loguru` | 0.7+ | 14K | **ADOPT** | Simplified structured logging (better UX than stdlib) |
| `structlog` | 24.1+ | 2.5K | **EVALUATE** | Structured logging library (verbose setup) |
| `python-json-logger` | 2.0+ | 800 | **EVALUATE** | JSON logging handler (minimal) |
| `pydantic-logfire` | bundled | - | **EVALUATE** | Logfire integration (Pydantic observability platform) |

**Impacted Repos:** All Python services (heliosApp, heliosCLI, AgilePlus)
**Action:** Standardize on loguru for consistent logging across all Python services

---

## AI/LLM Integration — 2026 Latest Versions

### Anthropic Claude SDK

| Package | Version | Recommendation | Rationale | Status |
|---------|---------|----------------|-----------|--------|
| `anthropic` | 0.25+ | **ADOPT** | Official Claude SDK (Python, async first, structured outputs) | v0.25+ stable |
| `anthropic[bedrock]` | 0.25+ | **ADOPT** | AWS Bedrock integration for enterprise | production-ready |
| `anthropic[vertex]` | 0.25+ | **ADOPT** | Google Vertex AI integration | production-ready |

**Impacted Repos:** heliosCLI, thegent (agent command dispatch), AgilePlus specs
**Action:** Use official Anthropic SDK for all Claude API calls; support streaming, tool use, vision, batches

---

### Claude Agent SDK

| Package | Version | Recommendation | Rationale |
|---------|---------|----------------|-----------|
| `@anthropic-ai/claude-agent-sdk` | 0.2.71 (Node.js) | **ADOPT** | Official agent framework (same architecture as Claude Code) |
| `claude-agent-sdk-python` | 0.1.48 (Python) | **ADOPT** | Python version (active development, v0.2 incoming) |

**Impacted Repos:** thegent (replace custom agent loop with SDK), heliosCLI agent dispatch
**Action:** Evaluate Claude Agent SDK for thegent agent orchestration; use tool registry, hooks, MCP support directly

---

### Agent Orchestration Frameworks

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `langgraph` | 1.0.10+ (stable Oct 2025) | 6K | **WRAP** | State graph orchestration for agents (mature, v1 production) |
| `crewai` | 0.47+ | 21K | **WRAP** | Multi-agent frameworks with A2A protocol support, MCP ready |
| `autogen` | 0.3.x (stable) | 32K | **MONITOR** | AutoGen v0.4 graph-based; Microsoft maintenance mode |
| `openagents` | 1.0+ | 800 | **EVALUATE** | Open-source agent framework (emerging) |

**Impacted Repos:** thegent, heliosCLI (agent orchestration)
**Action:** LangGraph v1.0+ is stable; CrewAI good for multi-agent with A2A; both better than custom loops

---

### Tokenization & Cost Tracking

| Package | Version | GitHub Stars | Recommendation | Rationale |
|---------|---------|--------------|----------------|-----------|
| `tiktoken` | 0.5+ | 2K | **ADOPT** | OpenAI BPE tokenizer (works with Claude via adapters) |
| `tokenizers` | 0.20+ | 9K | **EVALUATE** | HuggingFace tokenizers (more comprehensive, slower) |

**Impacted Repos:** heliosCLI (agent budget tracking), AgilePlus token accounting
**Action:** Use tiktoken for token counting in agent budgets; track cost per turn

---

## WASM & Plugin Architecture — 2026 Latest Versions

### Extism Framework

| Package | Version | Recommendation | Rationale | Status |
|---------|---------|----------------|-----------|--------|
| `extism` (Rust) | 1.x+ | **ADOPT** | WASM plugin framework (mature, production, Apache 2.0) | stable |
| `extism` (Go) | 1.x+ | **ADOPT** | Go SDK with Wazero runtime (rewritten 2026 for performance) | stable |
| `extism` (Node.js) | 1.x+ | **ADOPT** | JavaScript WASM runtime | stable |
| `extism` (Python) | 1.x+ | **ADOPT** | Python WASM support | stable |

**Impacted Repos:** phenotype-vessel (if building extensible plugins), harbor-skills fork
**Action:** Consider Extism for skill plugins; cross-language plugin system for agents

---

### WebAssembly Tools

| Package | Version | Recommendation | Rationale |
|---------|---------|----------------|-----------|
| `wasm-pack` | 1.3+ | **ADOPT** | Rust → WASM compiler (npm integration) |
| `wasm-opt` | 120+ | **ADOPT** | WASM optimization (binaryen) |
| `wasmer` | 4.2+ | **EVALUATE** | Alternative WASM runtime (vs Wasmtime) |
| `wasmtime` | 16+ | **ADOPT** | Bytecode Alliance WASM runtime (standard) |

**Action:** If building WASM plugins, use wasm-pack + wasmtime

---

## Observability & Monitoring — 2026 Latest Versions

### OpenTelemetry (All Languages)

| Component | Version | Recommendation | Rationale |
|-----------|---------|----------------|-----------|
| `opentelemetry-rust` | 0.22+ | **ADOPT** | Traces (beta), Logs (stable), Metrics (stable) |
| `opentelemetry-go` | 1.26+ | **ADOPT** | Go SDK (v1 stable) |
| `opentelemetry-js` | 0.52+ | **ADOPT** | JavaScript SDK |
| `opentelemetry-python` | 1.26+ | **ADOPT** | Python SDK |
| `opentelemetry-otlp` | 0.15+ (all langs) | **ADOPT** | OTLP exporter (unified protocol for all backends) |
| `opentelemetry-jaeger` | 0.21 | **DEPRECATED** | Use opentelemetry-otlp instead |

**Impacted Repos:** phenotype-observability, all services
**Action:** Migrate from jaeger exporter → OTLP exporter; allows backend flexibility (Jaeger, Grafana Tempo, Datadog, etc.)

**Estimated Savings:** 100 LOC; centralized configuration for telemetry backend

---

### Distributed Tracing Backend

| Tool | Recommendation | Rationale |
|------|----------------|-----------|
| `jaeger` | **EVALUATE** | Still supported via OTLP; can migrate backend without code change |
| `grafana-tempo` | **EVALUATE** | Alternative trace backend (simpler setup, lower cost) |
| `signoz` | **EVALUATE** | All-in-one observability (OpenTelemetry native) |

---

## Performance & Build Tools — 2026 Latest Versions

### Rust Build & Test Acceleration

| Tool | Version | Recommendation | Rationale |
|------|---------|----------------|-----------|
| `cargo-nextest` | 0.9+ | **ADOPT** | Parallel test runner (3-5x faster) |
| `cargo-hack` | 0.5+ | **EVALUATE** | Feature flag testing in CI |
| `sccache` | 0.8+ | **EVALUATE** | Shared cache for CI builds |
| `mold` | 1.0+ | **EVALUATE** | Fast linker (Linux; faster than lld) |

**Action:** Integrate cargo-nextest into CI; test sccache for GitHub Actions cost reduction

---

### Profiling & Debugging

| Tool | Version | Recommendation | Rationale |
|------|---------|----------------|-----------|
| `cargo-flamegraph` | 0.6+ | **ADOPT** - in use | Flame graph generation |
| `tokio-console` | 0.2+ | **ADOPT** | Async task inspection (TUI), task wakeup patterns |
| `tracing-flame` | 0.2+ | **EVALUATE** | Flame graphs from tracing spans |

---

## Summary: 2026 Recommended Tech Stack

### Rust Backend Services

**Core Stack:**
- Web: axum 0.8 + tokio 1.x + tower 0.4
- Database: sea-orm 2.0 (complex queries) or sqlx 0.8.6 (simple queries)
- Configuration: figment 0.10+ (multi-source)
- Authorization: casbin-rs 2.20+ (policy engine)
- Logging: tracing 0.27 + tracing-subscriber (JSON) + miette 0.7
- Observability: opentelemetry 0.22 + opentelemetry-otlp
- Container: bollard 0.18 (Docker API)
- Testing: cargo-nextest 0.9 + proptest 1.4 + criterion 0.5
- Concurrency: parking_lot 0.12, dashmap 5.5 (cache workloads)

**Estimated LOC Savings (vs hand-rolled):** 2,500-3,500 LOC across ecosystem

---

### Go Services & CLI

**Core Stack:**
- Logging: slog (stdlib) — remove logrus
- DI: Uber Fx 1.20+
- Database: ent 0.14+ (ORM) or sqlc 1.30+ (query-focused)
- Migrations: migrate 4.17+
- RPC: connectrpc 1.16+ (HTTP/1.1 + gRPC-compatible)
- Observability: opentelemetry-go 1.26 + OTLP exporter

**Action:** Immediate: migrate clipproxyapi-plusplus + KodeVibe-Go from logrus → slog

---

### TypeScript Web & API

**Core Stack:**
- Web: hono 4.x (edge-first) or express 4.18 (legacy)
- API: tRPC 11.x (type-safe RPC) + zod 3.23 (validation)
- Database: drizzle-orm 0.30+ (SQL-first, type-safe)
- Testing: vitest 1.2 (unit) + playwright 1.40 (E2E) + cucumber 9.6 (BDD)
- Build: Vite 5.x, Turbo 2.x

**Estimated LOC Savings:** 300-500 LOC (Drizzle schema + query generation)

---

### Python FastAPI & CLI

**Core Stack:**
- Web: FastAPI 0.110+ (async Python, Pydantic v2 first-class)
- Validation: Pydantic v2 (required)
- HTTP Client: httpx 0.26+ (async, type-annotated)
- Logging: loguru 0.7+ (structured logging)
- Workflows: temporalio 1.3+ (durable execution)
- Events: eventsourcing 5.0+ (event store)

**Action:** Migrate Pydantic v1 → v2; move from requests → httpx

---

### AI/LLM Integration (All Languages)

**Core Stack:**
- Claude SDK: anthropic 0.25+ (official)
- Agent SDK: claude-agent-sdk 0.2+ (Node.js) or 0.1.48 (Python)
- Orchestration: langgraph 1.0+ (state management) or CrewAI 0.47+ (multi-agent)
- Tokenization: tiktoken 0.5+ (token counting)
- WASM Plugins: Extism 1.x+ (cross-language)

---

### Observability (All Languages)

**Core Stack:**
- Tracing: OpenTelemetry 0.22+ (all languages) + OTLP exporter
- Tracing Backend: Jaeger (via OTLP) or Grafana Tempo
- Logging: tracing (Rust), slog (Go), loguru (Python), pino (Node.js)
- Metrics: OpenTelemetry metrics API (all languages)

**Action:** Centralize on OTLP; allows backend flexibility without code changes

---

## 2026 Technology Radar Chart

```
ADOPT (High confidence, production-ready):
✅ sea-orm 2.0 (Rust ORM, ActiveRecord)
✅ axum 0.8 (Rust web framework)
✅ drizzle-orm 0.30 (TypeScript SQL-first ORM)
✅ FastAPI 0.110 (Python async web)
✅ pydantic v2 (Python validation, required)
✅ slog (Go stdlib logging)
✅ opentelemetry-otlp (unified tracing backend)
✅ Extism 1.x (WASM plugins, cross-language)
✅ langgraph 1.0 (agent state graphs, v1 stable)
✅ anthropic 0.25+ (Claude SDK)
✅ cargo-nextest 0.9 (test acceleration, 3-5x faster)
✅ vitest 1.2 (Jest replacement, 3-5x faster)
✅ playwright 1.40 (E2E testing, overtook Cypress)

TRIAL (Moderate confidence, good for specific use cases):
🟡 casbin-rs 2.20 (cross-language policy engine)
🟡 crewai 0.47 (multi-agent, A2A protocol)
🟡 hono 4.x (edge deployment, universal runtime)
🟡 tRPC 11.x (type-safe RPC, excellent DX)
🟡 figment 0.10 (hierarchical config, multi-source)
🟡 temporalio 1.3 (durable execution, long-running workflows)

ASSESS (Emerging, niche use cases):
🔵 effect-ts 3.x (functional programming, error handling)
🔵 cedar-policy 3.1 (AWS policy-as-code)
🔵 tokenizers 0.20 (HuggingFace, more comprehensive)
🔵 connectrpc 1.16 (gRPC + HTTP/1.1 + browser)

HOLD (Wait or maintain only):
🔴 diesel (sync-focused; prefer sea-orm for async)
🔴 logrus (deprecated; migrate to slog)
🔴 typeorm (complex; prefer drizzle)
🔴 pydantic v1 (deprecated; upgrade to v2)
🔴 jest (slow; switch to vitest)
```

---

**Last Updated:** 2026-03-29
**Research Scope:** Rust, Go, TypeScript, Python, AI/LLM, WASM, Observability, Testing
**Total New Entries:** 25+ packages + frameworks
**Total LOC Savings Identified:** ~3,500-4,500 LOC across ecosystem
**Next Review:** 2026-06-29 (quarterly tech radar update)
