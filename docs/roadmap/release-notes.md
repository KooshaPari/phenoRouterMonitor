---
audience: [pms, developers]
---

# Release Notes

## v0.1.0 — Foundation (Current)

Initial release of AgilePlus with core spec-driven development capabilities.

### Features

- **Spec engine**: Create and manage structured feature specifications
- **Plan generation**: Decompose specs into work packages with dependency graphs
- **Worktree isolation**: Each work package gets its own git worktree
- **Agent dispatch**: Send structured prompts to AI coding agents
- **Governance**: Audit trail, constraint enforcement, review protocol
- **Triage & queue**: Priority-based work package scheduling
- **CLI**: Full command interface for all operations
- **Sync**: Bi-directional sync with Plane.so and GitHub

### Crates

| Crate | Purpose |
|-------|---------|
| `agileplus-core` | Domain model, entities, value objects |
| `agileplus-engine` | Business logic, orchestration |
| `agileplus-cli` | Command-line interface |
| `agileplus-sync` | Tracker integrations |
| `agileplus-agents` | Agent harness and dispatch |

### Known Limitations

- Single-agent execution only (no parallel WP processing)
- File-based storage only (no database backend)
- CLI-only interface (no web UI)
