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
---

## 2026-03-29 - Extended 2026 Crate Ecosystem Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Web research on emerging 2026 crates that could benefit the Phenotype ecosystem. Covers AI/LLM, observability, performance, and developer tooling.

---

### AI/LLM Integration (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `anthropic` | 0.3.0 | Claude SDK (official) | **ADOPT** - First-class async |
| `anthropic-sdk-core` | 0.3.0 | Core types | **ADOPT** - Streaming, tools |
| `llm-chain` | 0.5.0 | Multi-provider LLM | **EVALUATE** - Tool use, chains |
| `tiktoken` | 0.5.0 | BPE tokenization | **EVALUATE** - Cost tracking |
| `tokenizers` | 0.20.0 | HuggingFace tokenizer | **EVALUATE** - Full tokenizer |
| `transformers` | 0.3.0 | HuggingFace models | **WATCH** - Rust ML |

### Agent Frameworks (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `agent-P` | 0.2.0 | Agent primitives | **EVALUATE** - MCP integration |
| `open-agent` | 0.1.0 | OpenAI agents | **EVALUATE** - Tool calling |
| `mcp-sdk` | 0.1.0 | Model Context Protocol | **EVALUATE** - Standard tool protocol |
| `smol-ai` | 0.2.0 | Agent framework | **WATCH** - Emerging |

### Observability & Tracing (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `ratatui` | 0.28.0 | Terminal UI | **ADOPT** - TUI dashboards |
| `tokio-console` | 0.2.0 | Async debugging | **ADOPT** - Debugging |
| `tracing-flame` | 0.2.0 | Flame graphs | **EVALUATE** - Performance |
| `tracing-tracy` | 0.2.0 | Tracy profiler | **EVALUATE** - GPU profiling |
| `perf-monitor` | 0.1.0 | Runtime metrics | **EVALUATE** - Simple monitoring |

### Performance & Optimization (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `cargo-flamegraph` | 0.6.0 | Profiling | **ADOPT** - Already using |
| `cargo-nextest` | 0.9.0 | Test runner | **ADOPT** - Parallel tests |
| `cargo-hack` | 0.5.0 | Feature flags | **EVALUATE** - CI |
| `sccache` | 0.8.0 | Shared cache | **EVALUATE** - CI caching |
| `mold` | 1.0.0 | linker | **EVALUATE** - Faster builds |

### Async & Concurrency (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `parking_lot` | 0.12.0 | Synchronization | **EVALUATE** - Faster than Mutex |
| `dashmap` | 5.5.0 | Concurrent map | **EVALUATE** - Read-heavy |
| `flume` | 0.11.0 | Channels | **EVALUATE** - Higher throughput |
| `atomic-pool` | 0.2.0 | Object pooling | **EVALUATE** - Reduce allocations |
| `pretrace` | 0.1.0 | Tracing allocator | **WATCH** - Memory profiling |

### Database & Storage (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `sqlx` | 0.8.0 | Async SQL | **EVALUATE** - Migration from rusqlite |
| `sea-orm` | 1.0.0 | Async ORM | **EVALUATE** - Complex queries |
| `sled` | 0.34.0 | Embedded KV | **EVALUATE** - Local caching |
| `rocksdb` | 0.22.0 | RocksDB bindings | **EVALUATE** - Performance |
| `parquet` | 50.0.0 | Columnar storage | **EVALUATE** - Analytics |
| `arrow` | 45.0.0 | Apache Arrow | **EVALUATE** - Data frames |

### Serialization (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `rkyv` | 0.8.0 | Zero-copy | **EVALUATE** - Performance |
| `postcard` | 1.0.0 | No-std | **EVALUATE** - Embedded |
| `speedy` | 0.13.0 | Fast | **EVALUATE** - Cross-language |
| `abstreet` | 0.1.0 | MessagePack | **WATCH** - Alternative |
| `capnp` | 0.20.0 | Cap'n Proto | **EVALUATE** - RPC |

### CLI & Developer Tools (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `clap_complete` | 5.0.0 | Shell completions | **ADOPT** - CLI UX |
| `dialoguer` | 0.11.0 | Interactive prompts | **ADOPT** - CLI interactivity |
| `console` | 0.16.0 | Terminal styling | **ADOPT** - Colors, etc. |
| `colored` | 2.0.0 | Terminal colors | **EVALUATE** - Alternative |
| `indicatif` | 0.18.0 | Progress bars | **ADOPT** - Progress |
| `anyhow` | 1.0.0 | Error handling | ✅ Already using |

---

## 2026-03-29 - Fork Candidates Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0

### Summary

Deep analysis of fork candidates from GitHub repositories that could benefit Phenotype.

---

### 1. phenotype-process (FROM utils/pty)

**Source:** `platforms/heliosCLI/codex-rs/utils/pty/`

**Contents:**
```
utils/pty/
├── src/
│   ├── lib.rs          (exports, 200 LOC)
│   ├── pipe.rs         (non-interactive, 150 LOC)
│   ├── pty.rs          (PTY spawning, 300 LOC)
│   ├── process.rs      (ProcessHandle, 200 LOC)
│   └── process_group.rs (group management, 150 LOC)
└── tests/
    └── integration.rs   (1000+ LOC tests)
```

**Why Fork:**
- Cross-platform PTY (Unix + ConPTY)
- Process group semantics (kill all children)
- Built-in output streaming
- Well-tested (~1000 LOC of tests)

**Estimated Savings:** ~1,400 LOC across repos

---

### 2. phenotype-error (FROM CodexErr pattern)

**Source:** `platforms/heliosCLI/codex-rs/core/src/error.rs` (~1,148 LOC)

