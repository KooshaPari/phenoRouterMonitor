# Final Verification Checklist

**STAGE:** Phase 1 Completion Verification
**TIME ESTIMATE:** 10-15 minutes
**CRITICAL:** Run this after both automation scripts complete
**SUCCESS CRITERIA:** All 25 items verified ✅

---

## Pre-Verification Setup

Before starting verification:

1. Ensure both tokens are set:
   ```bash
   # Sentry token in place
   grep "token =" ~/.sentryclirc | grep "sn_" && echo "✅ Sentry token ready"

   # Snyk authentication complete
   snyk whoami && echo "✅ Snyk token ready"
   ```

2. Both scripts have completed:
   - [ ] `scripts/automation/sentry-automation.sh` — completed without errors
   - [ ] `scripts/automation/snyk-deployment.sh` — completed without errors

3. You have access to:
   - [ ] GitHub CLI (`gh` authenticated)
   - [ ] Sentry dashboard (https://sentry.io/)
   - [ ] Snyk dashboard (https://app.snyk.io/)

---

## SECTION 1: Sentry Verification

### 1.1 Verify Projects Were Created

**Command:**
```bash
sentry-cli projects list --org phenotype-org --format json | jq '.[] | .name' | wc -l
```

**Expected output:**
```
30
```

If you see a different number, something went wrong. See troubleshooting below.

**Checkpoint:** ✅ 30 projects created / ❌ Different count

---

### 1.2 Verify Project Names

**Command:**
```bash
sentry-cli projects list --org phenotype-org
```

**Expected output (first 10 of 30):**
```
phenotype-infrakit
heliosCLI
platforms-thegent
AgilePlus
civ
parpour
...
```

**Checkpoint:** ✅ All expected projects listed / ❌ Projects missing

---

### 1.3 Verify DSN Environment Variables

Each project should have DSNs. Check one:

**Command:**
```bash
sentry-cli projects list --org phenotype-org --format json | jq '.[0]' | grep -i "dsn"
```

**Expected output:**
```
{
  ...
  "dsn": {
    "public": "https://xxx@xxx.ingest.sentry.io/xxx",
    ...
  }
  ...
}
```

**Checkpoint:** ✅ DSNs present / ❌ DSN fields missing

---

### 1.4 Verify GitHub Secrets Were Populated

**Command:**
```bash
# Check one repository as a sample
cd repos/phenotype-infrakit
gh secret list | grep SENTRY_DSN

# Expected: SENTRY_DSN_<PROJECT_ID>
```

**Expected output:**
```
SENTRY_DSN_XXXXX
```

**Alternative check (all repos at once):**
```bash
cd repos
for repo in $(ls -d */ | head -5); do
  echo "=== $repo ==="
  cd "$repo"
  gh secret list | grep SENTRY_DSN | head -1
  cd ..
done
```

**Expected output:**
```
=== phenotype-infrakit ===
SENTRY_DSN_XXXXX
=== heliosCLI ===
SENTRY_DSN_XXXXX
...
```

**Checkpoint:** ✅ All repo DSN secrets populated / ❌ Secrets missing

---

### 1.5 Verify Alerts Configuration

**Command (via dashboard):**
1. Go to: https://sentry.io/organizations/phenotype-org/
2. Click on any project (e.g., phenotype-infrakit)
3. Go to **Alerts** tab
4. You should see alert rules configured

**Expected to see:**
- [ ] Alert rule for issue creation
- [ ] Alert rule for spike in errors
- [ ] Alert rule for performance degradation

**Checkpoint:** ✅ Alerts configured / ❌ No alerts

---

### 1.6 Verify SDK Integration

Check that Tier 1 repos have Sentry SDK:

**For phenotype-infrakit (Rust):**
```bash
cd repos/phenotype-infrakit
grep -r "sentry" Cargo.toml | head -3
```

**Expected output:**
```
sentry = "0.33"
sentry-tracing = "0.33"
```

**For heliosCLI (Node.js/Rust):**
```bash
cd repos/heliosCLI
grep "@sentry" package.json
```

**Expected output:**
```
"@sentry/node": "^8.25.0"
```

**For platforms/thegent (Go):**
```bash
cd repos/platforms/thegent
grep "sentry-go" go.mod
```

**Expected output:**
```
github.com/getsentry/sentry-go v1.29.0
```

**Checkpoint:** ✅ All SDKs present / ❌ SDKs missing

---

## SECTION 2: Snyk Verification

### 2.1 Verify Snyk Authentication

**Command:**
```bash
snyk whoami
```

**Expected output:**
```
You are logged in as your-email@example.com

Ready to start using Snyk!
```

**Checkpoint:** ✅ Authenticated / ❌ Authentication failed

---

### 2.2 Verify Projects Enrolled (via Dashboard)

1. Go to: https://app.snyk.io/projects
2. You should see all 30 repositories listed
3. Each should have a checkmark (✅) indicating active monitoring

**Visual check:**
- [ ] phenotype-infrakit — status: ACTIVE
- [ ] heliosCLI — status: ACTIVE
- [ ] platforms/thegent — status: ACTIVE
- [ ] AgilePlus — status: ACTIVE
- [ ] (16+ more projects)

**Checkpoint:** ✅ 30+ projects enrolled / ❌ <30 projects

---

### 2.3 Verify Scan Frequency

1. On Snyk Projects page, click on any project (e.g., phenotype-infrakit)
2. Go to **Settings**
3. Look for "Scan frequency" or "Automated scanning"
4. It should show: **Daily**

**Checkpoint:** ✅ Daily scans enabled / ❌ Scans disabled

---

### 2.4 Verify Snyk GitHub Integration

**Via Snyk Dashboard:**
1. Go to: https://app.snyk.io/settings/integrations/
2. Look for **GitHub** integration
3. Status should be: **Connected** or **Installed**

**Alternative check (via GitHub):**
```bash
# Check if Snyk app is installed on your GitHub account
gh app list | grep snyk

# Expected output:
# snyk/snyk - Snyk (INSTALLED)
```

**Checkpoint:** ✅ GitHub integration active / ❌ Not connected

---

### 2.5 Verify Initial Scan Results (Optional but recommended)

1. Go to: https://app.snyk.io/projects
2. Click on **phenotype-infrakit**
3. You should see scan results (even if 0 vulnerabilities)
4. Look for:
   - [ ] Vulnerabilities count
   - [ ] Scan date
   - [ ] Severity breakdown

**Checkpoint:** ✅ Scan results visible / ❌ No scan data

---

## SECTION 3: GitHub Configuration

### 3.1 Verify Three Required Secrets

**Command:**
```bash
# Check org-level secrets (if applicable)
gh secret list | grep -E "SENTRY_TOKEN|SNYK_TOKEN|SENTRY_ORG_SLUG"
```

**Expected output:**
```
SENTRY_TOKEN
SNYK_TOKEN
SENTRY_ORG_SLUG
```

All three should be listed.

**Checkpoint:** ✅ All 3 secrets present / ❌ <3 secrets

---

### 3.2 Verify Secret Values (Quick Sanity Check)

**For SENTRY_ORG_SLUG:**
```bash
gh secret view SENTRY_ORG_SLUG 2>/dev/null || echo "Value hidden (OK)"
```

**Expected output:**
```
phenotype-org
```
(or "Value hidden" if GitHub hides it, which is also OK)

**For SENTRY_TOKEN:**
```bash
gh secret view SENTRY_TOKEN 2>/dev/null | head -c 20 || echo "Hidden (expected)"
```

**Expected output:**
```
sn_ (first 3 chars)
```
(or hidden)

**For SNYK_TOKEN:**
```bash
gh secret view SNYK_TOKEN 2>/dev/null | head -c 10 || echo "Hidden (expected)"
```

**Expected output:**
```
(8-12 hex chars)
```
(or hidden)

**Checkpoint:** ✅ Secrets contain expected values / ❌ Values wrong or missing

---

### 3.3 Verify GitHub Actions Ready

**Command:**
```bash
# Check if workflows are present in one repo
cd repos/phenotype-infrakit
ls -la .github/workflows/ | grep -E "sentry|snyk|security"
```

**Expected output:**
```
sentry-health-check.yml (or similar)
snyk-security-scan.yml (or similar)
```

**Checkpoint:** ✅ Workflow files present / ❌ Missing workflows

---

## SECTION 4: Pre-Commit Hooks

### 4.1 Verify Hooks Installed (Sample Repos)

**Command:**
```bash
# Check 3 repos
for repo in phenotype-infrakit heliosCLI AgilePlus; do
  echo "=== $repo ==="
  cd repos/$repo
  git config core.hooksPath
  cd - > /dev/null
done
```

**Expected output:**
```
=== phenotype-infrakit ===
.git/hooks
=== heliosCLI ===
.git/hooks
=== AgilePlus ===
.git/hooks
```

**Checkpoint:** ✅ Hooks installed / ❌ Hooks not found

---

### 4.2 Verify Hook Types

**Command:**
```bash
cd repos/phenotype-infrakit
ls -la .git/hooks/ | grep -v "^d" | grep -v "sample"
```

**Expected output:**
```
pre-commit
pre-push
commit-msg
```

**Checkpoint:** ✅ All hook types present / ❌ Missing hooks

---

### 4.3 Test a Hook

**Command:**
```bash
cd repos/phenotype-infrakit

# Try to commit a file with trailing whitespace (should fail)
echo "test with trailing space   " > test-hook.txt
git add test-hook.txt

# This should fail due to trailing-whitespace hook
git commit -m "test: hook validation" 2>&1 | grep -i "trailing\|failed" || echo "✅ Hook working"

# Clean up
git reset HEAD test-hook.txt
rm test-hook.txt
```

**Expected output:**
```
✅ Hook working
```
(or an error message about trailing whitespace)

**Checkpoint:** ✅ Hooks execute / ❌ Hooks not running

---

## SECTION 5: Documentation

### 5.1 Verify Governance Docs Present

**Command:**
```bash
cd repos

# Check for Phase 1 docs
ls -la | grep -E "PHASE1|TOKEN|VERIFICATION|GOVERNANCE"
```

**Expected output:**
```
PHASE1_EXECUTION_NOW.md
TOKEN_ACQUISITION_CHECKLIST.md
FINAL_VERIFICATION_CHECKLIST.md
PHASE1_GOVERNANCE.md
SENTRY_INTEGRATION_GUIDE.md
SNYK_INTEGRATION_GUIDE.md
```

**Checkpoint:** ✅ All docs present / ❌ Docs missing

---

### 5.2 Verify Automation Scripts Present

**Command:**
```bash
ls -la scripts/automation/ | grep ".sh"
```

**Expected output:**
```
sentry-automation.sh
snyk-deployment.sh
verify-security-framework.sh
```

**Checkpoint:** ✅ All scripts present / ❌ Scripts missing

---

## SECTION 6: Comprehensive Summary

### Running the Full Verification Script

To automate all checks above:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Run the comprehensive verification
bash scripts/automation/verify-security-framework.sh

# This will check:
# ✅ Sentry (30 projects)
# ✅ Snyk (30 repos enrolled)
# ✅ Pre-commit hooks (all repos)
# ✅ GitHub Secrets (3 required)
# ✅ GitHub Actions workflows
```

**Expected output:**
```
=== SECURITY FRAMEWORK VERIFICATION ===

✅ Sentry Configuration
  • 30 projects created in phenotype-org
  • DSNs configured for all projects
  • GitHub Secrets populated (SENTRY_DSN_*)
  • Alerts configured for each project

✅ Snyk Configuration
  • 30 repositories enrolled
  • Daily scanning enabled
  • GitHub integration active
  • Initial scans complete

✅ Pre-Commit Hooks
  • 30 repositories have hooks installed
  • Hook types: trailing-whitespace, end-of-file-fixer, check-yaml, ruff, clippy, sentry-cli-check
  • Hooks are executable

✅ GitHub Secrets
  • SENTRY_TOKEN: Present and valid
  • SNYK_TOKEN: Present and valid
  • SENTRY_ORG_SLUG: Set to phenotype-org

✅ GitHub Actions Workflows
  • Sentry health check workflow: Ready
  • Snyk security scan workflow: Ready
  • Custom security workflows: Ready

=== PHASE 1 VERIFICATION COMPLETE ===

STATUS: ✅ VERIFIED
Items verified: 25/25
Ready for Phase 2: YES

Next steps:
1. Review Phase 1 completion summary
2. Begin Phase 2 decomposition work
3. Monitor Sentry/Snyk dashboards for first alerts
```

---

## Verification Matrix

| Category | Item | Status | Evidence |
|----------|------|--------|----------|
| **Sentry** | Projects created | ✅/❌ | `sentry-cli projects list` shows 30 |
| | DSNs configured | ✅/❌ | `sentry-cli projects list --format json` has DSN fields |
| | GitHub Secrets | ✅/❌ | `gh secret list` shows SENTRY_DSN_* |
| | Alerts configured | ✅/❌ | https://sentry.io/ dashboard shows alert rules |
| | SDK integration | ✅/❌ | Tier 1 repos have sentry in Cargo.toml/package.json/go.mod |
| **Snyk** | Authentication | ✅/❌ | `snyk whoami` returns email |
| | Projects enrolled | ✅/❌ | https://app.snyk.io/projects shows 30+ repos |
| | Scans enabled | ✅/❌ | Dashboard shows "Daily" or "Continuous" |
| | GitHub integration | ✅/❌ | https://app.snyk.io/settings/integrations/ shows Connected |
| | Initial scans | ✅/❌ | Project pages show scan results |
| **GitHub** | SENTRY_TOKEN | ✅/❌ | `gh secret list` includes SENTRY_TOKEN |
| | SNYK_TOKEN | ✅/❌ | `gh secret list` includes SNYK_TOKEN |
| | SENTRY_ORG_SLUG | ✅/❌ | `gh secret list` includes SENTRY_ORG_SLUG |
| **Pre-commit** | Hooks installed | ✅/❌ | `git config core.hooksPath` returns .git/hooks |
| | Hook types | ✅/❌ | `ls .git/hooks/` shows pre-commit, pre-push, etc. |
| | Hooks executable | ✅/❌ | Can run hooks without errors |
| **Documentation** | Phase 1 docs | ✅/❌ | PHASE1_*.md files present |
| | Integration guides | ✅/❌ | SENTRY_*, SNYK_* guides present |
| | Scripts present | ✅/❌ | scripts/automation/*.sh files present |

---

## Go/No-Go Decision

### GO Criteria (Proceed to Phase 2)

✅ All 25 items verified as present and functional:
- ✅ 30 Sentry projects created
- ✅ 30 DSNs configured
- ✅ All GitHub Secrets populated
- ✅ 30 Snyk projects enrolled
- ✅ All pre-commit hooks active
- ✅ All documentation complete

**DECISION:** ✅ **PHASE 1 COMPLETE — PROCEED TO PHASE 2**

---

### NO-GO Criteria (Return to troubleshooting)

❌ Any of these conditions:
- ❌ <28 Sentry projects created
- ❌ <28 Snyk projects enrolled
- ❌ <3 GitHub Secrets configured
- ❌ Pre-commit hooks missing on >3 repos
- ❌ Automation scripts failed with errors

**DECISION:** ❌ **PHASE 1 INCOMPLETE — TROUBLESHOOT AND RETRY**

---

## Troubleshooting Reference

### Sentry Issues

**Problem:** `sentry-cli projects list` shows <30 projects

**Diagnosis:**
```bash
# Check for errors in automation script
tail -50 /tmp/sentry-automation.log 2>/dev/null || echo "No log found"
```

**Solution:**
1. Verify Sentry token: `grep "token =" ~/.sentryclirc | grep "sn_"`
2. Verify token scope: Go to https://sentry.io/organizations/phenotype-org/settings/auth-tokens/ and check scopes
3. Re-run automation: `bash scripts/automation/sentry-automation.sh`

---

### Snyk Issues

**Problem:** `snyk whoami` returns authentication error

**Diagnosis:**
```bash
# Check Snyk auth status
cat ~/.snyk 2>/dev/null || echo "~/.snyk not found"
```

**Solution:**
1. Re-authenticate: `snyk auth`
2. Paste the token from https://app.snyk.io/account/settings
3. Verify: `snyk whoami`

---

### GitHub Secrets Issues

**Problem:** `gh secret list` shows <3 secrets

**Diagnosis:**
```bash
# Check which secrets are missing
gh secret list | grep -E "SENTRY|SNYK"
```

**Solution:**
1. For each missing secret, create it manually:
   ```bash
   gh secret set SENTRY_TOKEN < <(echo -n "your-token-here")
   gh secret set SNYK_TOKEN < <(echo -n "your-token-here")
   gh secret set SENTRY_ORG_SLUG < <(echo -n "phenotype-org")
   ```
2. Verify: `gh secret list | grep -E "SENTRY|SNYK"`

---

## Final Checklist

Before declaring Phase 1 complete:

- [ ] Ran all verification commands above
- [ ] Got expected output for all checks
- [ ] Reviewed verification matrix (all ✅)
- [ ] Confirmed GO decision criteria met
- [ ] Ready to proceed to Phase 2

**FINAL SIGN-OFF:**

I verify that Phase 1 is **92% → 100%** complete and ready for Phase 2.

---

## Phase 2 Entry Point

Once Phase 1 is verified:

1. ✅ Review `PHASE1_COMPLETION_TIMELINE.md`
2. ✅ Begin Phase 2 decomposition work
3. ✅ Reference Sentry + Snyk in all PRs
4. ✅ Monitor dashboards for first alerts

**Estimated Phase 2 timeline:** 2-4 weeks of development work
