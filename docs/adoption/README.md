# Adoption Guides

This directory contains guides for adopting the canonical phenotype crates.

## Available Guides

- [phenotype-port-traits.md](./phenotype-port-traits.md) - Async trait patterns
- [phenotype-logging.md](./phenotype-logging.md) - Structured logging setup
- [phenotype-time.md](./phenotype-time.md) - Duration and retry patterns
- [phenotype-string.md](./phenotype-string.md) - String utilities
- [phenotype-iter.md](./phenotype-iter.md) - Iterator extensions
- [phenotype-crypto.md](./phenotype-crypto.md) - Hashing and crypto
- [phenotype-retry.md](./phenotype-retry.md) - Retry with backoff
- [phenotype-error-core.md](./phenotype-error-core.md) - Error handling
- [phenotype-health.md](./phenotype-health.md) - Health checks
- [phenotype-config-core.md](./phenotype-config-core.md) - Configuration

## Quick Reference

### Add to Cargo.toml

```toml
[dependencies]
phenotype-port-traits = { path = "../crates/phenotype-port-traits" }
phenotype-logging = { path = "../crates/phenotype-logging" }
phenotype-time = { path = "../crates/phenotype-time" }
phenotype-error-core = { path = "../crates/phenotype-error-core" }
```

### Basic Usage Examples

```rust
// Use canonical error types
use phenotype_error_core::{AppError, ApiError};

// Use structured logging
use phenotype_logging::{setup_logging, LogLevel};

// Use duration constants
use phenotype_time::duration;

// Use async traits
use phenotype_port_traits::inbound::{UseCase, CommandHandler};
```

## Migration Checklist

- [ ] Replace custom error types with `phenotype-error-core`
- [ ] Adopt `phenotype-logging` in binaries
- [ ] Use `phenotype-time::duration` constants
- [ ] Replace custom duration calculations with `phenotype-time`
- [ ] Adopt `phenotype-retry` for retry logic
- [ ] Use `phenotype-health` for health checks
