# Directory Reorganization & Traversability Strategy

**Phenotype Repos Shelf — Structure Optimization for 30+ Projects**

**Status:** 2026-03-30 | **Scope:** `/Users/kooshapari/CodeProjects/Phenotype/repos/` | **Priority:** HIGH

---

## Executive Summary

**Current State:** 30+ projects scattered across flat/semi-organized top-level dirs. Traversability poor. Onboarding slow.

```
repos/
├── AgilePlus/                  # 24 crates, 63K LOC
├── heliosCLI/                  # Large project
├── phenotype-infrakit/         # Rust workspace
├── agent-wave/                 # Unclear purpose
├── phench/                     # Python project
├── platforms/
│   └── thegent/                # 5.34M LOC (huge)
├── .archive/                   # Archive (embedded git repos)
├── .worktrees/                 # Worktree staging
├── apps/                       # ???
├── libs/                       # ???
├── infra/                      # ???
├── crates/                     # ???
├── packages/                   # ???
├── docs/                       # Cross-project docs
├── scripts/                    # Cross-project scripts
├── governance/                 # Governance tooling
├── projects/                   # Project catalog
└── [11 other dirs]             # Unclear purpose, low activity
```

**Problem:**
- No clear metadata about each project (type, tier, status, owners)
- Hard to find "where should I add a new web component?" (apps/? libs/? packages/?)
- New agents can't quickly understand project landscape
- `.archive/` has embedded git repos (hangs on `git add -A`)
- Worktrees scattered across multiple dirs (`.worktrees/`, `PROJECT-wtrees/`, `repo-wtrees/`)

**Solution:** Implement **layered organization + metadata** with:
1. **Project tiers** (Tier-1 canonical, Tier-2 infrastructure, Tier-3 experimental)
2. **Project metadata** (`PROJECT.yml` at each root)
3. **Directory taxonomy** (clear naming → clear purpose)
4. **Quick navigation** (`PROJECTS_INDEX.md` machine-readable)
5. **Traversal helpers** (navigation scripts, VS Code workspace, tree commands)

---

## Part 1: Target Directory Structure

### 1.1 Top-Level Organization (After Reorganization)

```
repos/
│
├── canonical/                     # ← TIER-1 PROJECTS (production, stable)
│   ├── phenotype-infrakit/        # Rust workspace, published crates
│   ├── AgilePlus/                 # Spec-driven delivery framework
│   ├── platforms/thegent/         # Go monorepo, agent platform
│   └── PROJECT.yml                # Metadata for this tier
│
├── infrastructure/                # ← TIER-2 PROJECTS (enablement, shared)
│   ├── heliosCLI/                 # CLI framework
│   ├── phenotype-docs/            # Documentation chassis
│   ├── phenotype-design/          # Design system (if separate)
│   └── PROJECT.yml
│
├── experimental/                  # ← TIER-3 PROJECTS (research, prototypes)
│   ├── agent-wave/                # New agent coordination
│   ├── phench/                    # Performance benchmarking
│   ├── bifrost-extensions/        # Browser extensions
│   └── PROJECT.yml
│
├── ecosystem/                     # ← THIRD-PARTY / INTEGRATED PROJECTS
│   ├── phenotype-shared/          # Shared libs across repos
│   ├── civ/                       # CI/CD validation
│   ├── phenotype-design-system/   # If external
│   └── PROJECT.yml
│
├── archived/                      # ← DEPRECATED / REFERENCE ONLY (no active dev)
│   ├── old-agent-framework/       # Archived, reference only
│   ├── legacy-dashboard/          # No longer maintained
│   └── PROJECT.yml (status: archived)
│
├── infrastructure-libs/           # ← SHARED LIBRARIES (cross-repo reuse)
│   ├── phenotype-error-core/      # Error types
│   ├── phenotype-health/          # Health checks
│   ├── phenotype-config-core/     # Config management
│   └── phenotype-contracts/       # Shared contracts
│
├── .worktrees/                    # ← CANONICAL WORKTREE STAGING
│   ├── canonical/                 # Worktrees for Tier-1 projects
│   │   ├── phenotype-infrakit/
│   │   ├── AgilePlus/
│   │   └── thegent/
│   ├── infrastructure/            # Worktrees for Tier-2
│   ├── experimental/              # Worktrees for Tier-3
│   └── .gitignore (ignore *.lock, .node_modules)
│
├── .archive/                      # ← EMBEDDED GIT REPOS (read-only reference)
│   ├── removed-crates/            # Old Rust crates (archived, not active)
│   ├── old-specs/                 # Completed specs (reference)
│   └── ARCHIVE_MANIFEST.md        # What's here + why
│
├── docs/                          # ← CROSS-PROJECT DOCUMENTATION (unchanged)
│   ├── reference/                 # All reference docs (including new cloud agent docs)
│   ├── guides/                    # How-to guides
│   ├── reports/                   # Completion reports
│   ├── research/                  # Research summaries
│   └── worklogs/                  # Session worklogs
│
├── governance/                    # ← GOVERNANCE TOOLING (unchanged)
│   ├── CLAUDE.md
│   ├── AGENTS.md
│   ├── ADRs/
│   └── policies/
│
├── scripts/                       # ← CROSS-PROJECT UTILITIES (unchanged)
│   ├── project-discovery.sh       # Find all projects
│   ├── cost-report.sh             # Cloud agent cost analysis
│   ├── tree-command.sh            # Formatted tree output
│   └── workspace-setup.sh         # Onboarding helpers
│
├── projects/                      # ← PROJECT CATALOG (unchanged)
│   ├── INDEX.md                   # Master project list
│   └── [per-project metadata]
│
├── REPOS_INDEX.md                 # ← NAVIGATION HUB (master index, machine-readable)
├── DIRECTORY_STRUCTURE.md         # ← THIS DOCUMENT + ASCII tree
├── PROJECT_DISCOVERY.md           # ← Guide for finding projects
└── WORKSPACE.code-workspace       # ← VS Code workspace config (root-level projects)
```

