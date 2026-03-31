# Sentry SDK Configuration Examples

This document provides ready-to-use Sentry SDK configurations for Tier 1 repos (AgilePlus, phenotype-infrakit, heliosCLI).

## Quick Reference

| Repo | Language | SDK | Version | Integration Points |
|------|----------|-----|---------|-------------------|
| AgilePlus | Rust | sentry | 0.33+ | Main entry (CLI, API), test suite |
| phenotype-infrakit | Rust | sentry | 0.33+ | Binary entry points, benchmarks |
| heliosCLI | Rust | sentry | 0.33+ | TUI main, command handlers |

## 1. AgilePlus Configuration

### Workspace Setup

**File**: `AgilePlus/Cargo.toml` (workspace root)

```toml
[workspace.dependencies]
sentry = { version = "0.33", features = ["backtrace", "debug-images", "anyhow"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
```

### CLI Binary Entry Point

**File**: `AgilePlus/crates/agileplus-cli/src/main.rs`

```rust
use sentry::integrations::backtrace::BacktraceIntegration;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize Sentry early
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_DSN").ok(),
        release: sentry::release_name!(),
        environment: env::var("ENVIRONMENT")
            .ok()
            .map(|e| e.into()),
        integrations: vec![
            Box::new(BacktraceIntegration::new()),
            Box::new(sentry::integrations::panic::PanicIntegration::new()),
            Box::new(sentry::integrations::std_panic::StdPanicIntegration::new()),
        ],
        traces_sample_rate: 1.0,
        attach_stacktrace: true,
        ..Default::default()
    });

    // Add breadcrumb for startup
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: "app".into(),
        message: "Application started".into(),
        level: sentry::Level::Info,
        ..Default::default()
    });

    // Run CLI
    match run_cli().await {
        Ok(_) => {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: "app".into(),
                message: "Application completed successfully".into(),
                level: sentry::Level::Info,
                ..Default::default()
            });
        }
        Err(e) => {
            sentry::capture_error(&e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    // CLI logic
    Ok(())
}
```

### API Server Entry Point

**File**: `AgilePlus/crates/agileplus-api/src/main.rs`

```rust
use sentry::integrations::backtrace::BacktraceIntegration;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize Sentry
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_DSN").ok(),
        release: sentry::release_name!(),
        environment: env::var("ENVIRONMENT")
            .ok()
            .map(|e| e.into()),
        integrations: vec![
            Box::new(BacktraceIntegration::new()),
            Box::new(sentry::integrations::panic::PanicIntegration::new()),
            Box::new(sentry::integrations::std_panic::StdPanicIntegration::new()),
        ],
        traces_sample_rate: env::var("SENTRY_TRACES_SAMPLE_RATE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.1),
        ..Default::default()
    });

    // Start API server
    if let Err(e) = start_api_server().await {
        sentry::with_scope(
            |scope| {
                scope.set_tag("service", "api");
                scope.set_tag("startup_failed", "true");
                sentry::capture_error(&e);
            },
        );
        eprintln!("Failed to start API: {}", e);
        std::process::exit(1);
    }
}

async fn start_api_server() -> Result<(), Box<dyn std::error::Error>> {
    // Server initialization
    Ok(())
}
```

### Test Configuration

**File**: `AgilePlus/crates/agileplus-cli/src/lib.rs` (top of test module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn init_sentry() -> sentry::ClientGuard {
        sentry::init(sentry::ClientOptions {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: Some("test".into()),
            ..Default::default()
        })
    }

    #[test]
    fn test_error_handling() {
        let _guard = init_sentry();

        // Your test code
        let result: Result<(), String> = Err("Test error".to_string());

        if let Err(e) = result {
            sentry::capture_message(&format!("Test error: {}", e), sentry::Level::Error);
        }
    }

    #[tokio::test]
    async fn test_async_error_capture() {
        let _guard = init_sentry();

        // Async test code
        let error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "Async operation failed",
        );

        sentry::capture_error(&error);
    }
}
```

### .env Configuration

**File**: `AgilePlus/.env.example` (add to existing file)

```bash
# ── Sentry Configuration ────────────────────────────────────────────────
# Error tracking and performance monitoring
# Get DSN from: https://sentry.io/settings/phenotype/projects/agileplus/keys/
SENTRY_DSN=https://[your-key]@o[org-id].ingest.us.sentry.io/[project-id]
SENTRY_TRACES_SAMPLE_RATE=0.1
ENVIRONMENT=development
```

## 2. phenotype-infrakit Configuration

### Workspace Setup

**File**: `phenotype-infrakit/Cargo.toml` (workspace root)

```toml
[workspace.dependencies]
sentry = { version = "0.33", features = ["backtrace", "debug-images"] }
tokio = { version = "1", features = ["full"] }
```

### Binary Crate Example

**File**: `phenotype-infrakit/libs/example-cli/src/main.rs`

```rust
use sentry::integrations::backtrace::BacktraceIntegration;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize Sentry with minimal config
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_DSN").ok(),
        release: sentry::release_name!(),
        environment: env::var("ENVIRONMENT")
            .ok()
            .map(|e| e.into()),
        integrations: vec![
            Box::new(BacktraceIntegration::new()),
            Box::new(sentry::integrations::panic::PanicIntegration::new()),
        ],
        ..Default::default()
    });

    // Application code
    if let Err(e) = main_logic().await {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}

