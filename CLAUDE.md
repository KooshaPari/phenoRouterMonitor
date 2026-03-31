<<<<<<< HEAD
# CLAUDE.md — repos shelf root

<<<<<<< HEAD
**This project is managed through AgilePlus.**

## AgilePlus Mandate

All work MUST be tracked in AgilePlus:
- Reference: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- CLI: `cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>`

## Work Requirements

1. **Check for AgilePlus spec before implementing**
2. **Create spec for new work**: `agileplus specify --title "<feature>" --description "<desc>"`
3. **Update work package status**: `agileplus status <feature-id> --wp <wp-id> --state <state>`
4. **No code without corresponding AgilePlus spec**

## Branch Discipline

- Feature branches in `repos/worktrees/<project>/<category>/<branch>`
- Canonical repository tracks `main` only
- Return to `main` for merge/integration checkpoints

## UTF-8 Encoding

All markdown files must use UTF-8.

---



- Always evaluate and fix ALL CI check failures on a PR, including pre-existing failures inherited from main.
- Never dismiss a CI failure as "pre-existing" or "unrelated to our changes" — if it fails on the PR, fix it in the PR.
- This includes: build, lint, test, docs build, security scanning (CodeQL), code review gates (CodeRabbit), workflow guard checks, and any other CI jobs.
- When a failure is caused by infrastructure outside the branch (e.g., rate limits, external service outages), implement or improve automated retry/bypass mechanisms in CI workflows.
- After fixing CI failures, verify locally where possible (build, vet, tests) before pushing.
=======
## Identity
>>>>>>> origin/main

This is the **repos shelf** for `CodeProjects/Phenotype/organizational-shelf/repos`.
A shelf is a top-level organizational unit containing related but independent
project repositories. Think of it like a `/opt` or `~/code` directory, but
versioned and synced as a polyrepo (repo of repos).

**NOT AgilePlus.** AgilePlus is one of ~30 projects inside this shelf.
See `projects/INDEX.md` for the full catalog.

## Structure
=======
<<<<<<< HEAD
# CLAUDE.md — repos shelf root

## Identity
=======
<!-- Base: platforms/thegent/dotfiles/governance/CLAUDE.base.md -->
<!-- Last synced: 2026-03-29 -->

# phenotype-infrakit — CLAUDE.md

Extends thegent governance base. See `platforms/thegent/dotfiles/governance/CLAUDE.base.md` for canonical definitions.

## Project Overview

<<<<<<< HEAD
- Feature work goes in `.worktrees/<topic>/`
- Legacy `PROJECT-wtrees/` and `repo-wtrees/` roots are for migration only and must not receive new work.
- Canonical repository remains on `main` for final integration and verification.
=======
- **Name**: phenotype-infrakit
- **Description**: Rust workspace containing generic infrastructure crates extracted from the Phenotype ecosystem
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/` (monorepo)
- **Language Stack**: Rust (edition 2021)
- **Published**: Internal (shared across Phenotype org)
>>>>>>> origin/main

This is the **repos shelf** for `CodeProjects/Phenotype/organizational-shelf/repos`.
A shelf is a top-level organizational unit containing related but independent
project repositories. Think of it like a `/opt` or `~/code` directory, but
versioned and synced as a polyrepo (repo of repos).

**NOT AgilePlus.** AgilePlus is one of ~30 projects inside this shelf.
See `projects/INDEX.md` for the full catalog.

<<<<<<< HEAD
## Structure
=======
## Work Requirements

1. **Check for AgilePlus spec before implementing**
2. **Create spec for new work**: `agileplus specify --title "<feature>" --description "<desc>"`
3. **Update work package status**: `agileplus status <feature-id> --wp <wp-id> --state <state>`
4. **No code without corresponding AgilePlus spec**

## Branch Discipline

- Feature branches in `repos/worktrees/<project>/<category>/<branch>`
- Canonical repository tracks `main` only
- Return to `main` for merge/integration checkpoints

## UTF-8 Encoding

All markdown files must use UTF-8.

---

## Local Quality Checks

From this repository root:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Testing & Specification Traceability

All tests MUST reference a Functional Requirement (FR):

```rust
// Traces to: FR-XXX-NNN
#[test]
fn test_feature_name() {
    // Test body
}
```

**Verification**:
- Every FR in FUNCTIONAL_REQUIREMENTS.md MUST have >=1 test
- Every test MUST reference >=1 FR
- Run: `cargo test` to verify

---

## Project-Specific Configuration

This monorepo consists of domain-agnostic, independently consumable Rust crates:

### Crate Structure

```
crates/
  phenotype-event-sourcing/   # Append-only event store with SHA-256 hash chains
  phenotype-cache-adapter/    # Two-tier LRU + DashMap cache with TTL
  phenotype-policy-engine/    # Rule-based policy evaluation with TOML config
  phenotype-state-machine/    # Generic FSM with transition guards
  phenotype-contracts/        # Shared traits and types
  phenotype-error-core/       # Canonical error types
  phenotype-health/           # Health check abstraction
  phenotype-config-core/      # Configuration management
