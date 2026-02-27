---
work_package_id: "WP01"
subtasks:
  - "T001"
  - "T002"
  - "T003"
  - "T004"
  - "T005"
  - "T006"
title: "Rust Workspace & Build Scaffold"
phase: "Phase 0 - Foundation"
lane: "planned"
dependencies: []
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-02-27T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP01 -- Rust Workspace & Build Scaffold

## IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately.
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** -- Reviewers add detailed feedback here when work needs changes.

*[This section is empty initially. Reviewers will populate it if the work is returned from review.]*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ````python`, ````bash`

---

## Implementation Command

```bash
spec-kitty implement WP01
```

---

## Objectives & Success Criteria

1. **Cargo workspace compiles**: `cargo build --workspace` succeeds with zero errors.
2. **Cargo tests pass (empty)**: `cargo test --workspace` runs with 0 tests, 0 errors.
3. **All 9 crate stubs exist**: `agileplus-core`, `agileplus-cli`, `agileplus-api`, `agileplus-grpc`, `agileplus-sqlite`, `agileplus-git`, `agileplus-agents`, `agileplus-review`, `agileplus-telemetry`.
4. **Makefile targets work**: `make build`, `make test`, `make lint`, `make format` all succeed.
5. **Docker Compose starts**: `docker-compose up --build` creates Rust builder and Python MCP containers without errors.
6. **Proto stubs generated**: `make proto-gen` produces Rust (tonic) and Python (grpcio) stubs from `proto/agileplus.proto`.
7. **Rust 2024 edition** used in all `Cargo.toml` files.

---

## Context & Constraints

### Reference Documents
- **Spec**: `kitty-specs/001-spec-driven-development-engine/spec.md` -- FR requirements
- **Plan**: `kitty-specs/001-spec-driven-development-engine/plan.md` -- Project structure (lines 50-176), dependency graph, technical context
- **Data Model**: `kitty-specs/001-spec-driven-development-engine/data-model.md` -- Entity definitions for domain module stubs
- **Contracts**: `kitty-specs/001-spec-driven-development-engine/contracts/agileplus.proto` -- gRPC service definitions
- **Research**: `kitty-specs/001-spec-driven-development-engine/research.md` -- Technology choices and rationale

### Architectural Constraints
- **Hexagonal architecture**: Core crate has ZERO external dependencies except `serde`, `sha2`, `chrono`. All I/O goes through port traits.
- **Workspace path dependencies**: Adapter crates depend on `agileplus-core` via `path = "../agileplus-core"`, NOT crates.io.
- **Rust 2024 edition**: All crates use `edition = "2024"` for native async trait support.
- **Proto generation**: `tonic-build` for Rust, `grpcio-tools` for Python. `protoc` must be pinned in Makefile and Docker.

### Key Dependencies (from plan.md Technical Context)
- **Rust**: clap, axum, tokio, tonic, rusqlite, git2, opentelemetry, serde, sha2
- **Python**: fastmcp>=3.0, grpcio, opentelemetry-sdk
- **Build**: protoc (pinned version), buf (linting)

---

## Subtasks & Detailed Guidance

### Subtask T001 -- Create root `Cargo.toml` workspace manifest with all 9 crate members

- **Purpose**: Establish the Cargo workspace so all crates share a common `target/` directory, dependency resolution, and can be built/tested together with a single command.
- **Steps**:
  1. Create `Cargo.toml` at the repository root.
  2. Define `[workspace]` with `resolver = "3"` (Rust 2024 default).
  3. List all 9 members under `workspace.members`:
     ```toml
     members = [
         "crates/agileplus-core",
         "crates/agileplus-cli",
         "crates/agileplus-api",
         "crates/agileplus-grpc",
         "crates/agileplus-sqlite",
         "crates/agileplus-git",
         "crates/agileplus-agents",
         "crates/agileplus-review",
         "crates/agileplus-telemetry",
     ]
     ```
  4. Define `[workspace.dependencies]` for shared deps with version pinning:
     - `serde = { version = "1", features = ["derive"] }`
     - `serde_json = "1"`
     - `chrono = { version = "0.4", features = ["serde"] }`
     - `sha2 = "0.10"`
     - `tokio = { version = "1", features = ["full"] }`
     - `thiserror = "2"`
     - `anyhow = "1"`
     - `tracing = "0.1"`
     - `tracing-subscriber = "0.3"`
  5. Add `[workspace.package]` with shared metadata: `edition = "2024"`, `license = "MIT"`, `repository`, `rust-version = "1.85"`.
