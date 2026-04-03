# Sentry Environment Variables Template

This file provides environment variable templates for Sentry configuration across all Tier 1 repos.

## Quick Start

Copy the appropriate template to `.env` in your repo root and fill in the values from Sentry dashboard.

## AgilePlus Environment Template

**File**: `AgilePlus/.env`

```bash
# ═══════════════════════════════════════════════════════════════════════════
# SENTRY ERROR TRACKING & PERFORMANCE MONITORING
# ═══════════════════════════════════════════════════════════════════════════
# Sentry Dashboard: https://sentry.io/organizations/phenotype/projects/agileplus/
# Get DSN from: Sentry → Project Settings → Client Keys

# Sentry Error Tracking DSN
# Format: https://[key]@o[org-id].ingest.us.sentry.io/[project-id]
SENTRY_DSN=https://your-key-here@o123456.ingest.us.sentry.io/1234567890

# Sampling rate for performance traces (0.0 - 1.0)
# 1.0 = 100% of requests traced (dev/test)
# 0.1 = 10% of requests traced (production)
SENTRY_TRACES_SAMPLE_RATE=1.0

# Sampling rate for errors (0.0 - 1.0)
# Usually keep at 1.0 unless dealing with high-error environments
SENTRY_SAMPLE_RATE=1.0

# Environment tag (development, staging, production, test)
ENVIRONMENT=development

# Sentry CLI authentication (for releases, source maps)
# Get token from: Sentry → Settings → Auth Tokens
SENTRY_AUTH_TOKEN=sntrys_your_auth_token_here

# Sentry Organization slug
SENTRY_ORG=phenotype

# Sentry Project slug
SENTRY_PROJECT=agileplus

# Enable/disable Sentry integration
SENTRY_ENABLED=true

# Enable debug logging for Sentry
# Set to "debug" for troubleshooting, "info" for normal operation
SENTRY_LOG_LEVEL=info

# ═══════════════════════════════════════════════════════════════════════════
# Related Logging Configuration
# ═══════════════════════════════════════════════════════════════════════════

RUST_LOG=info,agileplus=debug,sentry=info
```

## phenotype-infrakit Environment Template

**File**: `phenotype-infrakit/.env`

```bash
# ═══════════════════════════════════════════════════════════════════════════
# SENTRY ERROR TRACKING
# ═══════════════════════════════════════════════════════════════════════════
# Sentry Dashboard: https://sentry.io/organizations/phenotype/projects/phenotype-infrakit/
# Get DSN from: Sentry → Project Settings → Client Keys

SENTRY_DSN=https://your-key-here@o123456.ingest.us.sentry.io/1234567890

# Performance monitoring (traces)
# For library/infrastructure: usually lower sample rate
SENTRY_TRACES_SAMPLE_RATE=0.1

# Environment
ENVIRONMENT=development

# Sentry CLI token
SENTRY_AUTH_TOKEN=sntrys_your_auth_token_here

# Organization & Project (for CLI tools)
SENTRY_ORG=phenotype
SENTRY_PROJECT=phenotype-infrakit

# Enable Sentry
SENTRY_ENABLED=true

# Logging
RUST_LOG=info,phenotype=debug,sentry=info
```

## heliosCLI Environment Template

**File**: `heliosCLI/.env`

```bash
# ═══════════════════════════════════════════════════════════════════════════
# SENTRY ERROR TRACKING & CLI INSTRUMENTATION
# ═══════════════════════════════════════════════════════════════════════════
# Sentry Dashboard: https://sentry.io/organizations/phenotype/projects/helioscli/
# Get DSN from: Sentry → Project Settings → Client Keys

SENTRY_DSN=https://your-key-here@o123456.ingest.us.sentry.io/1234567890

# For CLI, capture all traces (high sample rate)
SENTRY_TRACES_SAMPLE_RATE=1.0

# Environment
ENVIRONMENT=development

# Sentry CLI token
SENTRY_AUTH_TOKEN=sntrys_your_auth_token_here

# Organization & Project
SENTRY_ORG=phenotype
SENTRY_PROJECT=helioscli

# Enable Sentry
SENTRY_ENABLED=true

# Logging
RUST_LOG=info,helios=debug,sentry=info
```

