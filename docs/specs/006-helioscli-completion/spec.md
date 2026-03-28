# Spec: heliosCLI Completion

## Meta

- **ID**: 006-helioscli-completion
- **Title**: heliosCLI Multi-Runtime Agent CLI Completion
- **Created**: 2026-03-25
- **State**: specified
- **Repo**: /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI

## Overview

Complete heliosCLI multi-runtime AI coding CLI (153 commits since 2025-01-01).

## Architecture

### Components
- **codex-rs**: Rust core
- **codex-cli**: TypeScript CLI
- **Bazel monorepo**: Build system
- **thegent integration**: Agent orchestration

## Past Work (Completed)

### WP001: Expect Pattern Cleanup
- State: shipped
- Cleanup of test expect patterns

## Present Work (Current)

### WP010: Bazel Build Optimization
- Build caching
- Remote execution
- Incremental builds

## Future Work (Planned)

### WP020: Multi-Runtime Integration
- Codex runtime
- Claude runtime
- Gemini runtime
- Cursor runtime
- Copilot runtime

### WP021: thegent Orchestration
- Full thegent integration
- Agent lifecycle management

## Work Packages

| ID | Description | State |
|----|-------------|-------|
| WP001 | Expect cleanup | shipped |
| WP010 | Bazel optimization | in_progress |
| WP020 | Multi-runtime | specified |
| WP021 | thegent integration | specified |

## Traces

- Related: 001-spec-driven-development-engine
- Related: 007-thegent-completion

## Status

Migrated from kitty-specs. Tracked in AgilePlus.
