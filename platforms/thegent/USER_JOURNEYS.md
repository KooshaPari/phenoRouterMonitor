# User Journeys: thegent

**Version:** 1.0
**Status:** Approved
**Date:** 2026-03-30

---

## Overview

This document describes the key user journeys for thegent, the unified agent orchestration CLI for Factory skills and droids. Journeys cover the perspectives of:

1. **Agent Operators** — day-to-day agent invocation and task execution
2. **Researchers** — model discovery, evaluation, and benchmarking
3. **Platform Engineers** — governance, policy enforcement, and compliance
4. **System Integrators** — MCP server setup, installer workflows, cross-platform sync

Each journey maps to functional requirements (FR-{DOMAIN}-NNN) and demonstrates how thegent supports the target actor's goals.

---

## UJ-001: Agent Operator Runs a Single Agent Task

**Actor:** Agent Operator
**Goal:** Execute a prompt against a specific agent with proper governance checks, output normalization, and audit trail recording
**Time:** 1-5 minutes per run

### Preconditions

- thegent CLI installed (`thegent install`)
- At least one agent binary available (claude, codex, gemini, copilot)
- Settings configured via `.env` or `THGENT_*` environment variables
- Session directory writable

### Flow

```
┌────────────┐    ┌──────────────┐    ┌────────────────┐    ┌───────────────┐
│  Operator  │───▶│  CLI (Typer) │───▶│ Input          │───▶│ Agent         │
│  types     │    │  resolves    │    │ Guardrails     │    │ Registry      │
│  command   │    │  agent/model │    │ check prompt   │    │ get_runner()  │
└────────────┘    └──────────────┘    └────────────────┘    └───────┬───────┘
                                                                    │
                         ┌──────────────────────────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐    ┌───────────────┐
                  │ Runner       │───▶│ Output         │───▶│ Contract      │
                  │ executes     │    │ Parser         │    │ Normalization │
                  │ subprocess   │    │ extract_       │    │ XML/Generic   │
                  └──────────────┘    │ condensed()    │    │ adapter       │
                                      └────────────────┘    └───────┬───────┘
                         ┌──────────────────────────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐    ┌───────────────┐
                  │ Semantic     │───▶│ Policy Engine  │───▶│ Run Registry  │
                  │ Validation   │    │ evaluate()     │    │ hash-chained  │
                  │ CSM checks   │    │ OPA / fallback │    │ JSONL audit   │
                  └──────────────┘    └────────────────┘    └───────────────┘
```

### Steps

1. **Operator invokes CLI** (FR-CLI-001)
   - `thegent run claude "Refactor auth module" --cwd ~/project --timeout 120`
   - Typer framework parses arguments, resolves working directory (FR-CLI-002)

2. **Agent and model resolution** (FR-CLI-003, FR-AGT-007, FR-MOD-002)
   - CLI resolves "claude" to canonical agent name via registry
   - Default model determined from settings (FR-CFG-002) or `--model` override
   - Model alias normalized (e.g. "haiku" to "claude-haiku-4.5")

3. **Input guardrails** (FR-GOV-003, FR-GOV-004, FR-GOV-005, FR-GOV-006)
   - Prompt length validated against `prompt_max_chars`
   - Prompt checked against blocklist patterns
   - Agent checked against allowlist (if configured)
   - CWD validated against allowed prefixes

4. **Time budget injection** (FR-CLI-004)
   - Preamble injected: timeout / 2.3s = approximate tool-call budget

5. **Runner execution** (FR-AGT-001, FR-AGT-002)
   - `AgentRunner.run()` invoked with prompt, cwd, timeout, streaming flags
   - Native CLI binary resolved via env vars or fallback paths
   - Noisy stderr filtered (FR-AGT-003)

6. **Retry on transient failure** (FR-AGT-009, FR-AGT-010)
   - Failure classified: RATE_LIMIT, TRANSIENT, USAGE_LIMIT, UNKNOWN
   - Exponential backoff retry (max 4 attempts, 2s-60s wait)

7. **Output parsing** (FR-OUT-001, FR-OUT-002, FR-OUT-003, FR-OUT-005)
   - `extract_condensed()` attempts JSONL extraction, falls back to plain text
   - Think tags removed, noise stripped
   - ParseResult returned with error classification (FR-OUT-004)

8. **Contract normalization** (FR-CTR-001, FR-CTR-002, FR-CTR-003, FR-CTR-005)
   - Output adapted to CSM via provider-specific XML adapter or generic fallback (FR-CTR-004)
   - Adapter selected from registry; fallback normalization if unregistered

9. **Semantic validation** (FR-CTR-011)
   - CSM invariants checked (COMPLETED requires progress >= 1.0, etc.)

10. **Policy evaluation** (FR-EXE-008, FR-EXE-009)
    - PolicyEngine evaluates run (OPA or Python fallback)
    - Critical lane and production trust gates enforced

11. **Audit recording** (FR-EXE-001, FR-EXE-002, FR-EXE-003)
    - RunMeta persisted to `run_registry.jsonl` with SHA-256 hash chain
    - Schema version marker maintained

12. **Telemetry** (FR-CTR-006, FR-CTR-007)
    - Normalization event written to `contract_telemetry.jsonl`
    - Drift budget checked

### Involved FRs

