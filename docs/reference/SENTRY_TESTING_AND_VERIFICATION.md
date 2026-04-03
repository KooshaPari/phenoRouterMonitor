# Sentry Testing & Verification Guide

This guide provides step-by-step procedures to test and verify Sentry integration across Tier 1 repos.

## Prerequisites

- Sentry projects created for AgilePlus, phenotype-infrakit, heliosCLI
- DSN tokens available (from environment or GitHub Secrets)
- Sentry Rust SDK (v0.33+) integrated into each repo
- Access to Sentry dashboard
- Network connectivity to sentry.io

## Test Environment Setup

### 1. Configure Local Environment

Create `.env.test` in each repo:

**AgilePlus/.env.test**
```bash
SENTRY_DSN=https://[your-dsn-key]@o[org-id].ingest.us.sentry.io/[project-id]
ENVIRONMENT=test
RUST_LOG=sentry=debug
```

**phenotype-infrakit/.env.test**
```bash
SENTRY_DSN=https://[your-dsn-key]@o[org-id].ingest.us.sentry.io/[project-id]
ENVIRONMENT=test
RUST_LOG=sentry=debug
```

**heliosCLI/.env.test**
```bash
SENTRY_DSN=https://[your-dsn-key]@o[org-id].ingest.us.sentry.io/[project-id]
ENVIRONMENT=test
RUST_LOG=sentry=debug
```

### 2. Verify Network Connectivity

```bash
# Test connectivity to Sentry
curl -I https://o[org-id].ingest.us.sentry.io/

# Should return HTTP 200 or 404 (not connection refused)
```

## Test Suite 1: Basic Error Capture

### Test 1.1: Panic Capture

**AgilePlus/crates/agileplus-cli/src/lib.rs**

```rust
#[cfg(test)]
mod sentry_tests {
    use super::*;

    fn init_test_sentry() -> sentry::ClientGuard {
        sentry::init(sentry::ClientOptions {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: Some("test".into()),
            ..Default::default()
        })
    }

    #[test]
    #[should_panic]
    fn test_panic_capture() {
        let _guard = init_test_sentry();

        // This panic should be captured by Sentry
        panic!("Test panic for Sentry capture");
    }
}
```

**Run the test**:
```bash
export SENTRY_DSN="https://[key]@o[org-id].ingest.us.sentry.io/[project-id]"
cargo test test_panic_capture --lib -- --ignored 2>&1 | tee test-output.log
```

**Verification**:
1. Wait 5-10 seconds for async transport
2. Go to Sentry Dashboard → AgilePlus → Issues
3. Look for issue: "Test panic for Sentry capture"
4. Verify:
   - ✅ Stack trace shows panic location
   - ✅ Environment: "test"
   - ✅ Status: "New" or "Regressed"

### Test 1.2: Exception Capture

**phenotype-infrakit/libs/example-crate/src/lib.rs**

```rust
#[cfg(test)]
mod sentry_tests {
    use super::*;

    fn init_sentry() -> sentry::ClientGuard {
        sentry::init(sentry::ClientOptions {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: Some("test".into()),
            ..Default::default()
        })
    }

    #[test]
    fn test_error_capture() {
        let _guard = init_sentry();

        let error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test I/O error for Sentry",
        );

        sentry::capture_error(&error);

        // Give async transport time to send
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
```

**Run the test**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
export SENTRY_DSN="https://[key]@o[org-id].ingest.us.sentry.io/[project-id]"
cargo test test_error_capture --lib
```

**Verification**:
1. Go to Sentry Dashboard → phenotype-infrakit → Issues
2. Look for: "Test I/O error for Sentry"
3. Verify:
   - ✅ Error type: "io"
   - ✅ Message captured correctly
   - ✅ Test environment shows

### Test 1.3: Async Error Capture

**heliosCLI/crates/harness_orchestrator/src/lib.rs**

```rust
#[cfg(test)]
mod sentry_tests {
    use super::*;

