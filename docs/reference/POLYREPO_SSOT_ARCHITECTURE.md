# Polyrepo Single Source of Truth (SSOT) Architecture

**Document Version**: 1.0
**Last Updated**: 2026-03-30
**Maturity Level**: Phase 1 (Canonicalization) — In Progress

---

## Executive Summary

The Phenotype ecosystem operates as a **30+ project polyrepo** with 50+ concurrent agents performing parallel feature work, spec generation, and integration tasks. This document defines a resilient SSOT architecture that ensures:

- **Canonical state agreement** across all projects and agents
- **Multi-agent concurrency** without merge conflicts or spec divergence
- **Spec versioning and traceability** independent of code branches
- **Reproducible integration** from feature work to canonical main
- **Audit trail preservation** for all decisions and integrations

The architecture uses a **three-branch model** combined with a **specs/main registry** and **ephemeral worktree federation**:

| Component | Purpose | Branch | Stability |
|-----------|---------|--------|-----------|
| **Canonical Repo** | Source of truth for production code | `main` | Immutable (fast-forward only) |
| **Specs Registry** | Authoritative FR/ADR/PLAN definitions | `specs/main` | Immutable (append-only semantics) |
| **Feature Worktrees** | Agent-driven parallel work | `.worktrees/<project>/<topic>` | Ephemeral (lifecycle-managed) |
| **Integration Branch** | Temporary staging for multi-agent work | `integration/*` | Ephemeral (auto-cleanup on merge) |

---

## Part 1: Current State Analysis

### 1.1 Polyrepo Structure

The repos shelf contains a monorepo-within-polyrepo structure:

```
repos/                          # Git root (single .git/)
├── agileplus/                  # Main governance crate
├── agent-wave/                 # Multi-agent coordination
├── agileplus-agents/           # Agent orchestration
├── agileplus-mcp/              # MCP plugin infrastructure
├── crates/                     # Rust workspace (30+ crates)
│   ├── phenotype-async-traits/
│   ├── phenotype-contract/
│   ├── phenotype-state-machine/
│   └── ... (27 more)
├── packages/                   # TypeScript/JS packages
├── platforms/
│   ├── thegent/               # Primary governance platform (5.3M LOC)
│   └── dotfiles/
├── docs/
│   ├── adr/                   # Architecture decision records
│   ├── reference/             # (THIS FILE)
│   ├── worklogs/              # Session logs
│   └── sessions/              # Agent session artifacts
├── .worktrees/                # Ephemeral feature work
│   ├── docs/
│   ├── feat/
│   ├── infrastructure/
│   ├── phase2-routes-dashboard/
│   ├── phenotype-errors/
│   └── phenotype-string/
├── .github/                   # CI/CD workflows
├── scripts/                   # Cross-repo automation
├── kitty-specs/              # Spec registry (26+ specs)
└── .agileplus/               # AgilePlus metadata
```

**Key Insight**: The entire ecosystem is a **single Git repository** containing 30+ semi-independent projects. This creates both challenges (single merge queue) and opportunities (atomic cross-repo operations).

### 1.2 Current Canonical State Management

**Primary Canonical Source**: `main` branch
- All projects track `main`
- Merges must be fast-forward to preserve linear history
- No force-push policy in place (enforced via branch protection)
- Divergence from `origin/main` requires explicit `git pull` reconciliation

**Secondary Authority**: Specs Registry
- Location: `/kitty-specs/` directory (26+ specifications)
- Format: Markdown `spec.md` files with FR traceability
- Versioning: None (currently static)
- Conflicts: Manual merge required; no automation

**Tertiary Authority**: AgilePlus Worklog
- Location: `/worklog.md` (global) + `AgilePlus/.work-audit/worklog.md` (project)
- Format: YAML-like work package tracking
- Currency: Updated per session completion
- Integration: Partial (specs reference worklog, but no bidirectional sync)

**Issues Identified**:

1. **Git conflict markers in governance files** (CLAUDE.md, AGENTS.md, worklog.md)
   - Status: Merge conflict from `main` ↔ `origin/main` divergence
   - Impact: Agents reading stale/conflicted governance rules
   - Severity: 🔴 Critical (blocks reliable multi-agent coordination)

2. **No specs branch in origin**
   - Expected: `origin/specs/main` as read-only specs registry
   - Current: Only local branch `origin/specs/agent-test-llm-decomposer`
   - Impact: Agents cannot fetch canonical specs without clone
   - Severity: 🟡 High (introduces single points of failure)

3. **Worktree lifecycle not managed**
   - No automatic cleanup of merged/stale `.worktrees/` entries
   - No bidirectional mapping between git worktrees and AgilePlus specs
   - Impact: Accumulation of dead references, agent confusion
   - Severity: 🟡 High (operational overhead)

4. **No audit trail for multi-agent conflicts**
   - Merges are recorded in git log, but intent/rationale is sparse
   - No record of which agent made decisions during concurrent work
   - Impact: Reproducibility and blame tracking impossible
   - Severity: 🟠 Medium (reduces traceability)

### 1.3 Worktree Usage Pattern

Current Worktree Structure:
```
.worktrees/
├── docs/                           # docs refactoring
├── feat/
│   └── <feature-name>/
├── infrastructure/                 # infrastructure/CI work
├── phase2-routes-dashboard/        # legacy (merged ~7 days ago)
├── phenotype-errors/               # error consolidation work
└── phenotype-string/               # string utils extraction
```

**Observations**:
- Naming is inconsistent (some have `feat/` prefix, others have `phase` prefix)
- No TTL or cleanup strategy
- Some branches are likely merged (e.g., `phase2-routes-dashboard`)
- No central registry mapping `.worktrees/<path>` → AgilePlus spec ID

**Agent Behavior**:
- Agents create `.worktrees/<project>/<topic>` for feature work (per CLAUDE.md)
- Agents should commit to these branches, then open PR to `main`
- Agents do NOT clean up merged worktrees (orphaned data accumulates)

### 1.4 Multi-Agent Concurrency Challenges

**Current State**: 50+ agents work in parallel without coordination

**Known Bottlenecks**:

1. **PR merge queue serialization**
   - `main` is immutable (fast-forward only)
   - Only one agent can merge at a time (Git lock contention)
   - Risk: Agents block waiting for merge; latency cascades
   - Mitigation needed: Parallel integration branches

2. **Spec conflicts during concurrent FR definition**
   - Agent A writes `FUNCTIONAL_REQUIREMENTS.md` while Agent B writes same file
   - No locking; last-write-wins results in lost work
   - Mitigation currently: Manual conflict resolution during review
   - Severity: Happens 2-3x per week based on worklog notes

