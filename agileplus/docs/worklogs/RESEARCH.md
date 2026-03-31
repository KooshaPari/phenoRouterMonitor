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