async fn main_logic() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running phenotype-infrakit");
    Ok(())
}
```

### Benchmark with Error Tracking

**File**: `phenotype-infrakit/libs/example-crate/benches/benchmark.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn init_sentry() {
    sentry::init(sentry::ClientOptions {
        dsn: std::env::var("SENTRY_DSN").ok(),
        environment: Some("benchmark".into()),
        traces_sample_rate: 0.5,
        ..Default::default()
    });
}

fn benchmark_example(c: &mut Criterion) {
    init_sentry();

    c.bench_function("example_operation", |b| {
        b.iter(|| {
            let result = black_box(42);
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: "benchmark".into(),
                message: format!("Iteration: {}", result),
                level: sentry::Level::Debug,
                ..Default::default()
            });
        })
    });
}

criterion_group!(benches, benchmark_example);
criterion_main!(benches);
```

### .env Configuration

**File**: `phenotype-infrakit/.env.example` (add to existing file)

```bash
# ── Sentry Configuration ────────────────────────────────────────────────
SENTRY_DSN=https://[your-key]@o[org-id].ingest.us.sentry.io/[project-id]
ENVIRONMENT=development
```

## 3. heliosCLI Configuration

### Workspace Setup

**File**: `heliosCLI/Cargo.toml` (workspace root)

```toml
[workspace.dependencies]
sentry = { version = "0.33", features = ["backtrace", "debug-images", "anyhow"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
```

### TUI Main Entry Point

**File**: `heliosCLI/crates/harness_orchestrator/src/main.rs`

```rust
use sentry::integrations::backtrace::BacktraceIntegration;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize Sentry before TUI
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_DSN").ok(),
        release: sentry::release_name!(),
        environment: env::var("ENVIRONMENT")
            .ok()
            .map(|e| e.into()),
        integrations: vec![
            Box::new(BacktraceIntegration::new()),
            Box::new(sentry::integrations::panic::PanicIntegration::new()),
            Box::new(sentry::integrations::std_panic::StdPanicIntegration::new()),
        ],
        traces_sample_rate: 1.0,
        ..Default::default()
    });

    // TUI loop with error handling
    match run_tui().await {
        Ok(_) => {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: "tui".into(),
                message: "TUI exited normally".into(),
                level: sentry::Level::Info,
                ..Default::default()
            });
        }
        Err(e) => {
            sentry::with_scope(
                |scope| {
                    scope.set_tag("component", "tui");
                    scope.set_tag("fatal", "true");
                    sentry::capture_error(&e);
                },
            );
            eprintln!("TUI Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // TUI event loop
    Ok(())
}
```

### Command Handler with Context

**File**: `heliosCLI/crates/harness_runner/src/lib.rs`

```rust
use std::error::Error as StdError;

pub async fn handle_command(cmd: &str) -> Result<(), Box<dyn StdError>> {
    // Add breadcrumb for command execution
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: "command".into(),
        message: format!("Executing command: {}", cmd),
        level: sentry::Level::Info,
        ..Default::default()
    });

    // Execute with error context
    match execute(cmd).await {
        Ok(result) => {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: "command".into(),
                message: "Command completed".into(),
                level: sentry::Level::Info,
                ..Default::default()
            });
            Ok(())
        }
        Err(e) => {
            // Capture with context
            sentry::with_scope(
                |scope| {
                    scope.set_tag("command", cmd);
                    scope.set_tag("error_type", std::any::type_name_of_val(&e));
                    sentry::capture_error(&e);
                },
            );
            Err(e)
        }
    }
}

async fn execute(_cmd: &str) -> Result<String, Box<dyn StdError>> {
    // Command logic
    Ok("Success".to_string())
}
```

### .env Configuration

**File**: `heliosCLI/.env.example` (add to existing file)

```bash
# ── Sentry Configuration ────────────────────────────────────────────────
SENTRY_DSN=https://[your-key]@o[org-id].ingest.us.sentry.io/[project-id]
ENVIRONMENT=development
```

## 4. Common Patterns

### Pattern: Result Type with Sentry Integration

```rust
use std::error::Error as StdError;

pub type AppResult<T> = Result<T, Box<dyn StdError>>;

