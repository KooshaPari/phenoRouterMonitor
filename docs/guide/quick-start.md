---
audience: [developers, agents]
---

# Quick Start

Get AgilePlus running in under 5 minutes.

## Prerequisites

- Rust toolchain (`rustup`)
- Git
- A project tracker account (Plane.so or GitHub Issues)

## Install

```bash
cargo install agileplus
```

## Initialize a Project

```bash
agileplus init my-project
cd my-project
```

This creates:
- `kitty-specs/` — your specification directory
- `.kittify/` — local config and agent metadata
- `AGENTS.md` — agent governance file

## Create Your First Spec

```bash
agileplus specify "Add user login with email and password"
```

AgilePlus generates a structured specification in `kitty-specs/001-user-login/spec.md`.

## Plan and Execute

```bash
agileplus plan 001           # Generate work packages
agileplus implement WP01     # Start a work package
```

## What's Next?

- [Core Workflow](/guide/workflow) — understand the full pipeline
- [Spec-Driven Development](/concepts/spec-driven-dev) — learn the methodology
- [Configuration](/guide/configuration) — customize AgilePlus for your project
