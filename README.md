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

## Running Plane.so + Dragonfly (OrbStack)

AgilePlus ships two complementary approaches for running Plane.so and Dragonfly locally.
Both require [OrbStack](https://orbstack.dev) as the container runtime (macOS).

### Approach A — Native processes + OrbStack containers (default, recommended)

This is the default `process-compose` stack. Dragonfly and PostgreSQL run as OrbStack
containers; Plane.so API/worker/beat run as native Python processes; Plane web runs
natively via Node. This gives fast file-watch reloads and easy breakpoint debugging.

```bash
# 1. One-time: bootstrap Plane sources and Python env
bash scripts/setup-plane.sh

# 2. Copy env template and fill in secrets
cp .env.example .env
# Set PLANE_SECRET_KEY to output of: openssl rand -hex 32

# 3. Start full stack (OrbStack must be running)
process-compose up

# Services become available at:
#   AgilePlus API  http://localhost:3000
#   Plane.so web   http://localhost:3100
#   Plane.so API   http://localhost:8000
#   Dragonfly      redis://localhost:6379
#   PostgreSQL     postgresql://agileplus:agileplus-dev@localhost:5432/plane
#   NATS           nats://localhost:4222
#   MinIO          http://localhost:9000 (console: :9001)
#   Neo4j          bolt://localhost:7687 (browser: :7474)
```

The `orb-containers` process in `process-compose.yml` calls `scripts/orb-up.sh` which
starts (or reuses) the `agileplus-dragonfly` and `agileplus-postgres` containers.
All Plane processes depend on `orb-containers: process_healthy` before starting.

### Approach B — Fully containerized (CI or no Python/Node runtime)

Use `docker-compose.plane.yml` to run the entire Plane.so stack as OrbStack containers.
This is useful for CI pipelines or environments where native Python/Node setup is not
feasible.

```bash
# Start only the backing stores (Dragonfly + Postgres)
docker compose -f docker-compose.plane.yml up dragonfly plane-db -d

# Start the full containerized Plane stack
docker compose -f docker-compose.plane.yml up -d

# Health check
redis-cli -p 6379 ping                              # expected: PONG
pg_isready -h localhost -p 5432                     # expected: accepting connections
curl -s http://localhost:8000/api/health | jq .     # expected: {"status":"ok"}
curl -s -o /dev/null -w "%{http_code}" http://localhost:3100/  # expected: 200

# Tear down
docker compose -f docker-compose.plane.yml down
```

### Container names and ports

| Container | Image | Port |
|-----------|-------|------|
| `agileplus-dragonfly` | `dragonflydb/dragonfly:latest` | `6379` |
| `agileplus-postgres` | `postgres:16-alpine` | `5432` |
| `agileplus-plane-api` | `makeplane/plane-backend:stable` | `8000` |
| `agileplus-plane-web` | `makeplane/plane-frontend:stable` | `3100` |
| `agileplus-plane-worker` | `makeplane/plane-backend:stable` | — |
| `agileplus-plane-beat` | `makeplane/plane-backend:stable` | — |

### Troubleshooting

```bash
# Check OrbStack status
orb status

# Inspect running containers
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# Dragonfly memory/stats
redis-cli -p 6379 info memory | grep used_memory_human

# Follow process-compose logs for a specific service
process-compose logs -f plane-api
process-compose logs -f orb-containers

# Restart a single container without disrupting the stack
docker restart agileplus-dragonfly
```

## Contributing

1. Edit code or proto files as needed.
2. Run `task lint` to validate.
3. Run `task test` to ensure no regressions.
4. Submit a PR — CI will run full quality gates.
