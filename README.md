# AgilePlus

Spec-driven development engine — from specification to shipped feature with full governance and audit trail.

## Overview

AgilePlus implements a 7-stage development pipeline:

```
specify → research → plan → implement → validate → ship → retrospective
```

Each stage transitions features through a governed state machine with SHA-256 audit chains.

## Architecture

- **8 Rust crates** in a clean-architecture workspace
- **Port-based adapters** (SQLite, Git, Plane.so, GitHub)
- **Rule-based triage** with keyword-weighted intent classification
- **25 hidden sub-commands** for advanced agent workflows
- **Multi-agent support** (Claude Code, Cursor, Codex, Copilot)

## Quick Start

```bash
# Install
cargo install --path crates/agileplus-cli

# Initialize a project
agileplus init

# Create a feature
agileplus specify --title "My Feature" --description "What it does"

# Full pipeline
agileplus research my-feature
agileplus plan my-feature
agileplus implement my-feature --wp WP01
agileplus validate my-feature
agileplus ship my-feature
agileplus retrospective my-feature
```

## Documentation

Run `npm run dev` to preview docs locally, or see the [docs/](docs/) directory.

## License

MIT