FR-CLI-001, FR-CLI-002, FR-CLI-003, FR-CLI-004, FR-AGT-001, FR-AGT-002, FR-AGT-003, FR-AGT-007, FR-AGT-009, FR-AGT-010, FR-MOD-002, FR-CFG-001, FR-CFG-002, FR-CFG-003, FR-GOV-003, FR-GOV-004, FR-GOV-005, FR-GOV-006, FR-OUT-001, FR-OUT-002, FR-OUT-003, FR-OUT-004, FR-OUT-005, FR-CTR-001, FR-CTR-002, FR-CTR-003, FR-CTR-004, FR-CTR-005, FR-CTR-011, FR-EXE-001, FR-EXE-002, FR-EXE-003, FR-EXE-008, FR-EXE-009, FR-CTR-006, FR-CTR-007

### Success Metrics

- Run completes within timeout with exit code 0
- CSM output has confidence >= 0.4
- Audit trail hash chain is unbroken
- No guardrail violations

---

## UJ-002: Agent Operator Executes Fallback Chain Across Providers

**Actor:** Agent Operator
**Goal:** Submit a task that automatically fails over between providers when one hits usage limits, producing normalized output regardless of which provider ultimately succeeds
**Time:** 2-10 minutes (depends on fallback depth)

### Preconditions

- Multiple agent backends configured (direct CLIs, CLIProxyAPIPlus, cursor-api)
- Proxy process available or auto-startable
- Fallback policy defined

### Flow

```
┌────────────┐    ┌───────────────┐    ┌────────────────┐
│  Operator  │───▶│  Fallback     │───▶│  Provider A    │──╳ USAGE_LIMIT
│  runs task │    │  State        │    │  (claude)      │
└────────────┘    │  Machine      │    └────────────────┘
                  │               │    ┌────────────────┐
                  │  iterates     │───▶│  Provider B    │──╳ RATE_LIMIT
                  │  providers    │    │  (codex-proxy) │     retry x4
                  │               │    └────────────────┘
                  │               │    ┌────────────────┐
                  │               │───▶│  Provider C    │──✓ SUCCESS
                  │               │    │  (gemini)      │
                  └───────┬───────┘    └────────────────┘
                          │
                          ▼
                  ┌──────────────┐    ┌────────────────┐
                  │  Normalize   │───▶│  Policy        │
                  │  via adapter │    │  evaluate      │
                  │  (XML/plain) │    │  drift budget  │
                  └──────────────┘    └────────────────┘
```

### Steps

1. **Operator submits task** (FR-CLI-001, FR-CLI-003)
   - `thegent run claude "Analyze security vulnerabilities" --cwd ~/project`

2. **Fallback chain constructed** (FR-AGT-008, FR-AGT-011)
   - `get_fallback_agents()` returns ordered provider list excluding current
   - FallbackStateMachine initialized with provider list and retry config

3. **Primary provider attempted** (FR-AGT-002, FR-AGT-009)
   - Direct runner invokes claude CLI binary
   - On USAGE_LIMIT: classified via stderr regex (FR-AGT-010), no retry

4. **Proxy provider attempted** (FR-AGT-004, FR-AGT-006)
   - CLIProxyAPIPlus lifecycle: binary resolved, config YAML generated
   - Provider blocks configured (minimax, glm, antigravity via iFlow)
   - Proxy started with health-check polling (5s timeout)
   - Codex CLI invoked with `OPENAI_BASE_URL` pointing to proxy

5. **Cursor-API backend attempted** (FR-AGT-005)
   - Reachability verified via `GET /v1/models`
   - `OPENAI_BASE_URL` and `OPENAI_API_KEY` configured from settings

6. **Output normalized per provider** (FR-CTR-003, FR-CTR-005)
   - Winning provider's adapter selected from `ADAPTER_REGISTRY`
   - XML tags mapped to CSM fields
   - Confidence computed

7. **Fallback policy evaluated** (FR-CTR-008)
   - `FallbackPolicy` checks min_confidence_threshold, max_fallback_rate
   - Strict provider enforcement applied

8. **Contract versioning checked** (FR-CTR-009, FR-CTR-010)
   - `ContractRegistry` verifies version compatibility
   - `MigrationController` checks deprecation status

9. **Drift telemetry recorded** (FR-CTR-006)
   - Structural and semantic drift events emitted
   - Drift budget status checked

### Involved FRs

FR-CLI-001, FR-CLI-003, FR-AGT-002, FR-AGT-004, FR-AGT-005, FR-AGT-006, FR-AGT-008, FR-AGT-009, FR-AGT-010, FR-AGT-011, FR-CTR-003, FR-CTR-005, FR-CTR-006, FR-CTR-008, FR-CTR-009, FR-CTR-010, FR-MOD-001

### Success Metrics

- Task completes via fallback with normalized output
- Fallback rate stays below configured threshold (30%)
- Drift budget not exceeded (5% structural, 10% semantic)
- All provider transitions recorded in telemetry

---

## UJ-003: Researcher Discovers and Routes Models

**Actor:** Researcher
**Goal:** Explore available models, understand routing policies, evaluate cost-performance trade-offs, and select optimal model configurations for workloads
**Time:** 5-15 minutes

### Preconditions

- thegent CLI installed
- At least one provider backend reachable (proxy or direct)
- Model catalog populated

