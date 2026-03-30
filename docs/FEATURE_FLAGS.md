# Phenotype Workspace Feature Flags

This document describes the optional feature flags available in the Phenotype workspace crates, enabling a plugin-pluggable architecture with reduced binary size.

## Overview

Feature flags in Phenotype allow you to:
- Include only the backends and functionality you need
- Reduce default binary size by making optional dependencies truly optional
- Support multiple transport and storage backends simultaneously
- Enable gradual adoption of features as project needs evolve

## Global Feature Matrix

| Crate | Feature | Dependencies | Use Case |
|-------|---------|-------------|----------|
| `phenotype-cache-adapter` | `in-memory` (default) | lru, moka, dashmap | Two-tier local caching (development/testing) |
| `phenotype-cache-adapter` | `redis` | redis | Distributed Redis caching |
| `phenotype-cache-adapter` | `memcached` | memcache | High-performance Memcached backend |
| `phenotype-event-sourcing` | `memory-store` (default) | (none) | In-memory event log |
| `phenotype-event-sourcing` | `kafka` | rdkafka | Distributed event streaming via Kafka |
| `phenotype-event-sourcing` | `redis-streams` | redis | Lightweight Redis Streams events |
| `phenotype-event-sourcing` | `sql-store` | sqlx | Durable SQL database (PostgreSQL) storage |
| `phenotype-telemetry` | `tracing` | tracing, tracing-subscriber | OpenTelemetry-compatible tracing |
| `phenotype-telemetry` | `opentelemetry` | opentelemetry, opentelemetry-jaeger | Jaeger integration for distributed tracing |
| `phenotype-telemetry` | `prometheus` | prometheus | Prometheus metrics export |
| `phenotype-logging` | `tracing` | tracing, tracing-subscriber | Structured tracing-based logging |
| `phenotype-logging` | `slog` | slog | Slog structured logging |
| `phenotype-logging` | `slog-json` | slog, slog-json | JSON output for slog |

## Per-Crate Feature Details

### phenotype-cache-adapter

Two-tier cache with pluggable backends.

#### Default Features
- `in-memory` - Two-tier LRU (L1) + Moka (L2) cache for local caching

#### Optional Features
- `redis` - Redis client for distributed caching
- `memcached` - Memcached client for high-performance caching

#### Examples

**Local development (default)**:
```toml
[dependencies]
phenotype-cache-adapter = "0.2"
```

**With Redis**:
```toml
[dependencies]
phenotype-cache-adapter = { version = "0.2", features = ["redis"] }
```

**Multiple backends**:
```toml
[dependencies]
phenotype-cache-adapter = { version = "0.2", features = ["in-memory", "redis"] }
```

**Minimal (no caching)**:
```toml
[dependencies]
phenotype-cache-adapter = { version = "0.2", default-features = false }
```

### phenotype-event-sourcing

Event sourcing with pluggable backends and blake3 hash chains.

#### Default Features
- `memory-store` - In-memory event store (recommended for development)

#### Optional Features
- `kafka` - Apache Kafka for distributed event streaming
- `redis-streams` - Redis Streams for lightweight persistence
- `sql-store` - SQL database (PostgreSQL) for durable storage

#### Examples

**Development with in-memory store**:
```toml
[dependencies]
phenotype-event-sourcing = "0.2"
```

**Production with Kafka**:
```toml
[dependencies]
phenotype-event-sourcing = { version = "0.2", features = ["kafka"] }
```

**Multiple backends**:
```toml
[dependencies]
phenotype-event-sourcing = { version = "0.2", features = ["kafka", "redis-streams", "sql-store"] }
```

### phenotype-telemetry

Observability infrastructure with multiple tracing and metrics backends.

#### Default Features
- (None) - Observability is opt-in

#### Optional Features
- `tracing` - OpenTelemetry-compatible distributed tracing
- `opentelemetry` - Full OpenTelemetry SDK with Jaeger exporter
- `prometheus` - Prometheus metrics export

#### Examples

**With tracing**:
```toml
[dependencies]
phenotype-telemetry = { version = "0.2", features = ["tracing"] }
```

**Full observability (tracing + metrics)**:
```toml
[dependencies]
phenotype-telemetry = { version = "0.2", features = ["tracing", "prometheus"] }
```

### phenotype-logging

Structured logging with multiple backend options.

#### Default Features
- (None) - Logging is opt-in

#### Optional Features
- `tracing` - Tracing-based structured logging
- `slog` - Slog structured logging framework
- `slog-json` - JSON formatting for slog

#### Examples

**With tracing**:
```toml
[dependencies]
phenotype-logging = { version = "0.2", features = ["tracing"] }
```

