# Sentry Setup & Error Tracking Guide

This guide documents the Sentry initialization for Phase 1 of the Security & QA implementation. Sentry provides real-time error tracking, performance monitoring, and release management across Tier 1 repos.

## Overview

Sentry is configured for three Tier 1 projects:
- **AgilePlus** (Rust workspace, 24 crates)
- **phenotype-infrakit** (Rust workspace)
- **heliosCLI** (Rust workspace, 18 crates)

All projects use the Sentry Rust SDK v0.33+ for error capture, performance monitoring, and release tracking.

## Sentry Projects & DSN Tokens

### Project URLs

| Project | Type | Dashboard | DSN | GitHub Repo |
|---------|------|-----------|-----|------------|
| AgilePlus | Rust | `https://sentry.io/organizations/phenotype/projects/agileplus/` | `SENTRY_DSN_AGILEPLUS` | `KooshaPari/AgilePlus` |
| phenotype-infrakit | Rust | `https://sentry.io/organizations/phenotype/projects/phenotype-infrakit/` | `SENTRY_DSN_INFRAKIT` | `KooshaPari/phenotype-infrakit` |
| heliosCLI | Rust | `https://sentry.io/organizations/phenotype/projects/helioscli/` | `SENTRY_DSN_HELIOSCLI` | `KooshaPari/heliosCLI` |

### Accessing Sentry Dashboard