**Key Patterns:**
```rust
pub enum CodexErr {
    TurnAborted,
    ContextWindowExceeded,
    ThreadNotFound(ThreadId),
    Stream(String, Option<Duration>),  // retryable
    Io(#[from] io::Error),
    Json(#[from] serde_json::Error),
    // ...
}

impl CodexErr {
    pub fn is_retryable(&self) -> bool { ... }
    pub fn to_codex_protocol_error(&self) -> CodexErrorInfo { ... }
}
```

**Why Fork:**
- Single enum with From impls
- Retryable trait for automatic retry
- Protocol-aware error translation
- Comprehensive test coverage

**Estimated Savings:** ~400 LOC (75% reduction)

---

### 3. phenotype-git (FROM utils/git)

**Source:** `platforms/heliosCLI/codex-rs/utils/git/`

**Contents:**
```
utils/git/
├── src/
│   ├── apply.rs        (cherry-pick, patches)
│   ├── branch.rs       (branch CRUD)
│   ├── ghost_commits.rs (orphaned commits)
│   ├── operations.rs   (clone, fetch, push)
│   └── lib.rs
```

**Why Fork:**
- Git operations already implemented
- Pattern-based rather than full-featured
- Could be enhanced with worktree support

**Estimated Savings:** ~300 LOC

---

### 4. phenotype-executor (FROM SpawnContext)

**Source:** `vibe-kanban/backend/src/executor.rs:72-151`

**Pattern:**
```rust
pub struct SpawnContext {
    pub executor_type: ExecutorType,
    pub task_id: Option<TaskId>,
    pub working_dir: Option<PathBuf>,
    pub env_vars: HashMap<String, String>,
}

impl From<&tokio::process::Command> for SpawnContext { ... }
```

**Why Fork:**
- Rich context for process spawning
- Builder pattern for configuration
- Error context (executor type, task ID, working dir)

**Estimated Savings:** ~150 LOC

---

## 2026-03-29 - Inactive Folders Audit Summary

**Project:** [cross-repo]
**Category:** research
**Status:** in_progress
**Priority:** P1

### Worktrees to Verify

| Worktree | Status | Action |
|----------|--------|--------|
| `ccusage-wtrees/` | Unknown | CHECK git state |
| `zen-wtrees/` | Unknown | CHECK git state |
| `fix-dead-code/` | Experimental | EVALUATE + ARCHIVE |

### Cleanup Protocol

1. **Verify on main**: `git checkout main && git pull`
2. **Check stashes**: `git stash list`
3. **Extract stashes**: `git stash pop` if valuable
4. **Delete**: `git worktree remove <path>`

### Non-Worktree Directories

| Directory | Purpose | Action |
|----------|---------|--------|
| `worktree/` | Legacy overlay | CONFIRM status |
| `platforms/thegent` | Project ref | CONFIRM status |
| `docs/node_modules` | Generated | OK (gitignored) |
| `.worktrees/*` | Local clones | CLEANUP if stale |

---

_Last updated: 2026-03-29_
**Project:** [phenotype-infrakit]
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

---

## 2026-03-29 - Error Propagation Patterns Research

**Project:** [cross-repo]
**Category:** research
**Status:** in_progress
**Priority:** P1

### Summary
Research into error handling patterns for distributed systems across Rust, TypeScript, and Python services.

### Error Propagation Patterns

| Pattern | Use Case | Assessment |
|---------|----------|------------|
| **Result<T, E>** | Synchronous Rust | ✅ Standard |
| **Try/Catch** | TypeScript/Python | ✅ Standard |
| **Error Channels** | Async boundaries | ✅ ADOPT |
| **Circuit Breakers** | Service resilience | 🟡 EVALUATE |

### Cross-Language Error Mapping

```rust
// Rust Error
pub enum ServiceError {
    NotFound(String),
    Validation(String),
    Internal(anyhow::Error),
}

// Convert to JSON-RPC error
impl From<ServiceError> for jsonrpc::Error {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(msg) => jsonrpc::Error::not_found(Some(msg)),
            ServiceError::Validation(msg) => jsonrpc::Error::invalid_params(msg),
            ServiceError::Internal(_) => jsonrpc::Error::internal_error(),
        }
    }
}
```

### Tasks

- [ ] ERROR-001: Standardize error code ranges across services
- [ ] ERROR-002: Add error code documentation

_Last updated: 2026-03-29_

---

## 2026-03-29 - LLM Infrastructure Research

**Project:** [cross-repo]
**Category:** research
**Status:** in_progress
**Priority:** P1

### LLM Provider Comparison

| Provider | Latency | Cost | Complexity | Assessment |
|---------|---------|------|-------------|------------|
| **OpenAI** | Low | $$$ | Low | ✅ Use for prod |
| **Anthropic** | Low | $$$ | Low | ✅ Use for prod |
| **Ollama** | Medium | $ | Low | ✅ EVALUATE for local |
| **vLLM** | Low | $ | High | 🔲 EVALUATE for scale |
| **SGLang** | Low | $ | High | 🔲 EVALUATE for scale |

### Ollama for Local Development

**What:** Local LLM inference with simple API.

**Key Features:**
- Model management
- OpenAI-compatible API
- GPU support
- Docker support

**Status:** ✅ ADOPT - For development and testing

**Opportunity:** Use Ollama in CI/CD for agent testing

### vLLM/SGLang for Scale

**What:** High-throughput LLM serving with PagedAttention.

**Key Features:**
- Continuous batching
- PagedAttention
- Tensor parallelism
- Streaming

**Status:** 🔲 EVALUATE - For production at scale

**Opportunity:** Evaluate for high-volume agent workloads

---

_Last updated: 2026-03-30_
