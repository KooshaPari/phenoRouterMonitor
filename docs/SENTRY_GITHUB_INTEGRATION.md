# Sentry GitHub Integration Setup Guide

Complete walkthrough for integrating Sentry with GitHub for automatic issue creation from error events.

## Prerequisites

- Sentry account with projects created for AgilePlus, phenotype-infrakit, and heliosCLI
- Admin access to KooshaPari GitHub organization
- Admin access to Sentry organization

## Step-by-Step Setup

### Phase 1: Create Sentry Projects

1. **Go to Sentry.io:**
   - https://sentry.io/organizations/

2. **Create Projects (if not already done):**
   - Click "Projects" → "Create Project"
   - Choose platform: **Rust**
   - Name: AgilePlus
   - Team: Default
   - Alert frequency: Real-time
   - Click "Create Project"
   - Copy DSN (e.g., `https://key@sentry.io/project-id`)

3. **Repeat for other repos:**
   - phenotype-infrakit
   - heliosCLI

### Phase 2: GitHub Integration Setup

#### Install GitHub App in Sentry

1. **Go to Sentry Integration Settings:**
   - https://sentry.io/settings/integrations/

2. **Search for GitHub:**
   - Click "GitHub"
   - Click "Install"

3. **Authorize GitHub:**
   - You'll be redirected to GitHub authorization
   - Repository selector will show up
   - Select repositories:
     - AgilePlus
     - phenotype-infrakit
     - heliosCLI
   - Click "Install" to complete

4. **Verify Installation:**
   - Return to Sentry
   - Should see "GitHub installed" confirmation
   - Status shows "Installed"

#### Configure GitHub per Project

1. **Go to AgilePlus Project Settings:**
   - https://sentry.io/settings/projects/agileplus/

2. **Navigate to Integrations:**
   - Left sidebar → "Integrations"
   - Find "GitHub"
   - Click to configure

3. **Enable Issue Creation:**
   - Toggle "Create GitHub issues"
   - Select repository: `KooshaPari/AgilePlus`
   - Click "Save"

4. **Repeat for:**
   - phenotype-infrakit project → `KooshaPari/phenotype-infrakit`
   - heliosCLI project → `KooshaPari/heliosCLI`

### Phase 3: Configure Alert Rules

#### Create Alert for New Errors

1. **Go to Project Alerts:**
   - https://sentry.io/alerts/

2. **Create Alert Rule:**
   - Click "Create Alert Rule"
   - **Condition:**
     - When: "A new issue is created"
     - AND "Error level" is "error" or higher
   - **Actions:**
     - Click "Create GitHub Issue"
     - Repository: Select target repo
     - Title: `[{project}] {title}`
     - Description: Include error message, stack trace
     - Labels: `sentry`, `auto-issue`
   - Click "Save Rule"

3. **Example Rule Configuration:**

```
Name: "High Severity Errors → GitHub Issues"
Conditions:
  - New event in error level
  - Ignore ignored issues
  - Ignore until fixed
Actions:
  - Create GitHub issue
  - Notify team via Slack (optional)
```

### Phase 4: Test Integration

#### Trigger Test Error

1. **In AgilePlus:**
   ```bash
   cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
   SENTRY_DSN="your-dsn-here" cargo test --lib sentry_config -- --nocapture
   ```

2. **Monitor Sentry:**
   - Go to AgilePlus project
   - Issues tab
   - Should see new issue appear within 30 seconds

3. **Verify GitHub Issue Created:**
   - Go to GitHub: https://github.com/KooshaPari/AgilePlus/issues
   - Should see new issue with title like:
     ```
     [AgilePlus] Test error for Sentry capture
     ```
   - Issue body should include:
     - Error message
     - Stack trace
     - Environment (development/production)
     - Release version
     - Sentry link

### Phase 5: GitHub Secrets for CI/CD

#### Add Repository Secrets

1. **Go to AgilePlus Repository Settings:**
   - https://github.com/KooshaPari/AgilePlus/settings/secrets/actions

2. **New Repository Secret:**
   - Name: `SENTRY_DSN`
   - Value: `https://your-key@sentry.io/your-project-id`
   - Click "Add secret"

3. **Repeat for:**
   - phenotype-infrakit
   - heliosCLI

#### Use in GitHub Actions

Add to `.github/workflows/build.yml`:

```yaml
name: Build & Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    env:
      SENTRY_DSN: ${{ secrets.SENTRY_DSN }}
      SENTRY_ENVIRONMENT: ci

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test --all
        # Tests will automatically send errors to Sentry
```

## Monitoring & Maintenance

### Daily Checks

1. **Sentry Dashboard:**
   - Check error count
   - Review new errors
   - Assign and resolve issues