## CI/CD Environment Variables (GitHub Actions)

### AgilePlus (.github/workflows/ci.yml)

```yaml
env:
  # Sentry credentials (from GitHub Secrets)
  SENTRY_DSN: ${{ secrets.SENTRY_DSN_AGILEPLUS }}
  SENTRY_AUTH_TOKEN: ${{ secrets.SENTRY_AUTH_TOKEN_AGILEPLUS }}

  # Environment context
  ENVIRONMENT: ${{ github.event_name == 'release' && 'production' || 'staging' }}

  # Logging
  RUST_LOG: info,agileplus=debug,sentry=info

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release
        env:
          SENTRY_DSN: ${{ secrets.SENTRY_DSN_AGILEPLUS }}
```

### phenotype-infrakit (.github/workflows/ci.yml)

```yaml
env:
  SENTRY_DSN: ${{ secrets.SENTRY_DSN_INFRAKIT }}
  SENTRY_AUTH_TOKEN: ${{ secrets.SENTRY_AUTH_TOKEN_INFRAKIT }}
  ENVIRONMENT: test
  RUST_LOG: info
```

### heliosCLI (.github/workflows/ci.yml)

```yaml
env:
  SENTRY_DSN: ${{ secrets.SENTRY_DSN_HELIOSCLI }}
  SENTRY_AUTH_TOKEN: ${{ secrets.SENTRY_AUTH_TOKEN_HELIOSCLI }}
  ENVIRONMENT: test
  RUST_LOG: info
```

## GitHub Secrets Configuration

### AgilePlus Secrets

**URL**: https://github.com/KooshaPari/AgilePlus/settings/secrets/actions

Add the following secrets:

```
SENTRY_DSN_AGILEPLUS
  Value: https://[key]@o[org-id].ingest.us.sentry.io/[project-id]

SENTRY_AUTH_TOKEN_AGILEPLUS
  Value: sntrys_[your-token]
```

### phenotype-infrakit Secrets

**URL**: https://github.com/KooshaPari/phenotype-infrakit/settings/secrets/actions

```
SENTRY_DSN_INFRAKIT
  Value: https://[key]@o[org-id].ingest.us.sentry.io/[project-id]

SENTRY_AUTH_TOKEN_INFRAKIT
  Value: sntrys_[your-token]
```

### heliosCLI Secrets

**URL**: https://github.com/KooshaPari/heliosCLI/settings/secrets/actions

```
SENTRY_DSN_HELIOSCLI
  Value: https://[key]@o[org-id].ingest.us.sentry.io/[project-id]

SENTRY_AUTH_TOKEN_HELIOSCLI
  Value: sntrys_[your-token]
```

## How to Get Values

### SENTRY_DSN