### Flow

```
┌────────────┐    ┌───────────────┐    ┌────────────────┐
│ Researcher │───▶│ models list   │───▶│ Static Catalog │
│            │    │ models refresh│    │ + Dynamic      │
└────────────┘    └───────────────┘    │ Scraping       │
                                       └───────┬────────┘
                         ┌─────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Alias        │───▶│ Blacklist      │
                  │ Normalization│    │ Filter         │
                  │ haiku→4.5   │    │ reject old     │
                  └──────────────┘    └───────┬────────┘
                         ┌────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Route        │───▶│ Cost           │
                  │ Resolution   │    │ Estimation     │
                  │ by priority  │    │ $/1k tokens    │
                  └──────────────┘    └────────────────┘
```

### Steps

1. **List available models** (FR-MOD-001)
   - `thegent models list`
   - Static catalog displays canonical model IDs with routes (provider, backend_type, priority, cost_weight)

2. **Refresh dynamic models** (FR-MOD-004)
   - `thegent models refresh`
   - Scrapes cursor `--list-models` and proxy `GET /v1/models`
   - Results cached in `~/.cache/thegent/models-cache.json` (300s TTL)

3. **Classify proxy models** (FR-MOD-005)
   - Scraped models bucketed by provider substring matching
   - Unmatched models default to antigravity
   - Empty results use static fallbacks (minimax-m2.5, glm-5)

4. **Normalize aliases** (FR-MOD-002)
   - Researcher uses short names: "haiku", "sonnet", "opus"
   - System normalizes to full IDs (claude-haiku-4.5, claude-sonnet-4.5, claude-opus-4.6)
   - Route policies validated (prefer_direct, prefer_proxy, failover, round_robin, cheapest)

5. **Blacklist enforcement** (FR-MOD-003)
   - Old model versions rejected (Claude 3.x, Gemini 1.x, GPT-4)
   - Only current-generation models available for selection

6. **Cost estimation** (FR-GOV-001)
   - `CostEstimator.estimate()` computes $/run from pricing table
   - Per-model input/output token costs displayed

7. **Route contract audit** (FR-MOD-006)
   - `route_contract()` exposes schema_version, backend_types, policy_names
   - Researcher verifies compatibility with downstream consumers

8. **Select and run with explicit model** (FR-CLI-003)
   - `thegent run gemini "Summarize this codebase" --model gemini-3-flash`

### Involved FRs

FR-MOD-001, FR-MOD-002, FR-MOD-003, FR-MOD-004, FR-MOD-005, FR-MOD-006, FR-CLI-001, FR-CLI-003, FR-GOV-001

### Success Metrics

- All available models discoverable via `models list`
- Cache refresh completes under 5 seconds
- No blacklisted models appear in listings
- Cost estimates within 10% of actual billing

---

## UJ-004: Platform Engineer Configures Governance and Enforces Policies

**Actor:** Platform Engineer
**Goal:** Set up guardrails, cost controls, policy evaluation, trust boundaries, and retention policies to ensure safe agent operations in production
**Time:** 30-60 minutes initial setup; ongoing monitoring

### Preconditions

- thegent installed with access to environment variable configuration
- OPA server available (optional, for policy delegation)
- Production environment with defined trust boundaries

### Flow

```
┌────────────────┐    ┌────────────────┐    ┌────────────────┐
│ Platform Eng   │───▶│ Configure      │───▶│ Validate       │
│ sets env vars  │    │ guardrails     │    │ setup          │
│ THGENT_*       │    │ allowlists     │    │ fail-fast      │
└────────────────┘    └────────────────┘    └───────┬────────┘
                                                    │
                         ┌──────────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Policy       │───▶│ Trust          │
                  │ Engine       │    │ Boundary       │
                  │ OPA / local  │    │ Validator      │
                  └──────────────┘    └───────┬────────┘
                                              │
                         ┌────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Cost         │───▶│ Retention      │
                  │ Aggregation  │    │ Policy         │
                  │ daily by     │    │ tiered         │
                  │ owner        │    │ per-domain     │
                  └──────────────┘    └────────────────┘
```

### Steps

1. **Configure settings** (FR-CFG-001, FR-CFG-006)
   - Set `THGENT_*` environment variables or populate `.env`
   - Pydantic validates all fields on startup
   - `validate_setup()` ensures session directory exists and is writable

2. **Set up input guardrails** (FR-GOV-003, FR-GOV-004, FR-GOV-005, FR-GOV-006, FR-GOV-007)
   - `THGENT_PROMPT_MAX_CHARS=32768` — limit prompt size
   - `THGENT_PROMPT_BLOCKLIST_PATTERNS=rm -rf,DROP TABLE` — block dangerous patterns
   - `THGENT_AGENT_ALLOWLIST=claude,gemini` — restrict agent selection
   - `THGENT_CWD_ALLOWED_PREFIXES=/home/deploy/projects` — restrict working directories

3. **Configure timeouts** (FR-CFG-003)
   - `THGENT_DEFAULT_TIMEOUT=90` for standard agents
   - `THGENT_DEFAULT_TIMEOUT_CLAUDE=300` for Claude (longer context processing)