    fn init_sentry() -> sentry::ClientGuard {
        sentry::init(sentry::ClientOptions {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: Some("test".into()),
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn test_async_error_capture() {
        let _guard = init_sentry();

        // Simulate async operation failure
        let result: Result<(), Box<dyn std::error::Error>> =
            Err("Async operation failed".into());

        if let Err(e) = result {
            sentry::capture_message(&format!("Async error: {}", e), sentry::Level::Error);
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
```

**Run the test**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI
export SENTRY_DSN="https://[key]@o[org-id].ingest.us.sentry.io/[project-id]"
cargo test test_async_error_capture --lib
```

**Verification**:
1. Go to Sentry Dashboard → heliosCLI → Issues
2. Look for: "Async error: Async operation failed"
3. Verify captured and categorized correctly

## Test Suite 2: Breadcrumb Tracing

### Test 2.1: Breadcrumb Trail

**Create test in AgilePlus**:

```rust
#[test]
fn test_breadcrumb_trail() {
    let _guard = init_test_sentry();

    // Add breadcrumbs for operation steps
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: "operation".into(),
        message: "Step 1: Initialize".into(),
        level: sentry::Level::Info,
        ..Default::default()
    });

    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: "operation".into(),
        message: "Step 2: Process".into(),
        level: sentry::Level::Info,
        ..Default::default()
    });

    // Trigger error
    let error = std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed after processing",
    );

    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: "operation".into(),
        message: "Step 3: Error occurred".into(),
        level: sentry::Level::Error,
        ..Default::default()
    });

    sentry::capture_error(&error);
    std::thread::sleep(std::time::Duration::from_secs(2));
}
```

**Run and verify**:
```bash
cargo test test_breadcrumb_trail -- --nocapture
```

In Sentry:
1. Find the "Failed after processing" error
2. Click to view details
3. Scroll to **Breadcrumbs** section
4. Verify all 3 breadcrumbs appear in order:
   - ✅ "Step 1: Initialize"
   - ✅ "Step 2: Process"
   - ✅ "Step 3: Error occurred"

## Test Suite 3: Context Tags

### Test 3.1: Tag Capture

```rust
#[test]
fn test_context_tags() {
    let _guard = init_test_sentry();

    sentry::with_scope(
        |scope| {
            scope.set_tag("operation", "database_query");
            scope.set_tag("user_id", "test-user-123");
            scope.set_tag("database", "sqlite");

            let error = std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database query failed",
            );

            sentry::capture_error(&error);
        },
    );

    std::thread::sleep(std::time::Duration::from_secs(2));
}
```

**Run and verify**:
```bash
cargo test test_context_tags
```

In Sentry:
1. Find the error "Database query failed"
2. Click to expand **Tags** section
3. Verify all tags present:
   - ✅ operation: "database_query"
   - ✅ user_id: "test-user-123"
   - ✅ database: "sqlite"

## Test Suite 4: Release Tracking

### Test 4.1: Release Version Detection

**In AgilePlus/Cargo.toml**:
```toml
[package]
name = "agileplus"
version = "1.0.0-test"
```

**In main.rs**:
```rust
let _guard = sentry::init(sentry::ClientOptions {
    release: sentry::release_name!(),
    ..Default::default()
});

panic!("Testing release tracking");
```

**Trigger and verify**:
```bash
cargo run 2>&1 | head -20
```

In Sentry:
1. Go to **Issues** tab
2. Find the panic
3. Check **Release** field
4. Should show: "1.0.0-test@src"

### Test 4.2: Manual Release Creation

```bash
# Create test release
sentry-cli releases create -p agileplus 1.0.0-test

# Verify in dashboard
sentry-cli releases list -p agileplus | grep 1.0.0-test
```

Verification:
1. Go to Sentry Dashboard → AgilePlus → **Releases**
2. Should show: "1.0.0-test"
3. Click to view release details

## Test Suite 5: Performance & Latency

### Test 5.1: Capture Latency

**Test script** (save as `test-latency.sh`):

```bash
#!/bin/bash

PROJECT=${1:-agileplus}
SENTRY_DSN=${2:?SENTRY_DSN required}

echo "Testing Sentry capture latency for $PROJECT"
echo "==========================================="

# Create test binary
cat > /tmp/latency_test.rs << 'EOF'
use std::time::Instant;

fn main() {
    let start = Instant::now();

    let _guard = sentry::init(sentry::ClientOptions {
        dsn: std::env::var("SENTRY_DSN").ok(),
        ..Default::default()
    });

    sentry::capture_message("Latency test message", sentry::Level::Info);

    let elapsed = start.elapsed();
    println!("Capture initiated in: {:.2}ms", elapsed.as_secs_f64() * 1000.0);

    // Wait for async send
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("Async send should be complete");
}
EOF

# Compile and run
SENTRY_DSN="$SENTRY_DSN" cargo run --release

echo ""
echo "Check Sentry dashboard for 'Latency test message'"
echo "Expected latency: <30 seconds from message send to dashboard"
```

**Run the test**:
```bash
chmod +x test-latency.sh
./test-latency.sh agileplus "$SENTRY_DSN"
```

**Verification**:
1. Note the time you ran the test
2. Go to Sentry Dashboard → AgilePlus → **Issues**
3. Check the timestamp of "Latency test message"
4. Calculate: Dashboard timestamp - Run time = Capture latency
5. **Expected**: <10 seconds for local, <30 seconds for production

## Test Suite 6: GitHub Integration

### Test 6.1: GitHub Issue Auto-Creation

**Setup**: Ensure GitHub integration is enabled (see SENTRY_GITHUB_INTEGRATION.md)

**Trigger test error 5+ times**:
```bash
# Run test multiple times to trigger alert rule
for i in {1..6}; do
    echo "Run $i..."
    cargo test test_github_issue_creation 2>&1 | grep -E "test_github|panicked" || true
    sleep 2
done
```

**Verification**:
1. Go to Sentry Dashboard → AgilePlus → **Issues**
2. Find the test error
3. Check if it has a **GitHub Issue** indicator
4. Go to [GitHub Issues](https://github.com/KooshaPari/AgilePlus/issues)
5. Look for issue: "[Sentry] panicked at 'assertion failed'"
6. Verify it:
   - ✅ Links to Sentry issue
   - ✅ Contains stack trace
   - ✅ Has label "sentry"

### Test 6.2: Release Deployment Tracking

```bash
# Create test release tag
git tag -a v1.0.0-test -m "Testing release tracking"
git push origin v1.0.0-test

# Wait 10 seconds for webhook
sleep 10

# Verify in Sentry
sentry-cli releases list -p agileplus | grep 1.0.0-test
```

**Verification**:
1. Go to Sentry → AgilePlus → **Releases**
2. Should show: "v1.0.0-test"
3. Click release to see:
   - ✅ Associated commits
   - ✅ Changes
   - ✅ Errors before/after release

## Test Suite 7: Multi-Project Verification

### Test 7.1: Parallel Error Capture

**Run errors in all 3 repos simultaneously**:

```bash
# Terminal 1: AgilePlus
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
SENTRY_DSN="$AGILEPLUS_DSN" cargo test test_panic_capture --lib -- --ignored

# Terminal 2: phenotype-infrakit
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
SENTRY_DSN="$INFRAKIT_DSN" cargo test test_error_capture --lib

# Terminal 3: heliosCLI
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI
SENTRY_DSN="$HELIOSCLI_DSN" cargo test test_async_error_capture --lib
```

**Verification**:
1. Go to Sentry Dashboard
2. Check each project received errors:
   - ✅ AgilePlus → Issues: "Test panic for Sentry capture"
   - ✅ phenotype-infrakit → Issues: "Test I/O error for Sentry"
   - ✅ heliosCLI → Issues: "Async error: Async operation failed"

## Test Suite 8: Edge Cases

### Test 8.1: DSN Missing

**Test behavior when DSN is not set**:

```bash
# Unset SENTRY_DSN
unset SENTRY_DSN

# Run test
cargo test test_error_capture --lib

# Expected: Test passes, error not sent (graceful degradation)
```

### Test 8.2: Network Failure

**Test behavior with network timeout**:

```bash
# Block network to Sentry (Mac example)
sudo ifconfig en0 down

# Run test (should timeout gracefully)
timeout 5 cargo test test_error_capture --lib

# Restore network
sudo ifconfig en0 up

# Expected: Test completes despite network failure
```

### Test 8.3: Invalid DSN

**Test with malformed DSN**:

```bash
SENTRY_DSN="https://invalid-dsn" cargo test test_error_capture --lib

# Expected: Error logged to RUST_LOG, no crash
```

## Verification Checklist

Use this checklist to verify complete Sentry setup:

### Infrastructure
- [ ] Sentry organization created: phenotype
- [ ] 3 projects created: AgilePlus, phenotype-infrakit, heliosCLI
- [ ] DSN tokens stored in GitHub Secrets
- [ ] GitHub integration installed and authorized
- [ ] Repositories linked to Sentry projects

### SDK Integration
- [ ] Sentry v0.33+ in Cargo.toml (AgilePlus)
- [ ] Sentry v0.33+ in Cargo.toml (phenotype-infrakit)
- [ ] Sentry v0.33+ in Cargo.toml (heliosCLI)
- [ ] Main entry points initialize Sentry
- [ ] .env.example files include SENTRY_DSN
- [ ] Environment variables properly configured

### Error Capture
- [ ] Panics captured automatically
- [ ] Exceptions captured with sentry::capture_error()
- [ ] Breadcrumbs traced in stack traces
- [ ] Tags added to errors for filtering
- [ ] Async errors captured in Tokio tasks

### Testing
- [ ] Local tests pass with Sentry enabled
- [ ] Errors appear in dashboard within 10 seconds
- [ ] Multiple errors properly grouped/deduped
- [ ] Stack traces show full context
- [ ] Environment tags correct (test/dev/prod)

### GitHub Integration
- [ ] GitHub integration working (Issues, Commits, Releases)
- [ ] Alert rules created for each project
- [ ] Auto-issue creation tested and working
- [ ] Slack notifications configured (optional)
- [ ] Release tracking tested

### Performance
- [ ] Capture latency < 30 seconds
- [ ] No performance degradation in app
- [ ] Async transport doesn't block main thread
- [ ] Memory usage stable under load

## Troubleshooting Test Failures

### Error not appearing in dashboard

1. Check DSN is valid:
   ```bash
   echo $SENTRY_DSN
   ```

2. Verify Sentry project exists:
   ```bash
   curl -I -H "Authorization: Bearer $SENTRY_AUTH_TOKEN" \
     https://sentry.io/api/0/projects/phenotype/agileplus/
   ```

3. Check logs for send errors:
   ```bash
   RUST_LOG=sentry=debug cargo test test_error_capture
   ```

4. Verify network connectivity:
   ```bash
   curl -v https://o[org-id].ingest.us.sentry.io/
   ```

### Test hanging/timeout

1. Check async guard is released:
   ```rust
   let _guard = sentry::init(...);
   // guard dropped here automatically
   std::thread::sleep(Duration::from_secs(2));
   ```

2. Ensure DSN present before init:
   ```bash
   SENTRY_DSN=... cargo test
   ```

### Incorrect environment/tags

1. Verify scope setup:
   ```rust
   sentry::with_scope(|scope| {
       scope.set_tag("key", "value");
       sentry::capture_error(&e);
   });
   ```

2. Check ClientOptions environment:
   ```rust
   sentry::init(sentry::ClientOptions {
       environment: Some("test".into()),
       ..Default::default()
   })
   ```

## Summary

This test suite verifies:
- ✅ Error capture (panic, exception, async)
- ✅ Breadcrumb tracing
- ✅ Context tags
- ✅ Release tracking
- ✅ Latency metrics (<30s)
- ✅ GitHub integration
- ✅ Multi-project setup
- ✅ Edge case handling

All tests should pass within 30-60 minutes of execution.