1. Go to [Sentry Dashboard](https://sentry.io)
2. Select project (AgilePlus, phenotype-infrakit, or heliosCLI)
3. Go to **Settings** → **Projects** → **[Project Name]** → **Client Keys**
4. Copy the **DSN** value

Format: `https://[key]@o[org-id].ingest.us.sentry.io/[project-id]`

### SENTRY_AUTH_TOKEN

1. Go to [Sentry Settings](https://sentry.io/settings/account/api/auth-tokens/)
2. Click **Create New Token**
3. Name: `[Project]-CI-Token`
4. Scopes:
   - ✅ `project:read`
   - ✅ `project:write`
   - ✅ `org:read`
   - ✅ `releases:read`
   - ✅ `releases:write`
5. Copy the token

## Example .env File

**Template** (copy and fill in):

```bash
# ═══════════════════════════════════════════════════════════════════════════
# SENTRY CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

SENTRY_DSN=https://[KEY]@o[ORG_ID].ingest.us.sentry.io/[PROJECT_ID]
SENTRY_TRACES_SAMPLE_RATE=1.0
SENTRY_SAMPLE_RATE=1.0
ENVIRONMENT=development
SENTRY_AUTH_TOKEN=sntrys_[YOUR_TOKEN]
SENTRY_ORG=phenotype
SENTRY_PROJECT=[PROJECT_NAME]
SENTRY_ENABLED=true
SENTRY_LOG_LEVEL=info

# ═══════════════════════════════════════════════════════════════════════════
# LOGGING
# ═══════════════════════════════════════════════════════════════════════════

RUST_LOG=info,[project]=debug,sentry=info
```

## Loading Environment Variables

### From .env file

**In Rust** (using `dotenvy` crate):

```rust
use std::env;

fn main() {
    // Load from .env file
    dotenvy::dotenv().ok();

    let sentry_dsn = env::var("SENTRY_DSN").ok();
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let _guard = sentry::init(sentry::ClientOptions {
        dsn: sentry_dsn,
        environment: Some(environment.into()),
        ..Default::default()
    });
}
```

### From environment directly

```bash
export SENTRY_DSN="https://[key]@o[org-id].ingest.us.sentry.io/[project-id]"
export ENVIRONMENT=development
cargo run
```

### From GitHub Secrets in CI

```yaml
- name: Test
  env:
    SENTRY_DSN: ${{ secrets.SENTRY_DSN_AGILEPLUS }}
  run: cargo test
```

## Environment-Specific Configurations

### Development

```bash
ENVIRONMENT=development
SENTRY_TRACES_SAMPLE_RATE=1.0        # Capture all traces
SENTRY_SAMPLE_RATE=1.0                # Capture all errors
SENTRY_LOG_LEVEL=debug                # Verbose logging
RUST_LOG=debug,sentry=debug
```

### Staging

```bash
ENVIRONMENT=staging
SENTRY_TRACES_SAMPLE_RATE=0.5         # 50% sampling
SENTRY_SAMPLE_RATE=0.5                # 50% sampling
SENTRY_LOG_LEVEL=info
RUST_LOG=info,sentry=info
```

### Production

```bash
ENVIRONMENT=production
SENTRY_TRACES_SAMPLE_RATE=0.1         # 10% sampling (reduce cost)
SENTRY_SAMPLE_RATE=1.0                # Capture all errors (important)
SENTRY_LOG_LEVEL=warn
RUST_LOG=info,sentry=warn
```

### Testing

```bash
ENVIRONMENT=test
SENTRY_TRACES_SAMPLE_RATE=1.0         # Full trace capture
SENTRY_SAMPLE_RATE=1.0                # Full error capture
SENTRY_LOG_LEVEL=debug
RUST_LOG=debug,sentry=debug
```

## Validation Checklist

Before running your application, verify:

- [ ] `.env` file exists in project root
- [ ] `SENTRY_DSN` is set and valid format
- [ ] `ENVIRONMENT` matches current context (dev/test/prod)
- [ ] `SENTRY_AUTH_TOKEN` is set (for releases)
- [ ] No sensitive data leaked in `RUST_LOG`
- [ ] `.env` file is in `.gitignore` (never commit DSN)
- [ ] GitHub Secrets are configured (for CI/CD)

## Testing Configuration

```bash
# Verify DSN is readable
echo $SENTRY_DSN

# Check Sentry connectivity
curl -I "https://$(echo $SENTRY_DSN | cut -d'@' -f2)"

# Test with Sentry CLI
sentry-cli -x url
sentry-cli info

# Run tests with Sentry
SENTRY_DSN="$SENTRY_DSN" RUST_LOG=sentry=debug cargo test --lib
```

## Troubleshooting

### DSN Not Loading

```bash
# Check variable is set
env | grep SENTRY

# Check .env file exists
ls -la .env

# Check .env format (no quotes needed)
cat .env | grep SENTRY_DSN
```

### Invalid DSN Format

Valid format:
```
https://[public-key]@o[org-id].ingest.us.sentry.io/[project-id]
```

Invalid (don't use):
```
https://[public-key]:[secret-key]@[host]/[project-id]
```

### GitHub Secrets Not Available in CI

1. Check secret name matches exactly
2. Verify secret is created in repo settings
3. Use correct syntax in workflow: `${{ secrets.SECRET_NAME }}`
4. Check branch has access to secrets

## Summary

This template provides:
- ✅ Complete environment variable setup for all 3 Tier 1 repos
- ✅ CI/CD integration examples
- ✅ GitHub Secrets configuration
- ✅ Environment-specific settings
- ✅ Troubleshooting guide
- ✅ Validation checklist

Store sensitive values (DSN, tokens) in GitHub Secrets, not in repository code.
