# Phenotype Dependency Standards

Standardized dependency choices for the Phenotype ecosystem across all supported languages.

## Purpose

This document defines approved dependencies for each language in the Phenotype ecosystem. These standards ensure:

- **Consistency**: Similar projects use similar tools
- **Maintainability**: Reduced cognitive load when switching between projects
- **Security**: Pre-vetted dependencies with proven track records
- **Performance**: Battle-tested libraries optimized for production
- **Hiring**: Easier onboarding for new team members

## Rust Standards

| Function | Standard | Reason |
|----------|----------|--------|
| Web Framework | `axum` | Tokio-native, modern, excellent middleware system |
| Database | `sqlx` | Async-first, compile-time checked queries, no ORM bloat |
| CLI | `clap` v4 | Industry standard, derive macros, shell completions |
| HTTP Client | `reqwest` | Simple API, widely used, async support |
| Serialization | `serde` | De facto standard, zero-cost abstractions |
| Async Runtime | `tokio` | Standard async runtime, mature ecosystem |
| Testing | built-in + `tokio-test` | Native support, async test utilities |
| Error Handling | `thiserror` + `anyhow` | Library vs application error patterns |
| Logging | `tracing` | Structured logging, async-aware, OpenTelemetry support |
| Metrics | `prometheus` | Industry standard, pull-based, efficient |
| Configuration | `config` | Layered config (env, file, CLI) with validation |
| Validation | `validator` | Derive-based validation, JSON Schema support |
| Auth/JWT | `jsonwebtoken` | Most popular JWT library, `ring` backend |
| Password Hashing | `argon2` | Modern, secure, OWASP recommended |
| Cryptography | `ring` | High-quality, safe Rust crypto |
| Caching | `moka` | High-performance concurrent cache |
| Task Queue | `apalis` | Async job processing with multiple backends |
| WebSockets | `tokio-tungstenite` | Standard WebSocket implementation |
| gRPC | `tonic` | Async gRPC with `prost` codegen |
| Testing (Mock) | `mockall` | Powerful mocking with derive macros |
| Testing (HTTP) | `wiremock` | HTTP mocking for integration tests |
| Documentation | `rustdoc` + `cargo doc` | Built-in, comprehensive |
| Benchmarking | `criterion` | Statistics-aware benchmarking |

### Rust Version Policy

- **MSRV**: 1.80+ (6 months behind latest stable)
- **Edition**: 2021
- **Lints**: `#![deny(warnings)]` in CI, `#![warn(missing_docs)]`

## Python Standards

| Function | Standard | Reason |
|----------|----------|--------|
| Web Framework | `FastAPI` | Performance, modern, OpenAPI gen, type hints |
| Validation | `Pydantic` v2 | Fast Rust core, type-safe, JSON Schema |
| Testing | `pytest` | Feature-rich, fixtures, plugins |
| Async Runtime | `asyncio` + `anyio` | Native support, compatibility layer |
| CLI | `typer` | Modern, type hints, `click` backend |
| HTTP Client | `httpx` | Async support, `requests`-compatible API |
| Database (ORM) | `SQLAlchemy` 2.x | Mature, async support, type hints |
| Database (Migrations) | `alembic` | SQLAlchemy companion, battle-tested |
| Database (Driver) | `asyncpg` (PostgreSQL), `aiosqlite` (SQLite) | Native async drivers |
| Caching | `redis-py` | Standard Redis client, async support |
| Task Queue | `celery` + `redis` | Mature, feature-rich, monitoring |
| Data Science | `pandas`, `numpy`, `polars` | Standard, Polars for performance |
| ML/AI | `transformers`, `torch`, `litellm` | Industry standard, multi-provider LLM |
| Configuration | `pydantic-settings` | Pydantic-based, env var support |
| Logging | `structlog` | Structured logging, JSON output |
| Observability | `opentelemetry` | OTel standard, vendor-neutral |
| HTTP Server | `uvicorn` | ASGI server, `uvloop` on Unix |
| Type Checking | `mypy` + `basedpyright` | Dual validation for safety |
| Linting | `ruff` | Fast Rust-based linter/formatter |
| Testing (Mock) | `pytest-mock`, `respx` | Mocking and HTTP mocking |
| Testing (Coverage) | `pytest-cov` | Coverage reporting |
| Package Management | `uv` | Fast Rust-based resolver/installer |
| Environment | `python-dotenv` | `.env` file loading |
| Data Parsing | `msgspec` | Fast validation/serialization, JSON/msgpack |
| Image Processing | `Pillow` | Standard, wide format support |
| Process Management | `psutil` | Cross-platform process info |
| Scheduling | `APScheduler` | Cron-like scheduling |
| File Watching | `watchdog` | Cross-platform file events |
| CLI Testing | `typer-cli` + `pytest` | CLI testing utilities |

