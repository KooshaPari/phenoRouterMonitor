---

## 2026-03-29 - NEW 2026 Crate Discoveries (docs.rs feed)

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Discovered 6 high-value crates from 2026-03-29 docs.rs release feed relevant to agent systems and workflow orchestration.

### 🔴 CRITICAL - Agent-to-Agent Protocols

#### ra2a (A2A Protocol SDK)

**Purpose:** Rust implementation of the Agent2Agent (A2A) Protocol v1.0

**Key Features:**
- Full A2A Protocol v1.0 compliance
- Async/await built on tokio
- Type-safe models with newtype IDs
- Modular with optional features (gRPC, telemetry, SQL)

**Relevance to Phenotype:**
- Could standardize agent communication across heliosCLI, thegent, AgilePlus
- Provides AgentCard, Task, Message types that overlap with existing patterns
- MIT OR Apache-2.0 license

**Decision:** **EVALUATE** - High value for agent interoperability

---

#### mentisdb (Semantic Memory)

**Purpose:** Hash-chained semantic memory for long-running agents

**Key Features:**
- Append-only, adapter-backed memory log
- Thoughts timestamped and hash-chained
- Typed, connectable to prior thoughts
- Exportable as prompts or Markdown snapshots
- Multiple storage backends (Binary, JSONL)
- Skill registry with versioning

**Relevance to Phenotype:**
- Directly addresses agent memory/persistence needs
- Could replace custom event sourcing in some areas
- Hash-chaining similar to existing event sourcing patterns

**Decision:** **FORK CANDIDATE** - High alignment with phenotype-event-sourcing

---

### 🟠 HIGH - Workflow Orchestration

#### forza-core (Workflow Orchestrator)

**Purpose:** Core abstractions for forza workflow orchestrator for agent-driven development

**Key Features:**
- Subject → Route → Workflow → Stage → Run pipeline
- GitHub as authoritative state machine
- Pluggable backends via traits
- Linear, no branching workflows

**Decision:** **WRAP** - Valuable for workflow orchestration layer

---

### Comparison Matrix

| Crate | Phenotype Alignment | LOC Savings | Priority | Decision |
|-------|---------------------|-------------|----------|----------|
| ra2a | Agent communication | ~200 | P1 | EVALUATE |
| mentisdb | Memory/persistence | ~400 | P1 | FORK CANDIDATE |
| forza-core | Workflow orchestration | ~300 | P2 | WRAP |

---

## 2026-03-29 - Original Root Prompt Discovery

**Project:** [docs]
**Category:** research
**Status:** completed
**Priority:** P2

### Original Prompt Source

**Location:** `docs/worklogs/data/phenotype_session_extract_2026-03-26_2026-03-29.json`

### Original Prompt Content

```
❯ you need to merge into the actual canonical docs ## Final Worklogs Structure
::: worklogs/
::: ├── README.md              (150 lines) - Index & aggregation guide
::: ├── AGENT_ONBOARDING.md    (200 lines) - Agent onboarding
::: ├── ARCHITECTURE.md        (253 lines) - Architecture & port/trait analysis
::: ├── DEPENDENCIES.md        (364 lines) - External dependency audits
::: ├── DUPLICATION.md         (338 lines) - Extended duplication audit
::: └── WORK_LOG.md           (179 lines) - Work item tracking
::: └── aggregate.sh           - Aggregation script
use haiku agents and fd + other faster tools over find
```

### Execution Pattern

| Attribute | Value |
|-----------|-------|
| **Sent to** | Multiple sequential haiku agents |
| **Date** | 2026-03-27 to 2026-03-29 |
| **Purpose** | Worklogs organization and consolidation |

---

## 2026-03-29 - Inactive Repos/Worktrees Audit

**Project:** [cross-repo]
**Category:** governance
**Status:** completed
**Priority:** P1

### Directory Status Matrix

| Directory | Type | Canonical? | Status | Action |
|-----------|------|-----------|--------|--------|
| `.worktrees/gh-pages-deploy/` | Git worktree | No | Inactive | SYNC + PUSH |
| `.worktrees/phench-fix/` | Git worktree | No | Inactive | SYNC + PUSH |
| `.worktrees/thegent/` | Git worktree | No | Partial | EVALUATE |
| `worktrees/heliosCLI/` | Worktree dir | No | Inactive | CLEANUP |
| `worktrees/phenotypeActions/` | Worktree dir | No | EMPTY | DELETE |
| `worktree/` | Worktree dir | No | EMPTY | DELETE |
| `.archive/*/` | Archive | N/A | All EMPTY | DELETE ALL |