4. **Set up policy engine** (FR-EXE-008, FR-EXE-009)
   - Configure `THGENT_OPA_URL` for OPA delegation
   - Or rely on Python fallback policy logic
   - Critical lane: confidence >= 0.9 required
   - Production: trust_score_threshold >= 0.8

5. **Configure trust boundaries** (FR-EXE-010)
   - `TrustBoundaryValidator` enforces environment transitions
   - Skip-level promotions (dev to prod) require explicit audit

6. **Set up cost controls** (FR-GOV-001, FR-GOV-002)
   - `CostEstimator` uses pricing table per model
   - `CostAggregator.daily_total()` monitors daily spend by owner

7. **Configure retention** (FR-CFG-004)
   - `THGENT_RETENTION_DAYS_SESSIONS=30`
   - `THGENT_RETENTION_DAYS_REGISTRY=90`
   - `THGENT_RETENTION_BY_DOMAIN='{"gdpr":365,"soc2":2555}'`

8. **Set normalization policy** (FR-CFG-005, FR-CTR-008)
   - `THGENT_MIN_CONFIDENCE=0.4`
   - `THGENT_MAX_FALLBACK_RATE=0.3`
   - `THGENT_STRICT_PROVIDERS=claude,gemini`

9. **Review policy** (FR-CLI-001)
   - `thegent policy show` — display active policy configuration

10. **Verify audit integrity** (FR-EXE-002)
    - `thegent history verify` — validate hash chain in `run_registry.jsonl`

11. **Configure federated policies** (FR-FED-001, FR-FED-002, FR-FED-005)
    - Set up hierarchical namespace: `org.project.environment`
    - Resolve policies from most specific to most general
    - "Most restrictive wins" for conflict arbitration

12. **Map jurisdiction profiles** (FR-FED-003)
    - Map regions to legal profiles (EU-AI-ACT, US-SEC)
    - Overlay retention and human-in-loop constraints

### Involved FRs

FR-CFG-001, FR-CFG-002, FR-CFG-003, FR-CFG-004, FR-CFG-005, FR-CFG-006, FR-GOV-001, FR-GOV-002, FR-GOV-003, FR-GOV-004, FR-GOV-005, FR-GOV-006, FR-GOV-007, FR-EXE-008, FR-EXE-009, FR-EXE-010, FR-CTR-008, FR-FED-001, FR-FED-002, FR-FED-003, FR-FED-005

### Success Metrics

- Startup validation passes without errors
- Guardrails reject malformed or dangerous prompts
- OPA policy evaluation responds under 100ms
- Daily cost stays within budget thresholds
- Audit hash chain verifies clean on every check

---

## UJ-005: System Integrator Sets Up MCP Server and Installs Dotfiles

**Actor:** System Integrator
**Goal:** Deploy thegent as an MCP server for multi-platform agent access, install dotfiles/skills to Claude/Factory environments, and configure cross-platform rules sync
**Time:** 15-30 minutes

### Preconditions

- Target machine with Python 3.11+
- Agent platforms installed (Claude Code, Cursor, Codex, Factory)
- Network access for MCP HTTP transport

### Flow

```
┌────────────────┐    ┌────────────────┐    ┌────────────────┐
│ Integrator     │───▶│ thegent        │───▶│ MCP Server     │
│ runs install   │    │ install both   │    │ FastMCP start   │
└────────────────┘    └────────────────┘    │ 127.0.0.1:3847 │
                                            └───────┬────────┘
                         ┌──────────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ MCP Client   │───▶│ MCP Tools      │
                  │ Config       │    │ run, bg, ps,   │
                  │ cursor,      │    │ status, stop,  │
                  │ claude-code  │    │ dag, models... │
                  └──────────────┘    └───────┬────────┘
                                              │
                         ┌────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Rules Sync   │───▶│ HAX: Queue,    │
                  │ .thegent/    │    │ Memory,        │
                  │ rules/ →     │    │ Gardener       │
                  │ all targets  │    │                │
                  └──────────────┘    └────────────────┘
```

### Steps

1. **Install dotfiles** (FR-INS-001, FR-INS-002, FR-INS-003)
   - `thegent install both` — syncs files to `~/.claude/` and `~/.factory/`
   - Source-to-destination mappings computed for claude and factory targets
   - Smart copy: only newer files copied, cache dirs excluded
   - Symlink mode available for editable development (FR-INS-004)

2. **Start MCP server** (FR-MCP-001, FR-MCP-002)
   - `thegent mcp serve` — starts FastMCP on 127.0.0.1:3847
   - Middleware stack: error handling, logging, timing, rate limiting, caching
   - Tools registered: run, bg, ps, status, stop, wait, logs, inspect, list_agents, list_droids, list_models, dag_list, session_contracts, observe_summary, etc.

3. **Configure MCP clients** (FR-MCP-003)
   - System writes `mcpServers` entries to cursor, claude-code, codex, claude-desktop, droid config files
   - thegent registered as RemoteMCPServer

4. **Test CWD elicitation** (FR-MCP-004)
   - MCP callers that omit working directory receive elicitation prompt
   - Owner elicitation similarly triggered when missing

5. **Sync cross-platform rules** (FR-HAX-002)
   - `thegent rules sync` reads canonical rules from `.thegent/rules/`
   - Writes to `.cursor/rules/` (.mdc), `CLAUDE.md`, `.codex/skills/` (SKILL.md)