### Python Version Policy

- **Minimum**: 3.11
- **Target**: 3.13+
- **Type Hints**: Required for public APIs
- **Typing Mode**: `strict` where feasible, `standard` minimum

## TypeScript Standards

| Function | Standard | Reason |
|----------|----------|--------|
| Web Framework | `Fastify` | Performance, modern plugin system |
| Web Framework (Alt) | `Express` | Ubiquitous, massive ecosystem |
| Validation | `Zod` | Type-safe, runtime validation, DX |
| Testing | `Vitest` | Fast, modern, Jest-compatible |
| Testing (E2E) | `Playwright` | Cross-browser, reliable |
| Build | `tsc` + `esbuild` | Type checking + fast bundling |
| HTTP Client | `axios` | Widely used, interceptors, browser/Node |
| HTTP Client (Alt) | `ky` | Modern, fetch-based, smaller |
| Database ORM | `Prisma` | Type-safe, migrations, excellent DX |
| Database ORM (Alt) | `Drizzle` | SQL-like, lightweight, fast |
| Database (Query) | `Kysely` | Type-safe SQL builder |
| CLI | `commander` | Standard, battle-tested |
| CLI (Modern) | `oclif` | Heroku's framework, plugins |
| Configuration | `cosmiconfig` | Standard config discovery |
| Logging | `pino` | Fast, structured, JSON |
| Observability | `otel` (OpenTelemetry) | Standard instrumentation |
| Monorepo | `Turborepo` + `pnpm` | Fast, caching, task orchestration |
| Linting | `eslint` + `typescript-eslint` | Standard, comprehensive |
| Formatting | `prettier` | Opinionated, consistent |
| Type Checking | `tsc --noEmit` | Strict mode required |
| Testing (Mock) | `msw` | API mocking for browser/Node |
| Testing (Coverage) | `v8` (built into Vitest) | Native coverage |
| React Framework | `Next.js` | Full-stack React, App Router |
| React Framework (Alt) | `Vite` + `React` | SPA, fast HMR, flexible |
| React State | `Zustand` | Simple, hooks-based |
| React Query | `TanStack Query` | Server state, caching, sync |
| React Forms | `React Hook Form` + `Zod` | Performance, validation |
| UI Components | `Radix` + `Tailwind` | Accessible, headless, customizable |
| CSS Framework | `Tailwind CSS` | Utility-first, design system friendly |
| Real-time | `Socket.io` | WebSocket fallback, rooms |
| Real-time (Alt) | `PartyKit` | Edge-deployed, stateful |
| Documentation | `VitePress` | Fast, Vue-based, MDX |
| Documentation (Alt) | `Nextra` | Next.js docs framework |
| Package Building | `tsup` | Fast bundling for libraries |
| Node Version | `Volta` | Pin versions per project |

### TypeScript Version Policy

- **Target**: ES2022
- **Module**: ESM (`"type": "module"`)
- **Strictness**: `strict: true`, `noUncheckedIndexedAccess: true`
- **Declaration**: Emit declarations for libraries

## Go Standards