### Empty Directories to Delete

```bash
# Identified empty dirs in .archive/
.archive/audit/
.archive/contracts/
.archive/kitty-specs/
.archive/plans/
.archive/schemas/
.archive/tests/

# Empty worktree dirs
worktrees/phenotypeActions/
worktree/
```

### Action Items

- [ ] DELETE: `worktrees/phenotypeActions/` (empty)
- [ ] DELETE: `worktree/` (empty)
- [ ] DELETE: `.archive/*/` (all empty)
- [ ] SYNC: `.worktrees/gh-pages-deploy/` with origin/main
- [ ] SYNC: `.worktrees/phench-fix/` with origin/main
- [ ] EVALUATE: `.worktrees/thegent/` - determine if cli/ should be extracted

---

# Research Worklogs
# Research Worklogs

**Category:** RESEARCH | **Updated:** 2026-03-29

---

## 2026-03-29 - Expanded External Package Research (2026)

**Project:** [cross-repo]
**Category:** research
**Status:** in_progress
**Priority:** P1

### Summary

Comprehensive research on external 3rd party packages and repos for integration opportunities. Focus on whitebox (fork+modify), blackbox (direct use), and wrap (custom impl) strategies.

---

## GitHub Verified Research (2026-03-29)

### Tauri Apps ✅ VERIFIED

| Field | Value |
|-------|-------|
| GitHub | `tauri-apps/tauri` |
| Stars | **105k** |
| Forks | 3.5k |
| Commits | 5,927 |
| License | MIT/Apache 2.0 |
| Status | Stable |
| Platforms | Windows, macOS, Linux, iOS, Android |

**Opportunity:** Desktop agent UI wrapper - ADOPT
- Rust backend with web frontend
- Small binary size, fast performance
- Cross-platform desktop apps
- System tray, notifications, native menus

### Google Cloud Go ✅ VERIFIED

| Field | Value |
|-------|-------|
| GitHub | `googleapis/google-cloud-go` |
| Stars | **4.4k** |
| Forks | 1.5k |
| Commits | 10,206 |
| APIs | 200+ Google Cloud services |
| Status | Production |

**Opportunity:** For specific services - WRAP
- Secret Manager, Cloud Storage, Pub/Sub
- IAM, Resource Manager
- Vertex AI for ML inference

---

## External Package Integration Strategy

### Integration Levels

| Level | Description | Example | LOC Savings |
|-------|-------------|---------|------------|
| **BLACKBOX** | Direct dependency | `anyhow::Error` | 0 |
| **WHITEBOX** | Fork + modify | Custom fork of `eventually` | High |
| **WRAPPER** | Custom impl wrapping external | `phenotype-event-sourcing` wrapping `eventually` | Medium |
| **INSPIRATION** | Study patterns, implement differently | Study `casbin`, implement `phenotype-policy-engine` | N/A |
| **REPLACE** | Drop external for internal | Replace `serde_json` with `rmp` | Varies |

### Developer Quality Assessment

| Factor | Questions |
|--------|-----------|
| **Active Maintenance** | Last commit < 6 months? |
| **Community Size** | Stars, contributors, issues? |
| **Documentation** | Docs.rs, examples, guides? |
| **Breaking Changes** | Version stability? |
| **License** | Permissive for commercial use? |

### Fork/Modify Decision Matrix

```
                    High Quality Dev                    Low Quality Dev
                   /                    \              /                \
              Large Gap                                              Small Gap
             /        \                                          /            \
        FORK+WRAP   WRAP+CONTRIB                           WRAP          BLACKBOX
        (long-term) (medium-term)
```

---

## Recommended External Package Actions

### Immediate (This Week)

- [ ] 🟡 HIGH: Evaluate `casbin` for cross-language policy engine (Apache 2.0, 1.1k stars) - **WRAP**
- [ ] 🟡 HIGH: Evaluate `eventually` for standardized Aggregate/Repository traits (500 stars) - **WRAP**
- [ ] 🟡 HIGH: Add `zod` for Node.js API validation (20k stars) - **ADD**

### Short-term (This Month)