6. **Configure prompt queue** (FR-HAX-001)
   - Unified Prompt Queue initialized at `.thegent/prompt_queue.jsonl`
   - Tasks stored with timestamp, prompt, project path, status

7. **Set up model routing** (FR-HAX-003)
   - LiteLLM integration configured for Pareto-optimal routing
   - TaskRouter maps task categories (FAST, COMPLEX) to optimal routes

8. **Connect memory provider** (FR-HAX-004)
   - SupermemoryProvider configured for L3 (graph) and L4 (archival) memory
   - Replaces local file-based context stores

9. **Enable documentation gardener** (FR-HAX-005)
   - Gardener agent configured for automated doc synthesis
   - Updates CLAUDE.md, ADR.md, PRD.md, PLAN.md from audit logs

### Involved FRs

FR-INS-001, FR-INS-002, FR-INS-003, FR-INS-004, FR-MCP-001, FR-MCP-002, FR-MCP-003, FR-MCP-004, FR-HAX-001, FR-HAX-002, FR-HAX-003, FR-HAX-004, FR-HAX-005

### Success Metrics

- `thegent install` completes with zero errors, only newer files copied
- MCP server starts and responds to health check within 5 seconds
- All MCP client configs updated with thegent server entry
- `rules sync` writes to all target platforms without conflicts
- Prompt queue persists across sessions

---

## UJ-006: Agent Operator Manages DAG Workflows and Sessions

**Actor:** Agent Operator
**Goal:** Create, execute, and recover DAG-based multi-step workflows with session continuity, checkpointing, and multi-agent coordination
**Time:** 10-60 minutes depending on DAG complexity

### Preconditions

- thegent CLI installed with session directory configured
- DAG tasks defined with dependencies
- Multiple agents available for parallel execution

### Flow

```
┌────────────┐    ┌───────────────┐    ┌────────────────┐
│ Operator   │───▶│ dag add/      │───▶│ DAG State      │
│ defines    │    │ update/remove │    │ Checkpoint     │
│ workflow   │    │               │    │ Registry       │
└────────────┘    └───────────────┘    └───────┬────────┘
                                               │
                         ┌─────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ dag run      │───▶│ Multi-Agent    │
                  │ execute DAG  │    │ Execution      │
                  │ parallel/    │    │ Modes          │
                  │ sequential   │    │ (5 modes)      │
                  └──────────────┘    └───────┬────────┘
                                              │
                         ┌────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Session      │───▶│ Recovery /     │
                  │ Continuation │    │ Rollback       │
                  │ multi-hop    │    │ Reconcile      │
                  └──────────────┘    └────────────────┘
```

### Steps

1. **Define DAG tasks** (FR-CLI-001)
   - `thegent dag add "lint" --agent gemini --prompt "Run linting"`
   - `thegent dag add "test" --depends-on lint --agent claude`
   - `thegent dag update test --timeout 300`

2. **Create checkpoint** (FR-EXE-007)
   - System persists DAG state to `checkpoint_registry.jsonl`
   - Checkpoint includes dag_content, session_dir, owner, reason

3. **Execute DAG** (FR-CLI-001, FR-AGT-013)
   - `thegent dag run` — executes tasks respecting dependency order
   - Execution mode selected: SEQUENTIAL_DELEGATION, PARALLEL_CONSENSUS, REVIEW_LOOP, ARBITRATION_QUORUM, or SOLO
   - Mode metadata includes min_agents, streaming support, coordination logic

4. **Monitor execution** (FR-CLI-001, FR-EXE-004)
   - `thegent ps` — list active runs
   - `thegent status <run_id>` — show run state (RUNNING, PAUSED, COMPLETED, FAILED)
   - `thegent logs <run_id>` — stream session output

5. **Background execution** (FR-CLI-001)
   - `thegent bg claude "Long analysis" --cwd ~/project`
   - `thegent wait <run_id>` — block until completion

6. **Session continuation** (FR-CLI-005)
   - On task resumption, system builds continuation prompt
   - Tails prior stdout (8000 chars) and stderr (2000 chars)
   - Multi-hop capped at 12000 chars total

7. **Recovery and rollback** (FR-CLI-001)
   - `thegent dag recover` — resume from last checkpoint
   - `thegent dag rollback` — revert to prior checkpoint state
   - `thegent dag reconcile` — resolve inconsistencies

8. **Track run state** (FR-EXE-004, FR-EXE-005)
   - State derived chronologically from registry events
   - Idempotency token lookup for deduplication

9. **Trust score calibration** (FR-EXE-006)
   - `get_calibration_factor()` computes ratio of feedback to confidence
   - Clamped to [0.5, 2.0], defaults to 1.0

10. **Feedback recording** (FR-EXE-002)
    - `thegent feedback <run_id> --score 4 --note "Good output"`
    - Feedback persisted to hash-chained registry

### Involved FRs

FR-CLI-001, FR-CLI-005, FR-AGT-013, FR-EXE-001, FR-EXE-002, FR-EXE-004, FR-EXE-005, FR-EXE-006, FR-EXE-007, FR-OPS-001, FR-OPS-002, FR-OPS-003

### Success Metrics

