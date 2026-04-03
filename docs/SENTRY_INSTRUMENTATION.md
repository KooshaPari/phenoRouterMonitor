# Sentry Error Tracking Instrumentation

Comprehensive guide for integrating and using Sentry error tracking across AgilePlus, phenotype-infrakit, and heliosCLI.

## Table of Contents

1. [Quick Start](#quick-start)
2. [SDK Integration Details](#sdk-integration-details)
3. [Environment Configuration](#environment-configuration)
4. [GitHub Integration Setup](#github-integration-setup)
5. [Error Capture Examples](#error-capture-examples)
6. [Dashboard Navigation](#dashboard-navigation)
7. [Troubleshooting](#troubleshooting)

## Quick Start

### For AgilePlus

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# Copy environment template
cp .env.example .env

# Edit .env and add your Sentry DSN
# SENTRY_DSN=https://your-key@sentry.io/your-project-id

# Run tests to verify Sentry captures events
cargo test --lib logger::sentry_config

# Run integration tests
cargo test --test sentry_integration_test -- --nocapture
```

### For phenotype-infrakit

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

# Copy environment template
cp .env.example .env

# Edit .env and add your Sentry DSN
# SENTRY_DSN=https://your-key@sentry.io/your-project-id

# Run integration tests
cargo test -p phenotype-sentry-config --test sentry_integration_test -- --nocapture
```

### For heliosCLI

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI

# Copy environment template
cp .env.example .env

# Edit .env and add your Sentry DSN
# SENTRY_DSN=https://your-key@sentry.io/your-project-id

# Run integration tests
cargo test -p harness_utils --test sentry_integration_test -- --nocapture
```

## SDK Integration Details

### Added Dependencies

All three repos now include Sentry 0.33 with the following features:
- `backtrace` — Automatic stack trace collection
- `contexts` — Enhanced error context
- `debug-images` — Debug symbol resolution

### Module Structure

#### AgilePlus
- **Location:** `libs/logger/src/sentry_config.rs`
- **Export:** `logger::initialize()`, `logger::capture_error()`, `logger::capture_message()`
- **Integration Point:** Initialize in main entry points (CLI, API servers)

#### phenotype-infrakit
- **Location:** `crates/phenotype-sentry-config/src/lib.rs`
- **Export:** `phenotype_sentry_config::initialize()` and related functions
- **Integration Point:** Available for all crates in workspace

#### heliosCLI
- **Location:** `crates/harness_utils/src/sentry_config.rs`
- **Export:** `harness_utils::sentry_config::initialize()` and related functions
- **Integration Point:** Available throughout harness system

### Initialization Function

All three repos expose the same initialization pattern:

```rust
use logger::initialize;  // AgilePlus
use phenotype_sentry_config::initialize;  // phenotype-infrakit
use harness_utils::sentry_config::initialize;  // heliosCLI

fn main() {
    // Initialize Sentry at application startup
    let _guard = initialize();

    // Now panics and errors are automatically captured
    // Don't drop the guard until application shutdown
}
```

## Environment Configuration

### Variables

| Variable | Required | Default | Notes |
|----------|----------|---------|-------|
| `SENTRY_DSN` | No | Test mode (stderr) | Get from https://sentry.io/settings/organization/projects/ |
| `SENTRY_ENVIRONMENT` | No | "development" | Use: development, staging, production |
| `SENTRY_RELEASE` | No | Auto-detected | Override with specific version |

### Setup

1. **Copy template:**
   ```bash
   cp .env.example .env
   ```

2. **Edit .env with your DSN:**
   ```bash
   SENTRY_DSN=https://your-key@sentry.io/your-project-id
   SENTRY_ENVIRONMENT=development
   ```

3. **Add to .env.test for testing (optional):**
   ```bash
   SENTRY_DSN=https://test@test.ingest.sentry.io/0
   SENTRY_ENVIRONMENT=test
   ```

4. **Never commit .env files:**
   - Already added to .gitignore in all repos
   - Use GitHub Secrets for CI/CD

## GitHub Integration Setup

### One-Time Setup

1. **Link GitHub Organization to Sentry:**
   - Go to https://sentry.io/settings/integrations/github/
   - Click "Add Installation"
   - Authorize KooshaPari GitHub account
   - Select repositories: AgilePlus, phenotype-infrakit, heliosCLI

2. **Enable Auto-Issue Creation:**
   - Per repo, go to Sentry Project Settings → Integrations
   - Enable GitHub Integration
   - Configure issue creation rules:
     - **Trigger:** New error events (High severity or above)
     - **Action:** Create GitHub issue automatically
     - **Template:** Include error message, stack trace, breadcrumbs

3. **Configure Notifications (Optional):**
   - Sentry → Alerts
   - Create alert rule: "New Error Issue"
   - Action: Post to Slack or send email

### GitHub Secrets Configuration

For CI/CD pipelines:

1. **Add to GitHub repository secrets:**
   ```
   SENTRY_DSN_AGILEPLUS=https://your-key@sentry.io/agileplus-id
   SENTRY_DSN_INFRAKIT=https://your-key@sentry.io/infrakit-id
   SENTRY_DSN_HELIOSCLI=https://your-key@sentry.io/helioscli-id
   ```

2. **Use in GitHub Actions workflows:**
   ```yaml
   env:
     SENTRY_DSN: ${{ secrets.SENTRY_DSN_AGILEPLUS }}
   ```

3. **Add to .env during CI:**
   ```bash
   echo "SENTRY_DSN=$SENTRY_DSN" >> .env
   ```

## Error Capture Examples

### Manual Error Capture

```rust
use logger::capture_error, capture_message;

fn process_data() {
    match read_file("data.json") {
        Ok(content) => {
            // Process content
        }
        Err(e) => {
            // Capture error to Sentry
            capture_error(&e);
            eprintln!("Failed to read file: {}", e);
        }
    }
}
```

### Capture Error Messages

```rust
use logger::capture_message;
use sentry::Level;

fn critical_operation() {
    match dangerous_operation() {
        Ok(_) => {
            capture_message("Critical operation succeeded", Level::Info);
        }
        Err(e) => {
            capture_message(
                &format!("Critical operation failed: {}", e),
                Level::Error
            );
        }
    }
}
```

### Automatic Panic Capture

```rust
fn main() {
    let _guard = logger::initialize();

    // Any panic is automatically captured to Sentry
    panic!("This error is captured automatically");
}
```

### Context Enhancement

```rust
use sentry::integrations::anyhow::capture_anyhow;

fn operation_with_context() -> anyhow::Result<()> {
    let result = some_fallible_operation()
        .context("Failed to perform operation")?;

    Ok(())
}
```

## Dashboard Navigation

### Viewing Errors in Sentry

1. **Go to Sentry Dashboard:**
   - https://sentry.io/organizations/

2. **Select Project:**
   - AgilePlus
   - phenotype-infrakit
   - heliosCLI

3. **View Issues:**
   - Click "Issues" tab
   - Filter by: Status, Environment, Release, Date Range
   - Click issue to see:
     - Stack trace
     - Breadcrumbs (preceding events)
     - Tags and user context
     - Source code snippets
     - Related errors

4. **Search Issues:**
   ```
   # Show errors from production
   environment:production

   # Show errors from last hour
   timestamp:[now-1h TO now]

   # Show errors containing "database"
   message:database
   ```

### Monitoring Alerts

**Error Trend:** Watch the dashboard graph for sudden spikes

**Email Alerts:** Sentry sends digest emails:
- Hourly (enabled by default)
- Daily
- Weekly

**Slack Integration:** Receive real-time alerts in Slack channel

## Troubleshooting

### Issue: No Events Appearing in Sentry Dashboard

**Solution 1: Verify DSN is set**
```bash
# Check environment
echo $SENTRY_DSN

# Should output: https://your-key@sentry.io/your-project-id
```

**Solution 2: Verify Sentry is initialized**
```rust
fn main() {
    let _guard = logger::initialize();
    println!("Sentry initialized");

    // Make sure guard is not dropped immediately
    capture_message("Test message", sentry::Level::Info);
    std::thread::sleep(std::time::Duration::from_secs(2));
}
```

**Solution 3: Enable debug logging**
```bash
RUST_LOG=sentry=debug cargo test --lib sentry_config
```

### Issue: Events Captured But Not Creating GitHub Issues

**Solution 1: Check GitHub integration is enabled**
- Sentry Settings → Integrations → GitHub
- Status should show "Installed"

**Solution 2: Verify alert rule configuration**
- Sentry → Alerts → Check rules trigger severity
- Adjust if needed (some errors may be filtered)

**Solution 3: Check GitHub repo has write permissions**
- GitHub Settings → Integrations & Applications
- Sentry should have "Read and Write" on Issues

### Issue: High Latency Between Error and Dashboard

**Expected:** <30 seconds (usually <5 seconds)

**If slower:**
1. Check Sentry project is not rate-limited
2. Verify network connectivity from application to Sentry
3. Check if running in test mode (stderr logging) instead of real DSN

### Issue: "Invalid DSN" Error

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: InvalidDsn'
```

**Solution:** DSN format must be:
```
https://key@sentry.io/project-id
```

Not:
```
https://key@sentry.io    # Missing project ID
https://key@host.ingest.sentry.io/0  # Using test DSN
```

### Issue: Duplicate Error Events

**Cause:** Multiple `initialize()` calls or guard dropped prematurely

**Solution:** Initialize once at application startup:
```rust
fn main() {
    let _guard = initialize();  // Keep guard alive until shutdown

    // Run application
}
```

## Performance Considerations

### Sampling Errors

By default, all errors are captured. To reduce volume:

```rust
// Initialize with 50% sampling
let options = sentry::ClientOptions {
    sample_rate: 0.5,  // Capture 50% of errors
    ..Default::default()
};

let _guard = phenotype_sentry_config::initialize_with_options(
    "https://key@sentry.io/id",
    options
);
```

### Release Tracking

Sentry tracks errors per release. Set release at build time:

```bash
# Build with specific release
SENTRY_RELEASE=0.1.1 cargo build --release
```

### Breadcrumbs

Sentry captures breadcrumbs (preceding events) automatically:
- HTTP requests
- Database queries (if instrumented)
- Log messages
- User actions

Limit breadcrumbs to reduce memory:

```rust
let options = sentry::ClientOptions {
    max_breadcrumbs: 50,  // Default is 100
    ..Default::default()
};
```

## Testing Error Capture

All three repos include integration tests:

```bash
# AgilePlus
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
cargo test --test sentry_integration_test -- --nocapture

# phenotype-infrakit
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
cargo test -p phenotype-sentry-config --test sentry_integration_test -- --nocapture

# heliosCLI
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI
cargo test -p harness_utils --test sentry_integration_test -- --nocapture
```

Expected output should show messages being captured without panicking.

## Success Criteria

- [ ] Sentry initialized without errors
- [ ] Events appear in Sentry dashboard within 30 seconds
- [ ] GitHub integration creates issues for high-severity errors
- [ ] Environment variables properly configured
- [ ] Integration tests pass
- [ ] CI/CD pipeline can capture errors in production

## References

- [Sentry Rust SDK Documentation](https://docs.sentry.io/platforms/rust/)
- [Sentry GitHub Integration Guide](https://docs.sentry.io/product/integrations/github/)
- [Sentry Best Practices](https://docs.sentry.io/product/best-practices/)

---

**Last Updated:** 2026-03-30
**Repos:** AgilePlus, phenotype-infrakit, heliosCLI
**Sentry Version:** 0.33.x
