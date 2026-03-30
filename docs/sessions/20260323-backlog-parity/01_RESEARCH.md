# Research Notes

## Codebase findings

- `crates/agileplus-domain/src/ports/storage.rs` is the canonical storage contract, so backlog
  semantics were added there instead of introducing a side channel.
- `crates/agileplus-sqlite/src/lib/storage_port.rs` is the storage adapter boundary, so it now
  delegates backlog CRUD/list/pop operations to a dedicated repository module.
- `crates/agileplus-api/src/routes/backlog.rs` is the right place for HTTP backlog list/create/
  transition/pop/import handlers because the router already nests `/api/v1/backlog` there.
- `python/src/agileplus_mcp/tools/queue.py` is the current MCP queue surface, so import/pop
  improvements were added there to keep the operator experience aligned with CLI/API semantics.

## Decision rationale

- Shared backlog concepts belong in the domain layer so CLI, API, MCP, and storage all use the same
  intent/priority/status vocabulary.
- Batch import should be available from both HTTP and MCP entry points so users do not need to seed
  SQL or bypass the normal work flow.
- Batch pop should behave consistently across surfaces, so the MCP queue tool now handles a count
  argument rather than only a single item.