- DAG executes with correct dependency ordering
- Checkpoints restore cleanly after interruption
- Session continuation preserves context within char limits
- Run states transition correctly through lifecycle

---

## UJ-007: Platform Engineer Monitors Observability and Federation Health

**Actor:** Platform Engineer
**Goal:** Monitor contract health, telemetry drift, operation taxonomy, planning analytics, and federated policy status across the agent orchestration platform
**Time:** 5-15 minutes per check; continuous for dashboards

### Preconditions

- thegent running with telemetry enabled
- Run registry populated with historical data
- Federation namespace configured (if using multi-tenant)

### Flow

```
┌────────────────┐    ┌────────────────┐    ┌────────────────┐
│ Platform Eng   │───▶│ session-       │───▶│ Telemetry      │
│ queries        │    │ contracts      │    │ Stats & KPIs   │
│ health         │    │ health-report  │    │ per-provider   │
└────────────────┘    └────────────────┘    └───────┬────────┘
                                                    │
                         ┌──────────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Drift        │───▶│ Conformance    │
                  │ Budget       │    │ Tests          │
                  │ structural   │    │ per-adapter    │
                  │ & semantic   │    │                │
                  └──────────────┘    └───────┬────────┘
                                              │
                         ┌────────────────────┘
                         ▼
                  ┌──────────────┐    ┌────────────────┐
                  │ Federation   │───▶│ Planning       │
                  │ Health &     │    │ PERT, Resource │
                  │ Consent      │    │ Contention,    │
                  │ Relay        │    │ Continuity     │
                  └──────────────┘    └────────────────┘
```

### Steps

1. **Review session contracts** (FR-CLI-001, FR-CTR-009)
   - `thegent session-contracts` — list active contract versions
   - ContractRegistry shows version, deprecation status, migration windows

2. **Check contract health** (FR-CTR-006, FR-CTR-007)
   - `thegent session-contract-health-report`
   - Telemetry stats: total events, success_rate, fallback_rate, avg_confidence
   - Per-provider breakdowns displayed

3. **Monitor drift budget** (FR-CTR-006)
   - `get_drift_budget_status()` — structural (5%) and semantic (10%) budgets
   - Alerts when drift exceeds thresholds

4. **Run conformance tests** (FR-CTR-012)
   - Test suite validates adapters: XML basic, partial XML, plain text, malformed, edge cases
   - CSMStatus correctness and confidence thresholds verified

5. **Review event schemas** (FR-CTR-013)
   - ChunkEvent, EvidenceEvent, PolicyEvent schemas available for audit trail
   - Pydantic validation on all canonical events

6. **Query operation taxonomy** (FR-OPS-001)
   - Five categories: ORCHESTRATE, GOVERN, RECOVER, OBSERVE, PLAN
   - Each CLI command mapped to operation type with constraints

7. **Evaluate orchestration mode fitness** (FR-OPS-002, FR-OPS-003)
   - `suggest_mode()` recommends mode based on risk, urgency, confidence
   - Low confidence -> PARALLEL_CONSENSUS
   - High risk, non-critical -> REVIEW_LOOP

8. **PERT analysis** (FR-PLN-001)
   - Compute expected duration and variance for workflow tasks
   - Critical path identification with p50/p90 confidence levels

9. **Resource contention simulation** (FR-PLN-002)
   - Identify resource contention windows
   - Peak demand vs capacity analysis with affected task lists

10. **Continuity risk assessment** (FR-PLN-003)
    - Risk score (0.0-1.0) based on open tasks and snapshot freshness
    - Recommendations: "Refresh snapshots before handoff"

11. **Federation health check** (FR-FED-006)
    - Namespace discovery and sync status
    - Drift detection across policy mesh

12. **Cross-namespace consent relay** (FR-FED-004)
    - Multi-tenant approval workflows with provenance signatures

13. **Verify audit trail integrity** (FR-EXE-002, FR-EXE-003)
    - `thegent history verify` validates SHA-256 hash chain
    - Schema version consistency verified

14. **Exit code diagnostics** (FR-EXIT-001)
    - `get_exit_message()` provides human-readable descriptions
    - EXIT_TIMEOUT=124, EXIT_HEALTH_GATE_FAILED=2

### Involved FRs

FR-CTR-006, FR-CTR-007, FR-CTR-009, FR-CTR-010, FR-CTR-012, FR-CTR-013, FR-OPS-001, FR-OPS-002, FR-OPS-003, FR-PLN-001, FR-PLN-002, FR-PLN-003, FR-FED-004, FR-FED-006, FR-EXE-002, FR-EXE-003, FR-EXIT-001

### Success Metrics

- Contract health report shows success_rate >= 95%
- Drift budgets within thresholds
- Conformance tests 100% passing
- PERT critical path identified with p90 confidence
- Federation namespace fully synced with zero drift

---

## FR-to-UJ Traceability Matrix

This matrix ensures 100% coverage: every FR maps to at least one user journey.