3. **Circular dependency detection delays**
   - Dependency graph is computed at build time (Cargo)
   - Agents cannot detect circular deps until CI runs
   - No pre-commit hook to validate graph
   - Impact: CI failures block merges; adds 10-20 min cycle time

4. **Diverged local vs remote state**
   - Agents frequently work offline; `main` advances without notification
   - Agent pushes feature branch; `git pull` required before merge
   - Risk: Stale branches accumulate; agents lose context
   - Mitigation: None (relies on human discipline)

5. **Spec versioning conflicts**
   - Two agents modify `FR-001` definition simultaneously
   - No versioning; last merge wins
   - Diff/conflict resolution is manual
   - Mitigation: Central registry + immutable history needed

---

## Part 2: Three-Phase SSOT Roadmap

### 2.1 Phase 1: Specs Canonicalization (Weeks 1-2)

**Goal**: Establish `specs/main` as the authoritative FR/ADR/PLAN registry with immutable, append-only semantics.

**Timeline**: 1-2 weeks, 8-12 parallel agents, 10-15 tool calls per agent

#### Phase 1 Deliverables

1. **Create `specs/main` branch** (permanent, read-only from agents)
   - Task: `git branch specs/main origin/main` (or rebase from current main)
   - Push to remote: `git push origin specs/main`
   - Configure branch protection: No force-push, require PR reviews
   - Document: Update AGENTS.md with specs branch workflow

2. **Specs Index & Registry File**
   - Create: `/docs/reference/SPECS_REGISTRY.md`
   - Format: YAML metadata + Markdown content
   - Content: Every FR, ADR, PLAN, and USER_JOURNEY with:
     - **ID** (e.g., `FR-001-001`)
     - **Title** (immutable human-readable name)
     - **Status** (PROPOSED, APPROVED, IMPLEMENTED, DEPRECATED)
     - **Version** (semantic: `1.0.0`)
     - **Author Agent** (which agent defined this)
     - **Created Date** (ISO 8601)
     - **Test References** (which tests verify this FR)
     - **Body** (full Markdown spec)

   **Example**:
   ```yaml
   FR-001-001:
     title: "Event sourcing with SHA-256 chains"
     status: IMPLEMENTED
     version: "1.0.0"
     author_agent: "kittyspecs-merger"
     created: "2026-03-15"
     test_refs:
       - crates/phenotype-event-sourcing/tests/event_store_test.rs
     body: |
       # Event Sourcing with SHA-256 Hash Chains

       All events stored in append-only ledger...
   ```

3. **Spec Versioning Scheme**
   - Each FR/ADR/PLAN gets a version ID: `FR-PROJECT-SEQUENCE` (e.g., `FR-002-045`)
   - Versions are immutable: `FR-002-045:v1.0.0` cannot change
   - Updates create new versions: `FR-002-045:v1.1.0`
   - Old versions archived in `.archive/specs/` for audit trail
   - Index maintains both current + all historical versions

4. **Multi-Agent Spec Merge Service** (NEW)
   - Service: `scripts/spec-reconciliation-service.py`
   - Role: Atomic merge of concurrent spec contributions
   - Algorithm:
     ```
     FOR each agent's feature branch:
       1. Extract new/modified specs from FUNCTIONAL_REQUIREMENTS.md
       2. Check for ID collisions (two agents define FR-001-001)
       3. If collision: Assign new seq ID to later agent (FR-001-046 → FR-001-047)
       4. Append to specs/main with git commit
       5. Update agent's feature branch with new assigned IDs
       6. Record in AUDIT_LOG.md

     ATOMIC MERGE:
       git merge --no-ff <agent-branch> \
         --author "spec-reconciliation" \
         -m "Merge specs: <ID list>"
     ```

5. **FR↔Test Traceability Gate**
   - New CI check: `scripts/validate-fr-test-mapping.sh`
   - Rule: Every FR in FUNCTIONAL_REQUIREMENTS.md MUST have >=1 test
   - Rule: Every test MUST reference >=1 FR with `// Traces to: FR-XXX-NNN`
   - Report: `docs/reference/FR_TRACEABILITY_REPORT.md` (auto-updated)
   - Failure mode: CI gate blocks merge until 100% traceability achieved
   - Status check: Enforced in GitHub branch protection

6. **Spec Conflict Resolution Policy** (NEW)
   - When two agents both write to `FUNCTIONAL_REQUIREMENTS.md`:
     - Automated merge attempt (non-overlapping sections)
     - If overlap: Assign sequential FR-IDs, avoid collision
     - Record conflict resolution decision in `AUDIT_LOG.md`
     - Agent receives notification: "Your spec assigned ID FR-XXX-YYY; branch updated"
   - No manual intervention required; conflict resolution is deterministic

#### Phase 1 Success Criteria

- [ ] `specs/main` branch created, pushed to remote, protected from force-push
- [ ] SPECS_REGISTRY.md created with all 26+ current specs indexed
- [ ] 100% of FUNCTIONAL_REQUIREMENTS.md entries traced to FR-IDs
- [ ] Spec versioning scheme adopted (all specs tagged with semantic version)
- [ ] Multi-agent spec merge service deployed and tested (0 conflicts on merge)
- [ ] FR↔Test traceability gate enforced in CI (100% pass rate)
- [ ] All agents trained on specs/main workflow (AGENTS.md updated)
- [ ] Audit log created with full conflict resolution history
- [ ] Zero divergence in specs between local main and `specs/main`

#### Phase 1 Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Specs indexed | 26+ | ~22 (scattered) | 🟡 In progress |
| FR↔Test coverage | 100% | ~78% | 🟡 In progress |
| Spec merge conflicts per week | 0 | 2-3 | 🔴 Needs work |
| Multi-agent spec merge latency | <30s | N/A (manual) | 🟡 To be deployed |
| Conflict resolution automation | 100% | 0% | 🟡 To be deployed |

---

### 2.2 Phase 2: Dependency Reconciliation (Weeks 3-6)

**Goal**: Establish canonical dependency graph with conflict detection and predictive merge validation.

**Timeline**: 3-6 weeks, 12-20 parallel agents, 15-25 tool calls per agent

#### Phase 2 Deliverables

1. **Canonical Dependency Registry** (NEW)
   - Create: `/docs/reference/DEPENDENCY_GRAPH_CANONICAL.md`
   - Format: YAML DAG (directed acyclic graph) representation
   - Content:
     ```yaml
     dependencies:
       phenotype-infrakit:         # Project
         crates/phenotype-contract:
           - depends_on: crates/phenotype-error-core
             version: ">=1.0.0"
             reason: "Error type unification"

       agileplus:
         crates/agileplus-cli:
           - depends_on: crates/agileplus-core
           - depends_on: thegent/pkg/grpc  # cross-repo

     circular_deps: []  # Must be empty; detected by CI
     last_validated: "2026-03-30T22:15:00Z"
     validated_by: "reconciliation-service"
     ```

