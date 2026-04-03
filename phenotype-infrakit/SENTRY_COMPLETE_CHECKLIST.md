# Sentry Tier 1 Deployment - Complete Checklist

**Status**: ✅ Deployment Complete (2026-03-31)
**Next Action**: Configure Secrets & Verify

## Deployment Completion Summary

### What Was Delivered

#### Part 1: SDK Enhancement (✅)
- [x] AgilePlus SDK configured with env DSN support
- [x] heliosCLI SDK configured with env DSN support
- [x] phenotype-infrakit SDK configured with env DSN support
- [x] All SDKs have unit tests (FR-SENTRY-001, FR-SENTRY-002)
- [x] Fallback to test mode if DSN not provided
- [x] Manual error/message capture utilities available

#### Part 2: GitHub Actions Workflows (✅)
- [x] AgilePlus: `.github/workflows/sentry-error-tracking.yml` created
- [x] heliosCLI: `.github/workflows/sentry-error-tracking.yml` created
- [x] phenotype-infrakit: `.github/workflows/sentry-error-tracking.yml` created
- [x] All workflows trigger on push/PR/schedule
- [x] All workflows include health check jobs
- [x] All workflows include integration setup checklist
- [x] All workflows include failure notifications

#### Part 3: GitHub Integration (✅ Ready)
- [x] Documentation for GitHub integration setup provided
- [x] Integration instructions included in workflows
- [x] Alert rule templates provided in docs
- [x] Label configuration examples included

#### Part 4: Documentation (✅)
- [x] Quick Start Guide: `/docs/guides/SENTRY_QUICK_START.md` (6.8 KB)
- [x] Finalization Report: `/docs/reports/SENTRY_TIER1_FINALIZATION.md` (16 KB)
- [x] Verification Checklist: `/docs/checklists/SENTRY_DEPLOYMENT_VERIFICATION.md` (8.8 KB)
- [x] Deployment Summary: `SENTRY_DEPLOYMENT_SUMMARY.md` (8.6 KB)
- [x] Integration Index: `SENTRY_INTEGRATION_INDEX.md` (comprehensive)

#### Part 5: Verification Tests (✅ Ready)
- [x] SDK unit tests prepared (FR-SENTRY-001, FR-SENTRY-002)
- [x] Local test procedure documented
- [x] Manual error trigger example provided
- [x] Expected latency baseline documented (15-70 seconds)

## Your Action Items (In Order)

### Step 1: Get DSNs from Sentry (5 minutes)

```bash
# Go to: https://sentry.io/settings/phenotype/projects/

# Find each project:
# - agileplus → Copy "DSN (Public)"
# - helioscli → Copy "DSN (Public)"
# - phenotype-infrakit → Copy "DSN (Public)"

# Each DSN looks like:
# https://key@domain.ingest.sentry.io/12345
```