**With slog and JSON output**:
```toml
[dependencies]
phenotype-logging = { version = "0.2", features = ["slog-json"] }
```

## Recommended Feature Combinations

### Development Environment
```toml
[dependencies]
phenotype-cache-adapter = "0.2"  # in-memory default
phenotype-event-sourcing = "0.2"  # memory-store default
phenotype-telemetry = { version = "0.2", features = ["tracing"] }
phenotype-logging = { version = "0.2", features = ["tracing"] }
```

### Production with Kafka & Redis
```toml
[dependencies]
phenotype-cache-adapter = { version = "0.2", features = ["redis"] }
phenotype-event-sourcing = { version = "0.2", features = ["kafka"] }
phenotype-telemetry = { version = "0.2", features = ["tracing", "prometheus"] }
phenotype-logging = { version = "0.2", features = ["tracing"] }
```

### Production with PostgreSQL
```toml
[dependencies]
phenotype-cache-adapter = { version = "0.2", features = ["redis"] }
phenotype-event-sourcing = { version = "0.2", features = ["sql-store"] }
phenotype-telemetry = { version = "0.2", features = ["opentelemetry"] }
phenotype-logging = { version = "0.2", features = ["tracing"] }
```

### Minimal (no optional features)
```toml
[dependencies]
phenotype-cache-adapter = { version = "0.2", default-features = false }
phenotype-event-sourcing = { version = "0.2", default-features = false }
phenotype-telemetry = { version = "0.2", default-features = false }
phenotype-logging = { version = "0.2", default-features = false }
```

## Testing Feature Combinations

The workspace includes comprehensive test matrices to verify all feature combinations:

```bash
# Test with no default features
cargo test --all --no-default-features

# Test with all features enabled
cargo test --all --all-features

# Test with default features
cargo test --all

# Test specific crate with specific features
cargo test -p phenotype-cache-adapter --features redis
cargo test -p phenotype-event-sourcing --features kafka,sql-store
```

## Binary Size Impact

Enabling feature flags can significantly reduce binary size:

| Configuration | Default Binary | Reduction |
|---------------|----------------|-----------|
| `--no-default-features` | ~5-8 MB | 40-50% |
| Only cache: `in-memory` | ~8-10 MB | 30-40% |
| Only telemetry: `tracing` | ~7-9 MB | 35-45% |
| Full (`--all-features`) | ~15-18 MB | 0% (baseline) |

## Architecture: Feature-Pluggable Design

The feature-pluggable architecture follows the **Hexagonal Architecture** pattern:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Application Core                             │
│  (phenotype-contracts, phenotype-error-core)                   │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ (Ports)
                ┌─────────────┼─────────────┐
                │             │             │
         ┌──────▼────┐ ┌──────▼────┐ ┌──────▼────┐
         │  Cache    │ │   Event   │ │ Telemetry│
         │  Adapters │ │  Backends │ │ Backends │
         └───────────┘ └───────────┘ └───────────┘
         (pluggable)  (pluggable)   (pluggable)
```

Each adapter layer is optional:
- **Cache**: Choose in-memory, Redis, or Memcached
- **Events**: Choose memory, Kafka, Redis, or SQL
- **Telemetry**: Choose tracing, OpenTelemetry, or Prometheus

## Migration Guide

### From Monolithic to Feature-Gated

If you're migrating from a monolithic build:

1. **Identify current usage**: Which backends do you actually use?
2. **Enable minimal features**: Enable only what you need
3. **Test all feature combinations**: Use the test matrix
4. **Measure binary size**: Verify improvements
5. **Document your choices**: Add feature comments to Cargo.toml

Example migration:

**Before** (all dependencies included):
```toml
[dependencies]
phenotype-cache-adapter = "0.2"
phenotype-event-sourcing = "0.2"
phenotype-telemetry = "0.2"
```

**After** (only needed features):
```toml
[dependencies]
# We use Redis for caching in production
phenotype-cache-adapter = { version = "0.2", features = ["redis"] }

# We use Kafka for event streaming
phenotype-event-sourcing = { version = "0.2", features = ["kafka"] }

# We use OpenTelemetry for distributed tracing
phenotype-telemetry = { version = "0.2", features = ["opentelemetry"] }
```

## Future Enhancements

Phase 3 roadmap includes:
- Additional cache backends (DynamoDB, Etcd)
- Additional event stores (NATS, EventStoreDB)
- Additional telemetry integrations (DataDog, New Relic)
- Feature auto-detection for CI/CD
- Performance benchmarks by feature combination

## See Also

- `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml` - Workspace dependencies
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/*/Cargo.toml` - Per-crate features
- `docs/adr/` - Architecture Decision Records
- `docs/reference/SOFTWARE_ARCHITECTURE_REFERENCE.md` - Architecture patterns