1. Navigate to [sentry.io](https://sentry.io)
2. Select organization: **phenotype**
3. Choose the project from the sidebar
4. View errors, performance metrics, and releases

### DSN Token Storage

DSN tokens are stored as GitHub Secrets in each repository:

**AgilePlus (`KooshaPari/AgilePlus`)**:
```
SENTRY_DSN_AGILEPLUS=https://[key]@o[org-id].ingest.us.sentry.io/[project-id]
```

**phenotype-infrakit (`KooshaPari/phenotype-infrakit`)**:
```
SENTRY_DSN_INFRAKIT=https://[key]@o[org-id].ingest.us.sentry.io/[project-id]
```

**heliosCLI** (if public repo):
```
SENTRY_DSN_HELIOSCLI=https://[key]@o[org-id].ingest.us.sentry.io/[project-id]
```

## Rust SDK Configuration

All Rust crates use the `sentry` crate (v0.33+) for instrumentation.

### Installation

Add to `Cargo.toml` in your workspace:

```toml
[workspace.dependencies]
sentry = { version = "0.33", features = ["backtrace", "debug-images", "anyhow"] }
tokio = { version = "1", features = ["full"] }
```

### Initialization (Binary Crates)

For binary entry points (main.rs), initialize Sentry at startup:

```rust
use sentry::integrations::backtrace::BacktraceIntegration;

#[tokio::main]
async fn main() {
    // Initialize Sentry
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: std::env::var("SENTRY_DSN")
            .ok()
            .or_else(|| option_env!("SENTRY_DSN").map(|s| s.to_string())),
        release: sentry::release_name!(),
        integrations: vec![
            Box::new(BacktraceIntegration::new()),
            Box::new(sentry::integrations::panic::PanicIntegration::new()),
            Box::new(sentry::integrations::std_panic::StdPanicIntegration::new()),
        ],
        // Performance monitoring
        traces_sample_rate: 1.0, // 100% for dev; lower for production
        environment: option_env!("ENVIRONMENT").map(|e| e.into()),
        ..Default::default()
    });

    // Your application code
    if let Err(e) = run_app().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    // Application logic here
    Ok(())
}
```

### Library Crates (non-binary)

Library crates should NOT initialize Sentry. Instead, provide error types that integrate with Sentry's capture mechanisms:

```rust
use std::fmt;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(String),
    UnknownError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::ParseError(s) => write!(f, "Parse error: {}", s),
            Error::UnknownError(s) => write!(f, "Unknown error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

// Capture errors in binary entry points
impl From<Error> for sentry::event::Event<'static> {
    fn from(err: Error) -> Self {
        sentry::event::Event {
            message: Some(format!("{:?}", err)),
            ..Default::default()
        }
    }
}
```

### Error Capture Patterns

**Pattern 1: Automatic Panic Capture**
```rust
// Panics are automatically sent to Sentry via PanicIntegration
panic!("This error will be captured by Sentry");
```

**Pattern 2: Manual Error Capture**
```rust
match risky_operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(e) => {
        // Capture error with context
        sentry::capture_error(&e);
        eprintln!("Operation failed: {}", e);
    }
}
```

**Pattern 3: With Context Tags**
```rust
sentry::with_scope(
    |scope| {
        scope.set_tag("operation", "database_query");
        scope.set_tag("user_id", "12345");
        sentry::capture_error(&error);
    },
);
```

**Pattern 4: Breadcrumbs for Tracing**
```rust
sentry::add_breadcrumb(sentry::Breadcrumb {
    category: "db".into(),
    message: "Query started".into(),
    level: sentry::Level::Info,
    ..Default::default()
});

// Perform operation...

sentry::add_breadcrumb(sentry::Breadcrumb {
    category: "db".into(),
    message: "Query completed".into(),
    level: sentry::Level::Info,
    ..Default::default()
});
```

## Environment Configuration

### Local Development

Create `.env` in the project root:

```bash
# Sentry Configuration
SENTRY_DSN=https://your-dsn-key@o[org-id].ingest.us.sentry.io/[project-id]
ENVIRONMENT=development
RUST_LOG=info,agileplus=debug,sentry=debug
```

### Testing

For tests, set the DSN environment variable:

```bash
SENTRY_DSN="https://test-key@o[org-id].ingest.us.sentry.io/[project-id]" cargo test
```

### Production

Store DSN in GitHub Secrets and inject via CI/CD:

```yaml
env:
  SENTRY_DSN: ${{ secrets.SENTRY_DSN_AGILEPLUS }}
  ENVIRONMENT: production
```

## GitHub Integration

Sentry's GitHub integration automatically creates issues and links commits/releases.

### Setup Steps

1. **Connect GitHub Organization**:
   - Go to Sentry Settings → Integrations
   - Search for "GitHub"
   - Click "Install"
   - Authorize Sentry app in GitHub

2. **Link Repository**:
   - For each project, go to Project Settings → Integrations
   - Select the repository (e.g., `KooshaPari/AgilePlus`)
   - Enable "Create issues" and "Link commits"

3. **Configure Alert Rules**:
   - Project Settings → Alert Rules
   - Create rule: "When an error reaches 10 occurrences"
   - Action: Create a GitHub issue with stack trace and context

### Auto-Issue Creation

When an error occurs, Sentry can automatically create a GitHub issue:

```
Title: [Sentry] panicked at 'assertion failed' in routes.rs
Body:
Error ID: {event_id}
Release: {release}
Environment: {environment}
Stack Trace: [snipped]
View in Sentry: {url}
```

## Release Tracking

Releases are tracked automatically when you create a git tag:

```bash
# Tag a release
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# Sentry receives notification and marks release
# Errors are now associated with the release version
```

### Manual Release Creation (via Sentry CLI)

```bash
# Install sentry-cli
curl -sL https://files.pythonhosted.org/packages/... | bash

# Create release
sentry-cli releases create -p agileplus v1.0.0

# Upload source maps (if applicable)
sentry-cli releases files -p agileplus v1.0.0 upload-sourcemap ./target/release

# Mark release as deployed
sentry-cli releases deploys -p agileplus v1.0.0 new --url https://agileplus.example.com
```

## Testing Error Capture

### Local Testing

**Test 1: Trigger Panic**
```bash
# Add this to a test file or main.rs
#[test]
fn test_sentry_panic_capture() {
    sentry::init(sentry::ClientOptions {
        dsn: Some("your-dsn".parse().unwrap()),
        ..Default::default()
    });

    panic!("Testing Sentry error capture");
}

# Run test
SENTRY_DSN="https://[key]@o[org-id].ingest.us.sentry.io/[project-id]" cargo test test_sentry_panic_capture
```

**Test 2: Manual Error Capture**
```rust
#[tokio::test]
async fn test_sentry_manual_capture() {
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: std::env::var("SENTRY_DSN").ok(),
        ..Default::default()
    });

    let error = std::io::Error::new(std::io::ErrorKind::Other, "Test error");
    sentry::capture_error(&error);

    // Wait for async transport
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}
```

### Verification in Dashboard

1. Navigate to [Sentry Dashboard](https://sentry.io/organizations/phenotype/)
2. Select the project
3. Go to Issues tab
4. Look for new error with:
   - Error message matching your test
   - Stack trace showing the panic location
   - Environment tag (development)
   - Release version (if tagged)
5. Click issue to view full context, breadcrumbs, and tags

### Expected Latency

- **Local Dev**: 1-5 seconds (to Sentry cloud)
- **CI/CD**: 2-10 seconds (depends on network)
- **Production**: <30 seconds (with batching)

## Integration with CI/CD

### GitHub Actions Workflow

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build release
        run: cargo build --release

      - name: Create Sentry release
        env:
          SENTRY_AUTH_TOKEN: ${{ secrets.SENTRY_AUTH_TOKEN }}
          SENTRY_ORG: phenotype
          SENTRY_PROJECT: agileplus
        run: |
          curl -sL https://files.pythonhosted.org/packages/.../sentry-cli | bash
          sentry-cli releases create -p agileplus "${GITHUB_REF#refs/tags/}"
          sentry-cli releases finalize -p agileplus "${GITHUB_REF#refs/tags/}"

      - name: Deploy
        run: |
          # Your deployment steps here
          echo "Deployed ${GITHUB_REF#refs/tags/}"
```

## Troubleshooting

### DSN Not Loading

**Problem**: Error capture not working, DSN not found.

**Solution**:
```bash
# Verify environment variable
echo $SENTRY_DSN

# Check .env file
cat .env | grep SENTRY

# Test with explicit DSN
SENTRY_DSN="https://[key]@o[org-id].ingest.us.sentry.io/[project-id]" cargo run
```

### Errors Not Appearing in Dashboard

**Problem**: Errors triggered locally but not visible in Sentry dashboard.

**Checks**:
1. Verify DSN is correct: `echo $SENTRY_DSN`
2. Check network connectivity: `curl -I https://o[org-id].ingest.us.sentry.io`
3. Increase verbosity: `RUST_LOG=sentry=debug cargo run`
4. Check Sentry project settings: Project Settings → Client Keys
5. Verify project is not in "Pause all errors" mode

### Performance Monitoring Not Working

**Problem**: Traces not appearing in Performance tab.

**Solution**:
```rust
// Ensure traces_sample_rate is > 0
let _guard = sentry::init(sentry::ClientOptions {
    dsn: Some(dsn),
    traces_sample_rate: 0.1, // 10% sampling
    ..Default::default()
});
```

## Best Practices

### 1. Set Release Version

Always set the release version for better error grouping:

```rust
let _guard = sentry::init(sentry::ClientOptions {
    release: sentry::release_name!(),
    ..Default::default()
});
```

### 2. Use Meaningful Tags

Add context tags to help group and filter errors:

```rust
sentry::with_scope(
    |scope| {
        scope.set_tag("crate", "agileplus-api");
        scope.set_tag("handler", "create_workspace");
        scope.set_tag("user_role", "admin");
        sentry::capture_error(&error);
    },
);
```

### 3. Sample Rates in Production

Don't capture 100% of errors in production:

```rust
let _guard = sentry::init(sentry::ClientOptions {
    traces_sample_rate: 0.1,  // 10% of traces
    sample_rate: 0.1,         // 10% of errors
    ..Default::default()
});
```

### 4. Add Breadcrumbs for Context

Breadcrumbs help trace the path to an error:

```rust
sentry::add_breadcrumb(sentry::Breadcrumb {
    category: "api".into(),
    message: format!("Request: {}", method),
    level: sentry::Level::Info,
    ..Default::default()
});
```

### 5. Clean Error Messages

Avoid leaking sensitive data in error messages:

```rust
// Bad
Err(format!("Failed to connect to {}: {}", db_url, error))

// Good
Err("Database connection failed".to_string())
```

## Adding Sentry to a New Crate

1. **Add dependency** to workspace `Cargo.toml`:
   ```toml
   sentry = { workspace = true }
   ```

2. **In binary crate** (main.rs):
   ```rust
   let _guard = sentry::init(sentry::ClientOptions {
       dsn: std::env::var("SENTRY_DSN").ok(),
       ..Default::default()
   });
   ```

3. **In library crate** (lib.rs):
   ```rust
   // No initialization needed
   // Just use sentry::capture_error(&error) when needed
   ```

4. **Add to tests**:
   ```rust
   #[test]
   fn test_error_handling() {
       let _guard = sentry::init(sentry::ClientOptions {
           dsn: std::env::var("SENTRY_DSN").ok(),
           ..Default::default()
       });
       // Your test code
   }
   ```

5. **Update .env.example**:
   ```bash
   # Sentry Configuration
   SENTRY_DSN=https://your-dsn-key@o[org-id].ingest.us.sentry.io/[project-id]
   ```

## Sentry Organization Settings

### Organization-Level Configuration

**URL**: https://sentry.io/settings/phenotype/

| Setting | Value | Purpose |
|---------|-------|---------|
| Org Name | phenotype | Identifies the organization |
| Default Role | Member | Base permission level for team members |
| SSO | Disabled | Single sign-on (enable for enterprise) |
| 2FA Required | Optional | Two-factor authentication enforcement |

### Team Management

```
Organization: phenotype
├── Team: Infrastructure (phenotype-infrakit, platform)
├── Team: Agents (AgilePlus, agent-wave)
└── Team: CLI (heliosCLI, pheno-cli)
```

## Monitoring & Alerts

### Alert Rules by Project

**AgilePlus**:
- Alert on: 5+ errors in 5 minutes
- Notify: #agileplus-alerts (Slack)
- Create issue: YES

**phenotype-infrakit**:
- Alert on: Any error in production
- Notify: #infrastructure-alerts (Slack)
- Create issue: YES

**heliosCLI**:
- Alert on: Unhandled exceptions
- Notify: #helioscli-alerts (Slack)
- Create issue: YES

### Slack Integration

```
1. Org Settings → Integrations → Slack
2. Authorize Slack workspace
3. For each project: Project Settings → Integrations → Slack
4. Select channel: #[project]-alerts
5. Configure alert rules to notify Slack
```

## References

- **Sentry Rust SDK**: https://docs.sentry.io/platforms/rust/
- **Sentry CLI**: https://docs.sentry.io/cli/
- **Error Reporting Best Practices**: https://docs.sentry.io/product/error-reporting/
- **Performance Monitoring**: https://docs.sentry.io/product/performance/

## Summary

This setup provides:
- Real-time error tracking across 3 Tier 1 repos
- Automatic panic and exception capture
- GitHub integration for issue creation
- Release version tracking
- Performance monitoring (traces)
- Team collaboration and alerting

All Tier 1 repos (AgilePlus, phenotype-infrakit, heliosCLI) are now instrumented with Sentry v0.33 and configured for secure error reporting to the Phenotype organization.
