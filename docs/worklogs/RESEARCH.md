# Research Worklogs

**Category:** RESEARCH | **Updated:** 2026-03-29

---

## 2026-03-29 - Cross-Repo GitHub Duplication Analysis

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0

### Summary

Full GitHub org scan identifying duplication clusters, agent-generated stubs, and consolidation targets.

### Cluster 1: `*kit` Stubs (15 repos — P0 Archive)

All 15 `*kit` repos (`logkit`, `tracingkit`, `metrickit`, `cachekit`, `configkit`, `authkit`, `evalkit`, `taskkit`, `eventkit`, `apikit`, `clikit`, `dbkit`, `httpkit`, `cryptokit`, `agentkit`) were created **2026-03-25** in a single agent session. Sizes: 5–58 kB. No real implementations. Each duplicates purpose with a more mature counterpart:

| Kit Stub | Mature Counterpart(s) |
|---|---|
| `logkit` | `helix-logging`, `phenotype-rust-logging` |
| `tracingkit` | `helix-tracing` |
| `metrickit` | `thegent-metrics` |
| `cachekit` | `thegent-cache`, `phenotype-cache-adapter` (×2) |
| `configkit` | `phenotype-config-ts`, `phenotype-rust-config` |
| `eventkit` | `phenotype-event-sourcing` (in infrakit + shared) |
| `agentkit` | `thegent-*` family |
| `authkit` | `phenotype-auth-ts` |

**Action:** Archive all 15. They are technical debt, not features.

### Cluster 2: `hexagon-*` Template Proliferation (11 repos — P2)

11 repos share identical descriptions, only language varies. `hexagon-rust` (9 kB) and `hexagon-rs` (39 kB) are direct duplicates. Most are empty stubs (0–1 kB).

**Action:** Consolidate into single `hexagon-templates` monorepo with per-language subdirectories. Delete `hexagon-rust` (9 kB) in favor of `hexagon-rs` (39 kB).

### Cluster 3: `phenotype-infrakit` vs `phenotype-shared` (4 duplicate crates — P1)

Both repos contain: `phenotype-cache-adapter`, `phenotype-event-sourcing`, `phenotype-policy-engine`, `phenotype-state-machine`. `phenotype-shared` is the superset (11 crates vs 5). `infrakit` was absorbed but not cleaned up.

**Action:** `phenotype-infrakit` crates → merge into `phenotype-shared`, archive `infrakit`.

### Cluster 4: Observability 3-4 Way Duplication (P1)

| Domain | Repos |
|---|---|
| Logging | `helix-logging`, `logkit`, `phenotype-rust-logging` |
| Tracing | `helix-tracing`, `tracingkit` |
| Metrics | `thegent-metrics`, `metrickit`, `phenotype-rust-metrics` |
| Caching | `thegent-cache`, `cachekit`, `phenotype-cache-adapter` (×2) |

**Action:** Consolidate all into `phenotype-shared/crates/phenotype-observability`.

### Summary Count

- **15** agent-stub repos to archive (`*kit` family)
- **4** duplicate crates between `infrakit` and `phenotype-shared`
- **11** template repos to consolidate into 1 monorepo
- **4** domains (logging, tracing, metrics, caching) each spread across 3-4 repos

---

## 2026-03-29 - 2026 Package Research: Python / TypeScript / Go / Zig / Mojo

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Python LLM Routing

| Package | Action | Notes |
|---|---|---|
| **LiteLLM v1.82.6** | WRAP (pinned) | 100+ provider unified API. WARNING: v1.82.7-v1.82.8 compromised in supply-chain attack (2026-03-25) — pin to v1.82.6 with hash verification until v1.82.9+ ships with provenance attestation |
| Portkey | BLACKBOX | Managed gateway; escape hatch for zero-ops teams |
| Bifrost (Maxim AI) | EVALUATE | Go-native, 54x p99 latency improvement at 5k RPS |

### Python Resilience

| Package | Action | Notes |
|---|---|---|
| **stamina 25.2.0** | ADOPT | hynek's opinionated retry wrapper over Tenacity; exponential backoff + jitter defaults, Prometheus + structlog built-in, async/trio, Python 3.10-3.14. Only retry primitive needed for phenoSDK. |
| Tenacity | WRAP via stamina | Use directly only for edge cases not covered by stamina |

