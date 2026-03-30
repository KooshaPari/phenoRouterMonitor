# phenotype-retry Adoption Guide

## Overview

`phenotype-retry` provides canonical retry logic with exponential backoff.

## Quick Start

### Add Dependency

```toml
[dependencies]
phenotype-retry = { path = "../crates/phenotype-retry" }
```

## Basic Retry

```rust
use phenotype_retry::{retry, RetryConfig};

let result = retry(
    || async { call_api().await },
    RetryConfig::default(),
).await?;
```

## Custom Configuration

```rust
use phenotype_retry::{retry, RetryConfig, RetryError};

let config = RetryConfig::default()
    .max_retries(5)
    .initial_delay(Duration::from_millis(100))
    .max_delay(Duration::from_secs(30))
    .backoff_multiplier(2.0)
    .jitter(0.1);

let result = retry(
    || async { call_api().await },
    config,
).await?;
```

## Builder Pattern

```rust
use phenotype_retry::{retry, ExponentialBackoff};

let result = retry(
    || async { call_api().await },
    ExponentialBackoff::builder()
        .max_retries(3)
        .base_delay(Duration::from_millis(100))
        .build(),
).await?;
```

## Available Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `max_retries` | 3 | Maximum retry attempts |
| `initial_delay` | 100ms | Initial delay |
| `max_delay` | 30s | Maximum delay cap |
| `backoff_multiplier` | 2.0 | Exponential factor |
| `jitter` | 0.0 | Random jitter (0.0-1.0) |

## Error Handling

```rust
use phenotype_retry::RetryError;

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(RetryError::Exhausted { attempts, last_error }) => {
        eprintln!("Failed after {} attempts: {:?}", attempts, last_error);
    }
    Err(RetryError::NonRetryable(e)) => {
        eprintln!("Non-retryable error: {:?}", e);
    }
}
```

## Related Crates

- `phenotype-time` - Duration constants for retry delays
- `phenotype-logging` - Log retry attempts
