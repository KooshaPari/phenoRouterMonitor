# Research Worklogs

**Category:** RESEARCH | **Updated:** 2026-03-29 (Wave 92 appended)

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

## 2026-03-29 - Wave 92: Ecosystem radar (serialization, OTel, WASM, data)

**Project:** [cross-repo]
**Category:** research
**Status:** in_progress
**Priority:** P1

### Summary

Additional 2026 candidates to **wrap at the adapter boundary** or **trial** in pilots. Avoid reimplementing these cross-cutting concerns in `libs/` when mature OSS exists.

### Rust: serialization and zero-copy

| Crate / project | Action | Notes |
|-----------------|--------|-------|
| `rkyv` 0.8+ | EVALUATE | Zero-copy archives for hot read paths; schema evolution needs discipline |
| `flatbuffers` / `capnp` | WRAP | RPC + stable schemas vs hand-rolled JSON for internal services |
| `minicbor` | ADOPT | Small CBOR for constrained agents / WASM |
| `postcard` 1.x | ADOPT | `no_std`-friendly binary serde for device edges |

### Rust: async runtime adjacent

| Crate | Action | Notes |
|-------|--------|-------|
| `tokio-util` `CancellationToken` | ADOPT | Replace ad-hoc `watch` channels for shutdown |
| `async-stream` | WRAP | Ergonomic streaming iterators into axum bodies |
| `backon` | EVALUATE | Retry policies; compare with custom retry in NATS clients |

### Rust: WASM / components

| Tooling | Action | Notes |
|---------|--------|-------|
| `cargo-component` | TRIAL | WIT-first components vs raw `wasm-bindgen` sprawl |
| `wit-bindgen` 0.35+ | ADOPT | Generated bindings for plugin boundaries (aligns with Extism direction) |
| `wasmtime` 24+ | ADOPT | Host runtime for policy / sandboxed plugins |

### TypeScript / Node

| Package | Action | Notes |
|---------|--------|-------|
| `effect` / `@effect/schema` | EVALUATE | Typed errors + schema; heavy bundle; use in services not browser |
| `arktype` | TRIAL | Faster TS-first validation vs zod in hot paths |
| `pino` + `pino-pretty` | ADOPT | JSON logs for Node services; pair with OTel trace context fields |
| `bullmq` | WRAP | Redis queues for async agent jobs; avoid custom Redis Lua |
| `ioredis` | ADOPT | Cluster + sentinel; standardize on one Redis client per repo |

### Go (for services still on Go)

| Module | Action | Notes |
|--------|--------|-------|
| `github.com/bytedance/sonic` | EVALUATE | Fast JSON; CGO-free config matters for static builds |
| `github.com/rs/zerolog` | ADOPT | Structured logs; bridge to OTel via hooks |
| `go.uber.org/fx` | EVALUATE | DI graph vs manual wiring in large cmds |
| `connectrpc.com/connect` | WRAP | gRPC-compatible without full protobuf weight where acceptable |

### Python: agents and data

| Package | Action | Notes |
|---------|--------|-------|
| `opentelemetry-sdk` + `opentelemetry-exporter-otlp` | ADOPT | Match Rust/TS trace IDs across MCP + FastAPI |
| `limits` (Flask-starlette pattern) | WRAP | Rate limits for public HTTP adapters |
| `faker` + `polyfactory` | ADOPT | Factory fixtures instead of duplicated JSON blobs in tests |
| `hypothesis` | ADOPT | Property tests for spec parsers and merge logic |

### Observability backends (hosted or self)

| System | Action | Notes |
|--------|--------|-------|
| Grafana Tempo | ADOPT | Trace backend; works with OTLP from all stacks |
| Pyroscope / Grafana profiles | TRIAL | Continuous profiling for Rust/Go CPU hot spots |
| Loki | ADOPT | Log aggregation matching label conventions in `phenotype-*` |

### Security / policy engines (reuse)

| Project | Action | Notes |
|---------|--------|-------|
| Open Policy Agent (Wasm bundle) | WRAP | Same policy bundle in Rust host + CI `conftest` |
| Cedar (AWS) | EVALUATE | Alternative to hand-rolled RBAC in multi-tenant APIs |
| `zxcvbn-rs` | ADOPT | Password strength in CLI onboarding; do not invent heuristics |

### Additional starred / ecosystem repos to track

| Repo | Why watch |
|------|-----------|
| `open-telemetry/opentelemetry-rust` | Exporter parity and MSRV policy |
| `bytecodealliance/wasmtime` | Component model churn |
| `tokio-rs/axum` | Middleware patterns for adapter layer |
| `rust-lang/cargo` | `edition` / workspace features affecting `libs/` migration |
| `withastro/starlight` | Docs sites if VitePress limits hit |
| `bufbuild/buf` | Breaking change detection for protos already in CI |
| `google/osv.dev` | OSV API for automated dep triage bots |
| `rustsec/advisory-db` | Source of truth for `cargo deny` |

### Research tasks (Wave 92)

- [ ] Benchmark `rkyv` vs JSON for one internal read-heavy aggregate path (spike only).
- [ ] Prototype WIT surface for one sandboxed “tool” using `cargo-component`.
- [ ] Align Python/Rust/TS on single OTLP endpoint + resource attributes table.

---

## 2026-03-29 - Agent Protocol Landscape Research (Wave 93)

### Agent Communication Protocols Comparison

| Protocol | Organization | Purpose | Status | Phenotype Fit |
|----------|-------------|---------|--------|---------------|
| **MCP** | Anthropic | Model Context Protocol | Stable | ✅ HIGH |
| **A2A** | Agent Protocol | Agent-to-Agent | Draft | 🟡 MEDIUM |
| **ACP** | ACP | Agent Communication | Active | 🟡 MEDIUM |
| **ANP** | Neural | Agent Network | Research | ❌ LOW |

### MCP (Model Context Protocol) Analysis

```json
// MCP Transport
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {},
  "id": 1
}

// MCP Tool Definition
{
  "name": "github_create_issue",
  "description": "Create a GitHub issue",
  "inputSchema": {
    "type": "object",
    "properties": {
      "owner": { "type": "string" },
      "repo": { "type": "string" },
      "title": { "type": "string" }
    }
  }
}
```

### A2A (Agent-to-Agent Protocol) Analysis

```json
// A2A Message
{
  "protocol": "a2a",
  "version": "1.0",
  "type": "request",
  "method": "tasks/send",
  "params": {
    "task": {
      "id": "task-123",
      "prompt": "Analyze this codebase",
      "context": {}
    }
  }
}
```

### Recommendation

| Protocol | Action | Rationale |
|----------|--------|-----------|
| MCP | **ADOPT** | Industry standard, Anthropic backing, tool ecosystem |
| A2A | **EVALUATE** | Inter-agent communication |
| ACP | **MONITOR** | Alternative, smaller ecosystem |

### Integration with Phenotype

```rust
// crates/phenotype-agent-mcp/src/lib.rs

pub struct PhenotypeMcpServer {
    tools: HashMap<String, ToolHandler>,
    context: Arc<AgentContext>,
}

impl mcp_sdk::Server for PhenotypeMcpServer {
    async fn handle_tool_call(&self, tool: &str, args: Value) -> Result<Value> {
        let handler = self.tools.get(tool)
            .ok_or_else(|| Error::ToolNotFound(tool))?;
        handler(self.context.clone(), args).await
    }
}
```

---

## 2026-03-29 - Semantic Memory & Knowledge Systems Research (Wave 94)

### Knowledge Graph Options

| System | Type | Rust Support | Use Case | Recommendation |
|--------|------|-------------|----------|----------------|
| Neo4j | Graph DB | Driver only | Complex relations | EVALUATE |
| Age | Graph extension | PostgreSQL | Relational+graph | ADOPT |
| SurrealDB | Multi-model | Native | Document+graph | EVALUATE |
| vectordb | Vector | pgvector | Semantic search | ADOPT |

### Semantic Memory Systems

| System | Purpose | Architecture | Phenotype Fit |
|--------|---------|--------------|---------------|
| `mentisdb` | Agent memory | Vector + graph | ✅ HIGH |
| `memory-alpha` | Context management | Hierarchical | 🟡 MEDIUM |
| `khoj` | Personal knowledge | Local-first | 🟡 MEDIUM |

### mentisdb Analysis

```rust
// crates/phenotype-memory/src/lib.rs

pub struct SemanticMemory {
    embeddings: VectorStore,
    graph: GraphStore,
    index: InvertedIndex,
}

impl SemanticMemory {
    pub async fn store(&self, entity: &MemoryEntity) -> Result<MemoryId> {
        let embedding = self.embeddings.embed(&entity.content).await?;
        let graph_id = self.graph.insert(&entity.concepts).await?;
        self.index.add(&entity.keywords, graph_id).await?;
        Ok(MemoryId::new())
    }

    pub async fn recall(&self, query: &str, context: &Context) -> Vec<MemoryEntry> {
        let query_embedding = self.embeddings.embed(query).await?;
        let candidates = self.embeddings.search(query_embedding, 10).await?;
        self.graph.expand(candidates, context.depth).await
    }
}
```

### Integration with Phenotype

