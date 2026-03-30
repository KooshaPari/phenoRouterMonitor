# phenotype-logging Adoption Guide

## Overview

`phenotype-logging` provides canonical logging setup with OpenTelemetry support.

## Quick Start

### Add Dependency

```toml
[dependencies]
phenotype-logging = { path = "../crates/phenotype-logging" }
tokio = { version = "1", features = ["rt-multi-thread"] }
```

### Basic Setup

```rust
use phenotype_logging::{init, Config, Level};

fn main() {
    init(&Config {
        level: Level::Info,
        ..Default::default()
    }).expect("failed to init logging");
}
```

### With OpenTelemetry

```rust
use phenotype_logging::{init_with_otel, Config};

fn main() {
    init_with_otel(
        "my-service",
        "http://localhost:4317"
    ).expect("failed to init logging with OTel");
}
```

### In Tokio Main

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    phenotype_logging::init(&Default::default());
    // Your code
    Ok(())
}
```

## Configuration

```rust
use phenotype_logging::{Config, Level, Format};

let config = Config {
    level: Level::Debug,
    format: Format::Json,
    include_thread_id: true,
    include_target: true,
    ..Default::default()
};
```

## Log Levels

| Level | Usage |
|-------|-------|
| `Error` | Failures that need immediate attention |
| `Warn` | Unexpected but recoverable situations |
| `Info` | Significant business events |
| `Debug` | Detailed debugging information |
| `Trace` | Very detailed tracing |

## OpenTelemetry Integration

```rust
// Initialize with custom OTel endpoint
init_with_otel("service-name", "https://otlp.example.com:4317");

// Spans are automatically created
tracing::info!("processing request");
```

## Related Crates

- `phenotype-error-core` - Error types with proper logging
- `phenotype-time` - Duration formatting in logs
