# Phenotype InfraKit

> Shared infrastructure crates for the Phenotype ecosystem - Generic, reusable infrastructure components

## Overview

Phenotype InfraKit provides foundational infrastructure components used across the entire Phenotype ecosystem. These crates are designed to be:

- **Generic**: Not tied to specific business logic
- **Reusable**: Usable across multiple services
- **Well-tested**: Comprehensive test coverage
- **Documented**: Full API documentation
- **Performance-oriented**: Optimized for production use

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Phenotype InfraKit Architecture                         │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Core Infrastructure Layer                        │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Error   │ │   Git    │ │  Health  │ │  Config  │         │   │
│  │   │  Core    │ │  Core    │ │  Check   │ │  Core    │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Cache   │ │  Event   │ │  Policy  │ │  State   │         │   │
│  │   │ Adapter  │ │ Sourcing │ │  Engine  │ │  Machine │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │Telemetry │ │Validation│ │Contracts │ │   ID     │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Application Layer                                 │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Agent   │ │   Task   │ │  Skill   │ │   Hub    │         │   │
│  │   │   Core   │ │  Engine  │ │ Registry │ │          │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Crates

### Core Infrastructure

| Crate | Description | Status | Docs |
|-------|-------------|--------|------|
| `phenotype-error-core` | Canonical error types with context and backtrace | Stable | [Docs](crates/phenotype-error-core/README.md) |
| `phenotype-git-core` | Git operations abstraction | Stable | [Docs](crates/phenotype-git-core/README.md) |
| `phenotype-health` | Health check abstraction with probes | Stable | [Docs](crates/phenotype-health/README.md) |
| `phenotype-config-core` | Configuration loading (TOML, YAML, JSON, env) | Stable | [Docs](crates/phenotype-config-core/README.md) |

### Data & State

| Crate | Description | Status | Docs |
|-------|-------------|--------|------|
| `phenotype-event-sourcing` | Append-only event store with SHA-256 chains | Beta | [Docs](crates/phenotype-event-sourcing/README.md) |
| `phenotype-cache-adapter` | Two-tier LRU + DashMap cache with TTL | Stable | [Docs](crates/phenotype-cache-adapter/README.md) |
| `phenotype-state-machine` | Generic FSM with transition guards | Stable | [Docs](crates/phenotype-state-machine/README.md) |
| `phenotype-validation` | Data validation with composable rules | Stable | [Docs](crates/phenotype-validation/README.md) |

### Cross-Cutting Concerns

| Crate | Description | Status | Docs |
|-------|-------------|--------|------|
| `phenotype-telemetry` | OpenTelemetry integration, metrics, tracing | Beta | [Docs](crates/phenotype-telemetry/README.md) |
| `phenotype-policy-engine` | Rule-based policy evaluation with TOML | Beta | [Docs](crates/phenotype-policy-engine/README.md) |
| `phenotype-contracts` | Shared traits and types across ecosystem | Stable | [Docs](crates/phenotype-contracts/README.md) |

## Quick Start

### Adding Dependencies

```toml
[dependencies]
# Core infrastructure
phenotype-error-core = { git = "https://github.com/KooshaPari/phenotype-infrakit.git" }
phenotype-config-core = { git = "https://github.com/KooshaPari/phenotype-infrakit.git" }
phenotype-health = { git = "https://github.com/KooshaPari/phenotype-infrakit.git" }

# Data layer
phenotype-cache-adapter = { git = "https://github.com/KooshaPari/phenotype-infrakit.git" }
phenotype-validation = { git = "https://github.com/KooshaPari/phenotype-infrakit.git" }
```

### Using Error Core

```rust
use phenotype_error_core::{Error, Result, Context};

fn main() -> Result<()> {
    let result = some_operation()
        .context("Failed to perform operation")?;
    
    Ok(())
}
```

### Using Config Core

```rust
use phenotype_config_core::{ConfigLoader, Source};

#[derive(Debug, serde::Deserialize)]
struct AppConfig {
    name: String,
    port: u16,
}

fn load_config() -> anyhow::Result<AppConfig> {
    let config = ConfigLoader::new()
        .add_source(Source::file("config.toml"))
        .add_source(Source::env("APP_"))
        .load::<AppConfig>()?;
    
    Ok(config)
}
```

### Using Cache Adapter

```rust
use phenotype_cache_adapter::{Cache, CacheConfig};

fn main() {
    let cache = Cache::new(CacheConfig {
        max_size: 1000,
        ttl: std::time::Duration::from_secs(300),
    });
    
    cache.set("key", "value");
    let value: Option<String> = cache.get("key");
}
```

## Crate Dependencies

```
phenotype-error-core (base)
    ↑
phenotype-contracts (uses error types)
    ↑
phenotype-config-core (uses contracts)
    ↑
phenotype-health (uses contracts)
    ↑
phenotype-telemetry (uses contracts)
    ↑
phenotype-validation (uses contracts)
    ↑
phenotype-cache-adapter (uses validation)
    ↑
phenotype-policy-engine (uses validation)
    ↑
phenotype-state-machine (uses policy)
    ↑
phenotype-event-sourcing (uses state machine)
    ↑
phenotype-git-core (uses telemetry)
```

## Development

### Prerequisites

- Rust 1.75+
- Cargo workspace tools
- just (task runner)

### Building

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p phenotype-error-core

# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace

# Generate docs
cargo doc --workspace
```

### Workspace Structure

```
phenotype-infrakit/
├── Cargo.toml           # Workspace manifest
├── README.md            # This file
├── crates/              # All infrastructure crates
│   ├── phenotype-error-core/
│   ├── phenotype-git-core/
│   ├── phenotype-health/
│   ├── phenotype-config-core/
│   ├── phenotype-telemetry/
│   ├── phenotype-validation/
│   ├── phenotype-event-sourcing/
│   ├── phenotype-cache-adapter/
│   ├── phenotype-policy-engine/
│   ├── phenotype-state-machine/
│   └── phenotype-contracts/
├── docs/                # Additional documentation
├── tests/               # Integration tests
└── benches/             # Benchmarks
```

## Performance Benchmarks

| Crate | Operation | Time | Memory |
|-------|-----------|------|--------|
| phenotype-error-core | Error creation | <1μs | 0 alloc |
| phenotype-cache-adapter | Cache lookup | <100ns | minimal |
| phenotype-validation | Simple validation | <1μs | minimal |
| phenotype-event-sourcing | Event append | <1ms | append-only |

## Security

All crates follow security best practices:
- No unsafe code unless necessary (and documented)
- Input validation on all public APIs
- Constant-time operations where required
- Audit logging for sensitive operations

## Documentation

- [Specification](SPEC.md) - Detailed system specification
- [Implementation Plan](PLAN.md) - Roadmap and milestones
- [Contributing](CONTRIBUTING.md) - Development guidelines

## License

MIT License - see [LICENSE](LICENSE) for details.

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Async Runtime](https://tokio.rs/)
- [ThisError](https://github.com/dtolnay/thiserror)
- [Serde](https://serde.rs/)