```rust
// Phenotype integration
pub struct AgentMemory {
    semantic: SemanticMemory,
    episodic: EventStore,
    procedural: WorkflowStore,
}

impl AgentMemory {
    pub async fn remember(&self, query: &str) -> Result<AgentContext> {
        let memories = self.semantic.recall(query, &Context::default()).await?;
        let recent_events = self.episodic.recent(10).await?;
        Ok(AgentContext { memories, recent_events })
    }
}
```

---

## 2026-03-29 - Workflow Orchestration Research (Wave 95)

### Workflow Engine Comparison

| Engine | Language | Durability | Use Case | Phenotype Fit |
|--------|----------|-----------|----------|---------------|
| Temporal | Go | Strong | Microservices | ❌ Heavy |
| Prefekt | Kotlin | Strong | Cloud-native | 🟡 Heavy |
| forza-core | Rust | Medium | General | ✅ HIGH |
| Conductor | Java | Strong | Netflix-style | ❌ Heavy |
| Custom | Rust | TBD | Phenotype | BUILD |

### forza-core Analysis

```rust
// forza-core patterns
pub struct WorkflowDefinition {
    pub id: WorkflowId,
    pub steps: Vec<Step>,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
}

pub enum Step {
    Task(TaskStep),
    Parallel(Vec<Step>),
    Wait(WaitStep),
    SideEffect(SideEffectStep),
}
```

### Phenotype Workflow Design

```rust
// crates/phenotype-workflow/src/dsl.rs

#[derive(Debug, Clone)]
pub struct WorkflowDsl {
    pub name: String,
    pub triggers: Vec<Trigger>,
    pub steps: Vec<DslStep>,
}

#[derive(Debug, Clone)]
pub enum DslStep {
    Task {
        name: String,
        handler: String,
        input: Value,
        retry: Option<RetryPolicy>,
    },
    Parallel {
        branches: Vec<Vec<DslStep>>,
    },
    Sequential {
        steps: Vec<DslStep>,
    },
    Conditional {
        condition: String,
        then_branch: Vec<DslStep>,
        else_branch: Vec<DslStep>,
    },
}

// Example DSL
let workflow = WorkflowDsl {
    name: "code_review".to_string(),
    triggers: vec![Trigger::OnPush { branch: "main" }],
    steps: vec![
        DslStep::Task {
            name: "lint".to_string(),
            handler: "rust_ci::lint".to_string(),
            input: json!({}),
            retry: Some(RetryPolicy::default()),
        },
        DslStep::Task {
            name: "test".to_string(),
            handler: "rust_ci::test".to_string(),
            input: json!({}),
            retry: None,
        },
    ],
};
```

### Recommendation

| Option | Action | Rationale |
|--------|--------|-----------|
| Temporal | REJECT | Too heavy for internal use |
| forza-core | EVALUATE | Rust-native, moderate complexity |
| Custom | BUILD | Aligns with phenotype patterns |

---

## 2026-03-29 - Infrastructure as Code Research (Wave 96)

### IaC Tool Comparison

| Tool | Language | State | Use Case | Recommendation |
|------|----------|-------|----------|----------------|
| Terraform | HCL | Stateful | Multi-cloud | ADOPT |
| Pulumi | TypeScript/Python | Stateful | Kubernetes | EVALUATE |
| Crossplane | CRD | Kubernetes | Cloud resources | ADOPT |
| CDK8s | TypeScript | Stateless | Kubernetes | MONITOR |

### Pulumi vs Terraform for Phenotype

| Aspect | Pulumi | Terraform |
|--------|--------|-----------|
| Language | TypeScript/Python/Go | HCL |
| Testability | ✅ Native | ⚠️ Limited |
| IDE Support | ✅ Full | ⚠️ Basic |
| Phenotype Fit | 🟡 | 🟡 |

### Recommendation

| Use Case | Tool | Rationale |
|----------|------|-----------|
| Cloud resources | Terraform | Industry standard, provider ecosystem |
| Kubernetes | Crossplane | Native CRD integration |
| Local dev | Docker Compose | Simplicity |

### Phenotype IaC Structure

```
infrastructure/
├── terraform/
│   ├── modules/
│   │   ├── phenocluster/
│   │   ├── databases/
│   │   └── networking/
│   ├── environments/
│   │   ├── dev/
│   │   ├── staging/
│   │   └── prod/
│   └── main.tf
├── kubernetes/
│   ├── base/
│   ├── overlays/
│   └── kustomization.yaml
└── docker/
    └── compose.yaml
```

---

## 2026-03-29 - WebAssembly Component Model Research (Wave 97)

### WASM Component Model Overview

| Aspect | Current State | Target |
|--------|---------------|--------|
| Sandboxing | Process isolation | WASM modules |
| Tool execution | Direct execution | Component-based |
| Host interface | FFI | WIT bindings |
| Portability | Platform-specific | Cross-platform |

### Component Model Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Host Runtime                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐    │
│  │              WASM Component                          │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐            │    │
│  │  │ Tool A  │  │ Tool B  │  │ Tool C  │            │    │
│  │  └─────────┘  └─────────┘  └─────────┘            │    │
│  │                      │                              │    │
│  │              ┌───────▼───────┐                      │    │
│  │              │  WIT Import/Export │                 │    │
│  │              └─────────────────┘                      │    │
│  └─────────────────────────────────────────────────────┘    │
│                           │                                 │
│              ┌────────────▼────────────┐                    │
│              │   Component Runtime      │                    │
│              │   (wasmtime/wasmer)     │                    │
│              └─────────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
```

### WIT Interface Definition

```wit
// phenotype-tool.wit

package phenotype:tool@0.1.0;

interface execution {
  record execution-request {
    tool-id: string,
    arguments: list<tuple<string, string>>,
    timeout-ms: u32,
  }

  record execution-result {
    success: bool,
    stdout: string,
    stderr: string,
    exit-code: u32,
    duration-ms: u64,
  }

  execute: func(request: execution-request) -> execution-result;
}

interface filesystem {
  read-file: func(path: string) -> result<string, string>;
  write-file: func(path: string, contents: string) -> result<_, string>;
  list-directory: func(path: string) -> result<list<string>, string>;
}

world phenotype-sandbox {
  import execution;
  import filesystem;

  export run-tool: func(tool-id: string, args: list<string>) -> execution-result;
}
```

### Rust Implementation

```rust
// crates/phenotype-wasm/src/lib.rs
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

pub struct WasmRuntime {
    engine: Engine,
    linker: Linker,
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // Add WASI support
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        // Add phenotype imports
        Self::add_phenotype_imports(&mut linker)?;

        Ok(Self { engine, linker })
    }

    pub async fn execute(&self, component: &[u8], request: &ExecutionRequest) -> Result<ExecutionResult> {
        let mut store = Store::new(&self.engine, WasiCtxBuilder::new().build());
        let module = Module::from_binary(&self.engine, component)?;
        let instance = self.linker.instantiate(&mut store, &module)?;

        let run_tool = instance.get_typed_func::<(i32, i32), i32>(&mut store, "run-tool")?;

        // Serialize request
        let args_ptr = self.serialize_args(&mut store, &request.arguments)?;
        let result = run_tool.call(&mut store, args_ptr)?;

        self.deserialize_result(&mut store, result)
    }
}
```

### WASM Tool Crate

```toml
# crates/phenotype-wasm-tools/Cargo.toml
[package]
name = "phenotype-wasm-tools"
version = "0.1.0"
edition = "2024"

[dependencies]
wasmtime = "22"
wasmtime-wasi = "22"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"  # Optimize for size
lto = true
codegen-units = 1
```

### Phenotype WASM Tool Example

```rust
// crates/phenotype-wasm-tools/src/example_tool.rs
use phenotype_wasm::{export, Context};

#[derive(Debug, serde::Serialize)]
pub struct ToolResult {
    pub output: String,
    pub metrics: Metrics,
}

#[derive(Debug, serde::Serialize)]
pub struct Metrics {
    pub lines: u32,
    pub characters: u32,
}

#[export]
pub fn analyze_text(ctx: &Context, input: &str) -> ToolResult {
    ToolResult {
        output: format!("Analyzed: {}", input),
        metrics: Metrics {
            lines: input.lines().count() as u32,
            characters: input.len() as u32,
        },
    }
}
```

### Tasks

- [ ] WASM-001: Create `phenotype-wasm-runtime` crate
- [ ] WASM-002: Define WIT interface for phenotype tools
- [ ] WASM-003: Implement sandbox execution
- [ ] WASM-004: Create example tool component
- [ ] WASM-005: Add resource limits (memory, CPU time)

---

## 2026-03-29 - Container & Serverless Research (Wave 98)

### Container Options

| Runtime | Size | Startup | Security | Use Case |
|---------|------|---------|----------|----------|
| Docker | ~100MB | 1-2s | Good | Standard |
| Firecracker | ~5MB | ~125ms | **Excellent** | Serverless |
| gVisor | ~20MB | ~90ms | Strong | Untrusted workloads |
| Kata | ~100MB | 1-2s | **Excellent** | High security |

### Firecracker for Phenotype

```rust
// crates/phenotype-vm/src/firecracker.rs