### Python Vector DB

| Package | Action | Notes |
|---|---|---|
| **Qdrant client v1.15** | ADOPT (direct, behind port) | Define `VectorStorePort`; implement Qdrant + Weaviate adapters |
| **Vextra** | WATCH | Academic Jan 2026, Pinecone/Weaviate/Qdrant adapters; architecture mirrors Phenotype hexagonal model exactly — adopt when PyPI package ships |

### Python MCP Framework

| Package | Action | Notes |
|---|---|---|
| **FastMCP v3.0 GA** (PrefectHQ) | ADOPT | 70% of all MCP servers use FastMCP. v3.0 adds component versioning, granular authorization, OpenTelemetry, OpenAPI providers. phenoSDK MCP layer should be built on this directly. |
| FastAPI-MCP | WRAP | Auto-exposes FastAPI endpoints as MCP tools; use as bridge adapter |

### Python DI / Hexagonal

| Package | Action | Notes |
|---|---|---|
| **lagom** | ADOPT | Type-safe DI container, auto-wiring, async, context managers. Wire port-to-adapter bindings. |
| Python `Protocol` (stdlib) | USE | Structural subtyping for port definitions — no ABC inheritance required |

### TypeScript Agents

| Package | Action | Notes |
|---|---|---|
| **Mastra v1.0** (YC W25, $13M) | ADOPT | TS-native agent framework built on Vercel AI SDK; built-in RAG, observability, memory, workflows. The correct bleeding-edge choice for Phenotype TS. |
| **Vercel AI SDK** | ADOPT (via Mastra) | Streaming-first, React Server Components, edge runtime; 2.8M weekly downloads |

### Go Hexagonal

| Package | Action | Notes |
|---|---|---|
| **google/wire** | ADOPT | Compile-time DI for Go; wire port-to-adapter at compile time |
| `go-hexagonal` (RanchoCooper) | SCAFFOLD REF | Use as layout reference, not runtime dep |
| ThreeDotsLabs clean-arch patterns | ADOPT patterns | Watermill + clean-arch is the reference impl for Phenotype Go services |

### Zig Observability

| Package | Action | Notes |
|---|---|---|
| **zlog** (hendriknielaender) | ADOPT | Zero-alloc structured logging + full OTel support for Zig 0.14 |
| logly.zig | FUTURE (Zig 0.15+) | 36M ops/sec, async I/O, JSON, distributed tracing; pin as upgrade target |

### Mojo

**Do not adopt for production in 2026.** Modular Platform 26.2 (Mar 2026) focuses on GPU kernel authoring and progressive Python interop. General application code stdlib is not stable. Revisit late 2026.

---

## 2026-03-29 - 2026 Rust Package Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Key Decisions

| Package | Action | Notes |
|---|---|---|
| **figment 0.10.19** | ADOPT (replace config-rs) | Superior error provenance, hierarchical overrides, array env var parsing; config-rs community recommends migration |
| **miette 7.6.0** | ADOPT | Fancy diagnostics; pairs with thiserror; requires rustc >= 1.82 |
| **pyo3 0.23.x** | ADOPT | Free-threaded Python 3.14 support; use maturin as build tool |
| **casbin-rs 2.8.0** | ADOPT (or Cerbos) | Now Apache-incubated; ACL/RBAC/ABAC via PERM model; Cerbos as policy-as-code alternative |
| **cqrs-es** | ADOPT (replace eventually) | eventually-rs 0.5.x is prerelease-quality, slow maintenance; cqrs-es is more production-ready for serverless Rust |
| **eventsourced** | EVALUATE | Akka Persistence-inspired, NATS+Postgres adapters |
| **eventastic** | EVALUATE | Fork of eventually-rs, enforces transactions + idempotency |
| **codex-rs (openai/codex)** | FORK CANDIDATE | v0.116.0 (Mar 19 2026), 67K stars, Apache 2.0, ~96% Rust, `app-server` + `core` crate architecture |
| **statig** | ADOPT (state machines) | Hierarchical state machines, tree-based, embedded + complex state hierarchies |
| **smlang** | EVALUATE | Procedural macro DSL state machines, `no_std`, async, generates Mermaid |

