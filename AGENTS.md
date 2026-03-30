<<<<<<< HEAD
# AGENTS.md — repos shelf agent rules

## Shelf Identity

This is the **repos shelf** — a polyrepo containing ~30 independent projects.
Agents working in this shelf must understand the difference between **shelf-level**
work and **project-level** work.

**Never treat this shelf as a single project.** Each subdirectory is an independent
git repository. Always identify which project you're working in before taking action.

## Agent Self-Reference

| Agent | Role | Default location |
|-------|------|-----------------|
| Forge | Main coding/impl agent | `@shelf/*` |
| Muse | Code review/quality | `@shelf/*` |
| Sage | Research/investigation | `@shelf/*` |
| Helios | Runtime/testing specialist | `@shelf/*` |

Agents should identify themselves at the start of significant tasks:
```
⏺ [HH:MM:SS] AgentName working on <project>:<task>
```

## Project Navigation Rules

### DO THIS FIRST
When starting any task, identify the project:
1. Does the user mention a project name? Use it.
2. Does the working directory contain a `.git/` subdirectory? You're in a project.
3. Does the task involve a file path like `heliosCLI/...` or `thegent/...`? Use that project.
4. Otherwise, assume shelf-level work.

### WORKING IN A PROJECT
```
cd <project-name>    # e.g., cd heliosCLI
pwd                   # Always verify
```
- Run all tests from inside the project directory
- All file paths are relative to the project, not the shelf
- Dotfiles (`.gitignore`, `.claude/`, etc.) are project-specific
=======
<!-- Base: platforms/thegent/governance/AGENTS.base.md -->
<!-- Last synced: 2026-03-29 -->

# AGENTS.md — phenotype-infrakit

Extends thegent governance base. See `platforms/thegent/governance/AGENTS.base.md` for canonical definitions of agent expectations, testing requirements, research patterns, and standard operating procedures.

## Project Identity & Work Management

### Project Overview

- **Name**: phenotype-infrakit
- **Description**: Rust workspace containing generic infrastructure crates extracted from the Phenotype ecosystem
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/`
- **Language Stack**: Rust (edition 2021)
- **Published**: Internal (shared across Phenotype org)

### AgilePlus Integration

All work MUST be tracked in AgilePlus:
- Reference: `/Users/kooshapari/CodeProjects/Phenotype/repos/.agileplus`
- CLI: `agileplus <command>` (from .agileplus directory)
- Specs: `.agileplus/specs/<feature-id>/`
- Worklog: `.agileplus/worklog.md`

**Requirements**:
1. Check for AgilePlus spec before implementing
2. Create spec for new work: `agileplus specify --title "<feature>"`
3. Update work package status as work progresses
4. No code without corresponding AgilePlus spec
>>>>>>> origin/main

### WORKING AT SHELF LEVEL
Shelf-level work is rare and includes:
- Organizing the shelf structure
- Cross-project governance
- Audit/investigation across multiple projects
- Creating or deleting projects

<<<<<<< HEAD
### NEVER DO THIS
- Run `npm test` or `cargo test` from shelf root — it's not a project
- Assume a file exists because you saw it in another project
- Create files at shelf root for project-specific work

## Naming Conventions

### Session/Conversation Naming
Format: `<project>:<brief-task-description>`
Good: `heliosCLI:auth-refactor`, `shelf:duplication-audit`
Bad: `fix`, `implementation`, `agent work`

### File Naming in Sessions
Format: `<project>-<YYYYMMDD>-<task>-<version>.md`
Good: `heliosCLI-20260329-cli-redesign-v1.md`
Location: `worktrees/<project>/sessions/` or `shelf-level: plans/`

### Branch Naming
Format: `<project>/<type>/<description>`
Good: `heliosCLI/feat/token-refresh`, `thegent/chore/update-deps`
Bad: `feature-branch`, `fix`

## Tool Usage Policy

### Read-Only Tools First
Before using write/edit tools:
1. Use `Read` to understand the current state
2. Use `fs_search`/`sem_search` to locate code
3. Use `shell` (read-only commands) to verify state

### Write Tools
Use `FSWrite`/`patch` only when:
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
3. **No overwrites**: If Agent B joins a task Agent A started, Agent B reads
   conversation history before taking action
4. **Conflict resolution**: If two agents conflict, pause and get user decision

## Ghostty/Forge.Terminal Policy

**NEVER**: Open or interact with terminal applications (ghostty, terminal multiplexers)
as part of an agent session. These are USER-ONLY.

**IF YOU SEE ghostty in traces**: Ignore and continue with your work. Do not
attempt to control or close it. It belongs to the user.

**IF YOU MADE `forge.real` or similar**: Do NOT touch it. Delete it if it causes
crashes. The real Forge session is separate from any agent-created instances.

## Project-Specific Agent Rules

Each project may have its own `AGENTS.md` or `.claude/rules/` directory.
**Project rules override shelf rules** for work inside that project.

When a project's rules conflict with shelf rules, prefer:
1. Project rules for work inside that project
2. Shelf rules for shelf-level or cross-project work

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

## Error Handling

### Rate Limits (429)
When encountering API rate limits:
1. Stop immediately — do not retry
2. Report the limit type to user
3. Wait for user instruction

### Crashes
If an agent crashes mid-task:
1. Save state to conversation
2. Report what was in progress
3. Wait for user or another agent to resume

### Tool Failures
If a tool fails repeatedly:
1. Try alternative approach
2. Report the failure with context
3. Ask user for guidance

## File Organization Responsibilities

### Shelf-Level (what you control here)
- `projects/INDEX.md` — master project list
- `docs/adr/` — architecture decisions
- `scripts/` — cross-project utilities
- `governance/` — governance tooling
- `.worktrees/` — worktree management

### Project-Level (each project controls its own)
- `CLAUDE.md`, `AGENTS.md` in project root
- `.claude/rules/` in project root
- README, CONTRIBUTING per project
- Dotfiles per project

## Onboarding a New Project

When adding a new project to the shelf:

1. Create project directory at shelf root: `mkdir newproject`
2. Add entry to `projects/INDEX.md`
3. Create minimal `CLAUDE.md` and `README.md` in project
4. Initialize git if needed: `git init && git remote add origin <url>`
5. Add worktree guidance to `WORKSTORES.md` if applicable

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
=======
## Repository Mental Model

### Project Structure

```
crates/
  phenotype-event-sourcing/     # Append-only event store with SHA-256 hash chains
  phenotype-cache-adapter/      # Two-tier LRU + DashMap cache with TTL
  phenotype-policy-engine/      # Rule-based policy evaluation with TOML config
  phenotype-state-machine/      # Generic FSM with transition guards
  phenotype-contracts/          # Shared traits and types
  phenotype-error-core/         # Canonical error types
  phenotype-health/             # Health check abstraction
  phenotype-config-core/        # Configuration management