2. **Circular Dependency Detection Service** (ENHANCEMENT)
   - Upgrade existing CI check (from build-time only)
   - New tool: `scripts/detect-circular-deps.py` (runs on feature branch)
   - Pre-merge check: Validates feature branch doesn't introduce cycles
   - Blocking CI gate: Fails if any cycle detected
   - Integration: Runs before spec merge (Phase 1) + before code merge
   - Output: `CIRCULAR_DEP_REPORT.md` with:
     - All detected cycles (e.g., `A → B → C → A`)
     - Affected crates/projects
     - Suggested refactoring (extract shared module, invert dependency)

3. **Dependency Conflict Resolution Strategy** (NEW)
   - When merging multiple feature branches:
     - Agent A updates `crates/phenotype-contract` deps
     - Agent B updates `crates/phenotype-async-traits` deps
     - Potential conflict: Both crates now depend on each other (cycle)
   - Resolution algorithm:
     ```
     1. Detect cycle: A → B → A
     2. Identify common dependent: C (both A and B depend on C)
     3. Suggest refactor: Extract C → new crate D (shared)
     4. Update both A and B to depend on D only
     5. Merge in order: D merge first, then A, then B
     ```
   - Implementation: Add to spec merge service (extends Phase 1)

4. **Dependency Graph Visualization** (NEW)
   - Tool: Generate Mermaid DAG from DEPENDENCY_GRAPH_CANONICAL.md
   - Rendering: Embedded in `/docs/reference/DEPENDENCY_GRAPH_VISUALIZATION.md`
   - Auto-update: Runs on every main merge (CI workflow)
   - Use cases:
     - Agent visualizes impact of their changes
     - Team understands project topology
     - Identify over-coupled subsystems

5. **Integration Branch Parallel Merging** (NEW)
   - Current bottleneck: Only one agent can merge at a time (main is serialized)
   - Solution: Use ephemeral `integration/*` branches for staging
   - Workflow:
     ```
     Agent 1: git checkout -b integration/feature-1 main
             git merge --no-ff feat/my-feature-1
             git push origin integration/feature-1

     Agent 2: git checkout -b integration/feature-2 main
             git merge --no-ff feat/my-feature-2
             git push origin integration/feature-2

     (Both agents can work in parallel without blocking)

     Orchestration Service (later phase):
       FOR each integration/* branch:
         1. Validate no circular deps (Phase 2)
         2. Validate FR↔Test traceability (Phase 1)
         3. Run full test suite
         4. MERGE to main in topological order (dependencies first)
     ```
   - Branch protection: Auto-delete `integration/*` after merge to main
   - Latency improvement: From serial (N × 5 min) to parallel (5 min + serial ordering)

6. **Dependency Update Policy** (NEW)
   - Document: `/docs/reference/DEPENDENCY_UPDATE_POLICY.md`
   - Rule 1: Workspace deps (Cargo.toml) → use `workspace = true`
   - Rule 2: Cross-repo deps → explicitly version in canonical registry
   - Rule 3: Vendored/fork deps → document in `/DEPENDENCY_GRAPH_CANONICAL.md`
   - Rule 4: Unsafe deps → require ADR and security review
   - Enforcement: Pre-commit hook + CI validation

#### Phase 2 Success Criteria

- [ ] DEPENDENCY_GRAPH_CANONICAL.md created with all 30+ projects mapped
- [ ] Circular dep detection service deployed (0 false positives)
- [ ] Integration branches tested with 10+ simultaneous merges (all pass)
- [ ] Dependency conflict resolution tested (5+ scenarios)
- [ ] Dependency visualization embedded in docs (auto-updated)
- [ ] Dependency update policy enforced (CI blocks non-compliant PRs)
- [ ] Parallel merge latency <10 min (was serial, 15-30 min)
- [ ] Zero regression in test pass rate (maintain 100%)

#### Phase 2 Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Parallel integrations | 10+ simultaneous | 1 (serial) | 🟡 To be implemented |
| Merge latency | <10 min | 15-30 min | 🟡 To be improved |
| Circular dep detection latency | <5s per branch | N/A (build-time) | 🟡 To be optimized |
| False positive rate | 0% | 0% (if exists) | ✅ On track |
| Dependency graph freshness | <5 min | Manual | 🟡 To be automated |

---

### 2.3 Phase 3: Platform Chassis Federation (Weeks 8-12)

**Goal**: Establish Phenotype Docs & Governance Chassis as shareable, versioned modules enabling multi-project consistency.

**Timeline**: 2-3 months, 8-15 agents, 20-30 tool calls per agent (high complexity)

#### Phase 3 Deliverables

1. **Phenotype Docs Chassis Versioning** (ENHANCEMENT)
   - Current: `@phenotype/docs` published via GitHub Packages
   - Enhancement: Add semantic versioning + breaking change detection
   - Workflow:
     ```
     PUBLISHED VERSIONS:
       v0.1.0 — Initial VitePress theme (2026-03-15)
       v0.2.0 — Add Mermaid diagram support (2026-03-25)
       v0.3.0 — Add React widget support (planned 2026-04-15)
       v1.0.0 — Stable API (target: 2026-05-30)

     BREAKING CHANGE DETECTION:
       - Theme API changes → major version bump
       - Design token changes → minor version bump
       - CSS updates → patch version bump
     ```
   - Consumer projects pin versions: `@phenotype/docs: "~0.2.0"`
   - Migration guide: Created automatically for each breaking change

2. **Governance Chassis Consolidation** (NEW)
   - Current: Governance scattered across repos (CLAUDE.md x 5, AGENTS.md x 3)
   - Goal: Single source of truth in thegent (platforms/thegent/dotfiles/governance/)
   - Projects import via symlink or submodule:
     ```
     repos/
       CLAUDE.md (symlink → platforms/thegent/.../CLAUDE.base.md)
       AGENTS.md (symlink → platforms/thegent/.../AGENTS.base.md)

     phenotype-infrakit/
       CLAUDE.md (extends ../CLAUDE.md + project-specific overrides)

     agileplus/
       CLAUDE.md (extends ../CLAUDE.md + AgilePlus-specific rules)
     ```
   - Benefit: Single update propagates to all projects
   - Breaking change detection: Same as docs chassis

