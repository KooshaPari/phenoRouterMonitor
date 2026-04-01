# CLAUDE.md — thegent-shm

## Project Overview

thegent is the agent framework for phenotypic development automation. It provides
CLI tools and agent infrastructure for the AgilePlus ecosystem.

## Repository

```
/Users/kooshapari/CodeProjects/Phenotype/repos/thegent
```

## Architecture

thegent follows domain-driven decomposition:

| Domain | Purpose |
|--------|---------|
| MODELS | Core data models and types |
| PLAN | Planning and task management |
| GOVERNANCE | Policy and compliance |
| TEAM | Multi-agent coordination |

## Current Work

See `docs/specs/007-thegent-completion/spec.md` for work package tracking.

Active work package: **WP011** — GitOps Refactor (shim split pattern)

## Key Files

| Path | Description |
|------|-------------|
| `src/cli.rs` | Main CLI entry point |
| `src/domain/` | Domain subpackages |
| `src/atlas/` | Codebase atlas generation |
| `Cargo.toml` | Workspace manifest |

## Development Commands

```bash
cargo check --all-features
cargo test --workspace
cargo clippy --all-features
```