| FR ID | Description | UJ Coverage |
|-------|-------------|-------------|
| **FR-AGT: Agents** | | |
| FR-AGT-001 | Base Runner Interface | UJ-001 |
| FR-AGT-002 | Direct Agent Invocation via Native CLIs | UJ-001, UJ-002 |
| FR-AGT-003 | Noisy Stderr Filtering | UJ-001 |
| FR-AGT-004 | Codex Proxy Runner via CLIProxyAPIPlus | UJ-002 |
| FR-AGT-005 | Cursor API Runner | UJ-002 |
| FR-AGT-006 | CLIProxyAPIPlus Lifecycle Management | UJ-002 |
| FR-AGT-007 | Agent Registry and Name Resolution | UJ-001 |
| FR-AGT-008 | Provider Fallback Chain | UJ-002 |
| FR-AGT-009 | Retry with Exponential Backoff | UJ-001, UJ-002 |
| FR-AGT-010 | Failure Classification | UJ-001, UJ-002 |
| FR-AGT-011 | Fallback State Machine Orchestration | UJ-002 |
| FR-AGT-012 | Droid Runner for Factory Droids | UJ-005 |
| FR-AGT-013 | Multi-Agent Execution Modes | UJ-006 |
| **FR-CTR: Contracts** | | |
| FR-CTR-001 | Canonical Structured Message Schema | UJ-001 |
| FR-CTR-002 | Incremental XML Parser | UJ-001 |
| FR-CTR-003 | XML Output Adapter Normalization | UJ-001, UJ-002 |
| FR-CTR-004 | Generic Output Adapter (Plain Text) | UJ-001 |
| FR-CTR-005 | Provider Adapter Registry | UJ-001, UJ-002 |
| FR-CTR-006 | Contract Telemetry and Drift Detection | UJ-001, UJ-002, UJ-007 |
| FR-CTR-007 | Telemetry Statistics and KPI Aggregation | UJ-001, UJ-007 |
| FR-CTR-008 | Normalization Fallback Policy Evaluation | UJ-002, UJ-004 |
| FR-CTR-009 | Contract Version Registry | UJ-002, UJ-007 |
| FR-CTR-010 | Contract Migration Controller | UJ-002, UJ-007 |
| FR-CTR-011 | Semantic Validation of CSM Invariants | UJ-001 |
| FR-CTR-012 | Conformance Test Suite | UJ-007 |
| FR-CTR-013 | Canonical Event Schemas | UJ-007 |
| **FR-GOV: Governance** | | |
| FR-GOV-001 | Cost Estimation per Run | UJ-003, UJ-004 |
| FR-GOV-002 | Daily Cost Aggregation by Owner | UJ-004 |
| FR-GOV-003 | Input Guardrail - Prompt Length | UJ-001, UJ-004 |
| FR-GOV-004 | Input Guardrail - Blocklist Patterns | UJ-001, UJ-004 |
| FR-GOV-005 | Input Guardrail - Agent/Model Allowlists | UJ-001, UJ-004 |
| FR-GOV-006 | Input Guardrail - CWD Restriction | UJ-001, UJ-004 |
| FR-GOV-007 | Guardrails from Environment Variables | UJ-004 |
| **FR-EXE: Execution** | | |
| FR-EXE-001 | Run Metadata Model | UJ-001, UJ-006 |
| FR-EXE-002 | Run Registry with Hash-Chained Audit | UJ-001, UJ-006, UJ-007 |
| FR-EXE-003 | Run Registry Schema Versioning | UJ-001, UJ-007 |
| FR-EXE-004 | Run State Tracking | UJ-006 |
| FR-EXE-005 | Idempotency Token Lookup | UJ-006 |
| FR-EXE-006 | Trust Score Calibration | UJ-006 |
| FR-EXE-007 | Checkpoint Registry for DAG State | UJ-006 |
| FR-EXE-008 | PolicyEngine Evaluation with OPA | UJ-001, UJ-004 |
| FR-EXE-009 | Critical Lane and Production Trust Gates | UJ-001, UJ-004 |
| FR-EXE-010 | Trust Boundary Validation | UJ-004 |
| **FR-MOD: Models** | | |
| FR-MOD-001 | Static Model Catalog with Route Resolution | UJ-002, UJ-003 |
| FR-MOD-002 | Model Alias Normalization | UJ-001, UJ-003 |
| FR-MOD-003 | Model Blacklist Enforcement | UJ-003 |
| FR-MOD-004 | Dynamic Model Scraping with Cache | UJ-003 |
| FR-MOD-005 | Proxy Model Classification | UJ-003 |
| FR-MOD-006 | Route Contract Metadata | UJ-003 |
| **FR-PLN: Planning** | | |
| FR-PLN-001 | PERT Forward Pass Analysis | UJ-007 |
| FR-PLN-002 | Resource Contention Simulation | UJ-007 |
| FR-PLN-003 | Continuity Risk Scoring | UJ-007 |
| **FR-CLI: CLI** | | |
| FR-CLI-001 | CLI Command Framework via Typer | UJ-001, UJ-002, UJ-003, UJ-006, UJ-007 |
| FR-CLI-002 | Working Directory Resolution | UJ-001 |
| FR-CLI-003 | Agent and Model Resolution | UJ-001, UJ-002, UJ-003 |
| FR-CLI-004 | Time Constraint Budget Injection | UJ-001 |
| FR-CLI-005 | Session Continuation with Multi-Hop | UJ-006 |
| **FR-MCP: MCP Server** | | |
| FR-MCP-001 | FastMCP Server with Tool Registration | UJ-005 |
| FR-MCP-002 | MCP Server Middleware Stack | UJ-005 |
| FR-MCP-003 | MCP Client Configuration Management | UJ-005 |
| FR-MCP-004 | MCP Server CWD and Owner Elicitation | UJ-005 |
| **FR-CFG: Configuration** | | |
| FR-CFG-001 | Pydantic Settings with Env Var Binding | UJ-001, UJ-004 |
| FR-CFG-002 | Agent-Specific Default Model Config | UJ-001, UJ-004 |
| FR-CFG-003 | Timeout Configuration | UJ-001, UJ-004 |
| FR-CFG-004 | Retention Policy Configuration | UJ-004 |
| FR-CFG-005 | Normalization and Contract Policy Settings | UJ-004 |
| FR-CFG-006 | Startup Configuration Validation | UJ-004 |
| **FR-OPS: Operations** | | |
| FR-OPS-001 | Operation Taxonomy Mapping | UJ-006, UJ-007 |
| FR-OPS-002 | Multi-Agent Orchestration Mode Catalog | UJ-006, UJ-007 |
| FR-OPS-003 | Mode Selection Policy | UJ-006, UJ-007 |
| **FR-INS: Install** | | |
| FR-INS-001 | Source-to-Destination Mapping | UJ-005 |
| FR-INS-002 | Smart Copy with Modification Time | UJ-005 |
| FR-INS-003 | Exclusion of Cache and Transient Dirs | UJ-005 |
| FR-INS-004 | Symlink Mode for Editable Installs | UJ-005 |
| **FR-OUT: Output Parser** | | |
| FR-OUT-001 | JSONL Stream Extraction | UJ-001 |
| FR-OUT-002 | Plain Text Noise Stripping | UJ-001 |
| FR-OUT-003 | Think Tag Removal | UJ-001 |
| FR-OUT-004 | ParseResult with Error Classification | UJ-001 |
| FR-OUT-005 | Condensed Output Extraction | UJ-001 |
| **FR-FED: Policy Federation** | | |
| FR-FED-001 | Hierarchical Policy Namespace | UJ-004 |
| FR-FED-002 | Federated Policy Resolution | UJ-004 |
| FR-FED-003 | Jurisdiction Profile Mapping | UJ-004 |
| FR-FED-004 | Cross-Namespace Consent Relay | UJ-007 |
| FR-FED-005 | Policy Conflict Arbitration | UJ-004 |
| FR-FED-006 | Federation Health and Drift Observability | UJ-007 |
| **FR-EXIT: Exit Codes** | | |
| FR-EXIT-001 | Standardized Exit Codes | UJ-007 |
| **FR-HAX: Harmonious Agent Experience** | | |
| FR-HAX-001 | Unified Prompt Queue | UJ-005 |
| FR-HAX-002 | Cross-Platform Rules Sync | UJ-005 |
| FR-HAX-003 | Pareto-Optimal Model Routing (LiteLLM) | UJ-005 |
| FR-HAX-004 | Universal Memory Provider (Supermemory) | UJ-005 |
| FR-HAX-005 | Automated Documentation Gardening | UJ-005 |