### 1.2 Project Tier Classification

**Tier-1 (Canonical, Production):** 3-5 core projects
```
Criteria:
- Published or widely-used
- Stable API
- Active maintenance
- >100K LOC or critical infrastructure
- 2+ external consumers

Examples:
- phenotype-infrakit (published Rust crates)
- AgilePlus (spec framework for org)
- platforms/thegent (agent orchestration)
```

**Tier-2 (Infrastructure, Enablement):** 5-8 projects
```
Criteria:
- Internal tooling/frameworks
- Supports Tier-1 projects
- Well-documented API
- 10K-100K LOC
- 1+ internal consumers

Examples:
- heliosCLI (execution framework)
- phenotype-docs (docs chassis)
```

**Tier-3 (Experimental, Research):** 5-10 projects
```
Criteria:
- Proof-of-concept or prototype
- May not have stable API
- Active research or development
- <10K LOC
- 0-1 consumers (exploratory)

Examples:
- agent-wave (new agent patterns)
- phench (benchmark harness)
```

**Archived (Reference Only):** N/A
```
Criteria:
- No active development
- Kept for reference/history
- Read-only (no commits)
- In .archive/ or deprecated/ branch

Examples:
- Old CLI frameworks
- Completed research projects
```

---

## Part 2: Project Metadata System

### 2.1 PROJECT.yml (Per-Project)

**Location:** Root of each project

```yaml
# phenotype-infrakit/PROJECT.yml
metadata:
  id: phenotype-infrakit
  name: "Phenotype Infrastructure Kit"
  tier: tier-1
  status: stable
  published: true

description: |
  Generic Rust infrastructure crates extracted from Phenotype ecosystem.
  Provides: error handling, health checks, config management, caching, state machines.
  Used by: AgilePlus, thegent, other Rust projects.

technology:
  language: rust
  edition: 2021
  workspace_members: 8
  loc: "~3.9K"

ownership:
  maintainers:
    - kooshapari
  reviewers:
    - team-rust
  slack_channel: "#infrastructure"

deployment:
  published_to: crates.io
  crate_name: phenotype-*
  docs_url: https://docs.rs/phenotype-infrakit/

versioning:
  strategy: semver
  current_version: v0.2.0
  last_release: 2026-03-29

ci_status: passing
  build: ✓
  tests: ✓
  security: ✓
  docs: ✓

links:
  github: https://github.com/KooshaPari/phenotype-infrakit
  docs: docs/
  api_reference: https://docs.rs/phenotype-infrakit/latest/phenotype/
  changelog: CHANGELOG.md
```

### 2.2 TIER_METADATA.yml (Per Tier)

**Location:** `canonical/PROJECT.yml`, `infrastructure/PROJECT.yml`, etc.

