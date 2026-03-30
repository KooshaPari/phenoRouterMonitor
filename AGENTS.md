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
- Reference: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- CLI: `cd AgilePlus && agileplus <command>`
- Specs: `AgilePlus/kitty-specs/<feature-id>/`
- Worklog: `AgilePlus/.work-audit/worklog.md`

**Requirements**:
1. Check for AgilePlus spec before implementing
2. Create spec for new work: `agileplus specify --title "<feature>"`
3. Update work package status as work progresses
4. No code without corresponding AgilePlus spec

---

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
// Traces to: FR-PHENO-NNN
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
```