**✅ Completion**: Save DSNs in secure location (don't commit)

### Step 2: Configure GitHub Secrets (5 minutes)

```bash
# For AgilePlus
gh secret set SENTRY_DSN_AGILEPLUS --body '<paste-dsn-here>'

# For heliosCLI
gh secret set SENTRY_DSN_HELIOSCLI --body '<paste-dsn-here>'

# For phenotype-infrakit
gh secret set SENTRY_DSN_PHENOTYPE_INFRAKIT --body '<paste-dsn-here>'

# Verify all were created:
gh secret list | grep SENTRY
```

**✅ Completion**: All 3 secrets created & visible

### Step 3: Enable GitHub Integration (3 minutes)

```
1. Go to: https://sentry.io/settings/phenotype/integrations/github/
2. Click "Install" if not done
3. Authorize KooshaPari GitHub organization
4. Grant necessary permissions
5. Verify status shows "Installed"
```

**✅ Completion**: GitHub org is authorized in Sentry

### Step 4: Create Alert Rules (5 minutes per project)

**For each project** (agileplus, helioscli, phenotype-infrakit):

```
1. Go to: https://sentry.io/organizations/phenotype/alerts/rules/
2. Click "Create Alert Rule"
3. Set:
   - Environment: production
   - Condition: (any) error event
   - Action: Create GitHub issue
   - Project: [Select project]
   - Labels: sentry-critical,error-tracking
4. Click "Save Rule"
```

**✅ Completion**: 3 alert rules created (one per project)

### Step 5: Run Verification Tests (30 minutes)

Follow: `/docs/checklists/SENTRY_DEPLOYMENT_VERIFICATION.md`

Checklist items:
- [ ] Local SDK tests pass with DSN
- [ ] Workflows trigger successfully
- [ ] Dashboard shows projects
- [ ] GitHub integration shows as authorized
- [ ] Trigger test error and verify capture
- [ ] Verify GitHub issue auto-created
- [ ] Check end-to-end latency (< 60 sec)

**✅ Completion**: All verification items checked

## Key Files & Links

### Documentation
| File | Purpose |
|------|---------|
| `/docs/guides/SENTRY_QUICK_START.md` | 5-min developer guide |
| `/docs/reports/SENTRY_TIER1_FINALIZATION.md` | Complete technical docs |
| `/docs/checklists/SENTRY_DEPLOYMENT_VERIFICATION.md` | Step-by-step verification |
| `SENTRY_DEPLOYMENT_SUMMARY.md` | High-level overview |
| `SENTRY_INTEGRATION_INDEX.md` | Navigation & reference |
| This file | Action checklist |

### Dashboard Links
| Project | Dashboard |
|---------|-----------|
| Sentry Org | https://sentry.io/organizations/phenotype/ |
| AgilePlus | https://sentry.io/organizations/phenotype/issues/?project=agileplus |
| heliosCLI | https://sentry.io/organizations/phenotype/issues/?project=helioscli |
| phenotype-infrakit | https://sentry.io/organizations/phenotype/issues/?project=phenotype-infrakit |
| GitHub Integration | https://sentry.io/settings/phenotype/integrations/github/ |
| Alert Rules | https://sentry.io/organizations/phenotype/alerts/rules/ |

### Code Files
| File | Purpose |
|------|---------|
| `AgilePlus/.github/workflows/sentry-error-tracking.yml` | Health check workflow |
| `heliosCLI/.github/workflows/sentry-error-tracking.yml` | Health check workflow |
| `phenotype-infrakit/.github/workflows/sentry-error-tracking.yml` | Health check workflow |
| `AgilePlus/libs/logger/src/sentry_config.rs` | SDK initialization |
| `heliosCLI/crates/harness_utils/src/sentry_config.rs` | SDK initialization |
| `phenotype-infrakit/crates/phenotype-sentry-config/src/lib.rs` | SDK initialization |

## GitHub Secrets Required

After Step 2, you should have these 3 secrets created:

```
✓ SENTRY_DSN_AGILEPLUS         (from agileplus project in Sentry)
✓ SENTRY_DSN_HELIOSCLI         (from helioscli project in Sentry)
✓ SENTRY_DSN_PHENOTYPE_INFRAKIT (from phenotype-infrakit project in Sentry)
```

Verify with:
```bash
gh secret list | grep SENTRY
```

## What Happens After Configuration

1. **Daily Health Checks** (6 AM UTC + on push):
   - Workflows run automatically
   - Check DSN is configured
   - Generate dashboard links
   - Notify if anything fails

2. **Error Capture** (Automatically):
   - Errors in production app → Sentry
   - Latency: 5-30 seconds to dashboard
   - Latency: 10-60 seconds to GitHub issue

3. **Issue Tracking** (Automated):
   - GitHub issue auto-created for each error
   - Links back to Sentry event
   - Labels applied (sentry-critical, error-tracking)
   - Can be assigned and tracked

4. **Monitoring** (Daily):
   - Check dashboard at: https://sentry.io/organizations/phenotype/
   - Filter by project, environment, severity
   - Investigate & respond to high-priority errors

## Success Criteria Checklist

### Pre-Deployment (✅ Done)
- [x] SDK configurations reviewed & verified
- [x] Workflows created & tested locally
- [x] Documentation complete & reviewed
- [x] Architecture validated
- [x] No breaking changes

### Post-Configuration (⏳ Your Turn)
- [ ] All 3 DSN secrets configured
- [ ] GitHub integration authorized
- [ ] Alert rules created (3 rules)
- [ ] Verification tests passing
- [ ] Team trained on dashboards
- [ ] Escalation procedures communicated

### Sign-Off
- [ ] Security team approves
- [ ] DevOps confirms production-ready
- [ ] Lead completes verification checklist
- [ ] Date: _______________

## Rollback Plan (If Needed)

If you need to disable error tracking:

```bash
# Option 1: Remove DSN secrets (errors won't be sent)
gh secret delete SENTRY_DSN_AGILEPLUS
gh secret delete SENTRY_DSN_HELIOSCLI
gh secret delete SENTRY_DSN_PHENOTYPE_INFRAKIT

# Option 2: Set dummy DSN (test mode only)
gh secret set SENTRY_DSN_AGILEPLUS --body 'https://test@test.ingest.sentry.io/0'

# Option 3: Disable alert rules (keep logging)
# Go to: https://sentry.io/organizations/phenotype/alerts/rules/
# Click each rule → Toggle "Enable" to OFF
```

## Timeline

| When | What | Owner |
|------|------|-------|
| 2026-03-31 | Deployment complete | Team |
| This week | Configure secrets | You |
| This week | Enable GitHub integration | Sentry admin |
| This week | Create alert rules | You |
| This week | Run verification | QA |
| ~2026-04-07 | Production ready | Team |
| 2026-04-14+ | Tier 2 expansion | Team |

## Quick Links

- **Start here**: This file
- **Quick setup**: `/docs/guides/SENTRY_QUICK_START.md`
- **Full guide**: `/docs/reports/SENTRY_TIER1_FINALIZATION.md`
- **Verification**: `/docs/checklists/SENTRY_DEPLOYMENT_VERIFICATION.md`
- **Dashboard**: https://sentry.io/organizations/phenotype/
- **Secrets setup**: https://github.com/KooshaPari/AgilePlus/settings/secrets/actions

## Troubleshooting

| Problem | Checklist |
|---------|-----------|
| Workflows failing | Secret set? DSN valid? Check workflow logs |
| Errors not in Sentry | SDK initialized? DSN correct? Check SDK logs |
| GitHub issues not creating | Alert rule created? GitHub authorized? Check rule status |
| Dashboard won't load | Org access? Projects created? Check Sentry status |
| Latency too high | Check Sentry API status, network issues |

See `/docs/guides/SENTRY_QUICK_START.md` for detailed troubleshooting.

---

## Next Actions

**Immediate (Today/Tomorrow)**:
1. Get DSNs from Sentry (5 min)
2. Configure GitHub secrets (5 min)
3. Enable GitHub integration (3 min)
4. Create alert rules (15 min)
5. Run verification (30 min)

**This Week**:
- [ ] Train team on dashboard navigation
- [ ] Monitor first errors in Sentry
- [ ] Adjust alert rules if needed
- [ ] Document any issues

**Next Week**:
- [ ] Complete sign-off
- [ ] Plan Tier 2 expansion
- [ ] Configure Slack integration (optional)

---

**Status**: ✅ Deployment Complete - Configuration Needed
**Owner**: DevOps Team
**Deadline**: 2026-04-07 (for production-ready status)
**Questions**: See documentation above or ask team lead
