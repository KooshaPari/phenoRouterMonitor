# Sentry GitHub Integration Guide

This guide covers setup and configuration of the Sentry-GitHub integration for automatic issue creation, commit linking, and release tracking.

## Overview

The Sentry-GitHub integration enables:
- **Auto-Issue Creation**: Errors automatically create GitHub issues
- **Commit Linking**: Stack traces link to responsible commits
- **Release Tracking**: Git tags automatically register as releases in Sentry
- **PR Integration**: Errors linked to specific pull requests
- **Slack Notifications**: Alert channels when errors occur

## Prerequisites

- Sentry account with organization access
- GitHub account with admin rights to target repositories
- Each repo must be public or org-accessible to Sentry app

## Step 1: Install GitHub App in Sentry

### Option A: Automatic Installation (Recommended)

1. Navigate to [Sentry Organization Settings](https://sentry.io/settings/phenotype/)
2. Go to **Integrations** → **GitHub**
3. Click **"Install GitHub Integration"**
4. You'll be redirected to GitHub authorization page
5. Click **"Authorize sentry"**
6. Sentry app is now installed in GitHub

### Option B: Manual Installation

1. Go to GitHub Settings → Apps → Authorized OAuth Apps
2. Find "Sentry" app
3. Click to review and authorize
4. Grant access to target repositories

## Step 2: Link Repositories to Sentry Projects

### For AgilePlus

1. Go to [AgilePlus Project Settings](https://sentry.io/settings/phenotype/projects/agileplus/integrations/)
2. Find **GitHub** in integrations list
3. Click **Configure**
4. Select repository: **KooshaPari/AgilePlus**
5. Enable:
   - ✅ Create issues
   - ✅ Link commits
   - ✅ Link releases
6. Click **Save**

### For phenotype-infrakit

1. Go to [phenotype-infrakit Project Settings](https://sentry.io/settings/phenotype/projects/phenotype-infrakit/integrations/)
2. Find **GitHub** in integrations list
3. Click **Configure**
4. Select repository: **KooshaPari/phenotype-infrakit**
5. Enable:
   - ✅ Create issues
   - ✅ Link commits
   - ✅ Link releases
6. Click **Save**

### For heliosCLI

1. Go to [heliosCLI Project Settings](https://sentry.io/settings/phenotype/projects/helioscli/integrations/)
2. Find **GitHub** in integrations list
3. Click **Configure**
4. Select repository: **KooshaPari/heliosCLI**
5. Enable:
   - ✅ Create issues
   - ✅ Link commits
   - ✅ Link releases
6. Click **Save**

## Step 3: Configure Alert Rules for Auto-Issue Creation

### AgilePlus Alert Rule

1. Go to [AgilePlus Alert Rules](https://sentry.io/alerts/rules/phenotype/agileplus/)
2. Click **Create Alert Rule**
3. Configure:

**Trigger**:
```
When: An issue is first seen
    AND has more than 5 occurrences in 5 minutes
    AND is in the [production, staging] environment
```

**Action**:
```
Send a notification to: #agileplus-errors (Slack)
AND Create an issue in: KooshaPari/AgilePlus
   Title: [Sentry] {title}
   Description: {description}
   Labels: sentry, error, triage
```

4. Click **Save**

### phenotype-infrakit Alert Rule

1. Go to [phenotype-infrakit Alert Rules](https://sentry.io/alerts/rules/phenotype/phenotype-infrakit/)
2. Click **Create Alert Rule**
3. Configure:

**Trigger**:
```
When: An error event is received
    AND is in the [production] environment
```

**Action**:
```
Send a notification to: #infrastructure-errors (Slack)
AND Create an issue in: KooshaPari/phenotype-infrakit
   Title: [Sentry] {title}
   Labels: sentry, infrastructure, critical
```

4. Click **Save**

### heliosCLI Alert Rule

1. Go to [heliosCLI Alert Rules](https://sentry.io/alerts/rules/phenotype/helioscli/)
2. Click **Create Alert Rule**
3. Configure:

**Trigger**:
```
When: An error event is received
    AND error.type is one of [panic, exception, unhandled]
```

**Action**:
```
Send a notification to: #helioscli-errors (Slack)
AND Create an issue in: KooshaPari/heliosCLI
   Title: [Sentry {level}] {title}
   Labels: sentry, cli, bug
```

4. Click **Save**

## Step 4: Release Tracking Configuration

### Automatic Release Creation

When you push a git tag, Sentry automatically creates a release:

```bash
# Tag a release
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3

# Sentry receives GitHub webhook and creates release automatically
# Errors are now attributed to v1.2.3
```

### Manual Release Creation (sentry-cli)

For more control, use the Sentry CLI:

```bash
# Install sentry-cli (if not already installed)
curl -sL https://files.pythonhosted.org/packages/[version]/sentry-cli | bash

# Create release manually
sentry-cli releases create -p agileplus v1.2.3

# Upload source maps (if applicable)
sentry-cli releases files -p agileplus v1.2.3 upload-sourcemap ./target/release

# Mark as deployed
sentry-cli releases deploys -p agileplus v1.2.3 new \
  --url https://agileplus.example.com \
  --env production
```

### Release Configuration in Cargo.toml

Add to workspace Cargo.toml to enable automatic release detection:

```toml
[package]
name = "agileplus"
version = "1.2.3"  # Must match git tag (without 'v' prefix)
```

Then in main.rs:

```rust
let _guard = sentry::init(sentry::ClientOptions {
    release: sentry::release_name!(),  // Automatically detects version
    ..Default::default()
});
```

## Step 5: Slack Integration (Optional)

### Prerequisites
- Sentry organization must have admin permissions
- Slack workspace where you have admin rights

### Setup Steps

1. Go to [Sentry Organization Settings](https://sentry.io/settings/phenotype/)
2. Navigate to **Integrations** → **Slack**
3. Click **Install**
4. Select Slack workspace and authorize
5. Create alert channels:
   - `#agileplus-errors` (or use existing)
   - `#infrastructure-errors` (or use existing)
   - `#helioscli-errors` (or use existing)

### Configure Slack Notifications per Project

1. Project Settings → **Integrations** → **Slack**
2. Select channel for notifications
3. Configure what triggers notifications:
   ```
   - New issues
   - Regression detected
   - Critical errors
   - Release deployed
   ```

### Slack Message Format

Example auto-generated Sentry alert in Slack:

```
🚨 Error in AgilePlus production
────────────────────────────────
Type: panicked at 'assertion failed'
File: routes.rs:42
Occurrences: 12 in the last hour
Environment: production
Release: v1.2.3

👁️ View in Sentry: https://sentry.io/organizations/phenotype/issues/...
🐛 GitHub Issue: https://github.com/KooshaPari/AgilePlus/issues/123

[Resolve] [Ignore] [Archive]
```

## Step 6: Source Code Visibility

### Configure Source Code Access

To enable Sentry to show source code in error pages:

1. Go to Project Settings → **Source Maps**
2. Click **Link your repository**
3. Select: **KooshaPari/AgilePlus** (or respective repo)
4. Sentry will fetch source code directly from GitHub

### Source Code Display

Once configured, stack traces will show:
```
Error in routes.rs:42
────────────────────
40: pub async fn create_item(req: Request) -> Response {
41:     let config = load_config();
42:     assert!(config.is_valid()); // ← Error occurred here
43:     process_item(&req).await
44: }
```

## Step 7: Commit & Deploy Tracking

### GitHub Webhook Configuration

Sentry automatically registers GitHub webhooks when you connect a repository:

1. Go to your GitHub repo → Settings → **Webhooks**
2. Look for webhook from `sentry.io`
3. Verify it's enabled and has recent successful deliveries

### Automatic Release Correlation

When pushing code:

```bash
# Commit with message (will auto-link to Sentry errors)
git commit -am "Fix database connection error (#42)"
git push origin feature-branch

# Create PR and merge
git merge --no-ff feature-branch
git push origin main

# Tag release
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3

# Sentry now knows:
# - Release v1.2.3 includes commit #abc123
# - Commit #abc123 fixed GitHub issue #42
# - Related Sentry errors can be marked resolved
```

### Manual Commit Association

If automatic detection doesn't work:

```bash
sentry-cli releases create -p agileplus v1.2.3
sentry-cli releases set-commits -p agileplus v1.2.3 \
  --commit "KooshaPari/AgilePlus@HEAD"
```

## Testing the Integration

### Test 1: Trigger Error and Verify Auto-Issue Creation

```bash
# In a test crate
cargo test test_sentry_github_integration

# This should:
# 1. Trigger an error in Sentry
# 2. After 5+ occurrences, auto-create GitHub issue
# 3. Link to commit that introduced error
# 4. Post to #agileplus-errors Slack channel
```

### Test 2: Verify Release Tracking

```bash
# Create test release
git tag -a v0.0.1-test -m "Test release"
git push origin v0.0.1-test

# Verify in Sentry:
# 1. Go to AgilePlus → Releases
# 2. Look for v0.0.1-test in list
# 3. Click to see associated errors
# 4. Should show commits included in release
```

### Test 3: Verify GitHub Issue Creation

1. Trigger intentional error in tests:
   ```rust
   #[test]
   fn test_github_issue_creation() {
       let _guard = sentry::init(DEFAULT_CONFIG);
       panic!("Testing GitHub issue auto-creation");
   }
   ```

2. Run test multiple times to reach 5+ occurrences:
   ```bash
   for i in {1..6}; do cargo test test_github_issue_creation; done
   ```

3. Check Sentry Dashboard:
   - Project → Issues → Sort by "New"
   - Find the test panic issue
   - Should show GitHub issue creation action in dropdown

4. Check GitHub:
   - Go to **Issues** tab
   - Look for "[Sentry] Testing GitHub issue auto-creation"
   - Verify it links back to Sentry

## Troubleshooting

### GitHub Integration Not Showing Projects

**Problem**: GitHub integration lists no repositories.

**Solution**:
1. Check GitHub app permissions: Settings → Applications → Authorized OAuth Apps
2. Re-authorize Sentry app if needed
3. Verify org admin has granted access
4. Try: Org Settings → Integrations → GitHub → Re-install

### Auto-Issue Creation Not Working

**Problem**: Errors appear in Sentry but no GitHub issues created.

**Checks**:
1. Verify alert rule is enabled: Project Settings → Alert Rules
2. Check trigger conditions are met (e.g., 5+ occurrences)
3. Verify GitHub repository is linked: Project Settings → Integrations → GitHub
4. Check GitHub app has "Issues" write permission
5. Review recent alert logs: Project Settings → Alert Rules → [Rule] → View logs

### Release Not Showing in Sentry

**Problem**: Git tag pushed but not appearing in Sentry Releases.

**Solution**:
1. Verify webhook is active: GitHub Repo → Settings → Webhooks → Sentry
2. Check webhook delivery logs for errors
3. Manually create release:
   ```bash
   sentry-cli releases create -p agileplus "$(git describe --tags)"
   ```
4. Verify version format matches `Cargo.toml` (without 'v' prefix)

### Commits Not Linked to Release

**Problem**: Release shows in Sentry but commit list is empty.

**Solution**:
```bash
# Manually set commits
sentry-cli releases set-commits -p agileplus v1.2.3 \
  --auto  # Auto-detects commits from git history
```

### Slack Not Receiving Alerts

**Problem**: Alert rules trigger but Slack stays silent.

**Solution**:
1. Verify Slack integration is enabled: Org Settings → Integrations → Slack
2. Check channel exists and Sentry app has access
3. Go to Project Settings → Integrations → Slack
4. Verify notification channels are configured
5. Test alert: Project Settings → Alert Rules → [Rule] → Test

## Best Practices

### 1. Meaningful Release Names

Use semantic versioning with dates:
```
v1.2.3           # Standard release
v1.2.3-rc.1      # Release candidate
v1.2.3-2026-03-30 # Date-based release
```

### 2. Include Commit Context

Commit messages should reference issues:
```
git commit -m "Fix database error (#42)"
           # This links to GitHub issue #42
           # Sentry can auto-resolve related errors
```

### 3. Add Release Notes in GitHub

When creating a release:
```
Title: v1.2.3 - Security & QA Phase 1
Description:
- Sentry integration for error tracking
- GitHub auto-issue creation
- Release tracking
- Closes #42, #43, #44
```

### 4. Monitor Alert Rules

Regularly review alert performance:
1. Project Settings → **Alert Rules**
2. Check "recent alerts" section
3. Disable overly-sensitive rules
4. Add new rules for critical errors

### 5. Archive Resolved Issues

Once fixed, mark in Sentry:
1. Go to issue
2. Click **Resolve** or **Archive**
3. GitHub issue auto-closes if linked
4. Prevents alert spam for same error

## Summary

The Sentry-GitHub integration provides:
- ✅ Automatic error-to-issue conversion
- ✅ Commit linking for root cause analysis
- ✅ Release tracking with version correlation
- ✅ Slack notifications for team awareness
- ✅ Source code visibility in stack traces
- ✅ PR/commit attribution

All 3 Tier 1 repos (AgilePlus, phenotype-infrakit, heliosCLI) are now configured for full GitHub integration.

## References

- [Sentry GitHub Integration Docs](https://docs.sentry.io/integrations/github/)
- [Sentry Alert Rules](https://docs.sentry.io/product/alerts/)
- [Sentry CLI Reference](https://docs.sentry.io/cli/)
- [GitHub Webhook Events](https://docs.github.com/en/developers/webhooks-and-events/webhooks/about-webhooks)