tests/                          # Integration and E2E tests
docs/
  adr/                          # Architecture decision records
  sessions/                     # Session-based work documentation
  reference/                    # Architecture docs and quick references
```

### Style Constraints

- **Line length**: 100 characters (Rust convention)
- **Formatter**: `cargo fmt` (mandatory)
- **Type checker**: Rust compiler (strict)
- **Linter**: `cargo clippy` with `-- -D warnings` (zero warnings)
- **File size target**: ≤350 lines per source file, hard limit ≤500 lines
- **Typing**: Full type annotations required; no `impl Trait` in public APIs unless necessary

### Key Constraints

- No inter-crate dependencies; each crate is independently consumable
- All public types must implement `Debug` and `Clone` where practical
- Error types must use `thiserror` with proper `#[from]` conversions
- Workspace-level dependency management in root `Cargo.toml`
- Tests are inline (`#[cfg(test)]` modules) within source files

---

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

### When to Document

- Research completions and findings
- Decisions made with rationale
- Issues found (duplication, performance, bugs)
- Work completions and status
- Planning for fork candidates or migration paths

---

## Quality Standards

### Code Quality Mandate

- **All linters must pass**: `cargo clippy --workspace -- -D warnings`
- **All tests must pass**: `cargo test --workspace`
- **No AI slop**: Avoid placeholder TODOs, lorem ipsum, generic comments
- **Backwards incompatibility**: No shims, full migrations, clean breaks

### Test-First Mandate

- **For NEW modules**: test file MUST exist before implementation file
- **For BUG FIXES**: failing test MUST be written before the fix
- **For REFACTORS**: existing tests must pass before AND after

### FR Traceability

All tests MUST reference a Functional Requirement (FR):

```rust
// Traces to: FR-XXX-NNN
#[test]
fn test_feature_name() {
    // Test body
}
```

---

## Governance Reference

See thegent governance base for complete guidance on:

1. **Core Agent Expectations** — Autonomous operation, when to ask vs. decide
2. **Standard Operating Loop (SWE Autopilot)** — Review, Research, Plan, Execute, Size-Check, Test, Review & Polish, Repeat
3. **File Size & Modularity Mandate** — ≤500 line hard limit, decomposition patterns
4. **Research-First Development** — Codebase research, web research, documentation
5. **Branch Discipline** — Worktree usage, PR workflow, git best practices
6. **Child-Agent and Delegation Policy** — When to spawn subagents, parallel vs. sequential
7. **Tool Usage & CLI Priority** — CLI as primary interface, read-only tools first
8. **Naming Conventions** — Session naming, file naming, branch naming

Location: `platforms/thegent/governance/AGENTS.base.md`

---

## Quick Reference Commands

```bash
# Run all quality checks
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Auto-format code
cargo fmt

# Run specific test
cargo test --package <crate-name> --lib <test_name>

# Build specific crate
cargo build -p <crate-name>

# View documentation locally
cargo doc --open
>>>>>>> origin/main
```