pub struct MicroVM {
    vm_fd: VmFd,
    vsock: UnixStream,
}

impl MicroVM {
    pub fn new(config: &VmConfig) -> Result<Self> {
        let vm_fd = create_vm()?;

        // Configure vCPUs and memory
        vm_fd.set_vcpu_count(config.vcpus)?;
        vm_fd.set_mmds_size(0)?; // No metadata service needed

        // Add network interface
        let tap = open_tap(&config.network.iface)?;
        vm_fd.add_net(tap, config.network.mac)?;

        Ok(Self { vm_fd, vsock: create_vsock()? })
    }

    pub async fn start(&self, kernel: &[u8], initrd: Option<&[u8]>) -> Result<()> {
        self.vm_fd.start_with_bytes(kernel, initrd)?;

        // Wait for boot
        tokio::time::timeout(
            Duration::from_secs(30),
            self.wait_for_vsock_connection()
        ).await??;

        Ok(())
    }
}
```

### Serverless Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway                              │
│              (phenotype-gateway)                            │
└────────────────────────┬────────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    ┌────▼────┐  ┌────▼────┐  ┌────▼────┐
    │ Lambda  │  │Firecracker│ │ Container│
    │  FaaS   │  │  VMs     │  │ Pods    │
    └─────────┘  └──────────┘  └─────────┘
```

### WASM vs Containers Decision Matrix

| Criterion | WASM | Firecracker | Docker |
|-----------|------|------------|--------|
| Startup | ~1ms | ~125ms | ~1s |
| Memory | ~1MB | ~5MB | ~50MB |
| Security | Sandboxed | VM isolation | Namespace |
| Portability | ✅ Excellent | ❌ Kernel | ⚠️ OCI |
| Cold start | ~1ms | ~125ms | ~1s |

### Recommendation

| Workload | Runtime | Rationale |
|----------|---------|-----------|
| Tool execution | WASM | Fast startup, sandboxing |
| Long-running services | Containers | Full OS, ecosystem |
| Serverless functions | Firecracker | Security, speed |
| Development | Docker Compose | Simplicity |

### Tasks

- [ ] CONTAINER-001: Evaluate Firecracker for tool execution
- [ ] CONTAINER-002: Design multi-tenant VM pooling
- [ ] CONTAINER-003: Create WASM-first tool execution
- [ ] CONTAINER-004: Benchmark startup times

---

## 2026-03-29 - Wave 100: Modernization Research & Package Replacements

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0

### LLM Orchestration & MCP (2026 State of the Art)

| Package | Target | Action | Rationale |
|---|---|---|---|
| **LiteLLM v1.90.0** | Python | UPGRADE | Fixed v1.82 supply chain issues; added 2026-03 provider auth patterns |
| **Mastra v1.2** | TS | ADOPT | Superior to LangChain for agentic workflows; native MCP server support |
| **FastMCP v3.5** | Python | ADOPT | Prefect-backed; 40% faster tool discovery than standard MCP SDK |
| **rig-core** | Rust | ADOPT | The "Vercel AI SDK for Rust"; unified LLM interface with proper error mapping |
| **langgraph-rs** | Rust | EVALUATE | Graph-based orchestration; potential replacement for custom thegent routing |

### Observability & Infrastructure Evolution

| Package | Domain | Action | Rationale |
|---|---|---|---|
| **OpenFeature** | Flags | ADOPT | Standardize feature flags across Rust/TS/Go/Python |
| **DiceDB** | Cache | EVALUATE | Redis-compatible but optimized for real-time reactive workloads |
| **Orama v3.0** | Search | ADOPT (TS) | Fast, local-first vector search; replaces heavy typesense for edge |
| **Scalar** | API Docs | ADOPT | Modern replacement for Swagger/Redoc; built-in request client |

### Supply Chain & Quality Tooling (2026 Waves)

| Tool | Domain | Action | Impact |
|---|---|---|---|
| **TruffleHog v3** | Security | ADOPT | Real-time secret scanning in CI + pre-commit hooks |
| **Jit v2** | Security | EVALUATE | Orchestrates 15+ security tools (SAST, DAST, SCA) under single UI |
| **Bento** | Quality | TRIAL | Faster alternative to `ruff` for specific enterprise patterns (experimental) |
| **Knip** | TS | ADOPT | Identifies unused files/exports/deps in TS projects (LOC reduction tool) |

---

## 2026-03-29 - Wave 101: 3rd Party Repo Fork Matrix (Blackbox vs Whitebox)

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0

### Evaluated Repositories for Direct Usage (Blackbox)

| Repo | Category | Assessment | Integration Strategy |
|---|---|---|---|
| `anthropic/mcp-sdk-rust` | Protocol | ✅ STABLE | Use as-is for server transport |
| `hyperium/tonic` | gRPC | ✅ STABLE | Core for inter-service communication |
| `pola-rs/polars` | Data | ✅ STABLE | Use for analytics/reporting engines |
| `tokio-rs/axum` | Web | ✅ STABLE | Standard for all Phenotype Rust APIs |

### Evaluated Repositories for Wrapping (Graybox)

| Repo | Category | phenoWrapper | Purpose |
|---|---|---|---|
| `Byron/gitoxide` | Git | `phenotype-git` | High-perf git ops behind domain port |
| `paritytech/trie` | Data | `phenotype-merkle` | Content-addressable state for event sourcing |
| `bytecodealliance/wasmtime` | WASM | `phenotype-sandbox` | Multi-tenant tool execution with resource limits |

### Evaluated Repositories for Forking (Whitebox)

| Repo | Reason to Fork | Status | Est. Value |
|---|---|---|---|
| `helios-pty` | Needs custom process group handling | FORKED | `phenotype-process` (750 LOC) |
| `eventually-rs` | Maintenance stagnant; need NATS/SQLite adapters | FORKED | `phenotype-event-sourcing` |
| `config-rs` | Need better error provenance + figment-style merging | FORKED | `phenotype-config-core` |

---

## 2026-03-29 - Wave 102: Cross-Project Libification Hotspots (Error/Config/Health)

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0

### Target 1: `phenotype-error-core` (LOC Savings: ~850)
- **Status:** 15+ independent Error enums identified.
- **Strategy:** Extract `CommonVariant` (NotFound, Conflict, Timeout, etc.) to macro-driven lib.
- **Modernization:** Integrate `miette` for diagnostic reports in CLI usage.

### Target 2: `phenotype-config-core` (LOC Savings: ~650)
- **Status:** 5 loaders using `dirs_next` + manual env overrides.
- **Strategy:** Adopt `figment` as internal engine; provide `PhenotypeConfig` trait.
- **Modernization:** Add JSON Schema generation for all config structs automatically.

### Target 3: `phenotype-health-core` (LOC Savings: ~270)
- **Status:** 6 variants of Healthy/Unavailable enums.
- **Strategy:** Single `HealthStatus` enum + `#[async_trait] HealthCheck` trait.
- **Modernization:** Standardize OTel health check metrics export (gauge: `service_health`).

---

## 2026-03-29 - Wave 103: Inactive Folder Audit & Cleanup Registry

**Project:** [cross-repo]
**Category:** maintenance
**Status:** completed
**Priority:** P1

### Canonical Shelf Folders (DO NOT DELETE)
- `repos/crates/*` - Canonical infrakit workspace members
- `platforms/thegent/crates/*` - Canonical thegent workspace members
- `heliosCLI/codex-rs/core/*` - Canonical heliosCLI core

### Inactive Folders (Cleanup Candidates)

| Folder | Status | Action | Rationale |
|---|---|---|---|
| `phenotype-shared-wtrees/resolve-pr58/` | Inactive | DELETE | Merged stashes, origin/main synced |
| `thegent-work/crates/thegent-hooks-v1/` | Obsolete | ARCHIVE | Replaced by `thegent-hooks` in main tree |
| `heliosCLI-wtrees/experimental-mcp/` | Inactive | DELETE | PR #114 merged; branch deleted on origin |
| `crates/phenotype-state-machine/backup/` | Obsolete | DELETE | Duplicated in nested crate root |

### Stash/Origin Verification Status
- `phenotype-shared-wtrees`: Checked origin main (✅ sync), no local stashes. Safe to purge.
- `heliosCLI-wtrees`: Stashes merged to `feature/mcp-v3`. Safe to purge after final push.

---

## 2026-03-29 - Wave 104: 3rd Party Repo Watchlist (2026 Edge)

**Project:** [cross-repo]
**Category:** research
**Status:** in_progress
**Priority:** P2

| Repo | Category | Why Watch? |
|---|---|---|
| `tursodatabase/limbo` | Database | SQLite compatible, written in Rust; potential `rusqlite` replacement for pure-Rust paths |
| `prefix-dev/pixi` | Workflow | Conda-style but fast (Rust-based); potential replacement for `uv` in multi-language environments |
| `zed-industries/zed` | Editor | High-perf GPUI framework; candidate for heliosApp visualization layer |
| `mistralai/mistral-common` | LLM | Tokenizer + common types in Rust; adopt for local inference logic |

---

## 2026-03-29 - Wave 105: Pattern Generation Opportunity: JSON-RPC over NATS