pub fn handle_result<T, E: StdError + 'static>(
    result: Result<T, E>,
    context: &str,
) -> AppResult<T> {
    match result {
        Ok(val) => {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: "operation".into(),
                message: format!("{}: success", context),
                level: sentry::Level::Info,
                ..Default::default()
            });
            Ok(val)
        }
        Err(e) => {
            sentry::with_scope(
                |scope| {
                    scope.set_tag("operation", context);
                    scope.set_context("error_details", {
                        let mut map = sentry::protocol::Map::new();
                        map.insert("message".to_string(), e.to_string().into());
                        sentry::protocol::Context::Other(map)
                    });
                    sentry::capture_error(&e);
                },
            );
            Err(Box::new(e))
        }
    }
}
```

### Pattern: Async Operation Tracing

```rust
pub async fn tracked_operation<F, T>(
    name: &str,
    f: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: "operation".into(),
        message: format!("{}: started", name),
        level: sentry::Level::Debug,
        ..Default::default()
    });

    match f.await {
        Ok(result) => {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: "operation".into(),
                message: format!("{}: completed", name),
                level: sentry::Level::Debug,
                ..Default::default()
            });
            Ok(result)
        }
        Err(e) => {
            sentry::with_scope(
                |scope| {
                    scope.set_tag("async_operation", name);
                    sentry::capture_error(&e);
                },
            );
            Err(e)
        }
    }
}
```

### Pattern: HTTP Error Logging (for API servers)

```rust
pub fn log_http_error(
    method: &str,
    path: &str,
    status: u16,
    error: Option<&str>,
) {
    if status >= 500 {
        sentry::with_scope(
            |scope| {
                scope.set_tag("http_method", method);
                scope.set_tag("http_path", path);
                scope.set_tag("http_status", status.to_string());
                if let Some(err_msg) = error {
                    scope.set_context("http_error", {
                        let mut map = sentry::protocol::Map::new();
                        map.insert("message".to_string(), err_msg.to_string().into());
                        sentry::protocol::Context::Other(map)
                    });
                }
                sentry::capture_message(
                    &format!("HTTP {} {} returned {}", method, path, status),
                    sentry::Level::Error,
                );
            },
        );
    }
}
```

## 5. GitHub Secrets Template

For each repository, add these secrets to GitHub Settings → Secrets and variables → Actions:

### AgilePlus (`KooshaPari/AgilePlus`)

```
SENTRY_DSN_AGILEPLUS=https://[project-key]@o[org-id].ingest.us.sentry.io/[project-id]
SENTRY_AUTH_TOKEN=[auth-token-for-releases]
```

### phenotype-infrakit (`KooshaPari/phenotype-infrakit`)

```
SENTRY_DSN_INFRAKIT=https://[project-key]@o[org-id].ingest.us.sentry.io/[project-id]
SENTRY_AUTH_TOKEN=[auth-token-for-releases]
```

### heliosCLI (`KooshaPari/heliosCLI`)

```
SENTRY_DSN_HELIOSCLI=https://[project-key]@o[org-id].ingest.us.sentry.io/[project-id]
SENTRY_AUTH_TOKEN=[auth-token-for-releases]
```

## 6. Testing Sentry Integration

### Unit Test Template

```rust
#[cfg(test)]
mod sentry_integration_tests {
    use super::*;

    fn setup_sentry() -> sentry::ClientGuard {
        sentry::init(sentry::ClientOptions {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: Some("test".into()),
            ..Default::default()
        })
    }

    #[test]
    fn test_error_capture() {
        let _guard = setup_sentry();

        let error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test error",
        );

        sentry::capture_error(&error);
        // Verify in Sentry dashboard after test runs
    }

    #[test]
    fn test_breadcrumb_tracking() {
        let _guard = setup_sentry();

        sentry::add_breadcrumb(sentry::Breadcrumb {
            category: "test".into(),
            message: "Test breadcrumb".into(),
            level: sentry::Level::Info,
            ..Default::default()
        });

        // Error triggered here will include breadcrumb
        sentry::capture_message("Test message with breadcrumb", sentry::Level::Info);
    }

    #[tokio::test]
    async fn test_async_error_capture() {
        let _guard = setup_sentry();

        let result: Result<(), _> = Err("Async error");
        if let Err(e) = result {
            sentry::capture_message(&format!("Async error: {}", e), sentry::Level::Error);
        }
    }
}
```

## Summary

This configuration provides:
- Ready-to-use Sentry SDK setup for all 3 Tier 1 repos
- Panic and exception auto-capture
- Breadcrumb tracing for operation flow
- Context tags for error grouping
- Async/await support
- Test instrumentation
- GitHub Secrets template for safe DSN storage

All configurations use Sentry Rust SDK v0.33+ with best practices for production-grade error tracking.