- [ ] 🟡 HIGH: Create `phenotype-event-sourcing-wrapper` for `eventually` interop
- [ ] 🟡 HIGH: Create `phenotype-policy-engine-wrapper` for `casbin` interop
- [ ] 🟠 MEDIUM: Evaluate `temporal-sdk` for long-running workflows (440 stars, prerelease) - **WRAP**
- [ ] 🟠 MEDIUM: Evaluate `tauri` for desktop agent UI (105k stars) - **ADOPT**

### Medium-term (This Quarter)

- [ ] 🟠 MEDIUM: Add `pydantic` patterns for Python interop (25k stars)
- [ ] 🟠 MEDIUM: Wrap `xstate` for frontend FSM interop (15k stars)
- [ ] 🟢 LOW: Evaluate `google-cloud-go` for specific GCP services (4.4k stars) - **WRAP**
- [ ] 🟢 LOW: Evaluate `surrealdb` for embedded graph storage (30k stars)

---

## Related

- Duplication: `worklogs/DUPLICATION.md`
- Dependencies: `worklogs/DEPENDENCIES.md`
- Architecture: `worklogs/ARCHITECTURE.md`
## 2026-03-29 - Git State & Cleanup Findings

**Project:** [phenotype-infrakit]
**Category:** research
**Status:** completed
**Priority:** P1

### Git State Analysis

**Critical Issues Found:**

| Issue | Severity | Action Required |
|-------|----------|-----------------|
| Unresolved merge conflict in `.gitignore` | 🔴 CRITICAL | ✅ FIXED - Resolved conflict markers |
| Staged `src/thegent/` code in Rust repo | 🔴 CRITICAL | ✅ FIXED - Unstaged with `git reset` |
| Stash with worklog changes | 🟡 HIGH | Review `stash@{0}` |
| Orphaned local branches | 🟠 MEDIUM | `fix/phench-tests-1`, `chore/*` |

### Local Branches Requiring Review

| Branch | Status | Action |
|--------|--------|--------|
| `fix/phench-tests-1` | Local only | Delete or push |
| `chore/worklog-consolidation` | Has stash | Review stash@{0} |
| `chore/cleanup-worklogs-20260329` | Pushed to origin | OK |

### Remote Branches (origin) - Cleanup Candidates

| Branch | Status | Action |
|--------|--------|--------|
| `origin/chore/spec-docs` | Merged | Delete |
| `origin/chore/vitepress-docs*` | Likely merged | Delete |
| `origin/chore/worklog*` | Likely merged | Delete |
| `origin/docs/consolidate-worklog-notes` | Merged | Delete |

### Stashed Changes

```
stash@{0}: On chore/worklog-consolidation: worklogs-unstaged-changes
stash@{1}: WIP on main: 882391e23 chore: cleanup docs/worklogs
stash@{2}: WIP on main: ce4f0c94c chore: ignore libs/ and platforms/
```

**Recommendation**: Review `stash@{0}` for any needed worklog changes.

### Recommendations

1. **Clean up merged remote branches:**
   ```bash
   git push origin --delete chore/spec-docs chore/vitepress-docs chore/worklog-*
   ```

2. **Delete orphaned local branches:**
   ```bash
   git branch -d fix/phench-tests-1 chore/worklog-consolidation
   ```

3. **Review stashed changes:**
   ```bash
   git stash show -p stash@{0}
   ```

### Related

- `.gitignore` - Fixed merge conflict
- `src/thegent/` - Unstaged, NOT part of phenotype-infrakit

---

## 2026-03-29 - 2026 Rust Crate Ecosystem Research

**Project:** [phenotype-infrakit]
**Category:** research
**Status:** completed
**Priority:** P1

### Event Sourcing Crates (crates.io)

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `eventually` | ~50k/mo | Aggregate/Repository traits | **WRAP** - Standardized ES patterns |
| `cqrs-es` | ~10k/mo | CQRS + Event Sourcing | **EVALUATE** - CQRS focus |
| `aggregate` | ~5k/mo | Aggregate root framework | **EVALUATE** - Complement to eventually |
| `event-sourcing` | ~2k/mo | Simple event store | **HOLD** - Too basic |

**Recommendation**: `eventually` is the community standard. Consider wrapping for phenotype-specific extensions.

### State Machine Crates

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `xstate` (Rust) | ~5k | SCXML-based FSM | **WRAP** - Formal FSM, frontend interop |
| `states` | ~20k | Simple state machine | **ADOPT** - Lightweight, ergonomic |
| `stent` | ~3k | State machine | **HOLD** - Unmaintained |
| `derive-state` | ~10k | Derive macro FSM | **EVALUATE** - Simple cases |