### Hexagonal Architecture

No dominant "hexagonal framework" crate in Rust. Pattern = multi-crate workspace (domain crate with port traits, adapters crate, entry-point crate). `hexser` (GitHub) worth watching for architectural validation tooling.

### Event Sourcing Replace Matrix

| From | To | Why |
|---|---|---|
| `eventually` 0.5.x | `cqrs-es` | Prerelease quality, slow maintenance |
| `eventually` | `eventsourced` | NATS+Postgres adapters, Akka Persistence-inspired |

---

## 2026-03-29 - Starred Repos Deep Analysis

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Deep research into 30 starred GitHub repositories. Identified patterns, gaps, and opportunities for the Phenotype ecosystem.

### High-Value Repos (Recommended)

| Repo | Value | Opportunity |
|------|-------|-------------|
| `harbor-framework/skills` | Agent skills framework | Create `harbor-skills` fork |
| `pathwaycom/pathway` | Real-time ML processing | Integrate with agileplus-events |
| `khoj-ai/khoj` | Local knowledge base | Create semantic search layer |
| `great-expectations/great_expectations` | Data validation | Create agent eval framework |
| `nitrojs/nitro` | Edge/serverless | Deploy MCP as serverless |
| `codecrafters-io/build-your-own-x` | Educational | Add to heliosCLI |

### Repo Analysis Summary

#### 1. harbor-framework/skills ⭐ (Agent Skills Framework)

**What:** Standardized skill definitions for AI agents with 40+ pre-built skills.

**Key Features:**
- Skill composition and chaining
- Integration with Claude Code, Copilot
- Development, testing, deployment skills
- Tool definitions and prompts

**Opportunity:** Create `platforms/harbor-skills` fork for AgilePlus domain:
- Custom skills: `specify`, `implement`, `validate`, `review`, `ship`
- Skill registry for agent dispatch
- Integration with existing CLI commands

**Overlap:** `agileplus-agent-dispatch`, `platforms/thegent/src/research_engine/`

---

#### 2. pathwaycom/pathway ⭐ (Real-Time ML)

**What:** Real-time data processing with LLM integration, 30+ connectors.

**Key Features:**
- Real-time stream processing
- MCP server capability
- RAG pipeline support
- Connectors: Kafka, PostgreSQL, S3, NATS

**Opportunity:** Create `platforms/pathway-xpack`:
- Real-time event processing for AgilePlus
- Semantic search for specs/plans (RAG)
- MCP server wrapper

**Overlap:** `agileplus-events`, `agileplus-mcp`, `agileplus-graph`

---

#### 3. khoj-ai/khoj ⭐ (Local AI Knowledge Base)

**What:** Local AI knowledge base with embeddings, semantic search, multiple interfaces.

**Key Features:**
- Semantic search over documents, notes, code
- Web, Obsidian, Emacs interfaces
- Agentic capabilities
- Local-first privacy

**Opportunity:** Create `platforms/knowledge-base`:
- Index AgilePlus specs and plans
- RAG for agent context injection
- Natural language queries over project knowledge

**Overlap:** `agileplus-graph`, `agileplus-cli/src/commands/specify.rs`

---

#### 4. antinomyhq/forgecode (Code Generation)

**What:** Code generation tool with agent-driven development patterns.

**Key Features:**
- Project scaffolding
- Template management
- Agent integration
- Context injection

**Opportunity:** Enhance AgilePlus agent dispatch with forgecode patterns.

---

#### 5. great-expectations/great_expectations ⭐ (Data Validation)

**What:** Data quality validation framework with expectation suites.

**Key Features:**
- Expectation suites and checkpoints
- Data profiling
- Pipeline integration
- HTML reports

**Opportunity:** Create `platforms/llm-eval`:
- Validate agent outputs against expectation suites
- Profile agent behavior and code quality
- Checkpoint-based validation

---

#### 6. nitrojs/nitro ⭐ (Edge/Serverless)

**What:** Edge/serverless deployment to 40+ targets with AI/LLM support.

**Key Features:**
- 40+ deployment targets
- Built-in AI/LLM support
- Hybrid rendering
- TypeScript-first