---

## Coverage Summary

| Domain | FR Count | UJ Coverage | Coverage % |
|--------|----------|-------------|------------|
| FR-AGT | 13 | UJ-001, UJ-002, UJ-005, UJ-006 | 100% |
| FR-CTR | 13 | UJ-001, UJ-002, UJ-004, UJ-007 | 100% |
| FR-GOV | 7 | UJ-001, UJ-003, UJ-004 | 100% |
| FR-EXE | 10 | UJ-001, UJ-004, UJ-006, UJ-007 | 100% |
| FR-MOD | 6 | UJ-001, UJ-002, UJ-003 | 100% |
| FR-PLN | 3 | UJ-007 | 100% |
| FR-CLI | 5 | UJ-001, UJ-002, UJ-003, UJ-006, UJ-007 | 100% |
| FR-MCP | 4 | UJ-005 | 100% |
| FR-CFG | 6 | UJ-001, UJ-004 | 100% |
| FR-OPS | 3 | UJ-006, UJ-007 | 100% |
| FR-INS | 4 | UJ-005 | 100% |
| FR-OUT | 5 | UJ-001 | 100% |
| FR-FED | 6 | UJ-004, UJ-007 | 100% |
| FR-EXIT | 1 | UJ-007 | 100% |
| FR-HAX | 5 | UJ-005 | 100% |
| **Total** | **91** | **UJ-001 through UJ-007** | **100%** |

---

## Design Principles

1. **Fail-Fast Governance** — Guardrails reject bad input before execution begins; no silent degradation
2. **Provider-Agnostic Normalization** — All agent output normalized to CSM regardless of source provider
3. **Hash-Chained Auditability** — Every run event is SHA-256 chained for tamper-evident audit trails
4. **Automatic Fallback** — Usage limits trigger provider failover without operator intervention
5. **Cross-Platform Parity** — MCP server exposes identical capabilities to all agent platforms
6. **Federated Policy** — Multi-tenant governance with namespace isolation and jurisdiction awareness

---

*Cross-ref: [FUNCTIONAL_REQUIREMENTS.md](./FUNCTIONAL_REQUIREMENTS.md) | [PRD.md](./PRD.md) | [PLAN.md](./PLAN.md) | [ADR.md](./ADR.md)*