| Function | Standard | Reason |
|----------|----------|--------|
| Web Framework | `gin` | Fast, popular, middleware ecosystem |
| Web Framework (Alt) | `echo` | Minimal, fast, maintainable |
| Web Framework (Alt) | `chi` | Composable, lightweight |
| CLI | `cobra` + `viper` | Industry standard (k8s, docker) |
| Testing | `testing` + `testify` | Built-in + assertions |
| HTTP Client | `net/http` (stdlib) | Standard library, excellent |
| HTTP Client (Enhanced) | `retryablehttp` | Auto-retry, backoff |
| Database (SQL) | `sqlx` | Extensions over `database/sql` |
| Database (ORM) | `gorm` | Feature-rich, migrations |
| Database (ORM Alt) | `bun` | Modern, fast, PostgreSQL-focused |
| Database (PG) | `pgx` | Native PostgreSQL driver |
| Database (Migrations) | `golang-migrate` | CLI and library |
| Configuration | `koanf` | Hierarchical, env/file/flag |
| Validation | `go-playground/validator` | Struct validation, i18n |
| Serialization | `encoding/json` (stdlib) | Standard, fast enough |
| Serialization (Fast) | `goccy/go-json` | Drop-in faster replacement |
| Logging | `slog` (stdlib 1.21+) | Structured, standard |
| Logging (Alt) | `zap` | High-performance |
| Observability | `otel` (OpenTelemetry) | Standard instrumentation |
| Metrics | `prometheus/client_golang` | Standard client |
| Auth/JWT | `golang-jwt/jwt` | Most popular JWT library |
| Auth/OAuth | `golang.org/x/oauth2` | Standard OAuth2 |
| Password Hashing | `golang.org/x/crypto/bcrypt` | Standard, secure |
| Password Hashing (Alt) | `argon2` | Modern alternative |
| Cryptography | `crypto` (stdlib) | Standard, sufficient |
| Caching | `ristretto` (dgraph) | Fast, bounded cache |
| Caching (Redis) | `go-redis` | Standard Redis client |
| Task Queue | `asynq` | Redis-based, monitoring UI |
| Task Queue (Alt) | `machinery` | Celery-like, AMQP |
| gRPC | `google.golang.org/grpc` | Official implementation |
| WebSockets | `gorilla/websocket` | Gorilla's battle-tested ws |
| Documentation | `swag` | Swagger/OpenAPI generation |
| Testing (Mock) | `gomock` + `mockgen` | Standard mocking |
| Testing (Coverage) | built-in | `go test -cover` |
| Benchmarking | built-in | `go test -bench` |
| Build Tool | `goreleaser` | Release automation |
| Version | `runtime.Version()` + `debug.ReadBuildInfo()` | Standard versioning |

### Go Version Policy

- **Minimum**: Go 1.23+
- **Module**: Go modules (go.mod)
- **Linter**: `golangci-lint` (comprehensive)
- **Vet**: `go vet` (standard analysis)

## Category Definitions

### Web Framework
Minimal HTTP server with routing, middleware, request/response handling.

### Validation
Input validation with type coercion, error messages, and schema support.

### Testing
Unit and integration testing with assertions, mocking, and coverage.

### Database
SQL/NoSQL connectivity, ORM/query building, migrations.

### CLI
Command-line interface parsing, help generation, shell completion.

### HTTP Client
Making external HTTP requests with retries, timeouts, JSON handling.

### Caching
In-memory or distributed caching with TTL and eviction policies.

### Task Queue
Background job processing with retry, scheduling, and monitoring.

### Logging
Structured logging with levels, formatting, and output destinations.

### Observability
Metrics, tracing, and health checks for monitoring systems.

### Authentication
JWT, OAuth, session management, and password handling.

### Cryptography
Hashing, encryption, and secure random generation.

## Updating Standards

To propose changes to these standards:

1. Create an RFC in `docs/rfcs/`
2. Include motivation, alternatives, and migration plan
3. Get approval from 2+ maintainers
4. Update this document and MIGRATION.md