**Opportunity:** Create `platforms/nitro-agent`:
- Deploy MCP server as serverless
- Agent runtime at edge locations
- Hybrid local + cloud architecture

---

#### 7. lightdash/lightdash (BI Tool)

**What:** BI tool with YAML-first approach and dbt integration.

**Key Features:**
- YAML-first configuration
- dbt integration
- Metrics layer
- MCP server support

**Opportunity:** Consider for metrics visualization.

---

#### 8. codecrafters-io/build-your-own-x (Educational)

**What:** Educational platform covering 50+ technologies.

**Key Features:**
- Build your own X tutorials
- Language-agnostic guides
- Progressive complexity
- Community contributions

**Opportunity:** Add educational mode to heliosCLI.

---

### Gap Analysis

| Gap | Solution | Priority |
|-----|----------|----------|
| No standardized skills | harbor-skills fork | P1 |
| No real-time processing | pathway integration | P1 |
| No semantic search | knowledge-base repo | P1 |
| No agent evaluation | llm-eval framework | P2 |
| No serverless support | nitro-agent | P2 |
| No Worktrunk integration | worktrunk-sync | P2 |

### Tasks Completed

- [x] Researched all 30 starred repos
- [x] Documented key features and opportunities
- [x] Identified overlaps with existing work
- [x] Created repo recommendations

### Related

- Plan: `plans/2026-03-29-CROSS_PROJECT_DUPLICATION_PLAN-v1.md`
- Research: `KushDocs/swe-practices-research-broughtToYouByKooshaForResearchDoNotDelete.md`

---

## 2026-03-29 - KushDocs Performance Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P2

### Summary

Analyzed KushDocs performance research document (649 lines). Contains valuable technical research on optimization strategies.

### Key Findings

| Topic | Relevance | Action |
|-------|-----------|--------|
| OrbStack alternatives | Medium | Monitor |
| Zero-copy architectures | High | Consider for agent communication |
| tmpfs/shared memory | Medium | Evaluate for hot paths |
| SGLang vs vLLM | High | Research for inference layer |
| Agentic harnesses | High | Evaluate Tabby, OpenHands |

### Recommendations

1. Evaluate SGLang for LLM inference in agents
2. Consider zero-copy for inter-process communication
3. Research Tabby/OpenHands for code completion

### Related

- Research: `KushDocs/Perf-research-broughtToYouByKooshaForResearchDoNotDelete.md`

---

## 2026-03-29 - KushDocs SWE Practices Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Analyzed KushDocs SWE practices research (680 lines). Contains excellent guidance on software engineering limits and agent-aware development.

### Key Findings

| Topic | Insight | Application |
|-------|---------|-------------|
| Code metrics | LOC, complexity, nesting matter | Add to llm-eval |
| Hexagonal architecture | Pattern already adopted | Good alignment |
| Polyrepo strategies | LoB > DRY for AI | Keep repos separated |
| DORA metrics | Track deployment frequency | Add to telemetry |
| Agent patterns | Special considerations | Document in AGENTS.md |

### Recommendations

1. Add code quality metrics to llm-eval
2. Track DORA metrics in agileplus-telemetry
3. Document agent patterns in AGENTS.md
4. Evaluate LoB > DRY for future decisions

### Related

- Research: `KushDocs/swe-practices-research-broughtToYouByKooshaForResearchDoNotDelete.md`

---

## 2026-03-28 - Technology Radar Update

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P2

### Summary

Quarterly technology radar update based on starred repo analysis.

### Adopt

| Technology | Rationale |
|------------|-----------|
| Pathway | Real-time ML with connectors |
| Nitro | Edge deployment simplicity |
| Harbor-skills | Standardized agent capabilities |

### Trial

| Technology | Rationale |
|------------|-----------|
| Khoj | Local knowledge base |
| Great Expectations | Agent output validation |
| Worktrunk | Linear alternative |

### Assess

| Technology | Rationale |
|------------|-----------|
| Forgecode | Code generation patterns |
| Lightdash | BI with YAML-first |
| Codecrafters | Educational platform |

### Hold

| Technology | Rationale |
|------------|-----------|
| Existing graph DBs | Consider Pathway instead |
| Custom MCP implementations | Use Pathway patterns |

---
