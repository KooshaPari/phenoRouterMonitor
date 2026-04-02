# Sentry Error Tracking - Quick Start Guide

**Last Updated**: 2026-03-31
**Target Audience**: Developers, DevOps, Security Team
**Time Required**: 5-10 minutes per repo

## What is This?

Sentry is an error tracking and performance monitoring platform that automatically captures, aggregates, and alerts on exceptions in production applications. The Tier 1 repos (AgilePlus, heliosCLI, phenotype-infrakit) now have Sentry SDKs deployed.

## Quick Links

| What | Where |
|------|-------|
| Sentry Dashboard | https://sentry.io/organizations/phenotype/ |
| AgilePlus Errors | https://sentry.io/organizations/phenotype/issues/?project=agileplus |
| heliosCLI Errors | https://sentry.io/organizations/phenotype/issues/?project=helioscli |
| phenotype-infrakit Errors | https://sentry.io/organizations/phenotype/issues/?project=phenotype-infrakit |
| GitHub Integration Setup | https://sentry.io/settings/phenotype/integrations/github/ |

## 5-Minute Setup Checklist

### Step 1: Get Your DSN (Sentry Admin Only)

```bash
# One-time per project:
# 1. Go to: https://sentry.io/settings/phenotype/projects/
# 2. Click "agileplus" (or project name)
# 3. Copy "DSN (Public)" from left sidebar
# 4. Example DSN looks like: https://key@domain.ingest.sentry.io/12345
```

### Step 2: Store DSN as GitHub Secret (DevOps Only)

```bash
# For AgilePlus:
gh secret set SENTRY_DSN_AGILEPLUS --body 'https://key@domain.ingest.sentry.io/12345'

# For heliosCLI:
gh secret set SENTRY_DSN_HELIOSCLI --body '<dsn>'

# For phenotype-infrakit:
gh secret set SENTRY_DSN_PHENOTYPE_INFRAKIT --body '<dsn>'

# Verify:
gh secret list | grep SENTRY
```

### Step 3: Enable GitHub Integration (Sentry Admin Only)

```
1. Go to: https://sentry.io/settings/phenotype/integrations/github/
2. Click "Install" if not already done
3. Grant access to KooshaPari GitHub organization
4. Authorize Sentry GitHub app
```

### Step 4: Create Alert Rule (Optional but Recommended)

```
1. Go to: https://sentry.io/organizations/phenotype/alerts/rules/
2. Click "Create Alert Rule"
3. Configure:
   - Condition: "Error events"
   - Action: "Create GitHub issue"
   - Project: Select "agileplus" (or your project)
   - Labels: Add "sentry-critical, error-tracking"
4. Click "Save Rule"
```

### Step 5: Test It

```bash
# In any Tier 1 repo:
export SENTRY_DSN="https://key@domain.ingest.sentry.io/12345"
cargo test --lib sentry_config

# Or trigger in code:
# Let an error happen in production, check dashboard in 20-60 seconds
```

## What Happens Automatically

Once deployed:

1. **Every error in production** is automatically captured
2. **Stack traces** are sent to Sentry dashboard
3. **GitHub issues are created** (if alert rule enabled)
4. **You get notified** via email or Slack

## Where to Look

### For Developers

**My errors**: https://sentry.io/organizations/phenotype/issues/

- See all errors across Tier 1 projects
- Filter by project, environment, version
- Click to view full stack trace, breadcrumbs, replay

### For DevOps

**Health check**: `.github/workflows/sentry-error-tracking.yml`

- Runs daily at 6 AM UTC
- Runs on every push to main
- Check workflow output → Sentry dashboard link

### For Security Team

**Alerts**: GitHub issues with label `sentry-critical`

- Auto-created from Sentry on high-severity errors
- Links to Sentry event for investigation
- Can assign and track response

## Common Tasks

### View an Error in Sentry

```
1. Go to: https://sentry.io/organizations/phenotype/issues/
2. Find error by:
   - Project (filter by "agileplus", etc.)
   - Time (Last 24h, This week, etc.)
   - Status (Unresolved, For review, etc.)
3. Click to view:
   - Full stack trace
   - Breadcrumbs (what happened before error)
   - Tags (version, environment, user)
   - Replays (if enabled)
```

### Create a GitHub Issue from Sentry Error

**Automatic** (if alert rule enabled):
- High-severity error triggers GitHub issue automatically
- Link appears in issue body

**Manual**:
```
1. View error in Sentry
2. Click "Recommended Actions" → "Create GitHub Issue"
3. Select repo and confirm
4. Issue created with labels: sentry-critical, error-tracking
```

### Investigate a Failed Sentry Health Check

If workflow fails:

```bash
# 1. Check if DSN is set
gh secret get SENTRY_DSN_AGILEPLUS

# 2. Test locally
export SENTRY_DSN="$(gh secret get SENTRY_DSN_AGILEPLUS)"
cargo test --lib sentry_config

# 3. Check Sentry project exists
# Go to: https://sentry.io/organizations/phenotype/

# 4. If still failing, check workflow logs
gh workflow view sentry-error-tracking.yml -R KooshaPari/AgilePlus
```

### Disable Error Tracking (Emergency Only)

```bash
# Remove DSN secret (errors won't be sent)
gh secret delete SENTRY_DSN_AGILEPLUS

# Or set dummy DSN:
gh secret set SENTRY_DSN_AGILEPLUS --body 'https://test@test.ingest.sentry.io/0'

# Restart workflow to apply changes
gh workflow run sentry-error-tracking.yml -R KooshaPari/AgilePlus
```

## Troubleshooting

### Errors not appearing in Sentry?

1. **Check DSN is set**:
   ```bash
   echo $SENTRY_DSN  # Should print https://key@...
   ```

2. **Check SDK is initialized**:
   - Look for `sentry_config::initialize()` call in main.rs
   - Should be near application startup

3. **Check error level**:
   - Sentry captures "error" and "fatal" by default
   - Warnings and info don't trigger alerts
   - Check your error is actually an error

4. **Check Sentry project**:
   - Go to: https://sentry.io/organizations/phenotype/projects/
   - Click your project
   - Look for "Client Keys" with active status

### GitHub issues not auto-creating?

1. **Check GitHub integration is enabled**:
   - Go to: https://sentry.io/settings/phenotype/integrations/github/
   - Should show "Installed"

2. **Check alert rule exists**:
   - Go to: https://sentry.io/organizations/phenotype/alerts/rules/
   - Should see a rule that creates GitHub issues

3. **Check error severity**:
   - Alert rule must match error level
   - Default: Only "error" and "fatal" create issues
   - Check your error isn't marked as "warning"

4. **Check GitHub permissions**:
   - Sentry app needs permission to create issues
   - Should see it in GitHub org settings
   - If missing, reinstall: https://sentry.io/settings/phenotype/integrations/github/

## Next Steps

- **Set up Slack notifications**: https://sentry.io/settings/phenotype/integrations/slack/
- **Enable Session Replays**: https://sentry.io/settings/phenotype/projects/<project>/replay/
- **Configure Performance Monitoring**: https://sentry.io/settings/phenotype/projects/<project>/performance/
- **Add custom tags**: See full docs at https://docs.sentry.io/

## Reference

- **Full Finalization Docs**: `/docs/reports/SENTRY_TIER1_FINALIZATION.md`
- **SDK Code**: `*/src/sentry_config.rs` (all 3 repos)
- **Workflow File**: `.github/workflows/sentry-error-tracking.yml`

---

**Need Help?** Check the troubleshooting section above or see the full finalization document.