- **Files**: `Cargo.toml` (root)
- **Parallel?**: No -- this must complete first; all other T002-T006 depend on it.
- **Validation**: `cargo metadata --format-version 1 | jq '.workspace_members | length'` returns 9.
- **Notes**: Do NOT add `[package]` to the root Cargo.toml -- it is workspace-only. The `resolver = "3"` is default for edition 2024 but being explicit is safer for CI tooling.

### Subtask T002 -- Scaffold `crates/agileplus-core/` with `lib.rs`, domain module stubs, port trait stubs

- **Purpose**: Create the domain layer crate that all adapters depend on. This crate defines the core types, domain logic, and port trait interfaces. It must compile independently with minimal deps.
- **Steps**:
  1. Create directory: `crates/agileplus-core/src/`
  2. Create `crates/agileplus-core/Cargo.toml`:
     ```toml
     [package]
     name = "agileplus-core"
     version = "0.1.0"
     edition.workspace = true

     [dependencies]
     serde.workspace = true
     serde_json.workspace = true
     chrono.workspace = true
     sha2.workspace = true
     thiserror.workspace = true
     ```
  3. Create `src/lib.rs` with module declarations:
     ```rust
     pub mod domain;
     pub mod ports;
     ```
  4. Create domain module stubs (each file has a comment header explaining its purpose):
     - `src/domain/mod.rs` -- re-exports `feature`, `work_package`, `governance`, `audit`, `state_machine`
     - `src/domain/feature.rs` -- `pub struct Feature {}` placeholder (fields come in WP03)
     - `src/domain/work_package.rs` -- `pub struct WorkPackage {}` placeholder
     - `src/domain/governance.rs` -- `pub struct GovernanceContract {}` placeholder
     - `src/domain/audit.rs` -- `pub struct AuditEntry {}` placeholder
     - `src/domain/state_machine.rs` -- `// State machine implementation placeholder`
  5. Create port trait stubs:
     - `src/ports/mod.rs` -- re-exports all port modules
     - `src/ports/storage.rs` -- `pub trait StoragePort {}` placeholder
     - `src/ports/vcs.rs` -- `pub trait VcsPort {}` placeholder
     - `src/ports/agent.rs` -- `pub trait AgentPort {}` placeholder
     - `src/ports/review.rs` -- `pub trait ReviewPort {}` placeholder
     - `src/ports/observability.rs` -- `pub trait ObservabilityPort {}` placeholder
  6. Create `src/error.rs` with a `DomainError` enum stub using `thiserror`:
     ```rust
     #[derive(Debug, thiserror::Error)]
     pub enum DomainError {
         #[error("not implemented")]
         NotImplemented,
     }
     ```
- **Files**: `crates/agileplus-core/Cargo.toml`, `crates/agileplus-core/src/lib.rs`, `src/domain/*.rs`, `src/ports/*.rs`, `src/error.rs`
- **Parallel?**: Yes -- independent after T001
- **Validation**: `cargo build -p agileplus-core` succeeds; `cargo doc -p agileplus-core --no-deps` generates docs.
- **Notes**: Keep structs empty (`{}`). WP03 and WP04 fill in fields. Port traits are empty -- WP05 adds methods. The goal here is compilable structure, not logic.

### Subtask T003 -- Scaffold remaining 8 adapter crates with `lib.rs` and dependency declarations

