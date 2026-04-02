# AGENTS.md — phenotype-infrakit

Extends thegent governance base. See `platforms/thegent/governance/AGENTS.base.md` for canonical definitions of agent expectations, testing requirements, research patterns, and standard operating procedures.

## Project Identity & Work Management

### Project Overview

- **Name**: phenotype-infrakit
- **Description**: Rust workspace containing generic infrastructure crates extracted from the Phenotype ecosystem
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit`
- **Language Stack**: Rust (edition 2021)
- **Published**: Internal (shared across Phenotype org)

### AgilePlus Integration

All work MUST be tracked in AgilePlus:
- Reference: `.agileplus/` directory
- CLI: `agileplus <command>` (from project root)
- Specs: `.agileplus/specs/<feature-id>/`
- Worklog: `.agileplus/worklog.md`

**Requirements**:
1. Check for AgilePlus spec before implementing
2. Create spec for new work: `agileplus specify --title "<feature>"`
3. Update work package status as work progresses
4. No code without corresponding AgilePlus spec

## Repository Structure

```
phenotype-infrakit/
├── crates/              # Crate sources
│   ├── phenotype-event-sourcing/
│   ├── phenotype-cache-adapter/
│   ├── phenotype-policy-engine/
│   ├── phenotype-state-machine/
│   ├── phenotype-contracts/
│   ├── phenotype-error-core/
│   ├── phenotype-health/
│   └── phenotype-config-core/
├── tests/               # Integration and E2E tests
├── docs/
│   ├── adr/           # Architecture decision records
│   └── reference/      # Architecture docs
└── Cargo.toml          # Workspace manifest
```

## Quality Standards

### Code Quality Mandate

- **All linters must pass**: `cargo clippy --workspace -- -D warnings`
- **All tests must pass**: `cargo test --workspace`
- **No AI slop**: Avoid placeholder TODOs, lorem ipsum, generic comments
- **Backwards incompatibility**: No shims, full migrations, clean breaks

### Style Constraints

- **Line length**: 100 characters (Rust convention)
- **Formatter**: `cargo fmt` (mandatory)
- **Type checker**: Rust compiler (strict)
- **File size target**: ≤350 lines per source file, hard limit ≤500 lines
- **Typing**: Full type annotations required

### Key Constraints

- No inter-crate dependencies; each crate is independently consumable
- All public types must implement `Debug` and `Clone` where practical
- Error types must use `thiserror` with proper `#[from]` conversions
- Tests are inline (`#[cfg(test)]` modules) within source files

## Quick Command Reference

```bash
# Run all quality checks
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Auto-format code
cargo fmt

# Build specific crate
cargo build -p <crate-name>

# Run specific test
cargo test -p <crate-name> --lib <test_name>
```