```yaml
# canonical/PROJECT.yml
tier: canonical
tier_name: "Canonical (Tier-1) Projects"
tier_description: "Production-ready, stable APIs, published to public registries"

projects:
  - id: phenotype-infrakit
    name: "Phenotype Infrastructure Kit"
    status: stable
    worktree: .worktrees/canonical/phenotype-infrakit/

  - id: agileplus
    name: "AgilePlus"
    status: stable
    worktree: .worktrees/canonical/AgilePlus/

  - id: thegent
    name: "The Agent Platform"
    status: stable
    worktree: .worktrees/canonical/thegent/

slas:
  availability: 99%
  response_time_critical: <1h
  response_time_normal: <4h

governance:
  required_approvals: 2
  security_scanning: required
  code_coverage_min: 80%
  breaking_changes: requires_major_version_bump
```

---

## Part 3: Navigation & Discovery

### 3.1 REPOS_INDEX.md (Master Navigation Hub)

**Location:** `repos/REPOS_INDEX.md`

```markdown
# Phenotype Repos: Master Index & Navigation

## Quick Start

- **New to Phenotype?** Start here: [PROJECT_DISCOVERY.md](PROJECT_DISCOVERY.md)
- **Find a project:** [Full Project List](#full-project-list-by-tier)
- **Need a feature?** [Technology Stack Guide](#technology-stack-guide)
- **Setup workspace:** [Workspace Setup](#workspace-setup)

---

## Full Project List by Tier

### Tier-1: Canonical (Production-Ready)

| Project | Type | Status | LOC | Worktree |
|---------|------|--------|-----|----------|
| **phenotype-infrakit** | Rust Workspace | Stable | 3.9K | `.worktrees/canonical/phenotype-infrakit/` |
| **AgilePlus** | Spec Framework | Stable | 63K | `.worktrees/canonical/AgilePlus/` |
| **thegent** | Go Monorepo | Stable | 9.8M | `.worktrees/canonical/thegent/` |

**Characteristics:**
- Published APIs (crates.io, npm, etc.)
- >80% test coverage
- Security scanning enabled
- 2+ external consumers
- Active maintenance

---

### Tier-2: Infrastructure (Enablement)

| Project | Type | Status | LOC | Worktree |
|---------|------|--------|-----|----------|
| **heliosCLI** | CLI Framework | Active | ~5K | `.worktrees/infrastructure/heliosCLI/` |
| **phenotype-docs** | Doc System | Active | varies | `.worktrees/infrastructure/phenotype-docs/` |

**Characteristics:**
- Internal-facing tools
- 1+ Tier-1 consumer
- Emerging stable API
- <80% test coverage accepted
- Regular maintenance

---

### Tier-3: Experimental (Research)

| Project | Type | Status | LOC | Worktree |
|---------|------|--------|-----|----------|
| **agent-wave** | Agent Research | Active | ~2K | `.worktrees/experimental/agent-wave/` |
| **phench** | Benchmark | Active | 3.8K | `.worktrees/experimental/phench/` |

**Characteristics:**
- Proof-of-concept or prototype
- API subject to change
- Exploratory research
- 0-1 consumer
- Optional maintenance

---

## Technology Stack Guide

**"I need to add a React component. Where does it go?"**
```
Feature: React/UI component
└─ Check: Is it sharable across projects?
   ├─ YES → packages/ui-shared/ or @phenotype/ui (npm)
   ├─ NO → Ask in #engineering
   └─ Decision: Update docs/guides/UI_ARCHITECTURE.md
```

**"I need to create a new Rust crate. Where?"**
```
Feature: Rust crate for infrastructure
└─ Check: Is it generalizable?
   ├─ YES → infrastructure-libs/ (phenotype-*)
   ├─ NO → Add to existing workspace (AgilePlus/crates/, phenotype-infrakit/crates/)
   └─ Publishing: Publish to crates.io if >1 consumer
```

**"I need to create a new CLI. Where?"**
```
Feature: New CLI tool
└─ Check: Is it a platform-level tool?
   ├─ YES → infrastructure/heliosCLI/ (or new framework)
   ├─ NO → Embed in project (AgilePlus/agileplus-cli/, etc.)
   └─ Publishing: Publish to npm/cargo if >1 consumer