- **Purpose**: Create all adapter crate directories with correct Cargo.toml files declaring their dependencies on `agileplus-core` and their external crates. Each must compile as an empty library.
- **Steps**:
  1. For each of the 8 adapter crates, create `crates/<name>/Cargo.toml` and `crates/<name>/src/lib.rs`.
  2. Crate-specific dependencies (workspace deps where possible):
     - **agileplus-cli**: `agileplus-core`, `clap = { version = "4", features = ["derive"] }`, `tokio.workspace`
     - **agileplus-api**: `agileplus-core`, `axum = "0.8"`, `tokio.workspace`, `serde.workspace`, `serde_json.workspace`
     - **agileplus-grpc**: `agileplus-core`, `tonic = "0.12"`, `prost = "0.13"`, `tokio.workspace`
     - **agileplus-sqlite**: `agileplus-core`, `rusqlite = { version = "0.32", features = ["bundled"] }`, `serde.workspace`
     - **agileplus-git**: `agileplus-core`, `git2 = "0.19"`, `serde.workspace`
     - **agileplus-agents**: `agileplus-core`, `tokio.workspace`
     - **agileplus-review**: `agileplus-core`, `reqwest = { version = "0.12", features = ["json"] }`, `tokio.workspace`, `serde.workspace`
     - **agileplus-telemetry**: `agileplus-core`, `opentelemetry = "0.28"`, `opentelemetry-otlp = "0.28"`, `tracing.workspace`, `tracing-subscriber.workspace`
  3. Each `lib.rs` should contain a doc comment describing the crate's role and an empty module structure matching plan.md:
     - `agileplus-cli/src/lib.rs`: `pub mod commands;` with `src/commands/mod.rs` stub
     - `agileplus-api/src/lib.rs`: `pub mod routes;` and `pub mod middleware;` stubs
     - `agileplus-grpc/src/lib.rs`: `pub mod server;` stub
     - `agileplus-sqlite/src/lib.rs`: `pub mod migrations;` and `pub mod repository;` stubs
     - `agileplus-git/src/lib.rs`: `pub mod worktree;`, `pub mod repository;`, `pub mod artifact;` stubs
     - `agileplus-agents/src/lib.rs`: `pub mod claude_code;`, `pub mod codex;`, `pub mod dispatch;`, `pub mod pr_loop;` stubs
     - `agileplus-review/src/lib.rs`: `pub mod coderabbit;`, `pub mod fallback;` stubs
     - `agileplus-telemetry/src/lib.rs`: `pub mod traces;`, `pub mod metrics;`, `pub mod logs;` stubs
  4. Create stub module files (empty `mod.rs` or named files) so the module declarations compile.
- **Files**: 8 `Cargo.toml` files + 8 `lib.rs` files + ~25 stub module files
- **Parallel?**: Yes -- independent after T001. Can also run parallel with T002.
- **Validation**: `cargo build --workspace` succeeds; `cargo check --workspace` is clean.
- **Notes**: Use `version.workspace = true` and `edition.workspace = true` in all crate Cargo.toml files. Pin exact major versions of external deps. The `rusqlite` bundled feature is critical to avoid requiring system SQLite headers.

### Subtask T004 -- Create `Makefile` with targets: build, test, lint, format, proto-gen, all

- **Purpose**: Provide a single polyglot build orchestration file that developers and CI use. Abstracts Cargo, uv, protoc, and Docker commands behind simple `make` targets.
- **Steps**:
  1. Create `Makefile` at repository root.
  2. Define variables at top:
     ```makefile
     PROTOC_VERSION := 28.3
     RUST_EDITION := 2024
     PYTHON_DIR := mcp
     PROTO_DIR := proto
     ```
  3. Implement targets:
     - `all`: depends on `build test lint`
     - `build`: `cargo build --workspace` then `cd $(PYTHON_DIR) && uv sync`
     - `test`: `cargo test --workspace` then `cd $(PYTHON_DIR) && uv run pytest`
     - `lint`: `cargo clippy --workspace -- -D warnings` then `cd $(PYTHON_DIR) && uv run ruff check .`
     - `format`: `cargo fmt --all` then `cd $(PYTHON_DIR) && uv run ruff format .`
     - `format-check`: `cargo fmt --all -- --check` then `cd $(PYTHON_DIR) && uv run ruff format --check .`
     - `proto-gen`: invoke `protoc` with `tonic-build` for Rust AND `grpcio-tools` for Python
     - `docker-build`: `docker-compose build`
     - `docker-up`: `docker-compose up -d`
     - `docker-down`: `docker-compose down`
     - `clean`: `cargo clean && rm -rf $(PYTHON_DIR)/.venv`
  4. Add `.PHONY` declarations for all targets.
  5. Add a `check-protoc` target that verifies protoc is installed at the correct version.