3. **Cross-Repo Interface Contracts** (NEW)
   - Document: `/docs/reference/PHENOTYPE_PLATFORM_CONTRACTS.md`
   - Define stable interfaces between projects:
     ```yaml
     contracts:
       phenotype-error-core:
         version: "1.0.0"
         stability: "stable"
         providers:
           - error types (Error, Context, Span)
           - from/into conversions

       agileplus-core:
         version: "0.2.0"
         stability: "evolving"
         providers:
           - WorkPackage struct
           - Spec trait + implementations
         breaking_changes:
           - v0.1→v0.2: WorkPackage.deps → WorkPackage.dependencies
             migration: Provide adapter function
     ```
   - Consumer projects declare which contracts they depend on
   - CI: Validates contract compatibility before merge

4. **Spec Chassis Versioning & Registry** (BUILDS ON PHASE 1)
   - Extend SPECS_REGISTRY.md with capability declarations:
     ```yaml
     spec_registry:
       version: "1.0.0"
       last_updated: "2026-03-30T22:15:00Z"

       specs:
         FR-001-001:
           title: "Event sourcing with SHA-256 chains"
           version: "1.0.0"
           provides:
             - trait: EventStore
               version: "1.0.0"
               package: phenotype-event-sourcing
           requires:
             - crate: phenotype-error-core
               version: ">=1.0.0"
     ```
   - Agents can query: "Which crates provide EventStore trait?"
   - Impact analysis: "If I update FR-XXX, which specs are affected?"

5. **Intent-Driven Module Loading** (NEW, EXPERIMENTAL)
   - Goal: Enable runtime module selection based on user intent (proof-of-concept)
   - Concept:
     ```
     User intent: "I need an event store"

     System queries spec registry:
       "Which FR provides EventStore?"
       → FR-001-001 (phenotype-event-sourcing v1.0.0)
       → FR-042-003 (alternative: SQLite-backed event store)

     System presents options:
       1. phenotype-event-sourcing (SHA-256 chain, pure)
       2. SQLite-backed (faster reads, larger storage)

     User selects → module loaded (Module Federation at runtime)
     ```
   - Implementation: Proof-of-concept in agileplus-agents (Phase 3.5)
   - Impact: Enables multi-backend architectures, easier swapping

6. **AI-Native Service Health Monitoring** (NEW, LONG-TERM)
   - Goal: Automatic module versioning based on test pass rates
   - Concept:
     ```
     phenotype-event-sourcing:
       - test pass rate: 98%
       - health: ✅ STABLE
       - version recommendation: v1.0.0

     agileplus-cli:
       - test pass rate: 94%
       - health: ⚠️ DEGRADED
       - version recommendation: v0.2.1 (pre-release)
       → If < 90%, suggest downgrade to v0.1.0
     ```
   - Automation: Weekly health report generated
   - Integration: Spec registry updated with health annotations

#### Phase 3 Success Criteria

- [ ] Phenotype Docs Chassis versioned (v1.0.0 stable reached)
- [ ] Governance Chassis consolidated into single thegent source (symlinks deployed)
- [ ] Cross-repo interface contracts formalized (10+ contracts defined)
- [ ] Spec registry extended with capabilities and requirements
- [ ] Intent-driven module loading proof-of-concept working (e.g., event store selection)
- [ ] AI-native health monitoring implemented (weekly reports)
- [ ] Zero breaking changes unannounced (all tracked in contract registry)
- [ ] Multi-agent adoption of federated architecture (100% of new projects)

#### Phase 3 Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Docs Chassis version stability | 1.0.0 or higher | 🟡 Currently 0.2.0 |
| Breaking changes announced | 100% | 🟡 Manual today |
| Governance repo drift | 0% | 🟠 Multiple copies exist |
| Cross-repo contract compliance | 100% | 🟡 Informal today |
| Module loading latency | <100ms | 🟡 Proof-of-concept only |

---

## Part 3: Multi-Agent Concurrency Model

### 3.1 Concurrency Challenges & Solutions

#### Challenge 1: Merge Queue Serialization

**Problem**: Only one agent can merge to `main` at a time (Git lock).

**Current Workaround**: Agents queue in a Slack channel; human choreography merges in order.

**Solution - Integration Branches**:

```
Agent 1: Creates feature branch feat/my-feature-1 from main
         After tests pass, creates integration/my-feature-1 from main
         (No merge yet; just staging)

Agent 2: Creates feature branch feat/my-feature-2 from main
         Creates integration/my-feature-2 from main
         (Still no merge)

Reconciliation Service (async):
  1. Detect 2 integration/* branches waiting
  2. Check for conflicts between them:
     - If no overlap: Can merge in parallel (using separate temp branches)
     - If overlap: Use topological sort (dependencies first)
  3. For each integration/* branch:
     - Run full test suite
     - Validate FR↔Test traceability (Phase 1 gate)
     - Validate no circular deps (Phase 2 gate)
  4. Merge to main in order (atomic, one at a time)
  5. Auto-cleanup integration/* branches

Result: Agents don't block on each other; service handles merge orchestration
```

#### Challenge 2: Concurrent Spec Conflicts

**Problem**: Two agents both define new FRs simultaneously; merge conflict in FUNCTIONAL_REQUIREMENTS.md.

**Current Workaround**: Manual conflict resolution; whoever reviews last wins.

**Solution - Spec Merge Service (Phase 1)**:

```
Agent 1: Adds FR-001-022 and FR-001-023 to FUNCTIONAL_REQUIREMENTS.md

Agent 2: Adds FR-001-024 and FR-001-025 (different sections)

Reconciliation Service:
  1. Detects both agents want to merge FUNCTIONAL_REQUIREMENTS.md
  2. Parses YAML/Markdown structure (non-overlapping sections)
  3. Performs semantic merge (not text merge):
     - Section 1 from Agent 1: FR-001-022, FR-001-023
     - Section 2 from Agent 2: FR-001-024, FR-001-025
  4. Merges into SPECS_REGISTRY.md (immutable, append-only)
  5. Both agents' specs accepted without conflict

No manual intervention; both agents' work preserved
```

#### Challenge 3: Circular Dependency Detection Delay

**Problem**: Agents don't know about circular deps until CI runs (10-20 min delay).

**Solution - Pre-commit Hook (Phase 2)**:

```
Agent updates Cargo.toml:
  phenotype-a = { path = "../crates/phenotype-a" }

Git hook runs:
  $ detect-circular-deps.py
  ✓ No cycles detected (OK to commit)

OR:
  ✗ Would create cycle: phenotype-a → phenotype-b → phenotype-a
    Suggested fix: Extract common module to phenotype-shared

Agent prevented from committing; cycle never reaches main
```

#### Challenge 4: Diverged Local vs Remote State

**Problem**: Agent's main branch is stale; feature branch is based on old commit.

