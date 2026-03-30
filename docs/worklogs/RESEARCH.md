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