- **Files**: `Makefile`
- **Parallel?**: Yes -- independent after T001
- **Validation**: `make build` succeeds; `make lint` runs without error; `make --dry-run all` shows correct target ordering.
- **Notes**: Use `$(MAKE)` for recursive make calls. The proto-gen target will fail until T006 creates the proto file, but the Makefile itself should be syntactically valid. Use tab indentation (Makefile requirement).

### Subtask T005 -- Create `docker-compose.yml` for dev environment

- **Purpose**: Provide a containerized development environment so developers can build and test without local toolchain installation. Also required for CI and integration testing.
- **Steps**:
  1. Create `docker-compose.yml` at repository root.
  2. Define services:
     - **rust-builder**: Rust nightly image, mounts workspace, builds all crates
       ```yaml
       rust-builder:
         build:
           context: .
           dockerfile: Dockerfile.rust
         volumes:
           - .:/workspace
           - cargo-cache:/usr/local/cargo/registry
         working_dir: /workspace
         command: cargo build --workspace
       ```
     - **python-mcp**: Python 3.13 image, mounts `mcp/` directory, runs FastMCP server
       ```yaml
       python-mcp:
         build:
           context: ./mcp
           dockerfile: Dockerfile.python
         ports:
           - "8000:8000"
         volumes:
           - ./mcp:/app
         depends_on:
           - rust-builder
       ```
     - **sqlite-volume**: Named volume for SQLite database persistence
  3. Create `Dockerfile.rust`:
     - Base: `rust:nightly-slim`
     - Install: protoc, buf, build-essential
     - Pin protoc version matching Makefile
  4. Create `Dockerfile.python` (in `mcp/`):
     - Base: `python:3.13-slim`
     - Install: uv
     - Copy pyproject.toml, install deps
  5. Define named volumes: `cargo-cache`, `sqlite-data`
  6. Add environment variables: `RUST_LOG=debug`, `SQLITE_PATH=/data/agileplus.db`
- **Files**: `docker-compose.yml`, `Dockerfile.rust`, `mcp/Dockerfile.python`
- **Parallel?**: Yes -- independent after T001
- **Validation**: `docker-compose config` validates without error; `docker-compose build` completes (may take time on first run).
- **Notes**: The Python MCP service depends on the Rust gRPC server, but at scaffold time both are stubs. Use `healthcheck` directives so `depends_on` works correctly. Pin all base image tags to specific versions for reproducibility.

### Subtask T006 -- Create `buf.yaml` + `proto/agileplus.proto` and generate Rust/Python stubs

- **Purpose**: Define the gRPC contract between Rust core and Python MCP service. Generate typed stubs in both languages so WP02 (Python) and WP14 (gRPC server) can immediately use them.
- **Steps**:
  1. Create `proto/` directory at repository root.
  2. Create `proto/agileplus.proto` based on `contracts/agileplus.proto` from the spec directory. If that file exists, copy and adapt it. If not, create the proto with these services:
     ```protobuf
     syntax = "proto3";
     package agileplus;

     service AgilePlusCore {
       rpc GetFeature (GetFeatureRequest) returns (FeatureResponse);
       rpc ListFeatures (ListFeaturesRequest) returns (ListFeaturesResponse);
       rpc TransitionFeature (TransitionRequest) returns (TransitionResponse);
       rpc GetAuditTrail (GetAuditTrailRequest) returns (AuditTrailResponse);
       rpc CheckGovernance (GovernanceCheckRequest) returns (GovernanceCheckResponse);
       rpc DispatchCommand (CommandRequest) returns (CommandResponse);
       rpc StreamAgentEvents (AgentEventRequest) returns (stream AgentEvent);
     }
     ```
  3. Define all message types matching the data model entities (Feature, WorkPackage, AuditEntry, etc.).
  4. Create `buf.yaml` for protobuf linting:
     ```yaml
     version: v2
     lint:
       use:
         - STANDARD
     breaking:
       use:
         - FILE
     ```
  5. Create `buf.gen.yaml` for code generation configuration.
  6. Add `build.rs` to `agileplus-grpc` crate for tonic-build:
     ```rust
     fn main() -> Result<(), Box<dyn std::error::Error>> {
         tonic_build::compile_protos("../../proto/agileplus.proto")?;
         Ok(())
     }
     ```
  7. Add tonic-build as a build-dependency in `agileplus-grpc/Cargo.toml`.
  8. Create a Python generation script at `mcp/scripts/generate_proto.py` using grpcio-tools.