**Solution - State Sync Notification Service (NEW)**:

```
Agent 1: Pushes feature/branch-1, opens PR at 14:30

System: origin/main advances (Agent 2 merges at 14:45)

System: Detects Agent 1's PR is now based on stale main
        Sends notification: "Your branch is 1 commit behind main"
        Provides auto-merge option: "Rebase on latest main" (1-click)

Agent 1: Clicks "Rebase", branch updated automatically
         No manual git rebase needed
```

### 3.2 Worktree Lifecycle State Machine

```
States:
  CREATED       → Feature branch created in .worktrees/<project>/<topic>
  ACTIVE        → Agent actively commits to this branch
  SUSPENDED     → Agent paused work; branch preserved
  MERGED        → All commits merged to main
  ARCHIVED      → Old branch moved to .archive/
  DELETED       → Branch permanently removed

Transitions:

CREATED ──→ ACTIVE (Agent makes first commit)
            ↓
        [commit loop]
            ↓
ACTIVE ──→ SUSPENDED (Agent calls: worktree-pause <branch>)
       ↓     ↓
       MERGED ARCHIVED (Agent calls: worktree-cleanup <branch>)
       ↓
    (deleted from remote)

Cleanup Policy:
  - MERGED branch: Auto-archive after 7 days (move to .archive/)
  - SUSPENDED branch: Warn after 30 days; delete after 60 days
  - ACTIVE branch: Keep indefinitely (agent owns lifecycle)

Registry Mapping:
  .worktrees/agileplus/feat/my-feature ──→ AgilePlus Spec ID: eco-001-WP07
  (Updated by: spec reconciliation service)
```

### 3.3 Conflict Resolution Decision Tree

```
When merging feature branch feat/my-feature to main:

┌─────────────────────────────────────────┐
│ Conflict Detected (git merge exit 1)    │
└─────────────────────────────────────────┘
        │
        ├─→ FUNCTIONAL_REQUIREMENTS.md?
        │   └─→ YES: Use spec merge service (Phase 1)
        │       - Extract new FRs from both branches
        │       - Assign sequential FR-IDs (no collision)
        │       - Merge specs into SPECS_REGISTRY.md
        │       - Auto-resolve ✓
        │
        ├─→ Cargo.toml or dependency file?
        │   └─→ YES: Check for circular deps (Phase 2)
        │       - If cycle detected: BLOCK merge, suggest refactor
        │       - If no cycle: Auto-merge workspace deps
        │       - Auto-resolve or BLOCK
        │
        ├─→ Code file (src/**/*.rs)?
        │   └─→ YES: Agent resolves manually
        │       - Merge conflict markers in code
        │       - Requires human judgment
        │       - Manual resolution required
        │
        └─→ Documentation file (*.md)?
            └─→ YES: Merge by sections (semantic merge)
                - If non-overlapping sections: Auto-merge
                - If same section: Concatenate with separator
                - Auto-resolve where possible
```

### 3.4 Agent Concurrency Limits

**Current**: 50+ agents can work in parallel (bounded by system resources)

**Safe Limits**:
- **Simultaneous merges to main**: 5 (prevents git lock contention)
- **Simultaneous integration/* branches**: Unlimited (no conflict)
- **Simultaneous spec conflicts**: 10+ (spec service can handle batches)
- **Simultaneous builds**: 3-5 (CI resource bottleneck)

**Throttling Strategy**:

```
Integration Queue (FIFO):
  integration/feature-1 → BUILDING (5 min) → QUEUED
  integration/feature-2 → QUEUED
  integration/feature-3 → QUEUED

When feature-1 merges to main:
  integration/feature-2 → BUILDING
  integration/feature-3 → QUEUED

No agent waits > 5 min per build step
All agents eventually merge without blocking
```

---

## Part 4: Branching Strategy

### 4.1 Three-Branch Model

| Branch | Purpose | Stability | Mutation Rules | Agent Access |
|--------|---------|-----------|----------------|--------------|
| `main` | Canonical production code | Immutable | Fast-forward only; no force-push | Read-only (via PR) |
| `specs/main` | Authoritative FR/ADR registry | Immutable | Append-only; no deletion | Read-only (via PR) |
| `.worktrees/*` | Feature/experiment work | Ephemeral | Agent-owned; force-push allowed | Read-write |
| `integration/*` | Staging for multi-agent merges | Ephemeral | Service-managed; auto-cleanup | Service-only |

### 4.2 Feature Branch Naming Convention

```
.worktrees/<project>/<category>/<name>

Examples:
  .worktrees/agileplus/feat/fr-consolidation
  .worktrees/phenotype-infrakit/fix/circular-deps
  .worktrees/thegent/refactor/auth-layer
  .worktrees/heliosCLI/docs/user-guide-expansion

Convention:
  <project>   = Target project or crate
  <category>  = Type: feat|fix|refactor|docs|chore|test
  <name>      = Kebab-case, descriptive (feature or bug name)

Maximum length: 80 chars total
Status tracking: Branch name maps to AgilePlus spec ID (via registry)
```

### 4.3 Merge Workflow

```
Step 1: Feature Development
  $ git checkout -b .worktrees/agileplus/feat/my-feature main
  $ git commit -am "feat: implement FR-001-022"
  $ git push origin .worktrees/agileplus/feat/my-feature

Step 2: Open Pull Request
  $ gh pr create --head .worktrees/agileplus/feat/my-feature --base main \
    --title "feat: Implement FR-001-022" \
    --body "Implements FR-001-022. Traces to WP-007 in AgilePlus."

Step 3: CI Validation
  [Automated]
  - Run tests
  - Validate FR↔Test traceability (Phase 1 gate)
  - Detect circular deps (Phase 2 gate)
  - Code review (CodeRabbit, Muse agent)

Step 4: Approval & Merge Staging
  [If all checks pass]
  Agent or Muse approves PR
  $ git checkout -b integration/my-feature main
  $ git merge --no-ff .worktrees/agileplus/feat/my-feature
  $ git push origin integration/my-feature

  [Merge Service now owns this branch]

Step 5: Orchestrated Merge to Main
  [Reconciliation Service]
  - Validate no conflicts with other integration/* branches
  - Run final test suite
  - Merge to main: git merge --no-ff integration/my-feature
  - Auto-delete integration/my-feature
  - Archive .worktrees/agileplus/feat/my-feature

Step 6: Cleanup
  [Automatic after 7 days]
  Move .worktrees/agileplus/feat/my-feature → .archive/2026-Q2/...
```

### 4.4 Integration Branch Orchestration

```
State Machine (Reconciliation Service):

  PR APPROVED
      ↓
  CREATE integration/* FROM main
      ↓
  VALIDATE (tests, deps, specs)
      ↓
  ┌───────────────────────────────┐
  │ Check for conflicts with      │
  │ other pending integration/*   │
  └───────────────────────────────┘
      │
      ├─→ CONFLICT DETECTED
      │   │
      │   └─→ Topological Sort
      │       (Merge dependencies first)
      │       ↓
      │   QUEUE this branch
      │       ↓
      │   WAIT for blocking PRs to merge
      │       ↓
      │   PROCEED when ready
      │
      └─→ NO CONFLICT
          │
          └─→ MERGE to main (immediately)
              ↓
          DELETE integration/*
              ↓
          ARCHIVE .worktrees/*

No agent waits for merge (service is deterministic)
All merges are atomic (no intermediate states visible)
```

### 4.5 Conflict Resolution Procedure

**Automatic** (no human intervention):

1. **Spec conflicts** (FUNCTIONAL_REQUIREMENTS.md)
   - Spec merge service (Phase 1)
   - Semantic merge, assign sequential IDs
   - Auto-resolve in 100% of cases

2. **Dependency conflicts** (Cargo.toml, package.json)
   - Circular dep detection (Phase 2)
   - If cycle: BLOCK merge, suggest refactor
   - If no cycle: Auto-merge

3. **Documentation conflicts** (*.md)
   - Semantic merge by sections
   - If non-overlapping: Auto-merge
   - If same section: Concatenate with separator

**Manual** (requires agent decision):

1. **Code conflicts** (src/**/*.rs, src/**/*.ts)
   - Agent resolves using git merge tools
   - Agent re-runs tests locally
   - Agent pushes resolved code

