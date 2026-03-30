# phenotype-time Adoption Guide

## Overview

`phenotype-time` provides canonical duration constants and timestamp utilities.

## Quick Start

### Add Dependency

```toml
[dependencies]
phenotype-time = { path = "../crates/phenotype-time" }
```

## Duration Constants

```rust
use phenotype_time::duration;

// Timeouts
let timeout = duration::NETWORK_TIMEOUT; // 60s

// Delays
let delay = duration::SHORT_DELAY; // 10ms

// Health checks
let health_timeout = duration::HEALTH_CHECK_TIMEOUT; // 30s
```

## Available Constants

| Constant | Duration | Use Case |
|----------|----------|----------|
| `SHORT_TIMEOUT` | 100ms | Quick operations |
| `DEFAULT_TIMEOUT` | 5s | Standard operations |
| `HEALTH_CHECK_TIMEOUT` | 30s | Health checks |
| `DB_QUERY_TIMEOUT` | 10s | Database queries |
| `NETWORK_TIMEOUT` | 60s | Network operations |
| `LONG_OPERATION` | 5min | Long-running tasks |
| `SHORT_DELAY` | 10ms | Brief pauses |
| `MEDIUM_DELAY` | 100ms | Medium pauses |
| `LONG_DELAY` | 1s | Longer pauses |

## Retry with Backoff

```rust
use phenotype_time::retry::retry_with_backoff;

let result = retry_with_backoff(
    || async { fetch_data().await },
    3,              // max retries
    100,            // initial delay ms
    2.0,            // backoff multiplier
).await?;
```

## Instant Utilities

```rust
use phenotype_time::Instant;

let deadline = Instant::now() + duration::DEFAULT_TIMEOUT;
if Instant::now() < deadline {
    // Continue operation
}
```

## Timestamp Formatting

```rust
use phenotype_time::Timestamp;

let ts = Timestamp::now();
println!("{}", ts); // ISO 8601 format
```

## Related Crates

- `phenotype-logging` - Logging with timestamps
- `phenotype-retry` - Retry with duration constants