```

### Conventions

- All public types implement `Debug`, `Clone` where possible
- Error types use `thiserror` with `#[from]` for conversions
- Serialization via `serde` with `Serialize`/`Deserialize` derives
- No inter-crate dependencies; each crate stands alone
- Workspace-level dependency versions in root `Cargo.toml`
- Tests are inline (`#[cfg(test)]` modules) within each source file

### Adding a New Crate

1. Create `crates/<name>/` with `Cargo.toml` and `src/lib.rs`
2. Add to `members` in root `Cargo.toml`
3. Use `workspace = true` for shared dependencies
4. Include inline tests with `#[cfg(test)]`
5. Update `README.md` crate table

---

## Architecture

### Hexagonal Architecture (Ports & Adapters)

This project follows Hexagonal Architecture with clear separation of concerns:
>>>>>>> origin/main
>>>>>>> origin/main

```
repos/                          # ← YOU ARE HERE (shelf root)
├── .worktrees/                 # Canonical worktree staging area
├── .archive/                   # Archived/rejected items
├── apps/                       # Application projects (user-facing)
├── libs/                       # Shared libraries (internal packages)
├── tooling/                    # Developer tools, CLIs, scripts
├── infra/                      # Infrastructure, deployment, devops
├── platforms/                  # Platform-as-product projects
├── crates/                     # Rust workspace members
├── packages/                   # JS/TS monorepo packages
├── docs/                       # Cross-project documentation
│   ├── adr/                   #   Architecture decision records
│   └── guides/                #   How-to guides
├── scripts/                    # Cross-project utility scripts
├── governance/                 # Governance tooling (policy, scoring)
├── projects/                   # Project catalog & metadata
│   └── INDEX.md               #   Master project list
├── WORKSTORES.md               # Worktree management guide
└── REPOS_INDEX.md              # Detailed shelf index
```

<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> origin/main
## Agent Rules

**READ `AGENTS.md` FIRST.** It contains the authoritative agent interaction
rules for this shelf. Key points:

- When working on a project, cd into its directory first (e.g., `cd heliosCLI`)
- Never assume a project is at shelf root — always verify
- Test commands must run inside the target project directory, not shelf root
- File reads should specify the correct relative path from shelf root
<<<<<<< HEAD
=======
=======
### Design Principles
>>>>>>> origin/main
>>>>>>> origin/main

## Project Index

See `projects/INDEX.md` for the full catalog of all projects in this shelf.

## Phenotype Federated Hybrid Architecture

This shelf is part of the **Phenotype Federated Hybrid Architecture**, which provides two complementary chassis systems:

<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> origin/main
### Phenotype Docs Chassis

Provides VitePress configuration, design tokens, and theme components for consistent documentation across the organization.

**Location**: `@phenotype/docs` (GitHub Packages)
**Documentation**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
**Usage**: Add `@phenotype/docs` to `docs/package.json` in any project

### AgilePlus Governance Chassis

Defines specification-driven delivery framework: PRD, ADR, FUNCTIONAL_REQUIREMENTS, PLAN, USER_JOURNEYS, with FR traceability and worklog integration.

**Location**: AgilePlus project (this repo, crates/agileplus-*)
**Documentation**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
**Usage**: Create `/PRD.md`, `/FUNCTIONAL_REQUIREMENTS.md` at project root; tag tests with `@pytest.mark.requirement("FR-XXX-NNN")`

**See Also**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md` and `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md` for integration points and code examples.

## Quick Reference

| What you need | Where to look |
|---------------|---------------|
| Project list | `projects/INDEX.md` |
| Governance rules | `AGENTS.md` |
| Architecture decisions | `docs/adr/` |
| Cross-project scripts | `scripts/` |
| Docs Chassis Interface | `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md` |
| Governance Chassis Interface | `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md` |
<<<<<<< HEAD
=======
=======
See `docs/adr/` for architecture decisions.

---

## Governance Reference

See thegent governance base for:
- Complete CI completeness policy
- Phenotype Git and Delivery Workflow Protocol
- Phenotype Org Cross-Project Reuse Protocol
- Phenotype Long-Term Stability and Non-Destructive Change Protocol
- Worktree Discipline guidelines

Location: `platforms/thegent/dotfiles/governance/CLAUDE.base.md`
>>>>>>> origin/main
>>>>>>> origin/main
>>>>>>> origin/main