2. **Test failures** (appear during validation phase)
   - Agent diagnoses failure
   - Agent commits fix to feature branch
   - Feature branch re-validated

**Escalation** (requires team decision):

1. **Architectural conflicts**
   - Two agents propose incompatible designs
   - Escalate to team lead (decision via ADR)
   - ADR provides context for both approaches
   - Both agents refactor to agreed-upon design

2. **Business logic conflicts**
   - Two PRs implement contradictory features
   - Escalate to product/design
   - Resolution documented in PR

---

## Part 5: Reconciliation Service Requirements

### 5.1 Service Architecture

```
┌─────────────────────────────────────────┐
│  Reconciliation Service                 │
│  (runs every 5 minutes, or on webhook)  │
└─────────────────────────────────────────┘
        │
        ├─→ Phase 1: Spec Reconciliation Module
        │   ├─ Monitor: FUNCTIONAL_REQUIREMENTS.md changes
        │   ├─ Detect: Concurrent spec definitions
        │   ├─ Resolve: Assign FR-IDs, merge into SPECS_REGISTRY
        │   └─ Notify: Agents of assigned IDs
        │
        ├─→ Phase 2: Dependency Reconciliation Module
        │   ├─ Monitor: Cargo.toml, package.json changes
        │   ├─ Detect: Circular dependencies
        │   ├─ Resolve: Topological sort, merge order
        │   └─ Notify: Agents if merge is blocked
        │
        ├─→ Phase 3: Platform Chassis Module
        │   ├─ Monitor: @phenotype/docs, governance versions
        │   ├─ Detect: Breaking changes
        │   ├─ Resolve: Update contracts registry
        │   └─ Notify: Projects of breaking changes
        │
        └─→ Orchestration Layer
            ├─ Queue management (FIFO, topological)
            ├─ Atomic merges to main
            ├─ Cleanup (integration/*, .worktrees/*)
            └─ Audit logging (AUDIT_LOG.md)
```

### 5.2 Service API

```bash
# Trigger spec reconciliation manually
POST /api/reconcile/specs
  Body: { "branch": "feat/my-feature", "agent": "agent-1" }
  Response: { "status": "ok", "merged_ids": ["FR-001-022", "FR-001-023"] }

# Check merge readiness
GET /api/merge/integration/my-feature/readiness
  Response: {
    "ready": true,
    "tests_passing": true,
    "no_circular_deps": true,
    "fr_traceability": 100,
    "conflicts_with": []
  }

# Trigger merge to main
POST /api/merge/integration/my-feature
  Response: { "status": "merged", "commit": "abc123def", "time_ms": 450 }

# Archive worktree
POST /api/worktree/archive
  Body: { "path": ".worktrees/agileplus/feat/my-feature" }
  Response: { "status": "archived", "new_path": ".archive/2026-Q2/..." }

# Query spec registry
GET /api/specs?query=EventStore
  Response: {
    "specs": [
      { "id": "FR-001-001", "title": "Event sourcing", "version": "1.0.0" }
    ]
  }

# Get dependency graph
GET /api/deps/graph?format=mermaid
  Response: (Mermaid DAG markdown)
```

### 5.3 Service Deployment

**Technology Stack**:
- **Language**: Python 3.11+ (easy integration with GitPython, PyYAML)
- **Framework**: FastAPI (async, lightweight)
- **Persistence**: SQLite (local audit log) + Git (distributed state)
- **CI Integration**: GitHub Actions (webhook-triggered on PR events)

**Deployment Locations**:
1. **Local development**: `scripts/spec-reconciliation-service.py` (runs on agent machine)
2. **Shared CI**: GitHub Actions workflow `.github/workflows/reconciliation.yml`
3. **Scheduled**: Every 5 minutes (cron) or on webhook

**Failure Handling**:
- Service crash: Git operations are idempotent; re-run on restart
- Merge conflict (unexpected): Log to `AUDIT_LOG.md`, notify team lead
- Network failure: Retry with exponential backoff (3 attempts, max 5 min)
- Test failure: Don't merge; notify agent

---

## Part 6: Implementation Timeline & Phases

### 6.1 Detailed Phase Breakdown

#### Phase 1: Specs Canonicalization (Weeks 1-2)

**Week 1**:
- Days 1-2: Create `specs/main` branch, push to remote, configure protection
- Days 2-3: Build SPECS_REGISTRY.md, index all 26+ specs
- Days 3-4: Deploy spec merge service (local testing)
- Days 4-5: Implement FR↔Test traceability gate (CI check)
- Day 5: Train agents on specs/main workflow