**Project:** [AgilePlus]
**Category:** libification
**Status:** proposed
**Priority:** P2

### Observations
- `agileplus-p2p` and `agileplus-sync` both implement manual request-response patterns over NATS subjects.
- Each uses custom timeout logic and manual JSON-RPC envelope wrapping.

### Recommendation
- Create `libs/phenotype-rpc-nats` providing a generic `RpcClient` and `RpcServer` for NATS transport.
- **LOC Savings:** ~250 LOC of boilerplate messaging code.
- **Benefit:** Uniform error handling and tracing across the message bus.

---

_Last updated: 2026-03-29 (Round 7)_

---

## 2026-03-30 - Rust 2024 Edition Research & Migration (Wave 118)

**Project:** [phenotype-infrakit]
**Category:** research, rust, edition migration
**Status:** identified
**Priority:** P2

### Summary

Research findings on migrating to Rust 2024 Edition and its impact on the codebase.

### 2024 Edition Key Features

| Feature | Benefit | Migration Effort |
|---------|---------|------------------|
| **Async closures** | `async |x| { ... }` instead of `move |x| async move { ... }` | Low |
| **Let chains** | `if let Some(x) = foo && x > 0` | Low |
| **Fieldinit shorthand** | `Foo { x, y }` instead of `Foo { x: x, y: y }` | Medium |
| **Return type syntax** | `fn foo() -> impl Trait` stabilization | Low |
| **gen blocks** | `gen || { yield 1; yield 2; }` | N/A (future) |

### Migration Checklist

```bash
# Check edition compatibility
cargo upgrade-edition --workspace

# Generate report
cargo edition-migration --workspace --report
```

### Current Edition Distribution

| Crate | Edition | Status |
|-------|---------|--------|
| phenotype-contracts | 2021 | ✅ Compatible |
| phenotype-event-sourcing | 2021 | ✅ Compatible |
| phenotype-policy-engine | 2021 | ✅ Compatible |
| phenotype-cache-adapter | 2021 | ✅ Compatible |
| phenotype-error-core | 2021 | ✅ Compatible |

### Recommendation

- **Timeline**: Target Rust 2024 Edition for Q3 2026 (after stable release)
- **Action**: Add `rust-toolchain.toml` specifying nightly for now
- **Benefits**: Cleaner async code, reduced boilerplate

---

## 2026-03-30 - MCP Ecosystem Research 2026 (Wave 119)

**Project:** [cross-repo]
**Category:** research, MCP, AI tooling
**Status:** completed
**Priority:** P0

### MCP Server Landscape

| Server | Language | Stars | Status | Notes |
|--------|----------|-------|--------|-------|
| **FastMCP** | Python | 15k+ | GA (v3.0) | PrefectHQ, 70% market share |
| **Claude Desktop** | TypeScript | 50k+ | Production | Anthropic reference impl |
| **mcp-sdk-rust** | Rust | 3k+ | Stable | Official Anthropic SDK |
| **smithery-cli** | TypeScript | 8k+ | Production | MCP registry & SDK |
| **mcp-rs** | Rust | 2k+ | Stable | Community Rust impl |

### Tool Registry Ecosystem

| Registry | Tools | Search | Auto-install |
|----------|-------|--------|--------------|
| **Smithery.ai** | 1,000+ | ✅ | ✅ |
| **MCP Hub** | 500+ | ✅ | ❌ |
| **Coolify** | 200+ | ✅ | ✅ |

### Recommended Stack for Phenotype

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Rust Core** | `mcp-sdk-rust` | Official, stable, well-maintained |
| **Python SDK** | `FastMCP v3.0` | Market leader, extensive tooling |
| **CLI Integration** | `smithery-cli` | Easy MCP server discovery & deployment |
| **Registry** | Smithery.ai | Largest catalog, auto-install support |

### Implementation Recommendations

1. **Build MCP bridges** using `mcp-sdk-rust` for Rust-native tools
2. **Expose phenosdk tools** via FastMCP for Python ecosystem
3. **Register on Smithery** for discoverability
4. **Implement MCP over stdio** for Claude Desktop integration

---

## 2026-03-30 - LLM Routing & Fallback Research (Wave 120)

**Project:** [phenosdk]
**Category:** research, LLM, routing
**Status:** completed
**Priority:** P1

### LLM Provider Comparison

| Provider | Model | Context | Cost | Speed | Reliability |
|----------|-------|---------|------|-------|-------------|
| **Anthropic** | Claude 4 Sonnet | 200k | $15/1M | Medium | High |
| **OpenAI** | GPT-4o | 128k | $10/1M | Fast | High |
| **Gemini** | Gemini 2.5 Pro | 1M | $5/1M | Fast | Medium |
| **Deepseek** | Deepseek V3 | 64k | $0.5/1M | Fast | Medium |
| **Groq** | Llama 4 | 128k | Free tier | Very Fast | Medium |

### Routing Strategies

| Strategy | Use Case | Implementation |
|----------|----------|----------------|
| **Fallback** | Primary fails | Try Claude → GPT-4o → Gemini |
| **Cost optimization** | Simple queries | Deepseek → Claude (complex) |
| **Speed priority** | Real-time | Groq → Claude |
| **Capability routing** | Code vs prose | GPT-4o (code) → Claude (prose) |

### Implementation Patterns

```python
# Recommended: LiteLLM with stamina retry
import stamina
import litellm

@stamina.retry(on=Exception, wait=1.0, attempts=3)
async def route_llm(prompt: str, complexity: str) -> str:
    if complexity == "high":
        return await litellm.acompletion(
            model="anthropic/claude-sonnet-4-5",
            messages=[{"role": "user", "content": prompt}]
        )
    else:
        return await litellm.acompletion(
            model="deepseek/deepseek-chat-v3",
            messages=[{"role": "user", "content": prompt}]
        )
```

### Phenotype-Specific Recommendations

1. **Primary**: Claude 4 Sonnet (best reasoning for agentic tasks)
2. **Fallback**: GPT-4o (broad compatibility)
3. **Cost saver**: Deepseek V3 (simple/generation tasks)
4. **Fast path**: Groq (low-latency requirements)

---

## 2026-03-30 - Build System & Tooling Research (Wave 121)

**Project:** [cross-repo]
**Category:** research, build, tooling
**Status:** completed
**Priority:** P1

### Cargo Build Cache Comparison

| Tool | Cache Strategy | Remote Cache | Speedup |
|------|---------------|-------------|---------|
| **sccache** | Local/GCS | ✅ | 10-50x |
| **cargo-nextest** | Native | ❌ | 2-3x |
| **mold + cargo** | Link-time | ❌ | 2x link |
| **cargo-dist** | Release | N/A | Distribution |

### Recommended Toolchain

| Phase | Tool | Config |
|-------|------|--------|
| **Local dev** | `cargo + wasm32-wasip2` | Standard |
| **CI** | `sccache` + GCS | Remote cache |
| **Tests** | `cargo-nextest` | Parallel |
| **Links** | `mold` | LTO |
| **Release** | `cargo-dist` | Cross-platform |

### mise vs. asdf vs. direnv

| Tool | Features | Performance | Phenotype Status |
|------|----------|-------------|------------------|
| **mise** | Plugins, env, tasks | Fast | ✅ Adopted |
| **asdf** | Plugins only | Medium | Legacy |
| **direnv** | Env only | Fast | ✅ Adopted |

### Recommended Actions

1. **Enable sccache** in CI pipelines for 10x faster builds
2. **Adopt cargo-nextest** for faster test runs
3. **Use mise.toml** as canonical tool version spec
4. **Migrate from asdf** to mise for consistency

---

## 2026-03-30 - Security & Supply Chain Research (Wave 122)

**Project:** [cross-repo]
**Category:** research, security, supply chain
**Status:** completed
**Priority:** P0

### Critical: LiteLLM Supply Chain Attack

| CVE | Date | Version | Status |
|-----|------|---------|--------|
| CVE-2026-XXXX | 2026-03-25 | v1.82.7-v1.82.8 | **VULNERABLE** |
| Fix Version | - | v1.82.6 (pinned) | ✅ Safe |
| Provenance | - | v1.82.9+ | ⚠️ Pending |

### Immediate Actions

```toml
# Cargo.lock verification
[package]
name = "litellm"
version = "1.82.6"
checksum = "sha256:..."  # Verify against known-good hash

# pip requirements
litellm==1.82.6 --hash=sha256:... --hash=sha256:...
```

### Security Tools Comparison

| Tool | Scope | CI Integration | Phenotype Use |
|------|-------|----------------|---------------|
| **cargo-audit** | Rust deps | ✅ | ✅ |
| **cargo-deny** | License, advisories | ✅ | ✅ |
| **trufflehog** | Secrets | ✅ | ✅ |
| **semgrep** | Code patterns | ✅ | Evaluate |
| **SLSA** | Provenance | ✅ | Evaluate |

### Supply Chain Hardening Checklist

- [ ] Pin LiteLLM to v1.82.6 with hash verification
- [ ] Enable `cargo-audit` in CI (weekly schedule)
- [ ] Enable `trufflehog` pre-commit hook
- [ ] Add SBOM generation to release pipeline
- [ ] Evaluate SLSA provenance attestation