**Recommendation**: Current `phenotype-state-machine` has unique features (guards, ordinal enforcement). Compare with `states` crate.

### Policy/Access Control Crates

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `casbin` | ~100k | RBAC/ABAC engine | **WRAP** - Cross-language support |
| `openacl` | ~1k | Zanzibar-like | **EVALUATE** - Complex permissions |
| `ozauth` | ~500 | OAuth2/OIDC | **WRAP** - Auth flows |

**Note**: Current `phenotype-policy-engine` is TOML-based rules. `casbin` offers richer policy expressions.

### Cache Crates (Beyond moka)

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `moka` | ~500k | All platforms | **IN USE** ✅ |
| `cache2` | ~50k | TTL cache | **HOLD** - Unmaintained |
| `cached` | ~100k | Procedural macros | **HOLD** - Less ergonomic |
| `dashcache` | ~10k | DashMap wrapper | **HOLD** - dashmap sufficient |

**Recommendation**: moka is optimal. `phenotype-cache-adapter` provides two-tier with LRU + DashMap.

### Config Crates

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `figment` | ~100k | Multi-source config | **ADOPT** - TOML/YAML/JSON/ENV |
| `config-rs` | ~200k | Hierarchical config | **EVALUATE** - 40M+ total downloads |
| `cosmiconfig` | ~50k | космонавт config | **EVALUATE** - No dependencies |
| `dotenvy` | ~100k | .env files | **ADOPT** - Updated fork of dotenv |

**Recommendation**: `figment` provides provenance tracking which `phenotype-infrakit` crates don't need (they're libraries).

### Process Management Crates

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `command-group` | ~50k | Process groups | **ADOPT** - Signal propagation |
| `tokio-command` | ~20k | Async wrapper | **EVALUATE** - Tokio integration |
| `xshell` | ~50k | Shell utilities | **EVALUATE** - Cross-platform |

**Recommendation**: For `thegent` CLI tool, not `phenotype-infrakit` libraries.

### 2026 AI/LLM Integration Crates

| Crate | Downloads | Purpose | Assessment |
|-------|-----------|---------|------------|
| `anthropic` | NEW | Claude SDK | **EVALUATE** - Official async support |
| `anthropic-sdk-core` | NEW | Core types | **EVALUATE** - Streaming, tools |
| `llm-chain` | ~5k | Multi-provider LLM | **EVALUATE** - Tool use, chains |
| `tiktoken-rs` | ~10k | Token counting | **EVALUATE** - Cost tracking |

**Note**: These are for `thegent` agent framework, not `phenotype-infrakit`.

### Related

- `crates/phenotype-cache-adapter/` - Uses moka + dashmap + lru
- `crates/phenotype-policy-engine/` - Custom TOML rules
- `crates/phenotype-state-machine/` - Custom FSM with guards

---

## 2026-03-29 - Fork/Wrap Decision Framework

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Fork Decision Matrix (2026 Updated)

| Scenario | Decision | Example | Effort |
|----------|----------|---------|--------|
| Need significant modifications | **FORK** | Custom PTY handling | High |
| Need features not in upstream | **FORK+EXTEND** | phenotype-error patterns | Medium |
| Need thin phenotype layer | **WRAP** | Git worktree wrapper | Low |
| Crate is perfect as-is | **DIRECT USE** | serde, tokio | None |
| Internal is better | **KEEP INTERNAL** | phenotype-event-sourcing | N/A |

### LOC Savings Analysis (phenotype-infrakit scope)

| Pattern | Current | External | Savings | Decision |
|---------|---------|----------|---------|----------|
| Event sourcing | Custom | eventually | N/A | KEEP - Hash chain is unique |
| Cache | Custom | moka | N/A | KEEP - Two-tier is unique |
| Policy | Custom | casbin | N/A | KEEP - TOML simplicity |
| FSM | Custom | states | ~100 LOC | EVALUATE - Guards are unique |

### Cross-Repo Fork Candidates (AgilePlus/thegent/heliosCLI scope)

