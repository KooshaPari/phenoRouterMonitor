# AgilePlus Dev Stack

The full AgilePlus development stack is managed via [`process-compose`](https://github.com/F1bonacc1/process-compose) with OrbStack containers for stateful backing services.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    process-compose                          │
│                                                             │
│  Native processes          OrbStack containers              │
│  ─────────────────         ───────────────────              │
│  nats-server (JetStream)   dragonfly  :6379                 │
│  neo4j                     postgres   :5432                  │
│  minio                                                      │
│  plane-api (Python)                                         │
│  plane-web (Next.js)                                        │
│  plane-worker (Celery)                                      │
│  plane-beat  (Celery)                                       │
│  agileplus-api (Rust)                                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Service URLs

| Service | URL | Notes |
|---------|-----|-------|
| **agileplus-api** | `http://localhost:3000` | Rust API, primary backend |
| **agileplus-api health** | `http://localhost:3000/health` | Readiness probe |
| **Plane.so (web)** | `http://localhost:3100` | Project management UI |
| **Plane API** | `http://localhost:8000` | Django REST API |
| **NATS** | `nats://localhost:4222` | JetStream messaging |
| **NATS monitoring** | `http://localhost:8222/healthz` | Health + metrics |
| **Dragonfly** | `redis://localhost:6379` | Redis-compatible cache/queue |
| **PostgreSQL** | `postgresql://localhost:5432` | Plane database |
| **Neo4j** | `bolt://localhost:7687` | Graph DB (agileplus-api) |
| **MinIO** | `http://localhost:9000` | Object storage |
| **MinIO Console** | `http://localhost:9001` | Admin UI |

---

## Prerequisites

Install all dependencies with Homebrew:

```bash
brew bundle --file=Brewfile
```

Key requirements:

| Tool | Purpose |
|------|---------|
| `process-compose` | Process orchestrator |
| `orb` | OrbStack CLI (manages Dragonfly + PostgreSQL containers) |
| `nats-server` | NATS JetStream server |
| `minio` | S3-compatible object storage |
| `neo4j` | Graph database |
| `cargo` / Rust toolchain | Build agileplus-api |
| Python 3.11+ + `.venv` | Plane API/worker |
| Node.js + `npx` | Plane web frontend |

OrbStack must be installed and running: <https://orbstack.dev>

---

## Quick Start

```bash
# 1. Install deps (first time only)
brew bundle --file=Brewfile

# 2. Bootstrap Plane (first time only — clones + migrates)
bash scripts/setup-plane.sh

# 3. Start the full stack
task up

# 4. Watch logs (all services)
task logs

# 5. Check status
task status

# 6. Shut down
task down
```

---

## Task Reference

All tasks are defined in `Taskfile.yml`.

| Task | Equivalent | Description |
|------|-----------|-------------|
| `task up` | `process-compose up` | Start full stack (detached) |
| `task down` | `process-compose down` | Stop all services |
| `task logs` | `process-compose logs` | Tail all service logs |
| `task status` | `process-compose list` | Show service health |
| `task dev:restart` | down + up | Full restart |
| `task dev:deps` | — | Verify brew deps installed |

---

## OrbStack Containers

Dragonfly and PostgreSQL run as OrbStack containers managed by:

- `scripts/orb-up.sh` — start containers, block until healthy
- `scripts/orb-down.sh` — stop and remove containers

The `orb-containers` process in `process-compose.yml` wraps these scripts. All other processes wait for `orb-containers` to be healthy before starting.

---

## Logs

All service logs land in `.agileplus/logs/`:

```
.agileplus/logs/
  agileplus-api.log
  nats.log
  neo4j.log
  minio.log
  orb-containers.log
  plane-api.log
  plane-web.log
  plane-worker.log
  plane-beat.log
  process-compose.log
```

---

## Environment Variables

Copy `.env.example` to `.env` and adjust:

```bash
cp .env.example .env
```

Key variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `PLANE_POSTGRES_PASSWORD` | `agileplus-dev` | Postgres password for Plane |
| `PLANE_SECRET_KEY` | `agileplus-dev-secret-key` | Django secret key |
| `RUST_LOG` | `info,agileplus=debug` | Rust log level |

---

## Troubleshooting

**OrbStack containers fail to start**
- Ensure OrbStack is running: open OrbStack.app or `open -a OrbStack`
- Check: `orb list`

**Neo4j fails readiness probe**
- Neo4j takes ~15 s to start. If it times out: `process-compose restart neo4j`

**agileplus-api fails to compile**
- Run `cargo build -p agileplus-api` manually to see errors

**Port already in use**
- Check: `lsof -i :<port>` and kill the conflicting process