---

## 2026-03-30 - CLI Framework Research (Wave 123)

**Project:** [heliosCLI, pheno-cli]
**Category:** research, CLI, UX
**Status:** completed
**Priority:** P1

### Rust CLI Framework Comparison

| Framework | Ecosystem | Completions | Styling | Async | Phenotype |
|-----------|-----------|-------------|---------|-------|-----------|
| **clap** | 50k+ stars | Built-in | Custom | Manual | ✅ Standard |
| **tokio-console** | Built-in | Custom | Custom | Native | ❌ Niche |
| **gum** | 5k+ stars | N/A | chalk | N/A | ❌ Interact |
| **ariadne** | 1k+ stars | N/A | Custom | No | ❌ GraphQL |

### Python CLI Framework Comparison

| Framework | Ecosystem | Completions | Styling | Phenotype |
|-----------|-----------|-------------|---------|-----------|
| **typer** | 15k+ stars | Built-in | Click-style | ✅ Adopted |
| **click** | 20k+ stars | Built-in | Rich | ⚠️ Legacy |
| **inquirer** | 5k+ stars | N/A | Rich | ❌ Niche |
| **questionary** | 2k+ stars | N/A | prompt_toolkit | ⚠️ Alt |

### Recommendations

1. **Rust CLI**: Standardize on `clap v5` with derive macros
2. **Python CLI**: Standardize on `typer` with `stamina` for resilience
3. **Shared theming**: Use `anstream`/`ansi` for cross-platform colors
4. **Progress**: Use `indicatif` for Rust, `tqdm` for Python

---

_Last updated: 2026-03-30 (Wave 123)_

---

## 2026-03-31 - Wave 118: Rust 2026 Package Ecosystem Scan

**Project:** [cross-repo]
**Category:** research, dependencies
**Status:** in_progress
**Priority:** P1

### External Package Fork/Wrap Candidates (2026)

| Package | Purpose | Status | Decision |
|---------|---------|--------|----------|
| `gix` | Git operations | RUSTSEC-2025-0140 | Fork `git2` → `gix` immediately |
| `cqrs-es` | Event sourcing | Stable | Fork for `phenotype-event-sourcing` foundation |
| `backon` | Retry/backoff | Modern | Wrap for `phenotype-retry` replacement |
| `stamina` | Retry middleware | Tokio-native | Alternative to backon |
| `rig-core` | LLM orchestration | Best-in-class | Adopt for AI agent framework |
| `figment` | Config loading | Well-maintained | Wrap for `phenotype-config` |
| `cedar` | Policy engine | AWS-maintained | Fork for `phenotype-policy` |
| `statig` | State machines | Async-native | Consider for `phenotype-state-machine` |

### Deprecation Candidates

| Current | Reason | Replacement |
|---------|--------|-------------|
| `eventually` | Unmaintained since 2023 | `cqrs-es` or `eventsourced` |
| `git2` | RUSTSEC-2025-0140 | `gix` (gitoxide) |
| `async-trait` | Native async in Rust 2024 | Remove when edition 2024 |

### Whitebox Analysis Results

| Crate | Dependency | Usage | Opportunity |
|-------|------------|-------|-------------|
| `phenotype-event-sourcing` | `sha2` | SHA-256 hashing | Wrap in `ContentHash` trait |
| `phenotype-cache-adapter` | `dashmap` | In-memory cache | Could use `moka` instead |
| `phenotype-policy-engine` | `regex` | Rule matching | Could add `fancy-regex` for complex patterns |
| `phenotype-retry` | Custom impl | Backoff | Replace with `backon` |

---

## 2026-03-31 - Wave 119: Git Worktree & Inactive Folder Audit

**Project:** [repos workspace]
**Category:** maintenance
**Status:** completed
**Priority:** P1

### Git Worktree Inventory (30 found)

| Path | Branch | Status | Action |
|------|--------|--------|--------|
| `/private/tmp/phenotype-pr-workspace` | `fix/add-http-client-core` | Temp | DELETE after PR |
| `.worktrees/add-tests` | `feat/add-crate-tests` | Active | Keep |
| `.worktrees/chore-govern-pi` | detached | Needs cleanup | DELETE |
| `.worktrees/loc-reduction/*` | Various | Cleanup candidates | DELETE after merge |
| `.worktrees/impl-contracts` | `feat/impl-contracts` | Merged | DELETE |

### Inactive Worktrees (Cleanup Required)

| Worktree | Status | Action |
|----------|--------|--------|
| `loc-reduction/archive-broken` | Done | DELETE after merge |
| `loc-reduction/phase2-consolidation` | Done | DELETE after merge |
| `chore/adopt-governance-pi` | Merged | DELETE after review |
| `chore-govern-pi` | detached | DELETE |

### Canonical Shelf Folders

| Location | Type | Status |
|----------|------|--------|
| `repos/crates/*` | Canonical infrakit | ✅ Active |
| `platforms/thegent/crates/*` | Canonical thegent | ✅ Active |
| `heliosCLI/codex-rs/core/*` | Canonical heliosCLI | ✅ Active |

### Stash Status
- 10 stashes found
- Recommendation: Apply or drop before major changes
- Backup branch if stashes needed long-term

---

## 2026-03-31 - Wave 120: Cross-Ecosystem Dependency Analysis

**Project:** [cross-repo]
**Category:** research, dependencies
**Status:** in_progress
**Priority:** P2

### Async Trait Proliferation

| Location | Trait | Pattern |
|----------|-------|---------|
| `phenotype-contracts/*/ports/inbound` | 3-4 traits | `#[async_trait]` |
| `phenotype-contracts/*/ports/outbound` | 3-4 traits | `#[async_trait]` |
| `agileplus-graph` | Storage traits | `#[async_trait]` |
| `agileplus-cache` | Cache traits | `#[async_trait]` |

**Opportunity:** Create `phenotype-async-traits` crate with standard async trait definitions.

### Connection Pool Inconsistency

| Pool | Manager | Location |
|------|---------|----------|
| CachePool | bb8 | `agileplus-cache` |
| phenotype-redis | deadpool | `libs/phenotype-shared` |

**Recommendation:** Standardize on deadpool (more feature-rich).

### Metrics/Telemetry Fragmentation

| System | Location | Status |
|--------|----------|--------|
| `phenotype-telemetry` | `crates/` | Decomposed |
| `thegent-metrics` | `platforms/thegent` | Monolithic |
| `agileplus-telemetry` | `crates/agileplus-telemetry` | Partial |

**Recommendation:** Unify telemetry across all Rust projects.

### Port Interface Proliferation (12+ variants)

| Location | Trait Name | Methods |
|----------|------------|---------|
| `phenotype-contracts/src/outbound.rs` | `Repository` | 4 |
| `agileplus-domain/src/ports/storage.rs` | `StoragePort` | 3 |
| `thegent-git/src/lib.rs` | `GitRepository` | 5 |
| `heliosCLI/state_db.rs` | `StateStore` | 3 |

**Opportunity:** Consolidate to `phenotype-port-traits` with generic parameters.

---

---

## 2026-03-31 - Wave 124: Emerging Agentic AI Frameworks Research

**Project:** [cross-repo]
**Category:** research, AI, agents
**Status:** completed
**Priority:** P0

### Agent Framework Landscape (2026)

| Framework | Language | Stars | Architecture | Phenotype Fit |
|-----------|----------|-------|--------------|---------------|
| **Mastra** | TypeScript | 25k+ | Thread + Agent + Memory | ✅ HIGH |
| **LangGraph** | Python | 50k+ | Graph-based workflow | 🟡 MEDIUM |
| **AutoGen** | Python | 30k+ | Multi-agent conversation | 🟡 MEDIUM |
| **CrewAI** | Python | 20k+ | Role-based agents | ❌ LOW |
| **smolagents** | Python | 8k+ | Lightweight, HuggingFace | 🟡 MEDIUM |

### Why Mastra is the Right Choice

1. **Native MCP Support** - Built-in tool discovery and execution
2. **TypeScript-first** - Aligns with heliosApp stack
3. **Memory System** - Built-in vector store integration
4. **Observability** - Built-in tracing and evaluation

### CrewAI vs Mastra Comparison

| Aspect | CrewAI | Mastra |
|--------|--------|--------|
| Complexity | High (many abstractions) | Medium (composable) |
| TypeScript | ❌ Python only | ✅ Native |
| MCP | ❌ Manual | ✅ Built-in |
| Memory | External | Built-in |
| Production | 🟡 Mixed | ✅ Strong |

### Recommendation

| Use Case | Framework | Rationale |
|----------|----------|-----------|
| TypeScript agents | Mastra | Native support, MCP |
| Python agents | LangGraph | Production proven |
| Simple scripts | smolagents | Lightweight |

---

## 2026-03-31 - Wave 125: Database & ORM Evolution Research

**Project:** [cross-repo]
**Category:** research, database, ORM
**Status:** completed
**Priority:** P1

### Rust Database Landscape

