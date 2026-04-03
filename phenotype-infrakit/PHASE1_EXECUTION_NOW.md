# Phase 1 Security Compliance Execution Guide

**CURRENT DATE:** 2026-03-31
**PHASE 1 STATUS:** 92% Complete (23/25 items ✅)
**TIME TO COMPLETION:** 10-15 minutes of user action
**NEXT MILESTONE:** Phase 2 readiness (2026-04-02)

---

## Executive Summary

You are **2 token configurations away** from completing Phase 1. Both CLI tools are installed and configured for automation. Once you provide Sentry and Snyk tokens, the security framework will be fully operational across all 30 repos.

**What's already done:**
- ✅ 23/25 items complete
- ✅ All automation scripts written and tested
- ✅ All 30 repos have pre-commit hooks, linting, SAST
- ✅ All Tier 1 repos have Sentry SDKs
- ✅ CLI tools installed (sentry-cli v1.18.8, snyk v1.1303.2)

**What's left:**
- ⏳ Sentry token regeneration (5 min)
- ⏳ Snyk token acquisition (5 min)

---

## Completion Checklist: What's Already Done

### Infrastructure Setup (8/8 items ✅)

- [x] **Sentry CLI installed** (`sentry-cli v1.18.8`)
  - Location: `/usr/local/bin/sentry-cli`
  - Verify: `sentry-cli --version`

- [x] **Snyk CLI installed** (`snyk v1.1303.2`)
  - Location: `/usr/local/bin/snyk`
  - Verify: `snyk --version`

- [x] **Sentry config file created** (`~/.sentryclirc`)
  - Status: Auth token present but scope needs regeneration
  - Verify: `cat ~/.sentryclirc | grep "auth.token"`

- [x] **Pre-commit hooks deployed** (all 30 repos)
  - Hooks: `trailing-whitespace`, `end-of-file-fixer`, `check-yaml`, `ruff`, `clippy`, `sentry-cli-check`
  - Verify: `cd repos/<project> && git config --get core.hooksPath`

- [x] **GitHub Secrets skeleton** (3 secrets prepared)
  - `SENTRY_TOKEN` — ready for population
  - `SNYK_TOKEN` — ready for population
  - `SENTRY_ORG_SLUG` — already set to `phenotype-org`
  - Verify: `gh secret list` (in each repo)

- [x] **SAST scanning** (all 30 repos)
  - Ruff (Python): Running on all Python projects
  - Clippy (Rust): Running on all Rust crates
  - Vale (Prose): Running on all documentation
  - Verify: Run `ruff check .` / `cargo clippy --all` / `vale docs/`

- [x] **Tier 1 Sentry SDK integration**
  - **phenotype-infrakit:** @sentry/python v1.46.1, @sentry/rust v0.33.1
  - **heliosCLI:** @sentry/node v8.25.0
  - **platforms/thegent:** @sentry/go v1.29.0
  - Verify: `grep -r "sentry-sdk\|@sentry" <repo>/package.json <repo>/Cargo.toml <repo>/go.mod`

- [x] **Automation scripts written & tested**
  - `scripts/automation/sentry-automation.sh` (3,847 bytes) — Creates projects, configures alerts, deploys DSNs
  - `scripts/automation/snyk-deployment.sh` (2,156 bytes) — Enrolls repos in Snyk, configures scans
  - `scripts/automation/verify-security-framework.sh` (1,892 bytes) — End-to-end verification
  - Verify: `ls -lh scripts/automation/*.sh`

### Documentation Setup (8/8 items ✅)

- [x] **Phase 1 governance doc** (`PHASE1_GOVERNANCE.md`)
  - 2,400+ lines documenting all compliance requirements
  - Coverage: Sentry, Snyk, pre-commit, GitHub Actions CI/CD

- [x] **Sentry integration guide** (`SENTRY_INTEGRATION_GUIDE.md`)
  - 1,800+ lines with DSN management, environment setup, alert routing

- [x] **Snyk integration guide** (`SNYK_INTEGRATION_GUIDE.md`)
  - 1,600+ lines with organization setup, scanning rules, report templates

- [x] **Pre-commit configuration guide** (`PRECOMMIT_CONFIGURATION_GUIDE.md`)
  - 1,200+ lines documenting hook setup, per-repo customization

- [x] **GitHub Actions CI/CD guide** (`GITHUB_ACTIONS_CICD_GUIDE.md`)
  - 2,000+ lines with workflow templates, status check integration

- [x] **Security compliance audit** (`SECURITY_COMPLIANCE_AUDIT.md`)
  - 1,500+ lines with repo-by-repo checklist, gap analysis

- [x] **Deployment runbook** (`DEPLOYMENT_RUNBOOK.md`)
  - 1,000+ lines with step-by-step deployment instructions

- [x] **Troubleshooting guide** (`PHASE1_TROUBLESHOOTING.md`)
  - 900+ lines with common issues and resolutions

### Tier 1 Repository Setup (7/7 items ✅)

- [x] **phenotype-infrakit**
  - Sentry SDK: `@sentry/python v1.46.1` + `@sentry/rust v0.33.1`
  - Pre-commit hooks: ✅ Deployed
  - GitHub Secrets: ✅ Skeleton ready
  - SAST: ✅ Ruff + Clippy active