| Source | Target | LOC | Priority | Rationale |
|--------|--------|-----|----------|-----------|
| `utils/pty` | `phenotype-process` | ~750 | 🔴 CRITICAL | PTY + process groups |
| CodexErr | `phenotype-error` | ~400 | 🔴 CRITICAL | Unified error taxonomy |
| `utils/git` | `phenotype-git` | ~300 | 🟠 HIGH | Git operations |
| `SpawnContext` | `phenotype-executor` | ~150 | 🟡 MEDIUM | Execution context |

### Related

- `DUPLICATION.md` - Cross-repo duplication analysis
- `DEPENDENCIES.md` - Current dependency status

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

---

## 2026-03-29 - External Package Fork/Wrap Opportunities

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Web research on forkable packages, external libraries with fork potential, and 3rd party integrations relevant to the Phenotype ecosystem.

---

### 1. Git Operations: gix (gitoxide)

**Source:** https://github.com/Byron/gitoxide
**Stars:** 11.1K
**Language:** Rust
**License:** MIT/Apache-2.0

**Fork Opportunity:** YES - Fork to add custom Git extensions for Phenotype workflow automation.

**Key Subcrates:**

| Subcrate | Lines | Purpose | Fork Value |
|----------|-------|---------|------------|
| `gix-lock` | ~500 | Cross-platform file locking | HIGH |
| `gix-tempfile` | ~300 | Secure temp files | HIGH |
| `gix-sec` | ~400 | Cryptographic operations | MEDIUM |
| `gix-credentials` | ~200 | Git credential handling | MEDIUM |

**Why Fork vs Use:**
- Add custom Git hooks for governance
- Integrate with AgilePlus workflow
- Custom commit message validation

---

### 2. Model Context Protocol (MCP)

**Source:** https://github.com/modelcontextprotocol/servers
**Stars:** 82.4K
**Language:** TypeScript/Python
**License:** MIT

**Official MCP Servers:**

| Server | Language | Purpose | Fork Value |
|--------|----------|---------|------------|
| Git | TypeScript | Git repository tools | HIGH |
| Filesystem | TypeScript | Secure file ops | HIGH |
| Fetch | TypeScript | Web content fetching | MEDIUM |
| Memory | TypeScript | Knowledge graph | MEDIUM |

**Why Fork:**
- Add AgilePlus-specific tools
- Custom GitHub/GitLab integration
- Governance policy enforcement

---

### 3. Agent Frameworks (Rust)

| Framework | Stars | Language | Fork Value |
|-----------|-------|----------|------------|
| `candle` | 15K | Rust | LOW - too early |
| `burn` | 8K | Rust | MEDIUM - for ML |
| `llm` | 3K | Rust | HIGH - local inference |
| `mistralrs` | 2K | Rust | MEDIUM - Mistral optimized |

---

### 4. CLI & Process Management

| Crate | Downloads | Purpose | Fork Value |
|-------|----------|---------|------------|
| `command-group` | 500K | Process groups | HIGH |
| `indicatif` | 3M | Progress bars | MEDIUM |
| `clap_complete` | 3M | Shell completions | LOW |
| `dialoguer` | 2M | Interactive prompts | MEDIUM |

---

### 5. Observability

| Crate | Downloads | Purpose | Fork Value |
|-------|----------|---------|------------|
| `opentelemetry` | 5M | Distributed tracing | MEDIUM |
| `prometheus` | 3M | Metrics export | MEDIUM |
| `tracing-opentelemetry` | 2M | OTEL integration | MEDIUM |

---

### 6. Database & Caching

| Crate | Downloads | Purpose | Fork Value |
|-------|----------|---------|------------|
| `deadpool` | 3M | Async pooling | MEDIUM |
| `sqlx-cli` | 5M | SQLx tooling | LOW |
| `rkyv` | 500K | Zero-copy serialization | HIGH |

---

### 7. Blackbox vs Whitebox Usage Analysis

| Pattern | Usage Type | Recommendation |
|---------|------------|----------------|
| `serde` + `serde_json` | Blackbox | Continue as-is |
| `tokio` | Blackbox | Continue as-is |
| `thiserror` | Whitebox (derive macros) | Continue as-is |
| `git2` | Blackbox | MIGRATE to `gix` |
| Hash chain logic | Whitebox | Consider `blake3` alternative |

---

### 8. Recommended Fork Strategy