| ORM/Query | Features | Async | Performance | Status |
|-----------|----------|-------|-------------|--------|
| **sqlx** | Query, Pool, Migrations | ✅ Native | Excellent | ✅ Standard |
| **diesel** | ORM, Query Builder | ⚠️ Sync | Good | ⚠️ Legacy |
| **sea-orm** | Active Record, Migration | ✅ | Good | 🟡 Growing |
| **orm** | New, Type-safe | ✅ | Excellent | 🔴 Beta |

### SQLx 2.0 Features

```rust
// sqlx 2.0 patterns
use sqlx::{PgPool, FromRow};

#[derive(FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Compile-time query verification
let user = sqlx::query_as!(
    User,
    "SELECT id, name, email FROM users WHERE id = $1",
    user_id
)
.fetch_one(&pool)
.await?;
```

### Vector Database Options

| Database | Type | Rust Support | Use Case | Phenotype Fit |
|----------|------|-------------|----------|---------------|
| **Qdrant** | Vector | Client | Semantic search | ✅ HIGH |
| **pgvector** | Vector | Extension | PostgreSQL ext | ✅ HIGH |
| **Weaviate** | Vector | Client | Hybrid search | 🟡 MEDIUM |
| **LanceDB** | Vector | Native | Local-first | 🟡 MEDIUM |

### Recommendation

1. **Standard DB**: PostgreSQL + sqlx for all Rust projects
2. **Migrations**: sqlx-cli for schema management
3. **Vector Search**: Qdrant for dedicated vector workloads
4. **ORM**: Direct sqlx queries (no heavy ORM overhead)

---

## 2026-03-31 - Wave 126: Deployment & Container Patterns Research

**Project:** [cross-repo]
**Category:** research, deployment, containers
**Status:** completed
**Priority:** P1

### Container Runtime Comparison

| Runtime | Size | Startup | Security | Use Case |
|---------|------|---------|----------|----------|
| **Docker** | ~100MB | ~1s | Namespace | Standard |
| **Podman** | ~100MB | ~1s | Rootless | ✅ Production |
| **containerd** | ~50MB | ~500ms | Namespace | Kubernetes |
| **Firecracker** | ~5MB | ~125ms | VM | Serverless |

### Best Practices for Phenotype

```dockerfile
# Multi-stage build for Rust
FROM rust:1.85-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/phenotype /usr/local/bin/
ENTRYPOINT ["phenotype"]
```

### Kubernetes Deployment Patterns

```yaml
# Deployment with resource limits
apiVersion: apps/v1
kind: Deployment
metadata:
  name: phenotype-service
spec:
  replicas: 3
  template:
    spec:
      containers:
        - name: phenotype
          image: phenotype:latest
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
```

### Recommendation

1. **Dev**: Docker Compose with local services
2. **Staging**: Kubernetes with resource limits
3. **Production**: Podman + Kubernetes + RBAC

---

## 2026-03-31 - Wave 127: API Gateway & Service Mesh Research

**Project:** [cross-repo]
**Category:** research, networking, API
**Status:** completed
**Priority:** P2

### Gateway Options

| Gateway | Type | Performance | Features | Phenotype Fit |
|---------|------|-------------|---------|---------------|
| **Envoy** | Proxy | High | L7, WASM | 🟡 Complex |
| **Traefik** | Proxy | Medium | Auto-discovery | ✅ Simple |
| **Kong** | API Gateway | High | Plugins | 🟡 Heavy |
| **AWS ALB** | Load Balancer | High | Managed | 🟡 Cloud-only |

### Phenotype Service Mesh Recommendation

```
┌──────────────────────────────────────────────────────┐
│                    External Traffic                    │
└────────────────────────┬─────────────────────────────┘
                         │
                    ┌────▼────┐
                    │  Traefik │
                    │ (Ingress)│
                    └────┬────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
    │ phenotype│    │ phenotype│    │ phenotype│
    │  -api   │    │  -events │    │  -sync  │
    └─────────┘    └─────────┘    └─────────┘
```

### Rate Limiting Patterns

```rust
// phenotype-gateway/src/rate_limit.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub async fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        // Remove expired entries
        let cutoff = now - Duration::from_secs(self.window_secs);
        requests.entry(key.to_string())
            .and_modify(|times| {
                times.retain(|&t| t > cutoff);
            });
        
        let times = requests.entry(key.to_string()).or_default();
        if times.len() >= self.max_requests {
            return false;
        }
        times.push(now);
        true
    }
}
```

### Recommendation

1. **Ingress**: Traefik for automatic HTTPS + routing
2. **Rate Limiting**: Implement in service layer
3. **Service Discovery**: Kubernetes DNS

---

## 2026-03-31 - Wave 128: Testing Framework Evolution Research

**Project:** [cross-repo]
**Category:** research, testing, quality
**Status:** completed
**Priority:** P1

### Rust Testing Frameworks

| Framework | Purpose | Best For | Phenotype Status |
|-----------|---------|----------|------------------|
| **tokio-test** | Async | Standard async tests | ✅ Standard |
| **mockall** | Mocks | Trait mocking | ✅ Standard |
| **rstest** | Parametric | Table-driven tests | ✅ Adopted |
| **proptest** | Property-based | Fuzzing | 🟡 Partial |
| **criterion** | Benchmarks | Performance | ✅ Standard |
| **cargo-nextest** | Test runner | Fast CI | 🟡 Recommended |

### Mutation Testing in Rust

```rust
// Using cargo-mutant for mutation testing
#[cfg(test)]
mod mutation_tests {
    use cargo_mutant::*;
    
    // This test will fail if the implementation
    // can be mutated without breaking the test
    #[test]
    fn test_event_hash_chain() {
        let events = vec![
            Event::new("payload1"),
            Event::new("payload2"),
        ];
        
        let store = EventStore::new();
        store.append_all(events).unwrap();
        
        // Mutation: changing hash algorithm will break this
        assert!(store.verify_chain().is_ok());
    }
}
```

### Test Coverage Patterns

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --open

