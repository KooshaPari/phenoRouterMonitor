# Research Worklogs

**Category:** RESEARCH | **Updated:** 2026-03-29

---
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
**Status:** in_progress
**Priority:** P1

### Summary

Comprehensive 2026 analysis of external packages across Rust, npm, PyPI, and GitHub that could be forked, wrapped, or integrated into the Phenotype ecosystem.

---

### Rust Crates (crates.io) - 2026 Analysis

#### Event Sourcing & CQRS

| Crate | Version | GitHub Stars | Purpose | Recommendation |
|-------|---------|--------------|---------|----------------|
| [`eventually`](https://crates.io/crates/eventually) | 0.4.0 | ~500 | Aggregate, EventStore, Repository traits | **WRAP** - standardized DDD patterns |
| [`event-sourcing`](https://crates.io/crates/event-sourcing) | 0.1.20 | ~300 | Event store with adapters | **EVALUATE** - simpler alternative |
| [`cqrs-es`](https://crates.io/crates/cqrs-es) | 0.5.0 | ~200 | CQRS + Event Sourcing | **EVALUATE** - CQRS focus |
| [`aggregate`](https://crates.io/crates/aggregate) | 0.3.0 | ~100 | Aggregate root framework | **WRAP** - complement to eventually |

#### Policy & Access Control

| Crate | Version | GitHub Stars | Purpose | Recommendation |
|-------|---------|--------------|---------|----------------|
| [`casbin`](https://crates.io/crates/casbin) | 2.20.0 | ~2k | RBAC/ABAC policy engine | **WRAP** - cross-language support |
| [`openacl`](https://crates.io/crates/openacl) | 0.1.0 | ~50 | OpenACL implementation | **EVALUATE** - Zanzibar-like |
| [`ozauth`](https://crates.io/crates/ozauth) | 0.2.0 | ~30 | OAuth2/OIDC provider | **WRAP** - for auth flows |

#### Caching & Storage

| Crate | Version | GitHub Stars | Purpose | Recommendation |
|-------|---------|--------------|---------|----------------|
| [`moka`](https://crates.io/crates/moka) | 0.12+ | ~1k | Already using ✅ | N/A |
| [`redis-rs`](https://crates.io/crates/redis) | 0.25+ | ~2k | Redis client | **EVALUATE** - for distributed cache |
| [`rusqlite`](https://crates.io/crates/rusqlite) | 0.32+ | ~1.5k | SQLite | Already using |
| [`sqlx`](https://crates.io/crates/sqlx) | 0.8+ | ~3k | Async DB | **WRAP** - for async SQL patterns |

#### State Machines & Workflows

| Crate | Version | GitHub Stars | Purpose | Recommendation |
|-------|---------|--------------|---------|----------------|
| [`temporal-sdk`](https://crates.io/crates/temporal-sdk) | 0.1.0 | ~500 | Temporal workflow | **WRAP** - for long-running workflows |
| [`states`](https://crates.io/crates/states) | 0.7.0 | ~100 | State machine | **EVALUATE** - alternative FSM |
| [`xstate`](https://crates.io/crates/xstate) | 0.3.0 | ~200 | SCXML-based FSM | **EVALUATE** - formal FSM |

#### Configuration & Secrets

| Crate | Version | GitHub Stars | Purpose | Recommendation |
|-------|---------|--------------|---------|----------------|
| [`figment`](https://crates.io/crates/figment) | 0.10+ | ~300 | Multi-source config | **EVALUATE** - TOML/YAML/JSON/ENV |
| [`config-rs`](https://crates.io/crates/config) | 0.14+ | ~500 | Hierarchical config | **WRAP** - for config patterns |
| [`secret-service`](https://crates.io/crates/secret-service) | 3.0+ | ~50 | Secret management | **WRAP** - for credential store |

#### Observability & Telemetry

| Crate | Version | GitHub Stars | Purpose | Recommendation |
|-------|---------|--------------|---------|----------------|
| [`tracing`](https://crates.io/crates/tracing) | 0.1+ | ~2k | Already using ✅ | N/A |
| [`opentelemetry`](https://crates.io/crates/opentelemetry) | 0.22+ | ~1k | Distributed tracing | **WRAP** - for OTLP export |
| [`metrics`](https://crates.io/crates/metrics) | 0.22+ | ~200 | Metrics facade | **EVALUATE** - standardized metrics |
| [`prometheus`](https://crates.io/crates/prometheus) | 0.13+ | ~500 | Prometheus client | **WRAP** - for metrics export |

---

### npm Packages - Node.js Interoperability

#### Event Sourcing & Messaging

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `@eventually/core` | 0.5+ | ~500 | Node.js event sourcing | **WRAP** - cross-runtime ES |
| `eventemitter3` | 5.0+ | ~2k | Event emitter | **KEEP** - simple enough |
| `rxjs` | 7.8+ | ~25k | Reactive extensions | **WRAP** - for event streams |
| `ts-event sourcing` | 3.0+ | ~200 | TypeScript ES | **EVALUATE** - TS patterns |

#### Policy & Access Control

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `casbin` | 1.16+ | ~5k | RBAC/ABAC engine | **WRAP** - cross-runtime policy |
| `casbin-sequelize-adapter` | 1.0+ | ~100 | DB adapter for casbin | **WRAP** - for policy storage |
| `accesscontrol` | 2.0+ | ~500 | Role-based access | **EVALUATE** - simpler RBAC |

#### Caching & Storage

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `lru-cache` | 7.0+ | ~8k | LRU cache | Already using moka equivalent |
| `ioredis` | 5.0+ | ~10k | Redis client | **WRAP** - for distributed cache |
| `better-sqlite3` | 9.0+ | ~3k | SQLite for Node | **EVALUATE** - for local DB |

#### State Machines & Workflows

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `xstate` | 5.0+ | ~15k | State machines | **WRAP** - for frontend FSM |
| `@temporalio/client` | 1.0+ | ~2k | Temporal client | **WRAP** - for workflow orchestration |
| `statelyai/inspect` | 1.0+ | ~500 | FSM inspector | **WRAP** - for debugging |

#### Validation & Schema

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `zod` | 3.0+ | ~20k | Schema validation | **ADD** - for API input |
| `valibot` | 0.13+ | ~3k | Schema validation | **EVALUATE** - lighter than zod |
| `yup` | 1.0+ | ~15k | Object schema validation | **WRAP** - for form validation |
| `ajv` | 8.0+ | ~12k | JSON Schema validator | **WRAP** - for JSON validation |

---

### PyPI Packages - Python Interoperability

#### Event Sourcing & Messaging

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `eventsourcing` | 5.0+ | ~1k | Python ES library | **WRAP** - cross-runtime ES |
| `eventsourcing-sqlalchemy` | 5.0+ | ~200 | SQLAlchemy persistence | **WRAP** - for DB events |
| `pydantic` | 2.0+ | ~25k | Data validation | **ADD** - for Python APIs |
| `redis-py` | 5.0+ | ~15k | Redis client | **WRAP** - for distributed cache |

#### Policy & Access Control

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `casbin` | 1.0+ | ~3k | RBAC/ABAC engine | **WRAP** - cross-runtime policy |
| `permchain` | 0.1+ | ~100 | Permission chains | **EVALUATE** - alternative RBAC |

#### State Machines & Workflows

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `transitions` | 0.9+ | ~2k | State machine | **WRAP** - enhance phenotype-state-machine |
| `statelyai-python` | 1.0+ | ~500 | XState for Python | **WRAP** - for frontend FSM interop |
| `temporalio` | 1.0+ | ~1k | Temporal SDK | **WRAP** - for workflow orchestration |

#### Data Validation & Serialization

| Package | Version | GitHub Stars | Purpose | Recommendation |
|---------|---------|--------------|---------|----------------|
| `pydantic` | 2.0+ | ~25k | Data validation | **ADD** - for Python APIs |
| `msgspec` | 0.18+ | ~500 | Fast validation | **EVALUATE** - performance focus |
| `attrs` | 23.0+ | ~1k | Class validation | **EVALUATE** - simpler than pydantic |

---

### GitHub Fork Candidates (2026)

#### High-Value Forks

| Repo | Stars | Purpose | Fork Strategy |
|------|-------|---------|----------------|
| [`eventually-rs/eventually`](https://github.com/eventually-rs/eventually) | ~500 | Rust ES framework | **CONTRIBUTE** - upstream collaboration |
| [`casbin/casbin-rs`](https://github.com/casbin/casbin-rs) | ~2k | Policy engine | **WRAP** - keep as external dependency |
| [`temporalio/sdk-core`](https://github.com/temporalio/sdk-core) | ~2k | Workflow runtime | **WRAP** - for long-running workflows |
| [`tauri-apps/tauri`](https://github.com/tauri-apps/tauri) | ~105k | Desktop apps | **EVALUATE** - for desktop agent UI |
| [`LangChain-ai/langchain`](https://github.com/LangChain-ai/langchain) | ~90k | LLM orchestration | **WRAP** - for agent capabilities |

#### Specialized Libraries

| Repo | Stars | Purpose | Fork Strategy |
|------|-------|---------|----------------|
| [`BurntSushi/ripgrep`](https://github.com/BurntSushi/ripgrep) | ~45k | Search patterns | **STUDY** - for search implementation |
| [`astral-sh/ruff`](https://github.com/astral-sh/ruff) | ~35k | Python linting | **WRAP** - for linting integration |
| [`surrealdb/surrealdb`](https://github.com/surrealdb/surrealdb) | ~30k | In-memory DB | **EVALUATE** - for embedded graph DB |
| [`vectordotdev/vector`](https://github.com/vectordotdev/vector) | ~18k | Observability pipeline | **STUDY** - for telemetry design |
| [`mit-pdos/xv6-riscv`](https://github.com/mit-pdos/xv6-riscv) | ~12k | OS learning | **STUDY** - for OS concepts |

---

### Cross-Language Interop Strategy

#### Protocol-Based Wrappers

| Protocol | Implementations | Use Case | Recommendation |
|----------|-----------------|----------|----------------|
| **gRPC** | Rust, Node.js, Python | Service communication | **ADOPT** - already using tonic |
| **GraphQL** | Rust, Node.js, Python | API layer | **WRAP** - for flexible queries |
| **JSON-RPC** | Universal | Simple RPC | **ADD** - for lightweight IPC |
| **Apache Arrow** | Rust, Python | Columnar data | **EVALUATE** - for analytics |

#### Event Schema Compatibility

| Format | Rust | Node.js | Python | Recommendation |
|--------|------|---------|--------|----------------|
| JSON Schema | `jsonschema` | `ajv` | `pydantic` | **STANDARDIZE** - on JSON Schema |
| Protobuf | `prost` | `protobufjs` | `protobuf` | **ADOPT** - already using |
| MessagePack | `rmp` | `@msgpack` | `msgpack` | **EVALUATE** - for binary protocol |

---

### Recommended External Package Actions

#### Immediate (This Week)

- [ ] 🟡 HIGH: Evaluate `eventually` for standardized Aggregate/Repository traits
- [ ] 🟡 HIGH: Evaluate `casbin` for cross-language policy engine
- [ ] 🟡 HIGH: Add `zod` for Node.js API validation

#### Short-term (This Month)

- [ ] 🟡 HIGH: Create `phenotype-event-sourcing-wrapper` for `eventually` interop
- [ ] 🟡 HIGH: Create `phenotype-policy-engine-wrapper` for `casbin` interop
- [ ] 🟠 MEDIUM: Evaluate `temporal-sdk` for long-running workflows
- [ ] 🟠 MEDIUM: Evaluate `figment` for multi-source config

#### Medium-term (This Quarter)

- [ ] 🟠 MEDIUM: Add `pydantic` patterns for Python interop
- [ ] 🟠 MEDIUM: Wrap `xstate` for frontend FSM interop
- [ ] 🟢 LOW: Evaluate `tauri` for desktop agent UI
- [ ] 🟢 LOW: Evaluate `surrealdb` for embedded graph storage

---

### Related

- Duplication: `docs/worklogs/DUPLICATION.md`
- Dependencies: `docs/worklogs/DEPENDENCIES.md`
- Architecture: `docs/worklogs/ARCHITECTURE.md`

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
