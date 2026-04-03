# Session Overview: Backlog parity and import workflow

## Goal

Align AgilePlus backlog semantics across the domain model, SQLite storage, CLI, HTTP API, and MCP
surface so queue operations are storage-backed and batch import is available end to end. Extend the
same session with release hygiene work: semver, Keep a Changelog, and Codecov integration.

## Status

- Added shared backlog domain types in `crates/agileplus-domain`.
- Wired storage-backed backlog CRUD/pop methods into the SQLite adapter.
- Reworked CLI queue and triage commands to use storage and support file-based batch import.
- Added a dedicated `agileplus queue import` command so batch intake is discoverable without
  relying on the `add --from-file` flag path.
- Added HTTP backlog routes, including batch import.
- Added MCP queue tools, including batch import and batch pop support.
- Expanded the gRPC integrations contract so backlog create/list/get/import/pop/status flows are
  available end to end.
- Split MCP docs into `docs/sdk/mcp-tools.md` and `docs/sdk/mcp-runtime.md` to keep each document
  under the repo size target while preserving the full catalog.
- Added release hygiene surfaces: `CHANGELOG.md`, `codecov.yml`, and versioning policy docs.
- Updated workflow docs and CI uploads to make Codecov and changelog behavior explicit.
- Corrected package publish semantics and added repo-wide ignores for generated coverage output.
- Added a top-level `release.yml` workflow to orchestrate gate, changelog, and publish steps.
- Added release version validation so publish refuses manifest/version mismatches.
- Reconciled pre-commit hook paths and added a root `CODEOWNERS` governance contract.
- Added a PR template that forces release, versioning, and validation checks into review.
- Added issue templates that capture release/versioning context for bugs and features.
- Added `docs/process/governance.md` as the canonical intake/review/release flow entry point.
- Added docs navigation and homepage links to surface the governance flow visibly.
- Updated pilot rollout docs to match the manual `release.yml` orchestrator.
- Rewrote roadmap and full-pipeline examples to remove tag-first release wording.
- Updated the doc-system layer table so changelog generation points at `release.yml`.
- Removed the last tag-first release phrasing from the pilot guides.
- Replaced the last tag-triggered release wording in accept and constitution docs.
- Aligned the queue command documentation with the runtime output contract: plain text by default,
  JSON for machine-readable output.
- Removed stale queue/triage examples and aligned docs with the implemented `queue import` path.
- Split the Python gRPC client backlog RPCs into a dedicated mixin module and extracted shared gRPC
  exceptions into a tiny support module to restore the file-size budget.
- Split the remaining gRPC serialization and streaming helpers into dedicated mixins so the core
  client stays below the repo's soft size target.

## Notes

- External GitHub branch protection/ruleset changes remain out of repo scope.
- Validation has not been run yet; the change set now covers backlog parity, release hygiene,
  and the MCP/doc split needed to keep those surfaces under the file-size budget.