- **Files**: `proto/agileplus.proto`, `buf.yaml`, `buf.gen.yaml`, `crates/agileplus-grpc/build.rs`, `mcp/scripts/generate_proto.py`
- **Parallel?**: No -- depends on T001 completing and T003 creating the grpc crate
- **Validation**: `buf lint` passes; `cargo build -p agileplus-grpc` succeeds (generates Rust stubs); `cd mcp && python scripts/generate_proto.py` generates Python stubs.
- **Notes**: The proto file is the source of truth for the Rust-Python boundary. Changes here must be coordinated. Use `buf breaking` in CI to detect accidental breaking changes. The `tonic-build` requires `protoc` on the PATH -- this is why T004 (Makefile) and T005 (Docker) pin the protoc version.

---

## Test Strategy

- **Primary validation**: `cargo build --workspace && cargo test --workspace`
- **Expected**: 0 tests, 0 errors (all crates are stubs)
- **Lint**: `cargo clippy --workspace -- -D warnings` must pass
- **Format**: `cargo fmt --all -- --check` must pass
- **Proto**: `buf lint proto/` must pass
- **Docker**: `docker-compose config` must validate

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `tonic-build` requires system `protoc` | Build fails without it | Pin protoc version in Makefile, Docker, and CI; add check-protoc target |
| Rust 2024 edition may have breaking changes in nightly | Compilation failures | Pin nightly date in `rust-toolchain.toml`; test against latest stable as fallback |
| `git2` crate requires `libgit2` system library | Linking errors on some systems | Use `git2 = { features = ["vendored"] }` to bundle libgit2 |
| Workspace dependency version conflicts | Cargo resolve failures | Use `workspace.dependencies` for all shared deps; pin major versions |
| Docker image size bloat | Slow CI | Use multi-stage builds; cache cargo registry as named volume |

---

## Review Guidance

Reviewers should verify:

1. **Workspace completeness**: All 9 crates listed in root `Cargo.toml`, all compile.
2. **Edition correctness**: Every `Cargo.toml` uses `edition = "2024"` (via workspace inheritance or explicit).
3. **Dependency hygiene**: Core crate has ONLY serde, serde_json, chrono, sha2, thiserror. No I/O deps.
4. **Module structure**: Directory layout matches plan.md exactly (lines 50-176).
5. **Makefile targets**: All targets defined, correct commands, `.PHONY` declared.
6. **Docker**: Both Dockerfiles build, docker-compose.yml validates.
7. **Proto**: Proto file defines all services from contracts, `buf lint` passes.
8. **No logic**: This WP is scaffolding ONLY. No business logic, no tests beyond compilation.

---

## Activity Log

> **CRITICAL**: Activity log entries MUST be in chronological order (oldest first, newest last).

### How to Add Activity Log Entries

**When adding an entry**:
1. Scroll to the bottom of this Activity Log section
2. **APPEND the new entry at the END** (do NOT prepend or insert in middle)
3. Use exact format: `- YYYY-MM-DDTHH:MM:SSZ – agent_id – lane=<lane> – <action>`
4. Timestamp MUST be current time in UTC
5. Lane MUST match the frontmatter `lane:` field exactly

**Valid lanes**: `planned`, `doing`, `for_review`, `done`

- 2026-02-27T00:00:00Z – system – lane=planned – Prompt created.
