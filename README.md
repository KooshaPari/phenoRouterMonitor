# AgilePlus Workspace

## Overview

AgilePlus is a schema-driven project management platform built on Rust and gRPC. This repository is a **monorepo** containing the core domain logic, CLI, API server, and Protocol Buffer definitions for the entire ecosystem.

## Repository Layout

| Path | Description |
|------|-------------|
| `crates/agileplus-domain` | Core domain entities, business logic, and port abstractions |
| `crates/agileplus-cli` | The `agileplus` command-line interface |
| `crates/agileplus-api` | Axum-based HTTP API server |
| `crates/agileplus-sqlite` | SQLite storage adapter implementation |
| `crates/agileplus-git` | Git VCS adapter implementation |
| `crates/agileplus-plane` | Plane.so integration adapter |
| `crates/agileplus-events` | Event stream and audit log management |
| `proto/agileplus/v1/` | Protocol Buffer definitions for inter-service gRPC contracts |
| `rust/` | Rust crate (`agileplus-proto`) with tonic/prost codegen |
| `python/` | Python package (`agileplus-proto`) with grpcio stubs |

## Getting Started

### Prerequisites

- Rust toolchain (v1.86+)
- [Task](https://taskfile.dev/installation/) (Taskfile.yml replaces Makefiles)
- [buf](https://buf.build/docs/installation) v2+
- Python 3.14+ with [uv](https://docs.astral.sh/uv/)

### Common Tasks

```bash
# Lint the entire workspace
task lint

# Run all tests
task test

# Format all files
task fmt

# Build the CLI
cargo build -p agileplus-cli
```

### Protos and Codegen

```bash
# Lint proto files
task proto:lint

# Generate gRPC stubs for Rust and Python
task proto:gen
```

## Architecture

AgilePlus follows **Hexagonal Architecture** (Ports and Adapters):

1. **Domain**: Pure business logic and entity definitions in `crates/agileplus-domain`.
2. **Ports**: Trait definitions in `crates/agileplus-domain/src/ports/` for storage, VCS, and observability.
3. **Adapters**: Concrete implementations in `crates/agileplus-sqlite`, `crates/agileplus-git`, etc.
4. **Primary Adapters**: The CLI and API server that drive the domain logic.

## Breaking Change Policy (Protos)

All proto changes are checked against `main` using `buf breaking`. Breaking changes require:

1. A version bump in `buf.yaml` module path (e.g., `v1` → `v2`)
2. Explicit documentation in the PR description
3. Coordination with all downstream consumers

## Contributing

1. Edit code or proto files as needed.
2. Run `task lint` to validate.
3. Run `task test` to ensure no regressions.
4. Submit a PR — CI will run full quality gates.