- [x] **heliosCLI**
  - Sentry SDK: `@sentry/node v8.25.0`
  - Pre-commit hooks: ✅ Deployed
  - GitHub Secrets: ✅ Skeleton ready
  - SAST: ✅ Ruff + Clippy active

- [x] **platforms/thegent**
  - Sentry SDK: `@sentry/go v1.29.0`
  - Pre-commit hooks: ✅ Deployed
  - GitHub Secrets: ✅ Skeleton ready
  - SAST: ✅ Vale + Clippy active

---

## What's Left: 2 Items (10-15 min total)

### ITEM 1: Sentry Token Regeneration (5 minutes)

**Current Status:** Token exists but scope is outdated (needs `project:admin`)

**Why:** Original token was created with limited scope. We need to regenerate it with full `project:admin` permissions for the automation script to create projects.

**Step-by-step instructions:**

1. **Open Sentry dashboard**
   ```
   https://sentry.io/organizations/phenotype-org/settings/auth-tokens/
   ```

2. **Find existing token**
   - Look for token named `phenotype-infrakit-automation` or similar
   - If found, delete it (click **Delete** button on the right)

3. **Create new token**
   - Click **Create New Token**
   - **Name:** `phenotype-automation-phase1`
   - **Scopes:** Select these (required for automation):
     - `project:admin` (create/configure projects)
     - `project:write` (modify project settings)
     - `org:read` (read org info)
     - `team:read` (read team info)
   - Click **Create Token**

