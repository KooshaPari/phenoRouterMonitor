# Getting Started

## Prerequisites

- Rust 2024 edition (1.85+)
- Git 2.x
- SQLite 3.x (bundled via rusqlite)

## Installation

```bash
cargo install --path crates/agileplus-cli
```

## Initialize a Project

```bash
agileplus init
```

This detects your project type (brownfield/greenfield), scans for languages and frameworks, and generates:

- `.agileplus/config.toml` — project configuration
- `CLAUDE.md` — Claude Code governance
- `AGENTS.md` — cross-agent governance
- `.claudeignore` — context optimization
- Agent-specific configs (Cursor, Codex, Copilot)
- Git pre-commit hooks

## Core Workflow

```
specify → research → plan → implement → validate → ship → retrospective
```

Each stage transitions the feature through a governed state machine with full audit trail.

### Quick Example

```bash
# Create a feature specification
agileplus specify --title "User Authentication" --description "Add OAuth2 login flow"

# Research feasibility
agileplus research auth-feature

# Generate work packages
agileplus plan auth-feature

# Implement a work package
agileplus implement auth-feature --wp WP01

# Validate governance
agileplus validate auth-feature

# Ship it
agileplus ship auth-feature

# Retrospective
agileplus retrospective auth-feature
```

## Triage & Queue

```bash
# Classify incoming items
agileplus triage "login page crashes on mobile Safari"

# Manage the backlog
agileplus queue add --title "Fix Safari crash" --type bug
agileplus queue list
agileplus queue pop
```
