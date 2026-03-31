# Specifications

## Scope

Standardize backlog semantics across AgilePlus:

- Shared domain types for intent, priority, status, sort, and item shape.
- Storage-backed backlog CRUD/list/pop operations.
- CLI queue and triage commands that use storage instead of placeholders.
- HTTP backlog routes for list, create, get, transition, pop, and import.
- MCP queue tools for add, list, show, pop, and import.
- Canonical release history via `CHANGELOG.md`.
- CI coverage export and Codecov status checks for Rust and Python.

## Acceptance criteria

- Backlog items carry the same semantic fields across CLI, API, MCP, and storage.
- Batch import is available without ad hoc SQL seeding.
- Pop operations consume the next eligible backlog item rather than using a stub.
- Triage writes a backlog item through the storage port.
- Repository versioning follows semantic versioning and Keep a Changelog.

## Assumptions

- SQLite remains the active local storage backend.
- The gRPC integrations service exposes backlog create/list/get/import/pop/status for MCP use.

## Risks

- Validation has not been executed yet, so compile/runtime mismatches may still exist.
- MCP batch import relies on the gRPC integrations batch request; runtime validation is still
  pending.
