# Testing Strategy

## Coverage targets

- Domain parsing and enum conversion.
- Storage port backlog CRUD/list/pop methods.
- HTTP route request validation and import/pop behavior.
- MCP tool input validation and batch semantics.

## Recommended checks

- Add focused tests for backlog list/create/import/transition/pop.
- Add a CLI smoke test for file-based backlog import.
- Add a queue tool test that exercises `agileplus_queue_import` and batch pop behavior.
- Add a doc smoke check to confirm `docs/sdk/mcp-tools.md` and `docs/sdk/mcp-runtime.md` stay
  under the 500-line cap.

## Validation note

- Validation was intentionally deferred in this turn, so the first follow-up should be a targeted
  compile/test pass.
- The new MCP runtime split and session-doc updates do not require extra behavior tests by
  themselves, but they do need a doc consistency check in any future docs sweep.