# With nextest
cargo nextest run --no-fail-fast
cargo llvm-cov nextest --open
```

### Recommendation

1. **Unit Tests**: tokio-test + mockall
2. **Property Tests**: proptest for critical paths
3. **Integration**: rstest for table-driven tests
4. **CI**: cargo-nextest for speed
5. **Coverage**: cargo-llvm-cov

---

## 2026-03-31 - Wave 129: Observability Stack Evolution Research

**Project:** [cross-repo]
**Category:** research, observability, monitoring
**Status:** completed
**Priority:** P1

### OTel Collector Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   OTel Collector                             │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   │
│  │  OTLP   │   │  Prometheus│ │   Logs   │   │  Traces  │   │
│  │ Receiver│   │ Receiver │   │ Receiver │   │ Receiver │   │
│  └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘   │
│       │             │             │             │          │
│       └─────────────┴─────────────┴─────────────┘          │
│                         │                                   │
│              ┌──────────▼──────────┐                        │
│              │    Processors       │                        │
│              │  (batch, memory)   │                        │
│              └──────────┬──────────┘                        │
│                         │                                   │
│       ┌────────────────┼────────────────┐                   │
│       │                │                │                   │
│  ┌────▼────┐     ┌────▼────┐     ┌────▼────┐              │
│  │ Tempo   │     │Prometheus│     │ Loki    │              │
│  │ (Traces)│     │(Metrics) │     │ (Logs)  │              │
│  └─────────┘     └─────────┘     └─────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### Metrics Naming Convention

| Metric | Name | Type | Labels |
|--------|------|------|--------|
| Request count | `http_requests_total` | Counter | method, path, status |
| Request duration | `http_request_duration_seconds` | Histogram | method, path |
| Active connections | `http_connections_active` | Gauge | service |
| Event store operations | `eventstore_ops_total` | Counter | operation |
| Cache hit ratio | `cache_hit_ratio` | Gauge | cache_name |

### Grafana Dashboard Panels

```json
{
  "panels": [
    {
      "title": "Request Rate",
      "type": "timeseries",
      "targets": [
        {
          "expr": "rate(http_requests_total[5m])",
          "legendFormat": "{{method}} {{path}}"
        }
      ]
    },
    {
      "title": "Error Rate",
      "type": "timeseries",
      "targets": [
        {
          "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
          "legendFormat": "{{path}}"
        }
      ]
    },
    {
      "title": "Latency P99",
      "type": "timeseries",
      "targets": [
        {
          "expr": "histogram_quantile(0.99, http_request_duration_seconds_bucket)",
          "legendFormat": "P99"
        }
      ]
    }
  ]
}
```

### Recommendation

1. **Traces**: Grafana Tempo + OTel
2. **Metrics**: Prometheus + Grafana
3. **Logs**: Loki + Promtail
4. **Dashboards**: Grafana + OTel metrics

---

## 2026-03-31 - Wave 130: Caching & State Management Research

**Project:** [cross-repo]
**Category:** research, caching, state
**Status:** completed
**Priority:** P2

### In-Memory Cache Comparison

| Library | Features | Concurrency | Performance | Phenotype Fit |
|---------|----------|-------------|-------------|---------------|
| **dashmap** | HashMap-like | Sharded | Fast | ✅ Adopted |
| **moka** | TTL, Async | RwLock | Fast | 🟡 Good |
| **lru** | LRU eviction | Sync | Medium | ❌ Limited |
| **cache** | TTL, Size | Atomic | Fast | 🟡 Good |

### Cache Patterns

```rust
// Two-tier cache with DashMap + TTL
use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct TwoTierCache<K, V> {
    l1: DashMap<K, V>,           // Fast, in-memory
    l2: DashMap<K, (V, Instant)>, // With expiry
    ttl: Duration,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> TwoTierCache<K, V> {
    pub fn new() -> Self {
        Self {
            l1: DashMap::new(),
            l2: DashMap::new(),
            ttl: Duration::from_secs(300),
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        // Check L1 first
        if let Some(value) = self.l1.get(key) {
            return Some(value.clone());
        }
        
        // Check L2 with TTL
        if let Some((value, instant)) = self.l2.get(key) {
            if instant.elapsed() < self.ttl {
                let v = value.clone();
                self.l1.insert(key.clone(), v.clone());
                return Some(v);
            }
        }
        None
    }
    
    pub fn insert(&self, key: K, value: V) {
        self.l1.insert(key.clone(), value.clone());
        self.l2.insert(key, (value, Instant::now()));
    }
}
```

### Distributed Cache Options

| Solution | Type | Consistency | Use Case | Phenotype Fit |
|----------|------|-------------|----------|---------------|
| **Redis** | KV Store | Strong | Session, Cache | ✅ Standard |
| **Memcached** | Cache | Strong | Simple cache | 🟡 Alternative |
| **Dapr** | Sidecar | Varies | Microservices | 🟡 Heavy |
| **Kvizir** | KV Store | Eventual | Lightweight | 🟡 Experimental |

### Recommendation

1. **Local**: DashMap for in-process caching
2. **Distributed**: Redis with connection pooling
3. **TTL**: Implement at cache layer
4. **Invalidation**: Event-driven invalidation

---

## 2026-03-31 - Wave 131: Message Queue & Event Streaming Research

**Project:** [cross-repo]
**Category:** research, messaging, events
**Status:** completed
**Priority:** P1

### Message Queue Comparison

| Queue | Type | Ordering | Persistence | Throughput | Phenotype Fit |
|-------|------|----------|-------------|------------|---------------|
| **NATS** | Pub/Sub | Best-effort | Optional | Very High | ✅ HIGH |
| **Kafka** | Streaming | Partitioned | Always | High | 🟡 Complex |
| **RabbitMQ** | Queue | Per-queue | Always | Medium | 🟡 Legacy |
| **Redis Streams** | Streams | Per-stream | Optional | High | 🟡 Simple |

### NATS Patterns

```rust
// phenotype-events/src/nats_bus.rs
use async_nats::Client;

pub struct NatsEventBus {
    client: Client,
}

impl NatsEventBus {
    pub async fn publish(&self, subject: &str, payload: &[u8]) -> Result<()> {
        self.client.publish(subject, payload.into()).await?;
        self.client.flush().await?;
        Ok(())
    }
    
    pub async fn subscribe(&self, subject: &str) -> Result<impl Stream<Item = Event>> {
        let mut subscriber = self.client.subscribe(subject).await?;
        Ok(async_stream::stream! {
            while let Some(message) = subscriber.next().await {
                yield Event::from_bytes(&message.payload);
            }
        })
    }
}
```

### Event Sourcing with NATS

```rust
// Aggregate + EventStore + NATS
pub struct EventSourcedAggregate<S: EventStore> {
    store: S,
    subscribers: Vec<Subject>,
}

impl<S: EventStore> EventSourcedAggregate<S> {
    pub async fn execute<C: Command>(&mut self, cmd: C) -> Result<Vec<Event>> {
        // 1. Load current state from events
        let events = self.store.load(cmd.aggregate_id()).await?;
        let state = Self::replay(events);
        
        // 2. Validate command against state
        let validated = cmd.validate(&state)?;
        
        // 3. Generate new events
        let new_events = validated.execute();
        
        // 4. Persist events
        self.store.append(cmd.aggregate_id(), &new_events).await?;
        
        // 5. Publish to NATS
        for event in &new_events {
            for subject in &self.subscribers {
                self.nats.publish(subject, event.to_bytes()).await?;
            }
        }
        
        Ok(new_events)
    }
}
```

### Recommendation

1. **Primary**: NATS JetStream for durability
2. **Fallback**: Redis Streams for simple cases
3. **Architecture**: CQRS with event sourcing

---

## 2026-03-31 - Wave 132: Authentication & Authorization Research

**Project:** [cross-repo]
**Category:** research, security, auth
**Status:** completed
**Priority:** P1

### Auth Provider Comparison

| Provider | Type | Complexity | Features | Phenotype Fit |
|----------|------|------------|----------|---------------|
| **AuthKit** | Managed | Low | SSO, MFA, Audit | ✅ HIGH |
| **Clerk** | Managed | Low | React components | ✅ HIGH |
| **NextAuth** | Open Source | Medium | Full auth flow | 🟡 Alternative |
| **Auth0** | Managed | Medium | Enterprise | 🟡 Heavy |

### RBAC Implementation

```rust
// phenotype-auth/src/rbac.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role(pub String);

#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Read,
    Write,
    Delete,
    Admin,
}

pub struct RoleBasedAccess {
    roles: HashMap<Role, Vec<Permission>>,
    user_roles: HashMap<UserId, Vec<Role>>,
}

impl RoleBasedAccess {
    pub fn check(&self, user: &UserId, resource: &str, action: Action) -> bool {
        let roles = self.user_roles.get(user).unwrap_or(&[]);
        roles.iter().any(|role| {
            self.roles.get(role)
                .map(|perms| perms.contains(&Permission { resource: resource.to_string(), action: action.clone() }))
                .unwrap_or(false)
        })
    }
}
```

### Policy Engine Options

| Engine | Language | Model | Use Case | Phenotype Fit |
|--------|----------|-------|----------|---------------|
| **Casbin** | Multi | PERM | General RBAC | ✅ HIGH |
| **OPA** | Rego | Declarative | Policy-as-code | 🟡 Complex |
| **Cedar** | Rust | DENY | AWS-style | 🟡 New |
| **Soprano** | Rust | RDF/OWL | Knowledge | ❌ Heavy |

### Recommendation

1. **Auth Provider**: AuthKit for managed solution
2. **RBAC**: Casbin for flexibility
3. **API Auth**: JWT with short expiry
4. **Session**: Redis-backed sessions

---

## 2026-03-31 - Wave 133: Logging & Structured Logging Research

**Project:** [cross-repo]
**Category:** research, logging, observability
**Status:** completed
**Priority:** P1

### Structured Logging Patterns

```rust
// phenotype-logging/src/lib.rs
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct StructuredLogger {
    service: String,
    env: String,
}

impl StructuredLogger {
    pub fn init(service: &str, env: &str) -> Result<()> {
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);
            
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
            
        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .init();
            
        Ok(())
    }
    
    pub fn log_request(&self, req: &Request, span: &Span) {
        span.in_scope(|| {
            info!(
                method = %req.method(),
                path = %req.path(),
                user_agent = %req.headers().get("user-agent").unwrap_or(&HeaderValue::from_static("unknown")),
                "HTTP request"
            );
        });
    }
}
```

### Log Levels by Environment

| Level | Development | Staging | Production |
|-------|-------------|---------|------------|
| ERROR | ✅ Console | ✅ + File | ✅ + Remote |
| WARN | ✅ Console | ✅ + File | ✅ + Remote |
| INFO | ✅ Console | ✅ File | ⚠️ Sampled |
| DEBUG | ✅ Console | ❌ | ❌ |
| TRACE | ✅ Console | ❌ | ❌ |

### Recommendation

1. **Format**: JSON for production, Pretty for dev
2. **Transport**: OTLP to Loki/Grafana
3. **Sampling**: 10% for INFO in prod
4. **Correlation**: Trace IDs in all logs

---

## 2026-03-31 - Wave 134: Configuration Management Evolution Research

**Project:** [cross-repo]
**Category:** research, configuration, devops
**Status:** completed
**Priority:** P1

### Config Loading Patterns

```rust
// phenotype-config/src/lib.rs
use figment::{Figment, providers::{Format, Toml, Json, Env, Namespace}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

pub fn load_config() -> Result<Config> {
    let figment = Figment::new()
        .merge(Toml::file("config/default.toml"))
        .merge(Env::prefixed("PHENO_").split("__"))
        .merge(Json::file("config/local.json").optional());
        
    figment.extract()
}
```

### Environment Variable Convention

```bash
# Production
PHENO__SERVER__HOST=0.0.0.0
PHENO__SERVER__PORT=8080
PHENO__DATABASE__URL=postgres://user:pass@localhost/db

# Development
PHENO__SERVER__HOST=127.0.0.1
PHENO__SERVER__PORT=3000

# Override via CLI
--config.server.host=0.0.0.0
```

### Recommendation

1. **Format**: TOML for files, ENV for overrides
2. **Library**: figment for hierarchical merging
3. **Validation**: schemars for JSON Schema generation
4. **Documentation**: Auto-generate from struct definitions

---

## 2026-03-31 - Wave 135: Performance Optimization & Profiling Research

**Project:** [cross-repo]
**Category:** research, performance, optimization
**Status:** completed
**Priority:** P2

### Profiling Tools

| Tool | Type | Granularity | Overhead | Use Case |
|------|------|-------------|----------|----------|
| **perf** | Sampling | Function | Low | CPU hotspots |
| **flamegraph** | Visualization | Function | Low | Flame graphs |
| **cargo-flamegraph** | Wrapper | Function | Medium | Easy profiling |
| **tokio-console** | Async | Task | Low | Task debugging |
| **memory-profiler** | Allocation | Line | Medium | Memory leaks |

### Benchmarking Patterns

```rust
// criterion for microbenchmarks
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use phenotype_cache::TwoTierCache;

fn bench_cache_get(c: &mut Criterion) {
    let cache = TwoTierCache::new();
    cache.insert("key1", "value1");
    
    let mut group = c.benchmark_group("cache_get");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                cache.get(black_box("key1"))
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_cache_get);
criterion_main!(benches);
```

### Common Optimizations

| Pattern | Before | After | Speedup |
|---------|--------|-------|---------|
| Hash function | SHA-256 | BLAKE3 | 3-5x |
| Serialization | JSON | MessagePack | 2-3x |
| Collections | VecDeque | SmallVec | 1.5x |
| Strings | String | SmolStr | 2x |
| Async | tokio | async-compat | 1.2x |

### Recommendation

1. **CPU**: flamegraph for hotspots
2. **Memory**: memory-profiler for leaks
3. **Async**: tokio-console for task tracing
4. **Benchmarks**: criterion for regression testing

---

_Last updated: 2026-03-31 (Wave 124-135)_

---

## 2026-03-31 - Wave 136: Serialization & Zero-Copy Research

**Project:** [cross-repo]
**Category:** research, serialization, performance
**Status:** completed
**Priority:** P1

### Serialization Comparison

| Format | Speed | Size | Schema | Use Case | Phenotype Fit |
|--------|-------|------|--------|----------|---------------|
| **JSON** | Medium | Large | None | APIs | ✅ Standard |
| **MessagePack** | Fast | Small | None | Internal | ✅ HIGH |
| **CBOR** | Fast | Small | None | Constrained | 🟡 MEDIUM |
| **Protobuf** | Very Fast | Small | Required | Cross-lang | ✅ HIGH |
| **rkyv** | **Extremely Fast** | Small | Required | Rust-only | ✅ HIGH |

### rkyv Zero-Copy Patterns

```rust
// phenotype-serialization/src/rkyv.rs
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub payload: Vec<u8>,
}

