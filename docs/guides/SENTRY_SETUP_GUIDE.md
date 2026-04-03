# Sentry Project Setup Guide

**Status:** 🔴 Blocked — Auth token insufficient permissions (2026-03-31)

---

## Overview

This guide covers automated and manual setup of 3 Sentry projects for error tracking across:
- **AgilePlus** (Rust workspace)
- **phenotype-infrakit** (Rust workspace)
- **heliosCLI** (Rust workspace)

Each project will generate a DSN token, configured as GitHub Secrets for CI/CD integration.

---

## Quick Start (Automated)

### Prerequisites
- Sentry auth token with `project:admin` scope stored in `~/.sentryclirc`
- `sentry-cli` installed (v3.3.5+)
- GitHub CLI (`gh`) authenticated

### Steps

1. **Verify token permissions:**
   ```bash
   sentry-cli organizations list
   ```
   Expected: List of organizations you have access to

2. **Run the automated script:**
   ```bash
   bash /Users/kooshapari/CodeProjects/Phenotype/repos/scripts/create-sentry-projects.sh
   ```

3. **Verify GitHub Secrets:**
   ```bash
   gh secret list | grep SENTRY_DSN
   ```
   Expected:
   ```
   SENTRY_DSN_AGILEPLUS
   SENTRY_DSN_HELIOSCLI
   SENTRY_DSN_INFRAKIT
   ```

---

## Manual Setup (if automated fails)

### Step 1: Log into Sentry Dashboard

1. Go to https://sentry.io/
2. Sign in with your account
3. Navigate to Organization: **stealth-startup-3u**

### Step 2: Create AgilePlus Project

1. Click **Projects** → **Create Project**
2. **Project Name:** AgilePlus
3. **Platform:** Rust
4. **Alert Rule:** Default or skip
5. Click **Create Project**
6. Copy the **DSN** token (format: `https://<key>@<subdomain>.ingest.sentry.io/<id>`)

### Step 3: Create phenotype-infrakit Project

1. Repeat Step 2 with:
   - **Project Name:** phenotype-infrakit
   - **Platform:** Rust
2. Copy the DSN token

### Step 4: Create heliosCLI Project

1. Repeat Step 2 with:
   - **Project Name:** heliosCLI
   - **Platform:** Rust
2. Copy the DSN token

### Step 5: Configure GitHub Secrets

```bash
# Set secrets for AgilePlus
gh secret set SENTRY_DSN_AGILEPLUS --body '<DSN_FROM_STEP_2>'

# Set secrets for phenotype-infrakit
gh secret set SENTRY_DSN_INFRAKIT --body '<DSN_FROM_STEP_3>'

# Set secrets for heliosCLI
gh secret set SENTRY_DSN_HELIOSCLI --body '<DSN_FROM_STEP_4>'
```

### Step 6: Verify Configuration

```bash
gh secret list | grep SENTRY_DSN
```

Expected output:
```
SENTRY_DSN_AGILEPLUS    Updated 2026-03-31
SENTRY_DSN_HELIOSCLI    Updated 2026-03-31
SENTRY_DSN_INFRAKIT     Updated 2026-03-31
```

---

## Troubleshooting

### Issue: "You do not have permission to perform this action"

**Root Cause:** Auth token lacks `project:admin` scope

**Solution:**

1. Log into Sentry: https://sentry.io/settings/auth-tokens/
2. Find your token in the list
3. Check the **Scopes** column
4. If missing `project:admin`:
   - Delete the existing token
   - Create a new token with scopes:
     - `project:admin` ✓
     - `org:read` ✓
     - `team:admin` ✓
5. Copy the new token
6. Update `~/.sentryclirc`:
   ```bash
   nano ~/.sentryclirc
   # Replace the token= line with your new token
   ```
7. Verify:
   ```bash
   sentry-cli organizations list
   ```

### Issue: Token not found in `~/.sentryclirc`

**Solution:**

1. Create the config file:
   ```bash
   mkdir -p ~/.config/sentry
   cat > ~/.sentryclirc <<EOF
   [auth]
   token=<YOUR_TOKEN_HERE>
   EOF
   ```

2. Get token from: https://sentry.io/settings/auth-tokens/
3. Replace `<YOUR_TOKEN_HERE>` with your actual token
4. Verify:
   ```bash
   sentry-cli organizations list
   ```

### Issue: "Cannot authenticate with GitHub"

**Solution:**

1. Verify GitHub CLI is authenticated:
   ```bash
   gh auth status
   ```

2. If not authenticated:
   ```bash
   gh auth login
   ```

3. Select:
   - **What is your preferred protocol for Git operations?** → HTTPS
   - **Authenticate Git with your GitHub credentials?** → Yes
   - **How would you like to authenticate GitHub CLI?** → Login with a web browser

---

## Integration with Projects

Once DSN tokens are configured in GitHub Secrets, they can be used in:

### AgilePlus (`crates/agileplus-sentry/`)

```rust
// In Cargo.toml
[dependencies]
sentry = { version = "0.32", features = ["backtrace", "tracing-core"] }

// In main.rs or lib.rs
fn main() {
    let sentry_dsn = std::env::var("SENTRY_DSN_AGILEPLUS").unwrap_or_default();
    let _guard = sentry::init(sentry_dsn);
    // Your app code
}
```

### phenotype-infrakit (`crates/phenotype-*/`)

Similar integration using `SENTRY_DSN_INFRAKIT`

### heliosCLI (`src/main.rs`)

Integration using `SENTRY_DSN_HELIOSCLI`

---

## Verification Checklist

- [ ] 3 Sentry projects created (AgilePlus, phenotype-infrakit, heliosCLI)
- [ ] 3 DSN tokens obtained and verified
- [ ] GitHub Secrets configured (`SENTRY_DSN_*`)
- [ ] `sentry-cli` can authenticate: `sentry-cli organizations list`
- [ ] Projects appear in Sentry dashboard: https://sentry.io/organizations/stealth-startup-3u/projects/
- [ ] Each project has valid DSN in format: `https://<key>@<subdomain>.ingest.sentry.io/<id>`

---

## Next Steps

1. **Integrate SDKs** into each project (see Integration section above)
2. **Configure CI/CD** to pass DSN via environment variables
3. **Test** by triggering test errors and verifying they appear in Sentry dashboard
4. **Set up alerts** in Sentry for critical issues
5. **Create dashboards** for monitoring across all 3 projects

---

## References

- Sentry Auth Tokens: https://sentry.io/settings/auth-tokens/
- Sentry Rust SDK: https://docs.sentry.io/platforms/rust/
- sentry-cli Documentation: https://docs.sentry.io/cli/
- GitHub Secrets: https://docs.github.com/en/actions/security-guides/encrypted-secrets
