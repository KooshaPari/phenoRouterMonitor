# CLAUDE.md — Phenotype Repos Shelf

## Shelf Identity

This is the **repos shelf** — a polyrepo containing ~150 independent projects.
Agents working in this shelf must understand the difference between **shelf-level** work and **project-level** work.

**Never treat this shelf as a single project.** Each subdirectory is an independent git repository. Always identify which project you're working in before taking action.

## Quick Start

1. **Find your project**: Check `projects/INDEX.md` for the full catalog
2. **Enter the project**: `cd <project-name>` before any work
3. **Read project docs**: Each project has its own `CLAUDE.md` and `AGENTS.md`
4. **Run commands inside**: Never run `cargo test` or `npm test` from shelf root

## Repository Consolidation Status


## Consolidation Status (As of 2026-04-03)

| Registry | Merged From | Strategy |
| Repository | Consolidated From | Status |
|------------|-------------------|--------|
| **hexagon** | hexagon-go, hexagon-rust, hexagon-zig, HexaKit/*, template-lang-* | ✅ Active - Unified template registry |
| **PhenoConfig** | Settly, phenotype-config, Configra, phenotype-config-ts | ✅ Active - Polyglot config |
| **Kogito** | Kogito + bifrost-extensions | ✅ Active - Bifrost extensions |
| **thegent** | thegent-mesh, thegent-plugin-host, thegent-shm, thegent-subprocess (decomposition pattern) | ✅ Active - Satellite workspace |
| **phenotype-infrakit** | + 6 crates from thegent (crypto, fs, jsonl, utils, resources, path) | ✅ Active - Generic primitives |
| **agentapi-plusplus** | agentapi, agentapi-deprec, agentapi-temp | ✅ Active - API layer |
| **cliproxyapi-plusplus** | CLIProxyAPI | ✅ Active - CLI proxy |
| **Tracely** | Traceon, helix-logging, helix-tracing, thegent-cache (features) | ✅ Active - Observability |
| **Stashly** | thegent-cache (crate) | ✅ Active - Caching |
| **Profila** | phenoSDK quality/ (in progress) | 🔄 In Progress - Quality + profiling |
| **Tasken** | phenoSDK lib/tasks.py (in progress) | 🔄 In Progress - Task execution |

## Decomposition & Libification Plans

### AgilePlus - External Library Adoption

AgilePlus is already well-decomposed (~47 workspace crates). The optimization is to **replace custom implementations with proven external libraries**:

| Custom | External | Benefit |
|--------|----------|---------|
| cipher (AES/GCM) | rustls + ring | Audited, maintained |
| gauge (benchmarking) | Criterion + iai | De facto standard |
| middleware | tower + tower-http | Ecosystem compatibility |
| runtime (WASM) | wasmtime | Production-grade |

**Plan**: `AgilePlus/DECOMPOSITION_PLAN.md`

### Tracera - Monolith to Libraries

Tracera is a **massive 88K+ file monolith** (Go + Python + TypeScript) that needs libification:

**Proposed Split**:
- `tracera-core-py` - Traceability engine (Python)
- `tracera-api-go` - REST/gRPC API (Go)
- `tracera-ui-ts` - React frontend (TypeScript)
- `tracera-agent-py` - AI agent framework (Python)
- `tracera/` - Hub repo (coordination, integration tests)

**Target**: 88K files → ~7K files (-92% reduction)

**Plan**: `Tracera/LIBIFICATION_PLAN.md`

### phenoSDK - Component Extraction (In Progress)

The October 2025 monolithic SDK is being extracted into specialized crates:
- Quality/Fitness → **Profila**
- Storage → **Stashly** ✅
- Tasks → **Tasken**
- Connection Pool → phenotype-connection

**Plan**: `phenoSDK/EXTRACTION_PLAN.md`
| Component | Target | Status | Notes |
|-----------|--------|--------|-------|
| phenoSDK quality → | **Profila** | 🔄 IN PROGRESS | Quality metrics merge |
| phenoSDK storage → | **Stashly** | ✅ COMPLETE | Already merged |
| phenoSDK tasks → | **Tasken** | 🔄 IN PROGRESS | Task management |
| phenoSDK analytics → | phenotype-analytics-py | ⏳ PLANNED | Data structures |
| phenoSDK connection → | phenotype-connection | ⏳ PLANNED | Connection pooling |

**Goal:** Break down monolithic SDK into focused specialized crates

### Thegent Crate Extraction Candidates

| thegent Crate | Extraction Target | Status |
|---------------|-------------------|--------|
| thegent-crypto | phenotype-crypto | ⏳ Generic primitive candidate |
| thegent-fs | phenotype-fs | ⏳ Generic primitive candidate |
| thegent-jsonl | phenotype-jsonl | ⏳ Generic primitive candidate |
| thegent-utils | phenotype-utils | ⏳ Generic primitive candidate |
| thegent-path-resolve | phenotype-path | ⏳ Generic primitive candidate |
| thegent-parser | - | ✅ Keep in thegent (agent-specific) |
| thegent-discovery | - | ✅ Keep in thegent (agent-specific) |
| thegent-hooks | - | ✅ Keep in thegent (agent-specific) |

### Deleted Empty Shells

- pheno* directories (phenoCipher, phenoConfigTs, etc.) - phenotype-* equivalents exist
- thegent-* satellite directories (merged into thegent)

---


```
repos/                          # ← YOU ARE HERE (shelf root)
├── apps/                       # Application projects (user-facing)
├── libs/                       # Shared libraries (internal packages)
├── tooling/                    # Developer tools, CLIs, scripts
├── infra/                      # Infrastructure, deployment, devops
├── platforms/                  # Platform-as-product projects
├── crates/                     # Rust workspace members
├── packages/                   # JS/TS monorepo packages
├── docs/                       # Cross-project documentation
├── scripts/                    # Cross-project utility scripts
├── governance/                 # Governance tooling
├── projects/                   # Project catalog & metadata
│   └── INDEX.md               #   Master project list
├── WORKSTORES.md               # Worktree management guide
├── CLAUDE.md                   # ← This file
└── AGENTS.md                   # Agent interaction rules
```

### Project Categories

| Directory | Purpose | Examples |
|-----------|---------|----------|
| `apps/` | User-facing applications | heliosCLI, phenodocs |
| `libs/` | Shared libraries | Authvault, Stashly, Tracely |
| `tooling/` | Developer tools | Cmdra, Kogito, worktree-manager |
| `platforms/` | Platform-as-product | thegent, AgilePlus |
| `crates/` | Rust workspace | phenotype-* crates |
| `packages/` | JS/TS packages | @phenotype/* |
| `agent.s/` | Agent framework | Multi-agent orchestration |

## Agent Rules

**READ `AGENTS.md` FIRST.** It contains the authoritative agent interaction rules for this shelf.

Key points:
- When working on a project, `cd` into its directory first (e.g., `cd heliosCLI`)
- Never assume a project is at shelf root — always verify
- Test commands must run inside the target project directory, not shelf root
- File reads should specify the correct relative path from shelf root

### Working at Project Level

```bash
cd <project-name>    # e.g., cd heliosCLI
pwd                   # Always verify
```

- Run all tests from inside the project directory
- All file paths are relative to the project, not the shelf
- Dotfiles (`.gitignore`, `.claude/`, etc.) are project-specific

### Working at Shelf Level

Shelf-level work is rare and includes:
- Organizing the shelf structure
- Cross-project governance
- Audit/investigation across multiple projects
- Creating or deleting projects

### Never Do This

- Run `npm test` or `cargo test` from shelf root — it's not a project
- Assume a file exists because you saw it in another project
- Create files at shelf root for project-specific work

## Naming Conventions

### Session/Conversation Naming

Format: `<project>:<brief-task-description>`
- Good: `heliosCLI:auth-refactor`, `shelf:duplication-audit`
- Bad: `fix`, `implementation`, `agent work`

### File Naming in Sessions

Format: `<project>-<YYYYMMDD>-<task>-<version>.md`
- Good: `heliosCLI-20260329-cli-redesign-v1.md`
- Location: `worktrees/<project>/sessions/` or `shelf-level: plans/`

### Branch Naming

Format: `<project>/<type>/<description>`
- Good: `heliosCLI/feat/token-refresh`, `thegent/chore/update-deps`
- Bad: `feature-branch`, `fix`

## Quick Reference

| What you need | Where to look |
|---------------|---------------|
| Project list | `projects/INDEX.md` |
| Governance rules | `AGENTS.md` |
| Architecture decisions | `docs/adr/` |
| Cross-project scripts | `scripts/` |
| Worktree guide | `WORKSTORES.md` |

## Phenotype Federated Hybrid Architecture

This shelf is part of the **Phenotype Federated Hybrid Architecture**, which provides two complementary chassis systems:

### Phenotype Docs Chassis

Provides VitePress configuration, design tokens, and theme components for consistent documentation across the organization.

- **Location**: `@phenotype/docs` (GitHub Packages)
- **Documentation**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`

### AgilePlus Governance Chassis

Defines specification-driven delivery framework: PRD, ADR, FUNCTIONAL_REQUIREMENTS, PLAN, USER_JOURNEYS, with FR traceability and worklog integration.

- **Location**: AgilePlus project
- **Documentation**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`

## Tool Usage Policy

### Read-Only Tools First

Before using write/edit tools:
1. Use `Read` to understand the current state
2. Use `fs_search`/`sem_search` to locate code
3. Use `shell` (read-only commands) to verify state

### Write Tools

Use `patch`/`write` only when:
- You've read the file first
- You understand the full context
- You can explain what you're changing before changing it

### Shell Tools

- Always use `cwd` parameter, never `cd`
- Use `--` to separate file paths from arguments
- Never chain `cd` commands
- Use absolute paths when possible

## Multi-Agent Coordination

### Primary Agent Selection

| Task type | Primary agent |
|-----------|---------------|
| Feature implementation | Forge |
| Code review | Muse |
| Bug investigation | Sage |
| Testing/runtime | Helios |
| Cross-project architecture | Forge + Sage |
| Research/investigation | Sage |
| Documentation | Forge (with Muse review) |

### Coordination Protocol

1. **Single thread**: One agent owns a task at a time
2. **Handoff**: When switching agents, summarize state in conversation
3. **No overwrites**: If Agent B joins a task Agent A started, Agent B reads conversation history before taking action
4. **Conflict resolution**: If two agents conflict, pause and get user decision

## Session Documentation

All agents MUST maintain session documentation for research, decisions, and findings:

### Location

- Default: `docs/sessions/<session-id>/`

### Standard Session Structure

```
docs/sessions/<session-id>/
├── README.md           # Overview and context
├── 01_RESEARCH.md      # Findings and analysis
├── 02_PLAN.md          # Design and approach
├── 03_IMPLEMENTATION.md # Code changes and rationale
├── 04_VALIDATION.md    # Tests and verification
└── 05_KNOWN_ISSUES.md  # Blockers and follow-ups
```

## Commit/PR Policy

### Commit Messages

Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`

Good: `feat(heliosCLI): add token refresh with exponential backoff`
Bad: `fix stuff`, `update`, `WIP`

### PR Guidelines

- One logical change per PR
- PR title matches commit format
- Description explains WHY, not just WHAT
- Always link related issues

### Force Push Policy

**NEVER force push to `main`, `master`, or shared long-lived branches.**
Force push is acceptable ONLY for:
- Personal feature branches
- Your own worktrees
- Branches you're sure no one else is using

## Governance Reference

See thegent governance base for complete guidance on:
1. **Core Agent Expectations** — Autonomous operation, when to ask vs. decide
2. **Standard Operating Loop (SWE Autopilot)** — Review, Research, Plan, Execute, Size-Check, Test, Review & Polish, Repeat
3. **File Size & Modularity Mandate** — ≤500 line hard limit, decomposition patterns
4. **Research-First Development** — Codebase research, web research, documentation
5. **Branch Discipline** — Worktree usage, PR workflow, git best practices

Location: `platforms/thegent/dotfiles/governance/CLAUDE.base.md`

## Quick Command Reference

```bash
# List all projects
ls projects/INDEX.md  # then cat it

# Check if directory is a git repo
ls <dir>/.git 2>/dev/null && echo "GIT REPO" || echo "NOT A REPO"

# List worktrees
ls .worktrees/

# Add worktree
git worktree add .worktrees/<name> -b <branch>

# Remove worktree
git worktree remove .worktrees/<name>
```

---

*Last updated: 2026-04-03 — Post-consolidation cleanup*