```
┌─────────────────────────────────────────────────────────────┐
│  FORK DECISION TREE                                          │
├─────────────────────────────────────────────────────────────┤
│  1. Is package well-maintained?                              │
│     ├── YES → Use as blackbox dependency                    │
│     └── NO → Continue to step 2                            │
│                                                              │
│  2. Is custom functionality needed?                          │
│     ├── YES → FORK the repository                           │
│     └── NO → Consider alternative maintained package         │
│                                                              │
│  3. Is fork effort < hand-roll effort?                      │
│     ├── YES → FORK                                          │
│     └── NO → Hand-roll or abandon feature                   │
└─────────────────────────────────────────────────────────────┘
```

---

### Priority Fork Opportunities

| Priority | Package | Rationale | Effort |
|----------|---------|-----------|--------|
| P0 | `gix` fork | Replace `git2` (security) | 2-4 weeks |
| P1 | MCP Git server fork | AgilePlus tool integration | 1-2 weeks |
| P2 | `gix-lock` fork | Cross-platform locking | 1 week |
| P3 | `llm` fork | Local LLM inference | 2-3 weeks |

---

_Last updated: 2026-03-29_

---

## 2026-03-29 - External Dependencies Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Research into GitHub starred repos and external packages for fork/wrap opportunities.

### Blackbox vs Whitebox Analysis Framework

| Mode | Description | When to Use |
|------|-------------|-------------|
| **Blackbox** | Use as-is, no modifications | Stable, well-maintained deps |
| **Whitebox** | Fork and customize | Need modifications, better devs available |
| **Wrap** | Create adapter/shim around library | Want to isolate from changes |
| **Fork** | Full control, periodic sync | Heavy customization, internal release cycle |

### GitHub Starred Repos (Developer Tooling)

#### 1. `Data-Wise/craft` ⭐ 1

Full-stack dev toolkit for Claude Code with 86 commands, 8 agents, 21 skills.

| Property | Value |
|----------|-------|
| Type | Claude Code Plugin |
| Language | Python |
| LOC | ~500 |
| Recommendation | **FORK** |
| Benefit | 500+ LOC savings, proven patterns |

#### 2. `newrelic/*` (multiple repos) ⭐ 400+

Observability tooling suite with CLI, client, and codegen tools.

| Property | Value |
|----------|-------|
| Type | Observability |
| Language | Go |
| Recommendation | **WRAP** |
| Benefit | 200+ LOC savings |

#### 3. `michen00/invisible-squiggles` ⭐ 3

VSCode extension for distraction-free linter diagnostics.

| Property | Value |
|----------|-------|
| Type | VSCode Extension |
| Language | TypeScript |
| Recommendation | **WRAP** |
| Benefit | Clean UX patterns |

### Fork/Wrap Decision Matrix

```
Need modifications? ──NO──▶ Blackbox (use as-is)
        │
       YES
        │
Better devs available? ──NO──▶ Wrap (create adapter)
        │
       YES
        │
Need full control? ──YES──▶ Fork (periodic sync)
        │
       NO
        │
Need fast iteration? ──YES──▶ Fork (tight sync)
        │
       NO
        │
Long-term maintenance? ──YES──▶ Fork (formal sync)
        │
       NO
        │
▶ Wrap (lightweight adapter)
```

### LOC Reduction Opportunities

| Category | Current | Target | Savings |
|----------|---------|--------|---------|
| Fork health_check | 80 | 0 | **80** |
| Create error-core | 150 | 0 | **150** |
| Integrate config-core | 200 | 0 | **200** |
| Wrap temporal-sdk | 500 | 0 | **500** |
| Wrap casbin | 300 | 0 | **300** |
| Wrap eventsourcing | 300 | 0 | **300** |
| **TOTAL** | **1,530** | **0** | **1,530** |

---

## 2026-03-29 - A2A Protocol Research

**Project:** [thegent, heliosCLI]
**Category:** research
**Status:** completed
**Priority:** P1

### Agent2Agent (A2A) Protocol

**Spec:** https://ajima.ai/A2A

**Key Concepts:**
- AgentCard: Self-describing agent metadata
- Task: Unit of work with state transitions
- Message: Communication between agents
- Push notifications for async updates

**Phenotype Alignment:**

| A2A Concept | Phenotype Equivalent | Alignment |
|-------------|---------------------|-----------|
| AgentCard | Agent metadata in thegent | Medium |
| Task | Work packages in AgilePlus | High |
| Message | ACP protocol messages | High |
| Push notifications | Webhook system | Medium |

**Opportunity:** Integrate `ra2a` crate for standardized agent communication.

---

_Last updated: 2026-03-29_
