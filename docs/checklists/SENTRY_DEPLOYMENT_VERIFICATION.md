# Sentry Deployment Verification Checklist

**Date**: 2026-03-31
**Scope**: Tier 1 (AgilePlus, heliosCLI, phenotype-infrakit)
**Status**: Ready for Verification

## Pre-Deployment Verification

Use this checklist to verify Sentry integration before marking complete.

### SDK Configuration (✓ Verified)

- [x] AgilePlus: `libs/logger/src/sentry_config.rs` exists
- [x] AgilePlus: SDK reads `SENTRY_DSN` from environment
- [x] AgilePlus: Unit tests present (FR-SENTRY-001, FR-SENTRY-002)
- [x] heliosCLI: `crates/harness_utils/src/sentry_config.rs` exists
- [x] heliosCLI: SDK reads `SENTRY_DSN` from environment
- [x] heliosCLI: Unit tests present (FR-SENTRY-001, FR-SENTRY-002)
- [x] phenotype-infrakit: `crates/phenotype-sentry-config/src/lib.rs` exists
- [x] phenotype-infrakit: SDK reads `SENTRY_DSN` from environment
- [x] phenotype-infrakit: Unit tests present (FR-SENTRY-001, FR-SENTRY-002)

### GitHub Actions Workflows (✓ Verified)

- [x] AgilePlus: `.github/workflows/sentry-error-tracking.yml` exists
- [x] AgilePlus: Workflow triggers on push/pull_request/schedule
- [x] AgilePlus: Workflow has sentry-health-check job
- [x] AgilePlus: Workflow has integration-with-github-issues job
- [x] heliosCLI: `.github/workflows/sentry-error-tracking.yml` exists
- [x] heliosCLI: Workflow triggers on push/pull_request/schedule
- [x] heliosCLI: Workflow has sentry-health-check job
- [x] heliosCLI: Workflow has integration-with-github-issues job
- [x] phenotype-infrakit: `.github/workflows/sentry-error-tracking.yml` exists
- [x] phenotype-infrakit: Workflow triggers on push/pull_request/schedule
- [x] phenotype-infrakit: Workflow has sentry-health-check job
- [x] phenotype-infrakit: Workflow has integration-with-github-issues job

### Documentation (✓ Verified)

- [x] `/docs/reports/SENTRY_TIER1_FINALIZATION.md` - Comprehensive deployment guide
- [x] `/docs/guides/SENTRY_QUICK_START.md` - 5-minute developer guide
- [x] This checklist: `/docs/checklists/SENTRY_DEPLOYMENT_VERIFICATION.md`

## Post-Deployment Verification (Manual)

Complete these checks after DSNs are configured.

### Step 1: GitHub Secret Configuration

**Action**: Configure DSN secrets in GitHub

```bash
# For each repo, get DSN from Sentry (https://sentry.io/settings/phenotype/projects/)
# Then set secret:

cd AgilePlus
gh secret set SENTRY_DSN_AGILEPLUS --body '<dsn-from-sentry>'
gh secret list | grep SENTRY_DSN_AGILEPLUS  # Verify created

cd ../heliosCLI
gh secret set SENTRY_DSN_HELIOSCLI --body '<dsn-from-sentry>'
gh secret list | grep SENTRY_DSN_HELIOSCLI

cd ../phenotype-infrakit
gh secret set SENTRY_DSN_PHENOTYPE_INFRAKIT --body '<dsn-from-sentry>'
gh secret list | grep SENTRY_DSN_PHENOTYPE_INFRAKIT
```

**Verification**:
- [ ] AgilePlus: `SENTRY_DSN_AGILEPLUS` secret created
- [ ] heliosCLI: `SENTRY_DSN_HELIOSCLI` secret created
- [ ] phenotype-infrakit: `SENTRY_DSN_PHENOTYPE_INFRAKIT` secret created

### Step 2: Local SDK Test (Each Repo)

**Action**: Run Sentry SDK tests with DSN

```bash
# AgilePlus
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
export SENTRY_DSN="$(gh secret get SENTRY_DSN_AGILEPLUS)"
cargo test --lib sentry_config -- --nocapture
```

**Expected Output**:
```
test tests::test_environment_override ... ok
test tests::test_initialize_without_dsn ... ok
test result: ok. 2 passed; 0 failed
```

**Verification**:
- [ ] AgilePlus: Tests pass with DSN
- [ ] heliosCLI: Tests pass with DSN
- [ ] phenotype-infrakit: Tests pass with DSN

### Step 3: GitHub Actions Workflow Trigger

**Action**: Manually trigger sentry-error-tracking workflow

```bash
# AgilePlus
cd AgilePlus
gh workflow run sentry-error-tracking.yml

# Wait for workflow to complete (check Actions tab)
# Expected: sentry-health-check job succeeds, generates dashboard link
```

**Verification** (check GitHub Actions):
- [ ] AgilePlus: Workflow runs successfully
- [ ] AgilePlus: Step "Verify Sentry DSN is configured" passes
- [ ] AgilePlus: Step "Generate Sentry dashboard link" outputs URL
- [ ] heliosCLI: Workflow runs successfully
- [ ] heliosCLI: Step "Verify Sentry DSN is configured" passes
- [ ] phenotype-infrakit: Workflow runs successfully
- [ ] phenotype-infrakit: Step "Verify Sentry DSN is configured" passes