```

---

## Workspace Setup

### VS Code Multi-Root Workspace

**File:** `repos/WORKSPACE.code-workspace`

```json
{
  "folders": [
    {
      "path": "canonical/phenotype-infrakit",
      "name": "phenotype-infrakit"
    },
    {
      "path": "canonical/AgilePlus",
      "name": "AgilePlus"
    },
    {
      "path": "canonical/platforms/thegent",
      "name": "thegent"
    },
    {
      "path": "infrastructure/heliosCLI",
      "name": "heliosCLI"
    }
  ],
  "settings": {
    "python.linting.enabled": true,
    "rust.clippy_warn_all": true,
    "go.lintOnSave": "package"
  }
}
```

**Usage:**
```bash
cd repos/
code WORKSPACE.code-workspace
# Opens all 4 projects in single VS Code window
```

### Command-Line Helpers

**Find all projects:**
```bash
./scripts/project-discovery.sh
# Output: List of all projects with metadata
```

**Tree view:**
```bash
./scripts/tree-command.sh --depth=2
# Output: Formatted ASCII tree
```

**Get project metadata:**
```bash
./scripts/project-metadata.sh phenotype-infrakit
# Output: PROJECT.yml for that project
```

---

## Migration Plan (If Reorganizing)

### Phase 1: Create New Structure (Non-Destructive)
```bash
mkdir -p canonical/
mkdir -p infrastructure/
mkdir -p experimental/
mkdir -p infrastructure-libs/
mkdir -p .worktrees/{canonical,infrastructure,experimental}/
```

### Phase 2: Copy Projects (Keep Originals)
```bash
# Don't move; copy first, verify, then remove originals
cp -r phenotype-infrakit/ canonical/phenotype-infrakit/
cp -r AgilePlus/ canonical/AgilePlus/
# ... repeat for all projects
```

### Phase 3: Update All References
```bash
# Find all .git/config that reference old paths
grep -r "phenotype-infrakit" . --include=".git"

# Update CI/CD workflows
# Update CLAUDE.md references
# Update workspace paths
```

### Phase 4: Update Git Worktrees
```bash
# Rename existing worktrees
git worktree list  # See current
git worktree move phenotype-infrakit/ .worktrees/canonical/phenotype-infrakit/
```

### Phase 5: Verify & Commit
```bash
# All tests pass
# All CI checks pass
# Document migration in MIGRATION_LOG.md
# Create PR
```

---

## Current Pain Points & Solutions

| Pain Point | Current State | Solution |
|-----------|---------------|----------|
| "Where should I add X?" | No clear dir taxonomy | Technology Stack Guide + decision tree |
| Onboarding slow | 30+ projects scattered | PROJECT_DISCOVERY.md + metadata system |
| Hard to understand project landscape | No metadata | PROJECT.yml per project + REPOS_INDEX.md |
| Worktrees in 3 different locations | `.worktrees/`, `PROJECT-wtrees/`, `repo-wtrees/` | Consolidate in `.worktrees/<tier>/<project>/` |
| `.archive/` hangs git | Embedded git repos | Move to separate branch or read-only submodule |
| No cost tracking per project | Manual tracking | Add cost center to PROJECT.yml |
| New agents can't find projects | No machine-readable metadata | PROJECT.yml YAML schema + discovery script |

---

## Files to Create/Update

| File | Purpose | Owner | Status |
|------|---------|-------|--------|
| `REPOS_INDEX.md` | Master navigation hub | You | Create |
| `PROJECT_DISCOVERY.md` | Onboarding guide | You | Create |
| `DIRECTORY_STRUCTURE.md` | ASCII tree + rationale | You | Create |
| `canonical/PROJECT.yml` | Tier-1 metadata | You | Create |
| `infrastructure/PROJECT.yml` | Tier-2 metadata | You | Create |
| `experimental/PROJECT.yml` | Tier-3 metadata | You | Create |
| `phenotype-infrakit/PROJECT.yml` | Per-project metadata | You | Create |
| `AgilePlus/PROJECT.yml` | Per-project metadata | You | Create |
| `thegent/PROJECT.yml` | Per-project metadata | You | Create |
| `.github/workflows/validate-project-metadata.yml` | CI validation | CI Team | Create |
| `WORKSPACE.code-workspace` | VS Code workspace | You | Create |
| `scripts/project-discovery.sh` | Navigation helper | You | Create |
| `scripts/tree-command.sh` | Formatted output | You | Create |
| `scripts/project-metadata.sh` | Query metadata | You | Create |

---

## References

- **CLAUDE.md:** Governance rules (applies to all projects)
- **AGENTS.md:** Agent interaction guidelines per project
- **PROJECT_DISCOVERY.md:** Detailed onboarding guide (to be created)
- **Tech Stack Governance:** `docs/governance/language_governance_framework.md`

---

**Document Version:** 1.0
**Last Updated:** 2026-03-30
**Status:** Ready for Implementation