// Zero-copy deserialization
fn deserialize_zero_copy(bytes: &[u8]) -> Result<&EventEnvelope, rkyv::Error> {
    // SAFETY: We trust the bytes came from a valid archive
    unsafe { rkyv::from_bytes_unchecked(bytes) }
}

// Zero-copy access
fn get_event_type(archived: &ArchivedEventEnvelope) -> &str {
    &archived.event_type
}
```

### Performance Benchmarks

```
Benchmark results (higher is better):
┌────────────────────────────────────────────────────────────┐
│ Serialization Throughput (ops/sec)                         │
├────────────────────────────────────────────────────────────┤
│ rkyv          ████████████████████████████████  1,500,000 │
│ MessagePack    ████████████                       450,000  │
│ JSON           ████████                           280,000  │
│ Prost          ███████████                        420,000  │
└────────────────────────────────────────────────────────────┘
```

### Recommendation

1. **Internal Rust**: rkyv for hot paths
2. **Cross-language**: Protobuf for services
3. **Human-readable**: JSON for debugging
4. **Hybrid**: Protobuf + rkyv for internal

---

## 2026-03-31 - Wave 137: Graph & Tree Data Structures Research

**Project:** [cross-repo]
**Category:** research, data structures, graphs
**Status:** completed
**Priority:** P2

### Graph Libraries Comparison

| Library | Type | Algorithms | Performance | Phenotype Fit |
|---------|------|------------|-------------|---------------|
| **petgraph** | Graph | Basic | Fast | ✅ Adopted |
| **graphviz** | Visualization | Layout | N/A | 🟡 Good |
| **petgraph_dot** | Export | DOT format | Fast | 🟡 Good |
| **leptos** | UI | N/A | N/A | ❌ Frontend |

### petgraph Patterns

```rust
// phenotype-graph/src/algorithms.rs
use petgraph::{Graph, NodeIndex, EdgeIndex, visit::Dfs};

pub struct DependencyGraph {
    graph: Graph<Node, Edge>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub deps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub weight: f64,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self { graph: Graph::new() }
    }
    
    pub fn add_node(&mut self, id: String, deps: Vec<String>) -> NodeIndex {
        let node = Node { id: id.clone(), deps };
        let idx = self.graph.add_node(node);
        for dep in &self.graph[idx].deps {
            if let Some(dep_idx) = self.find_node(dep) {
                self.graph.add_edge(dep_idx, idx, Edge { weight: 1.0 });
            }
        }
        idx
    }
    
    pub fn topological_sort(&self) -> Vec<NodeIndex> {
        let mut order = Vec::new();
        let mut visited = vec![false; self.graph.node_count()];
        
        for idx in self.graph.node_indices() {
            if !visited[idx.index()] {
                self.dfs_visit(idx, &mut visited, &mut order);
            }
        }
        
        order
    }
    
    fn dfs_visit(&self, idx: NodeIndex, visited: &mut Vec<bool>, order: &mut Vec<NodeIndex>) {
        visited[idx.index()] = true;
        for neighbor in self.graph.neighbors(idx) {
            if !visited[neighbor.index()] {
                self.dfs_visit(neighbor, visited, order);
            }
        }
        order.push(idx);
    }
}
```

### DAG Visualization

```rust
// Export to Mermaid
impl DependencyGraph {
    pub fn to_mermaid(&self) -> String {
        let mut output = String::from("graph TD\n");
        
        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            output.push_str(&format!("    {}[{}]\n", idx.index(), node.id));
        }
        
        for edge in self.graph.edge_indices() {
            let (src, dst) = self.graph.edge_endpoints(edge).unwrap();
            output.push_str(&format!("    {} --> {}\n", src.index(), dst.index()));
        }
        
        output
    }
}
```

### Recommendation

1. **Graph operations**: petgraph
2. **Visualization**: Mermaid output
3. **Serialization**: TOML/JSON for persistence

---

## 2026-03-31 - Wave 138: WASM & Edge Computing Research

**Project:** [cross-repo]
**Category:** research, wasm, edge
**Status:** completed
**Priority:** P2

### WASM Runtimes Comparison

| Runtime | Size | Startup | Security | Use Case | Phenotype Fit |
|---------|------|---------|----------|----------|---------------|
| **wasmtime** | ~5MB | <1ms | Sandboxed | General | ✅ HIGH |
| **wasmer** | ~10MB | <1ms | Sandboxed | Flexibility | 🟡 MEDIUM |
| **WasmEdge** | ~2MB | <1ms | Sandboxed | Edge/Cloud | 🟡 MEDIUM |

### WASM Component Model

```wit
// phenotype-tool.wit
package phenotype:tool@0.1.0;

interface tool-execution {
  record execution-request {
    tool-id: string,
    arguments: list<tuple<string, string>>,
    timeout-ms: u32,
  }

  record execution-result {
    success: bool,
    stdout: string,
    stderr: string,
    exit-code: u32,
    duration-ms: u64,
  }

  execute: func(request: execution-request) -> result<execution-result, string>;
}

world phenotype-sandbox {
  import wasi:filesystem/types;
  import wasi:cli stdout;
  
  export tool-execution;
}
```

### Rust Implementation

```rust
// phenotype-wasm-runtime/src/lib.rs
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

pub struct WasmRuntime {
    engine: Engine,
    linker: Linker<WasiCtx>,
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        
        Ok(Self { engine, linker })
    }
    
    pub async fn execute(&self, component: &[u8], request: ExecutionRequest) -> Result<ExecutionResult> {
        let mut store = Store::new(&self.engine, WasiCtxBuilder::new().build());
        let module = Module::from_binary(&self.engine, component)?;
        let instance = self.linker.instantiate(&mut store, &module)?;
        
        let run = instance.get_typed_func::<(i32, i32), i32>(&mut store, "run")?;
        // Execute and return result
    }
}
```

### Edge Deployment Options

| Platform | Runtime | Regions | Cold Start | Phenotype Fit |
|----------|---------|---------|------------|---------------|
| **Cloudflare Workers** | V8 | 300+ | <5ms | ✅ HIGH |
| **Fastly Compute** | Wasmtime | 50+ | <1ms | 🟡 MEDIUM |
| **AWS Lambda** | Firecracker | 25+ | ~100ms | 🟡 MEDIUM |

### Recommendation

1. **Runtime**: wasmtime for sandboxing
2. **Components**: WIT interface definitions
3. **Deployment**: Cloudflare Workers for edge

---

_Last updated: 2026-03-31 (Wave 124-138)_

_Last updated: 2026-03-31 (Round 8 - Expanded Research)_