**Week 2**:
- Days 1-2: Soft launch spec merge service (log only, don't merge)
- Days 2-3: Monitor conflicts, fix edge cases
- Days 3-4: Full rollout (service actively merges specs)
- Days 4-5: Validate 100% FR↔Test traceability
- Day 5: Phase 1 complete, document learnings

**Parallel Work** (1-2 agents per task):
- Resolve git conflict markers in CLAUDE.md, AGENTS.md, worklog.md
- Create AUDIT_LOG.md template
- Set up GitHub branch protection for specs/main

**Expected Deliverables**:
- specs/main branch (live on GitHub)
- SPECS_REGISTRY.md (26+ specs indexed)
- Spec merge service (Python script)
- FR↔Test gate (CI check)
- Updated AGENTS.md (new workflow)
- Audit log (initial entries)

---

#### Phase 2: Dependency Reconciliation (Weeks 3-6)

**Week 3**:
- Days 1-2: Build DEPENDENCY_GRAPH_CANONICAL.md
- Days 2-3: Enhance circular dep detection service
- Days 3-4: Test with 5+ synthetic scenarios
- Day 5: Document dependency update policy

**Week 4**:
- Days 1-2: Implement integration branch workflow
- Days 2-3: Build merge orchestration service (topological sort)
- Days 3-4: Test with 10+ concurrent merges
- Day 5: Validate zero false positives

**Week 5**:
- Days 1-2: Deploy integration branch protection
- Days 2-3: Auto-cleanup service for integration/*
- Days 3-4: Stress test with high concurrency (20+ agents)
- Day 5: Monitor for issues, refine

**Week 6**:
- Days 1-3: Polish, document, train agents
- Days 4-5: Phase 2 complete, metrics collection

**Parallel Work**:
- Generate dependency graph visualization (Mermaid)
- Set up Slack notifications for merge events
- Create runbooks for common conflict scenarios

**Expected Deliverables**:
- DEPENDENCY_GRAPH_CANONICAL.md (30+ projects)
- Enhanced circular dep detection (pre-commit hook)
- Integration branch workflow (documented)
- Merge orchestration service (Python)
- Dependency update policy (enforced in CI)
- Visualization (auto-updated, embedded in docs)

---

#### Phase 3: Platform Chassis Federation (Weeks 8-12)

**Week 8**:
- Days 1-3: Determine @phenotype/docs versioning strategy
- Days 3-5: Version thegent governance (v1.0.0 or current)

**Week 9**:
- Days 1-2: Deploy governance symlinks (repos/ → thegent/)
- Days 2-3: Test symlink resolution in CI
- Days 3-5: Document breaking change detection

**Week 10**:
- Days 1-3: Define platform contracts (error-core, event-sourcing, etc.)
- Days 3-5: Create PHENOTYPE_PLATFORM_CONTRACTS.md

**Week 11**:
- Days 1-2: Implement contract validation in CI
- Days 2-3: Build intent-driven module loading POC
- Days 3-4: Test module selection workflow
- Day 5: Document POC results

**Week 12**:
- Days 1-2: Implement AI-native health monitoring
- Days 2-3: Generate first health report
- Days 3-5: Polish, document, train agents

**Expected Deliverables**:
- Phenotype Docs Chassis v1.0.0 (stable)
- Governance symlinks (all projects)
- PHENOTYPE_PLATFORM_CONTRACTS.md (10+ contracts)
- Intent-driven module loading POC
- AI-native health monitoring (weekly reports)
- Full documentation + agent training

---

### 6.2 Resource Allocation

| Phase | Duration | Agents | Tool Calls | Parallelism | Risk |
|-------|----------|--------|-----------|-------------|------|
| Phase 1 | 2 weeks | 8 | 12-15 per agent | High | Low |
| Phase 2 | 3-4 weeks | 12 | 15-20 per agent | High | Medium |
| Phase 3 | 4-5 weeks | 15 | 20-30 per agent | High | Medium-High |
| **Total** | **10-12 weeks** | **15-20 (peak)** | **50+ concurrent** | **High** | **Medium** |

**Total Cost (Agent-Days)**: ~40-50 agent-days (equivalent to 10-12 weeks with 4-5 agents full-time)

---

## Part 7: Success Criteria & Metrics

### 7.1 Phase 1 Success Metrics

| Metric | Target | Threshold | Measurement |
|--------|--------|-----------|-------------|
| Specs indexed | 26+ | 100% | Count entries in SPECS_REGISTRY.md |
| FR↔Test coverage | 100% | >=98% | CI gate report |
| Spec merge conflicts/week | 0 | <=1 | AUDIT_LOG.md entries |
| Conflict auto-resolution rate | 100% | >=95% | Automated merges / total conflicts |
| Spec merge latency | <30s | <1min | Service timing logs |
| Agent adoption rate | 100% | >=80% | Agents using specs/main in PRs |

### 7.2 Phase 2 Success Metrics

| Metric | Target | Threshold | Measurement |
|--------|--------|-----------|-------------|
| Parallel merges supported | 5 simultaneously | >=3 | Integration/* branch limit |
| Merge latency (serial → parallel) | <10 min | <15 min | End-to-end merge time |
| Circular dep detection rate | 100% | >=99% | Detected before merge / introduced |
| False positive rate | 0% | <1% | False blocks / total blocks |
| Dependency graph freshness | <5 min | <10 min | Last updated timestamp |
| Agent wait time | 0 min | <5 min | Queue depth tracking |

### 7.3 Phase 3 Success Metrics

| Metric | Target | Threshold | Measurement |
|--------|--------|-----------|-------------|
| Docs Chassis version stability | 1.0.0 | >=1.0.0 | Package version |
| Breaking changes announced | 100% | >=95% | Documented / total |
| Governance repo drift | 0% | <5% | Diff between repos / canonical |
| Contract compliance | 100% | >=90% | Passing CI checks / total |
| Module loading latency | <100ms | <500ms | Time to select module |
| Platform adoption | 100% | >=80% | Projects using contracts |

### 7.4 Overall Ecosystem Health

```
BEFORE SSOT Architecture:
  - Git conflict markers in governance files 🔴
  - Merge conflicts 2-3x/week 🟠
  - Serial merge queue (5-30 min latency) 🟠
  - No spec versioning 🔴
  - Dependency graph manual (no automation) 🟠
  - Governance scattered (5 CLAUDE.md copies) 🔴

  Overall Health Score: 42/100 (red zone)

AFTER Phase 1:
  - Git conflicts resolved ✅
  - Spec merge service 100% auto-resolution ✅
  - Spec versioning scheme ✅
  - FR↔Test traceability 100% ✅

  Expected Health Score: 68/100 (yellow → orange zone)

AFTER Phase 2:
  - Parallel merge orchestration ✅
  - Circular dep detection pre-commit ✅
  - Integration branch federation ✅
  - Dependency graph automated ✅

  Expected Health Score: 82/100 (green zone)

AFTER Phase 3:
  - Platform contracts formalized ✅
  - Intent-driven module loading ✅
  - AI-native health monitoring ✅
  - Governance centralized ✅

  Expected Health Score: 95/100 (stable production)
```

---

## Part 8: Risk Management

### 8.1 Critical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Spec merge service creates cycles (specs reference each other) | Medium | High | Implement DAG validation in service; cycle detection in Phase 2 |
| Agent doesn't adopt specs/main workflow | High | High | Mandatory training; CI gate blocks non-compliant PRs |
| Integration branch merge order causes test failures | Medium | High | Topological sort + full test suite on each ordering attempt |
| Reconciliation service crashes during merge | Low | Critical | Idempotent git operations; audit log for recovery |
| Governance symlinks break in CI environments | Medium | Medium | Test symlinks in CI; fallback to file copies if broken |

### 8.2 Mitigation Strategies

**Spec Merge Cycles**:
- Phase 2 dependency validation catches cycles
- Circular dep detection runs before spec merge
- If cycle detected: Spec merge blocked, agent notified

**Agent Adoption**:
- Mandatory training session (30 min)
- CI gate enforces adoption (blocks merges without specs/main usage)
- Early feedback loop (Slack notifications for every spec merge)

**Integration Branch Failures**:
- Full test suite runs before merge (catch failures early)
- If tests fail: Revert integration/* merge, notify agent
- Maintain detailed log of what failed (for debugging)

**Reconciliation Service Crashes**:
- All git operations are idempotent (safe to retry)
- Audit log stored in Git (distributed backup)
- Automatic recovery: Service restarts, re-processes failed merges

**Governance Symlinks**:
- Use `git config --add core.symlinks true` for CI environments
- Fallback: If symlinks fail, read from thegent directly
- Test symlink resolution as part of CI setup

---

## Part 9: Appendices

### A. Glossary

| Term | Definition |
|------|-----------|
| **SSOT** | Single Source of Truth — authoritative registry preventing divergence |
| **Polyrepo** | Multiple independent projects in a single Git repository |
| **Worktree** | Feature branch directory under `.worktrees/` for agent feature work |
| **Integration Branch** | Ephemeral `integration/*` branch for staging multi-agent merges |
| **Spec Registry** | SPECS_REGISTRY.md — immutable, versioned FR/ADR/PLAN registry |
| **FR** | Functional Requirement — testable specification of system behavior |
| **ADR** | Architecture Decision Record — rationale for design choices |
| **Reconciliation** | Process of merging concurrent contributions without conflicts |
| **Topological Sort** | Ordering of dependencies such that dependencies come first |
| **Circular Dependency** | A → B → A — forbidden structural pattern |

### B. Related Documents

- `AGENTS.md` — Multi-agent coordination rules (to be updated with specs/main workflow)
- `CLAUDE.md` — Governance rules (to be consolidated with Phase 3 symlinks)
- `docs/reference/SPECS_REGISTRY.md` — Authoritative FR/ADR registry (Phase 1 deliverable)
- `docs/reference/DEPENDENCY_GRAPH_CANONICAL.md` — Canonical dependency registry (Phase 2 deliverable)
- `docs/reference/PHENOTYPE_PLATFORM_CONTRACTS.md` — Platform contracts (Phase 3 deliverable)
- `ADR.md` — Architecture decision records at repos root

### C. Service Implementation Checklist

**Phase 1 - Spec Reconciliation**:
- [ ] `scripts/spec-reconciliation-service.py` created
- [ ] Service detects concurrent FUNCTIONAL_REQUIREMENTS.md changes
- [ ] Service assigns sequential FR-IDs
- [ ] Service merges specs into SPECS_REGISTRY.md
- [ ] Service notifies agents of assigned IDs
- [ ] CI integration: spec merge on every PR

**Phase 2 - Dependency Reconciliation**:
- [ ] `scripts/detect-circular-deps.py` enhanced for pre-commit hook
- [ ] `scripts/merge-orchestration-service.py` created
- [ ] Service handles topological sorting
- [ ] Service manages integration/* branch lifecycle
- [ ] Service auto-cleanup (delete merged branches)
- [ ] CI integration: dep validation on every commit

**Phase 3 - Platform Chassis**:
- [ ] Governance symlinks deployed
- [ ] Contract validation CI gate
- [ ] Module loading POC
- [ ] Health monitoring service
- [ ] Weekly health report generation

### D. Training Materials (To Be Created)

1. **Agents Guide: Using specs/main**
   - 15 min video walkthrough
   - Example PR with specs/main usage
   - FAQ: "What if my spec ID collides?"

2. **Operations Guide: Merge Orchestration**
   - Understanding integration/* branches
   - Resolving topological conflicts
   - Debugging service failures

3. **Architecture: Platform Contracts**
   - What is a contract?
   - How to declare requirements
   - How to handle breaking changes

---

## Part 10: Conclusion & Next Steps

### 10.1 Why SSOT Architecture?

With 50+ agents working in parallel, **consensus must be automatic and deterministic**. Manual conflict resolution doesn't scale:

- Spec conflicts: 2-3x/week (manual → 5 min each)
- Merge queue serialization: 15-30 min per merge (parallel → <10 min)
- Governance divergence: Accumulated over months (centralized → real-time sync)

SSOT architecture replaces human choreography with **automated, conflict-aware systems** that:

1. **Prevent conflicts** (pre-commit hooks detect issues early)
2. **Auto-resolve conflicts** (spec merge service, dep topologizer)
3. **Enable parallelism** (integration branches decouple agents)
4. **Preserve audit trail** (AUDIT_LOG.md records all decisions)

### 10.2 Timeline to Production

- **Weeks 1-2**: Phase 1 ready for soft launch (log-only mode)
- **Weeks 3-4**: Phase 1 full rollout + Phase 2 begins
- **Weeks 5-6**: Phase 2 ready for integration test
- **Weeks 8-12**: Phase 3 exploration + stabilization

**Go-Live**: Phase 1 by end of Week 2 (minimum viable SSOT)

### 10.3 Success Indicators (First 2 Weeks)

- [ ] Zero git conflict markers in governance files
- [ ] `specs/main` branch live on GitHub with 26+ specs indexed
- [ ] First multi-agent spec merge executed automatically (0 manual intervention)
- [ ] All agents adopt specs/main workflow (100% of new PRs reference specs)
- [ ] FR↔Test traceability reaches 100%
- [ ] Phase 2 work begins (dependency graph canonicalization)

---

## Document Control

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-03-30 | Claude Code | Initial design: 3-phase SSOT architecture |

---

**Next Action**:
1. Share this document with team leads for feedback
2. Resolve git conflict markers in CLAUDE.md, AGENTS.md, worklog.md
3. Create `specs/main` branch and push to origin
4. Begin Phase 1 Week 1 deliverables