### Step 4: Sentry Dashboard Access

**Action**: Verify Sentry dashboards are accessible

```
1. Go to: https://sentry.io/organizations/phenotype/issues/
2. Verify projects appear:
   - agileplus
   - helioscli
   - phenotype-infrakit
3. Each project should show:
   - Project name
   - Last event received (or "No events yet")
   - Error count
```

**Verification**:
- [ ] Sentry org dashboard loads
- [ ] All 3 projects visible in project list
- [ ] Can filter by individual project
- [ ] Project details page loads without errors

### Step 5: GitHub Integration Setup (Admin Only)

**Action**: Enable GitHub integration in Sentry

```
1. Go to: https://sentry.io/settings/phenotype/integrations/github/
2. If not authorized:
   - Click "Authorize GitHub"
   - Grant access to KooshaPari organization
3. Verify status shows "Installed"
```

**Verification**:
- [ ] GitHub integration authorized
- [ ] Status shows "Installed" (not "Pending")
- [ ] KooshaPari org authorized
- [ ] Can see available repositories

### Step 6: Alert Rule Creation (Recommended)

**Action**: Create alert rule for auto-issue creation

```
1. Go to: https://sentry.io/organizations/phenotype/alerts/rules/
2. Click "Create Alert Rule"
3. Configure:
   Environment: production
   Condition: (any) error event
   Action: Create GitHub issue
   Project: agileplus
   Labels: sentry-critical,error-tracking
4. Click "Save Rule"
5. Repeat for heliosCLI and phenotype-infrakit
```

**Verification**:
- [ ] Alert rule created for agileplus
- [ ] Alert rule created for helioscli
- [ ] Alert rule created for phenotype-infrakit
- [ ] Each rule shows "Enabled" status
- [ ] GitHub integration shows in alert action

### Step 7: End-to-End Error Capture Test

**Action**: Trigger test error and verify flow

**In Sentry Dashboard**:

```bash
# Trigger a test error capture in one repo
cd AgilePlus
export SENTRY_DSN="$(gh secret get SENTRY_DSN_AGILEPLUS)"

# Create test code (temporary):
# In main.rs or test file:
#
# let _guard = sentry_config::initialize();
# let result: Result<i32> = Err(std::io::Error::new(
#     std::io::ErrorKind::Other,
#     "Test error for Sentry integration"
# ));
# if let Err(e) = result {
#     sentry_config::capture_error(&e);
# }

# Or in test:
cargo test --lib sentry_config test_error_capture -- --nocapture
```

**Expected Flow**:
1. Error captured locally (stderr output)
2. Within 5-30 seconds: Error appears in Sentry dashboard
3. Within 60 seconds: GitHub issue auto-created (if rule enabled)

**Verification**:
- [ ] Error appears in Sentry dashboard
- [ ] Error shows correct environment (production)
- [ ] Error shows correct release version
- [ ] Stack trace is readable and complete
- [ ] GitHub issue created within 60 seconds
- [ ] Issue has labels: sentry-critical, error-tracking
- [ ] Issue links back to Sentry event

### Step 8: Health Check Workflow Verification

**Action**: Monitor next scheduled health check

```bash
# Health check runs daily at 6 AM UTC
# Or trigger manually:
gh workflow run sentry-error-tracking.yml -R KooshaPari/AgilePlus

# Check results:
gh workflow view sentry-error-tracking.yml -R KooshaPari/AgilePlus
```

**Verification**:
- [ ] Health check workflow completes successfully
- [ ] "Verify Sentry DSN is configured" step passes
- [ ] "Run Sentry initialization tests" step passes
- [ ] "Generate Sentry dashboard link" displays URL
- [ ] Workflow summary shows Sentry dashboard link

## Rollout Status

### Phase: Tier 1 Complete

- [x] SDK configurations deployed
- [x] GitHub Actions workflows deployed
- [x] Documentation complete
- [x] Local verification ready

### Phase: Manual Verification (In Progress)

- [ ] DSN secrets configured
- [ ] Local tests passing
- [ ] Workflow triggered and passing
- [ ] Dashboard accessible
- [ ] GitHub integration enabled
- [ ] Alert rules created
- [ ] End-to-end test successful
- [ ] Health check passing

### Phase: Production Ready (Pending)

- [ ] All manual verifications complete
- [ ] Team trained on dashboards
- [ ] Escalation procedures documented
- [ ] Monitoring alerts configured
- [ ] Tier 2 repos scheduled

## Sign-Off

**Component**: Sentry Error Tracking - Tier 1
**Deployment Date**: 2026-03-31
**Verification Date**: [To be filled]
**Verified By**: [Team member name]
**Status**: Ready for Manual Verification

---

**Next Steps After Verification**:
1. Mark all checkboxes above
2. Update "Verification Date" and "Verified By"
3. Create PR with verification summary
4. Schedule Tier 2 deployment for next week

**Reference Documents**:
- Full Details: `/docs/reports/SENTRY_TIER1_FINALIZATION.md`
- Quick Start: `/docs/guides/SENTRY_QUICK_START.md`
- Workflow File: `.github/workflows/sentry-error-tracking.yml` (all 3 repos)
