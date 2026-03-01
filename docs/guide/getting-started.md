# Getting Started

Install AgilePlus and run your first spec-driven pipeline.

## Prerequisites

- Rust 2024 edition (1.85+)
- Git 2.x

## Install

```bash
cargo install --path crates/agileplus-cli
```

## Initialize

```bash
agileplus init
```

This detects your project type, scans for languages and frameworks, and generates governance files. See [Project Setup](/guide/init) for details.

## Your First Feature

```bash
# 1. Create a specification
agileplus specify --title "User Auth" --description "Add login flow"

# 2. Research feasibility
agileplus research user-auth

# 3. Generate work packages
agileplus plan user-auth

# 4. Implement
agileplus implement user-auth --wp WP01

# 5. Validate governance
agileplus validate user-auth

# 6. Ship
agileplus ship user-auth

# 7. Retrospective
agileplus retrospective user-auth
```

## What's Next

- [Spec-Driven Development](/concepts/spec-driven-dev) — understand the philosophy
- [Core Workflow](/guide/workflow) — deep dive into each stage
- [Full Pipeline Example](/examples/full-pipeline) — end-to-end walkthrough
- [CLI Reference](/reference/cli) — all commands and flags