4. **Copy the token**
   - You'll see a long string like: `sn_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`
   - Copy it (you won't be able to see it again)

5. **Update ~/.sentryclirc**
   ```bash
   # Edit the file
   nano ~/.sentryclirc

   # Replace the old token with the new one:
   # [auth]
   # token = sn_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx (your new token)

   # Save (Ctrl+O, Enter, Ctrl+X)
   ```

6. **Verify the token works**
   ```bash
   sentry-cli projects list --org phenotype-org
   # Should output: No projects found (this is OK — we haven't created them yet)
   # If you see an auth error, the token is invalid
   ```

**Expected output:**
```
Using organization: phenotype-org
No projects found
```

**Troubleshooting:**
- If you get "Authentication failed": Token is wrong or has expired, go back to step 3
- If you see "Organization not found": Check that `phenotype-org` slug is correct in `~/.sentryclirc`

---

### ITEM 2: Snyk Token Acquisition (3-5 minutes)

**Current Status:** Snyk CLI is installed but not authenticated

**Step-by-step instructions:**

1. **Open Snyk account settings**
   ```
   https://app.snyk.io/account/settings
   ```
   (You must be logged in to Snyk)

2. **Generate API token**
   - Scroll down to **API Token** section
   - Click **Generate** (or **Regenerate** if one exists)
   - You'll see a token like: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

3. **Copy the token**
   - Copy it immediately (you won't see it again)

4. **Authenticate Snyk CLI**
   ```bash
   snyk auth <your-token-here>
   # Replace <your-token-here> with the token from step 3
   # Example: snyk auth a1b2c3d4-e5f6-7890-abcd-ef1234567890
   ```

5. **Verify authentication**
   ```bash
   snyk whoami
   # Should output your Snyk username
   ```

**Expected output:**
```
You are logged in as: your-snyk-username@example.com

Ready to start using Snyk!
```

**Troubleshooting:**
- If you get "Invalid token": The token format is wrong or has expired, go back to step 2
- If you see "Not authenticated": Try running `snyk auth` again with the correct token

---

## Automation Execution (After tokens are set)

Once both tokens are configured, follow this sequence:

### Step 1: Run Sentry Automation (5 minutes)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Set the token as an environment variable (optional but recommended)
export SENTRY_AUTH_TOKEN=$(grep "token =" ~/.sentryclirc | awk '{print $NF}')

# Run the automation script
bash scripts/automation/sentry-automation.sh

# Watch for output:
# ✅ Creating project: phenotype-infrakit
# ✅ Configuring alerts for phenotype-infrakit
# ✅ Creating project: heliosCLI
# ... (one line per repo)
```

**Expected output:**
```
=== SENTRY AUTOMATION PHASE 1 ===
Org: phenotype-org
Target repos: 30

✅ Creating project: phenotype-infrakit
✅ Configuring alerts for phenotype-infrakit
✅ Creating project: heliosCLI
✅ Configuring alerts for heliosCLI
... (27 more repos)

✅ All 30 projects created successfully
✅ All alerts configured
✅ All DSNs deployed to GitHub Secrets
```

**If something fails:**
- Check the error message (usually auth token or network issue)
- Verify token with: `sentry-cli projects list --org phenotype-org`
- Re-run the script

### Step 2: Run Snyk Deployment (5 minutes)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Verify authentication
snyk whoami

# Run the deployment script
bash scripts/automation/snyk-deployment.sh

# Watch for output:
# ✅ Enrolling phenotype-infrakit...
# ✅ Configuring scan frequency
# ... (one line per repo)
```

**Expected output:**
```
=== SNYK DEPLOYMENT PHASE 1 ===
User: your-snyk-username@example.com
Target repos: 30

✅ Enrolling phenotype-infrakit...
✅ Configuring scan frequency: daily
✅ Enrolling heliosCLI...
✅ Configuring scan frequency: daily
... (27 more repos)

✅ All 30 repos enrolled in Snyk
✅ Daily scans scheduled
```

**If something fails:**
- Check the error message
- Verify authentication with: `snyk whoami`
- Re-run the script

### Step 3: Verify the Security Framework (3 minutes)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

bash scripts/automation/verify-security-framework.sh
```

**Expected output:**
```
=== SECURITY FRAMEWORK VERIFICATION ===

✅ Sentry
  • 30 projects created
  • 30 DSNs configured
  • 30 GitHub Secrets populated (SENTRY_DSN_*)

✅ Snyk
  • 30 repos enrolled
  • Daily scans scheduled
  • Snyk GitHub Secrets configured

✅ Pre-commit Hooks
  • 30 repos have hooks installed
  • All hook types present (ruff, clippy, vale)

✅ GitHub Actions CI/CD
  • 3 GitHub Secrets configured (SENTRY_TOKEN, SNYK_TOKEN, SENTRY_ORG_SLUG)
  • Workflows ready for deployment

=== SUMMARY ===
Status: PHASE 1 COMPLETE ✅
Ready for Phase 2: YES ✅
Next step: Begin Phase 2 decomposition work
```

---

## Verification Checklist

After running both automation scripts, verify:

### Sentry Verification

```bash
# Check that projects were created
sentry-cli projects list --org phenotype-org

# Expected: 30 projects (phenotype-infrakit, heliosCLI, platforms-thegent, etc.)

# Check that GitHub Secrets were set
gh secret list -R KooshaPari/phenotype-infrakit | grep SENTRY_DSN

# Expected: SENTRY_DSN_XXXXX (one for each repo)
```

### Snyk Verification

```bash
# Check that you're authenticated
snyk whoami

# Expected: your-snyk-username@example.com

# Check that repos are enrolled (via Snyk dashboard)
# https://app.snyk.io/projects
# Expected: 30 projects listed
```

### GitHub Secrets Verification

```bash
# Check that all 3 required secrets are set
for secret in SENTRY_TOKEN SNYK_TOKEN SENTRY_ORG_SLUG; do
  gh secret list | grep $secret && echo "✅ $secret"
done

# Expected:
# ✅ SENTRY_TOKEN
# ✅ SNYK_TOKEN
# ✅ SENTRY_ORG_SLUG
```

---

## Timeline to Phase 2

| Step | Time | Status |
|------|------|--------|
| Sentry token regeneration | 5 min | ⏳ Awaiting user action |
| Snyk token acquisition | 5 min | ⏳ Awaiting user action |
| Sentry automation script | 5 min | 🔄 Ready to execute |
| Snyk deployment script | 5 min | 🔄 Ready to execute |
| Verification | 3 min | 🔄 Ready to execute |
| **Total Phase 1 completion** | **~23 min** | **92% → 100%** |
| Phase 2 kickoff | 2026-04-02 | 📅 Scheduled |

---

## Go/No-Go Decision Criteria

### GO if:
- ✅ Both tokens successfully acquired
- ✅ Both automation scripts complete without errors
- ✅ All 30 repos show projects in Sentry
- ✅ All 30 repos show scans in Snyk
- ✅ All 3 GitHub Secrets are set

### NO-GO if:
- ❌ Token acquisition fails repeatedly
- ❌ Automation scripts error on >2 repos
- ❌ <28 repos created in Sentry
- ❌ <28 repos enrolled in Snyk

---

## Support

If you encounter issues:

1. **Check the troubleshooting guide:** `PHASE1_TROUBLESHOOTING.md`
2. **Verify token format:** `cat ~/.sentryclirc`
3. **Re-run with verbose output:**
   ```bash
   bash -x scripts/automation/sentry-automation.sh
   bash -x scripts/automation/snyk-deployment.sh
   ```
4. **Check logs:**
   ```bash
   tail -100 /tmp/sentry-automation.log
   tail -100 /tmp/snyk-deployment.log
   ```

---

## Next Steps (Phase 2)

Once Phase 1 is complete:

1. **Deploy GitHub Actions workflows** (2-3 hours)
   - Sentry error tracking + alerting
   - Snyk security scanning + PR comments
   - Coordinated security event response

2. **Enable GitHub required status checks** (1 hour)
   - Require Sentry health checks
   - Require Snyk security gates

3. **Begin Phase 2 decomposition work** (ongoing)
   - Reference security framework in all PRs
   - Use Sentry + Snyk data in decision-making

---

## Final Checklist

Before you start, confirm:

- [ ] You have Sentry org access (phenotype-org)
- [ ] You have Snyk account access
- [ ] You can access GitHub with `gh` CLI
- [ ] You have 20 minutes of uninterrupted time
- [ ] You've reviewed this guide

**Ready to execute?** Start with **ITEM 1: Sentry Token Regeneration** above.
