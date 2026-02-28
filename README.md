# agileplus-proto

Protocol Buffer definitions for the AgilePlus gRPC API.

## Overview

This repository is the **single source of truth** for all inter-service contracts in the AgilePlus ecosystem. It defines three gRPC services and a set of shared message types used across five repositories.

## Repository Layout

| Path | Description |
|------|-------------|
| `proto/agileplus/v1/` | Protocol Buffer definitions (4 files) |
| `proto/agileplus/v1/common.proto` | Shared message types (Feature, AuditEntry, etc.) |
| `proto/agileplus/v1/core.proto` | `AgilePlusCoreService` — feature lifecycle, governance, audit |
| `proto/agileplus/v1/agents.proto` | `AgentDispatchService` — agent spawn, review loop |
| `proto/agileplus/v1/integrations.proto` | `IntegrationsService` — Plane.so, GitHub, triage |
| `rust/` | Rust crate (`agileplus-proto`) with tonic/prost codegen |
| `python/` | Python package (`agileplus-proto`) with grpcio stubs |
| `buf.yaml` | buf v2 lint and breaking change configuration |
| `buf.gen.yaml` | buf codegen plugin configuration |

## Getting Started

### Prerequisites

- [buf](https://buf.build/docs/installation) v2+
- Rust toolchain (for building the Rust crate)
- Python 3.12+ with [uv](https://docs.astral.sh/uv/) (for the Python package)

### Lint

```bash
make lint
```

### Generate Stubs

```bash
make generate
```

### Build Rust Crate

```bash
cd rust && cargo build
```

### Install Python Package

```bash
cd python && uv sync
```

### Check for Breaking Changes

```bash
make breaking
```

## Breaking Change Policy

All proto changes are checked against `main` using `buf breaking`. Breaking changes require:

1. A version bump in `buf.yaml` module path (e.g., `v1` → `v2`)
2. Explicit documentation in the PR description
3. Coordination with all downstream consumers (agileplus-core, agileplus-mcp, agileplus-agents, agileplus-integrations)

## Contributing

1. Edit proto files in `proto/agileplus/v1/`
2. Run `make lint` to validate
3. Run `make generate` to regenerate stubs
4. Run `cargo build` in `rust/` and `uv sync` in `python/` to verify
5. Submit a PR — CI will run lint, breaking change detection, and build checks
