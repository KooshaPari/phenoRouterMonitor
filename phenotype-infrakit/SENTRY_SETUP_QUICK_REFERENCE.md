# Sentry Setup — Quick Reference Card

## Current Status: 🔴 BLOCKED
**Issue:** Auth token insufficient permissions (HTTP 403)
**Action Required:** Regenerate Sentry auth token with `project:admin` scope

---

## 3-Step Unblock (5 minutes)

### Step 1: Regenerate Token (2 min)
```bash
# Go to Sentry auth tokens page
open https://sentry.io/settings/auth-tokens/

# OR manually navigate:
# 1. Log in to https://sentry.io/
# 2. Settings → Auth Tokens
# 3. Delete old token (if permissions insufficient)
# 4. Create new token with scopes:
#    ✓ project:admin
#    ✓ org:read
#    ✓ team:admin
# 5. Copy token to clipboard
```

### Step 2: Update Config (1 min)
```bash
# Edit config file
nano ~/.sentryclirc

# Replace token= line with your new token:
# token=sntrys_<YOUR_NEW_TOKEN_HERE>

# Save and exit (Ctrl+O, Ctrl+X)
```

### Step 3: Verify & Run (2 min)
```bash
# Test token
sentry-cli organizations list

# Expected: List of organizations (no 403 error)

# Run automation
bash /Users/kooshapari/CodeProjects/Phenotype/repos/scripts/create-sentry-projects.sh

# Expected: All 3 projects created + GitHub Secrets configured
```

---

## What Gets Created

**3 Sentry Projects:**
- AgilePlus
- phenotype-infrakit
- heliosCLI

**3 GitHub Secrets:**
- `SENTRY_DSN_AGILEPLUS`
- `SENTRY_DSN_INFRAKIT`
- `SENTRY_DSN_HELIOSCLI`

---

## Files & Locations

| File | Purpose | Location |
|------|---------|----------|
| Automation Script | Create projects + set secrets | `scripts/create-sentry-projects.sh` |
| Setup Guide | Detailed manual + troubleshooting | `docs/guides/SENTRY_SETUP_GUIDE.md` |
| Issue Tracker | Problem analysis + solution | `SENTRY_AUTH_ISSUE.md` |
| Full Report | Comprehensive execution report | `SENTRY_AUTOMATION_REPORT.md` |

---

## Verify Success

```bash
# 1. Check projects in Sentry dashboard
open https://sentry.io/organizations/stealth-startup-3u/projects/

# Expected: 3 projects listed (AgilePlus, phenotype-infrakit, heliosCLI)

# 2. Verify GitHub Secrets
gh secret list | grep SENTRY_DSN

# Expected:
# SENTRY_DSN_AGILEPLUS    Updated 2026-03-31
# SENTRY_DSN_HELIOSCLI    Updated 2026-03-31
# SENTRY_DSN_INFRAKIT     Updated 2026-03-31
```

---

## Fallback: Manual Creation (15 minutes)

If automation fails after token regeneration:

```bash
# 1. Go to projects page
open https://sentry.io/organizations/stealth-startup-3u/projects/

# 2. Click "Create Project" for each:
#    - Name: AgilePlus, Platform: Rust
#    - Name: phenotype-infrakit, Platform: Rust
#    - Name: heliosCLI, Platform: Rust

# 3. Copy DSN tokens and set secrets manually:
gh secret set SENTRY_DSN_AGILEPLUS --body '<DSN_FROM_PROJECT1>'
gh secret set SENTRY_DSN_INFRAKIT --body '<DSN_FROM_PROJECT2>'
gh secret set SENTRY_DSN_HELIOSCLI --body '<DSN_FROM_PROJECT3>'
```

---

## Common Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| HTTP 403 | Insufficient scopes | Regenerate token (Step 1 above) |
| Token not found | Config file issue | Create `~/.sentryclirc` with token |
| Cannot set secrets | GitHub auth issue | Run `gh auth login` |
| Project creation fails | API endpoint issue | Verify token, check firewall |

---

## Next Steps (After Success)

1. ✅ Integrate Sentry SDKs into projects
2. ✅ Configure CI/CD to pass DSN tokens
3. ✅ Test with sample errors
4. ✅ Set up alerts + dashboards

See `docs/guides/SENTRY_SETUP_GUIDE.md` for details.

---

**Created:** 2026-03-31
**Status:** Ready for user action