2. **GitHub Issues:**
   - Filter by label `sentry` or `auto-issue`
   - Triage new issues
   - Link to related code

### Weekly Review

1. **Error Trends:**
   - Most frequent errors
   - Error rate by release
   - Environment breakdown

2. **Performance:**
   - Error latency
   - Stack trace completeness
   - GitHub issue creation success rate

### Monthly Maintenance

1. **Alert Rule Tuning:**
   - Review false positives
   - Adjust severity thresholds
   - Add new patterns if needed

2. **Release Tracking:**
   - Mark releases as "released"
   - Tag commits with release versions
   - Update Sentry release notes

## Troubleshooting

### Issue: GitHub Integration Shows "Not Installed"

**Solution:**
1. Go to Sentry Settings → Integrations
2. Click GitHub
3. Re-authorize if needed
4. Confirm "Installed" status

### Issue: GitHub Issues Not Being Created

**Check:**
1. Alert rule is active and configured correctly
2. Repository selector shows correct repo
3. Sentry project has "Create issues" permission
4. Error severity matches alert rule conditions

**Fix:**
```bash
# Test manually
# 1. Go to Sentry issue
# 2. Click "Link Existing Issue" or "Create Issue"
# 3. Select repository
# 4. Click Create
```

### Issue: Duplicate GitHub Issues for Same Error

**Solution:**
1. Enable "Ignore until fixed" in alert rule
2. Sentry will only create issue for first occurrence per group
3. Subsequent errors link to same issue

### Issue: GitHub API Rate Limit Exceeded

**Solution:**
1. Reduce alert frequency
2. Increase severity threshold (e.g., only High/Critical)
3. Implement error grouping to reduce total issues

### Issue: Missing Stack Traces in GitHub Issues

**Solution:**
1. Ensure `backtrace` feature is enabled in Sentry SDK
2. Check Sentry project settings:
   - Settings → Source Maps
   - Upload debug symbols if using compiled Rust
3. Use custom issue template with full trace

## Advanced Configuration

### Custom GitHub Issue Template

In Sentry Project Settings → Integrations → GitHub:

```
Title: [{project}] {title}

Description:
### Error Details
- **Project:** {project}
- **Environment:** {environment}
- **Release:** {release}
- **URL:** {url}

### Stack Trace
```
{stack_trace}
```

### Context
- **Fingerprint:** {fingerprint}
- **First Seen:** {first_seen}
- **Last Seen:** {last_seen}
- **Count:** {count}

[View in Sentry]({url})
```

### Environment-Specific Rules

Create separate alert rules:

```
# Production errors only
Environment: production
Severity: error or critical
Action: Create GitHub issue + Slack notification

# Staging errors
Environment: staging
Severity: error
Action: Create GitHub issue (labeled "staging")

# Development
Environment: development
Severity: critical only
Action: Slack notification only
```

### Team Notifications

**Slack Integration:**

1. Go to Sentry → Integrations → Slack
2. Authorize Slack workspace
3. In alert rule, add action: "Post to Slack"
4. Select channel: `#engineering` or `#alerts`

**Email Notifications:**

1. Sentry → Settings → Notifications
2. Enable "Digest emails"
3. Frequency: Daily
4. Time: 9:00 AM UTC

## Success Criteria Checklist

- [ ] GitHub App installed in Sentry
- [ ] GitHub repositories linked to Sentry projects
- [ ] Alert rules created for all three projects
- [ ] Test error creates GitHub issue within 30 seconds
- [ ] GitHub issue includes stack trace and context
- [ ] GitHub Secrets configured for CI/CD
- [ ] CI/CD workflow captures errors in test environment
- [ ] Team receives notifications (Slack, email)
- [ ] Release tracking enabled
- [ ] No duplicate issues for same error

## Quick Reference

| Task | URL |
|------|-----|
| Sentry Org | https://sentry.io/organizations/ |
| AgilePlus Project | https://sentry.io/organizations/*/issues/?project=ID |
| GitHub Integration | https://sentry.io/settings/integrations/github/ |
| Alert Rules | https://sentry.io/alerts/ |
| AgilePlus Repo | https://github.com/KooshaPari/AgilePlus |
| phenotype-infrakit Repo | https://github.com/KooshaPari/phenotype-infrakit |
| heliosCLI Repo | https://github.com/KooshaPari/heliosCLI |

## Support

For issues or questions:
1. Check [Sentry Docs](https://docs.sentry.io/)
2. Review [GitHub Integration Guide](https://docs.sentry.io/product/integrations/github/)
3. Contact Sentry support: support@sentry.io

---

**Last Updated:** 2026-03-30
**Status:** Ready for deployment
**Repos:** AgilePlus, phenotype-infrakit, heliosCLI
