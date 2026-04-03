# Implementation Strategy

## Approach

- Put canonical backlog semantics in the domain layer.
- Expose backlog operations through `StoragePort`.
- Implement SQLite persistence in a dedicated repository module.
- Keep CLI/API/MCP surfaces thin and route them through the shared storage contract.

## Design choices

- Added `tags` to backlog items to support richer queue semantics without separate seed paths.
- Used batch import on the HTTP side and local item loops on the MCP side to avoid inventing a new
  gRPC contract without evidence from the proto surface.
- Added a dedicated CLI import subcommand on top of the file-backed queue path so batch intake is
  discoverable and consistent with the API surface.
- Kept pop behavior deterministic by selecting the next eligible backlog item in storage.
- Added `CHANGELOG.md`, `codecov.yml`, and a release policy doc to make versioning and coverage
  expectations explicit.
- Documented the changelog workflow permission requirement and the CI Codecov upload path in the
  reusable workflow README.
- Split MCP documentation into index/runtime documents to keep each file under the size limit.

## Follow-on work

- Add focused tests for the backlog import and pop paths.
- Add a CLI smoke test for `agileplus queue import`.
- Run a targeted compile/test pass for the CLI, API, and MCP surfaces.
- Update any remaining session or roadmap notes only if a concrete mismatch reappears.
