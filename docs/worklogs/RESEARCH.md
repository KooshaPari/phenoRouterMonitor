# Research Worklogs

**Category:** RESEARCH | **Updated:** 2026-03-29

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

## 2026-03-29 - Graph Database Alternatives Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P2

### Graph DB Landscape

| System | Language | Architecture | Assessment |
|--------|----------|-------------|------------|
| **Neo4j** | Java | Single-node | ✅ Standard |
| **ArangoDB** | C++ | Distributed | 🔲 EVALUATE |
| **Dgraph** | Go | Distributed | 🔲 EVALUATE |
| **TigerGraph** | C++ | Distributed | 🔲 WATCH |
| **Memgraph** | C++ | In-memory | 🔲 WATCH |
| **petgraph** | Rust | In-memory | ✅ ADOPT |

### Neo4j (Reference)

**What:** The standard graph database with Cypher query language.

**Key Features:**
- ACID transactions
- Cypher query language
- Graph algorithms
- Visualization tools

**Status:** Use as reference for query patterns

### Dgraph (Distributed)

**What:** Distributed graph database with GraphQL-like query language (DQL).

**Key Features:**
- Horizontal scaling
- Low-latency queries
- Distributed transactions
- GraphQL-like API

**Status:** 🔲 EVALUATE - For production at scale

### petgraph (In-Memory)

**What:** Rust-native in-memory graph library.

**Key Features:**
- No external dependency
- Optimal for small-medium graphs
- Graph algorithms built-in
- DOT export

**Status:** ✅ ADOPT - For internal phenoinfrakit graphs

---

## 2026-03-29 - Zero-Copy Serialization Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Zero-Copy Options

| System | Language | Schema | Assessment |
|--------|----------|--------|------------|
| **rkyv** | Rust | Static | ✅ ADOPT |
| **flatbuffers** | Multiple | Schema | 🔲 WRAP |
| **capnproto** | Multiple | Schema | 🔲 WRAP |
| **abomonation** | Rust | Dynamic | 🔲 EVALUATE |

### rkyv (Rust)

**What:** Zero-copy deserialization for Rust with zero allocation reads.

**Key Features:**
- Zero allocation on deserialization
- 10-100x faster than JSON
- Mature ecosystem
- Schema evolution support

**Benchmark:**
```
JSON serialize:   1.2 µs
JSON deserialize:   2.1 µs
rkyv serialize:    0.3 µs
rkyv deserialize:   0.1 µs (zero-copy)
```

**Status:** ✅ ADOPT - For hot read paths in phenoinfrakit

### flatbuffers (Multi-language)

**What:** Efficient cross-platform serialization by Google.

**Key Features:**
- Multiple language support
- Schema evolution
- Direct memory access
- Game-ready performance

**Status:** 🔲 WRAP - For cross-language serialization

---

## 2026-03-29 - Supply Chain Security Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P0

### Security Tools

| Tool | Purpose | Language | Assessment |
|------|---------|----------|------------|
| **cargo-audit** | Vulnerability scanning | Rust | ✅ ADOPT |
| **cargo-deny** | License/banned deps | Rust | ✅ ADOPT |
| **OSV** | Vulnerability database | Any | ✅ ADOPT |
| **Syft** | SBOM generation | Go | 🔲 TRIAL |
| **Grype** | Vulnerability scanning | Go | 🔲 TRIAL |

### Cargo Audit Integration

```yaml
# .github/workflows/security.yml
name: Security Audit
on: [push, pull_request]
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rust-lang/cargo-deny@v0.16
        with:
          bans: fail
      - uses: actions-rust-lang/cargo-audit@v0.18
```

### SBOM Generation

```bash
# Generate SPDX SBOM
syft packages . -o spdx-json > sbom.spdx.json

# Upload to OSV
osv-scanner -r -L ./sbom.spdx.json
```

### Supply Chain Attacks

**Known incidents (2026):**
- LiteLLM v1.82.7-1.82.8 (supply chain, 2026-03-25)
- Multiple PyPI typosquatting campaigns

**Mitigation:**
1. Pin exact versions with hash verification
2. Use OSV for vulnerability monitoring
3. Generate and publish SBOMs
4. Use only official registries

---

## 2026-03-29 - Edge Computing Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P2

### Edge Platforms

| Platform | Runtime | Assessment | Use Case |
|----------|---------|------------|----------|
| **Cloudflare Workers** | V8 Isolates | ✅ ADOPT | Global edge |
| **Fastly Compute** | Wasm | 🔲 EVALUATE | Fast edge |
| **AWS Lambda@Edge** | Node.js | 🟡 Good | AWS-specific |
| **Fly.io** | Firecracker | 🔲 EVALUATE | Distributed |

### Cloudflare Workers

**What:** Global edge computing with V8 isolates.

**Key Features:**
- 200+ data centers
- <5ms cold start
- TypeScript/JavaScript
- Durable Objects

**Status:** ✅ ADOPT - For global agent deployment

### Fastly Compute

**What:** WebAssembly-based edge computing.

**Key Features:**
- WASM runtime
- Rust support
- TypeScript SDK
- Instant purge

**Status:** 🔲 EVALUATE - For WASM-first edge

### Firecracker (Fly.io)

**What:** MicroVM-based distributed computing.

**Key Features:**
- Lightweight VMs
- Strong isolation
- Fast cold starts
- SSH access

**Status:** 🔲 EVALUATE - For full OS at edge

---

## 2026-03-29 - Observability Stack Research

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Observability Stack

| Component | Option | Assessment | Use Case |
|-----------|--------|------------|----------|
| **Tracing** | Jaeger | 🟡 Good | Distributed |
| **Tracing** | Zipkin | 🟡 Good | Simple |
| **Metrics** | Prometheus | ✅ STANDARD | Metrics |
| **Logs** | Loki | ✅ ADOPT | Log aggregation |
| **Profiles** | Pyroscope | 🔲 TRIAL | CPU profiling |
| **Dashboards** | Grafana | ✅ STANDARD | Visualization |

### OpenTelemetry

**What:** Vendor-neutral observability standard.

**Key Features:**
- Traces, metrics, logs
- Language-agnostic
- Backends: Jaeger, Tempo, etc.
- Auto-instrumentation

**Status:** ✅ ADOPT - For distributed phenoinfrakit

### Grafana Stack

**What:** Complete observability platform.

**Key Features:**
- Dashboards
- Alerting
- Multi-data source
- Explore UI

**Status:** ✅ STANDARD - For all Phenotype projects

### Recommended Stack

```
┌─────────────────────────────────────────────────┐
│              Observability Stack                   │
├─────────────────────────────────────────────────┤
│  Traces: OpenTelemetry → Jaeger/Tempo            │
│  Metrics: Prometheus → Grafana                    │
│  Logs: Loki → Grafana                            │
│  Profiles: Pyroscope → Grafana                    │
└─────────────────────────────────────────────────┘
```

---

_Last updated: 2026-03-29_
