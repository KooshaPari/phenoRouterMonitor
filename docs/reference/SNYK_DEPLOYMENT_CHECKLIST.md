# Snyk Phase 1 Deployment Checklist

**Status**: Ready for Execution
**Date**: 2026-03-30
**Phase**: 1 (Tier 1 Repos)

---

## Pre-Deployment (5 min)

### Infrastructure Verification

- [ ] **Deployment script exists**: `/scripts/snyk-deploy.sh`
  ```bash
  test -f /Users/kooshapari/CodeProjects/Phenotype/repos/scripts/snyk-deploy.sh && echo "✅"
  ```

- [ ] **Policy generator exists**: `/scripts/snyk-policy-generator.sh`
  ```bash
  test -f /Users/kooshapari/CodeProjects/Phenotype/repos/scripts/snyk-policy-generator.sh && echo "✅"
  ```

- [ ] **GitHub workflow exists**: `/.github/workflows/snyk-scan.yml`
  ```bash
  test -f /Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/snyk-scan.yml && echo "✅"
  ```

- [ ] **Documentation exists**:
  - `docs/reference/SNYK_SETUP_GUIDE.md`
  - `docs/reference/SNYK_QUICK_REFERENCE.md`
  - `SNYK_AUTOMATION_READY.md`
  ```bash
  test -f /Users/kooshapari/CodeProjects/Phenotype/repos/docs/reference/SNYK_SETUP_GUIDE.md && echo "✅"
  ```

### Local Environment

- [ ] **Snyk CLI installed**:
  ```bash
  which snyk && snyk --version
  # Output should show: snyk <version>
  ```

  If missing:
  ```bash
  brew install snyk    # macOS
  npm install -g snyk  # npm
  ```

- [ ] **jq installed** (JSON parser):
  ```bash
  which jq && jq --version
  ```

  If missing:
  ```bash
  brew install jq      # macOS
  apt-get install jq   # Linux
  ```

- [ ] **Git configured**:
  ```bash
  git config user.name && git config user.email
  ```

- [ ] **Bash 4.0+** (for script compatibility):
  ```bash
  bash --version  # Should be 4.0 or higher
  ```

---

## Token Acquisition (5 min)

### Step 1: Snyk Account

- [ ] **Account created** (if needed)
  - Go: https://app.snyk.io/auth/register
  - Sign up with GitHub or email
  - Verify email

- [ ] **Account active**
  - Log in: https://app.snyk.io/dashboard
  - Confirm you see the dashboard

### Step 2: API Token Generation

- [ ] **Navigate to settings**
  - Go: https://app.snyk.io/account/settings
  - Scroll to "API Token" section

- [ ] **Generate token**
  - Click "Generate Token" button
  - Copy the displayed token
  - Save to secure location (password manager, etc.)

- [ ] **Token saved securely**
  - Do NOT commit to git
  - Do NOT share in slack/email
  - Keep in password manager or local env var

### Step 3: Token Verification

- [ ] **Test token locally**
  ```bash
  export SNYK_TOKEN="your-token-here"
  snyk auth "$SNYK_TOKEN"
  snyk whoami  # Should show your account
  ```

---

## GitHub Secrets Setup (3 min)

### Option A: Organization Secret (Recommended)

- [ ] **Navigate to org settings**
  - Go: https://github.com/KooshaPari/settings/secrets
  - Or: GitHub > Settings (top-right) > Organization Settings > Secrets

- [ ] **Create new secret**
  - Click "New organization secret"
  - Name: `SNYK_TOKEN` (exact match, case-sensitive)
  - Value: Paste your Snyk token
  - Repository access: "All repositories" (selected)

- [ ] **Save secret**
  - Click "Add secret"
  - Verify it appears in the secrets list

### Option B: Per-Repository Secret (Alternative)

For each Tier 1 repo:

- [ ] **Navigate to repo secrets**
  - Go: https://github.com/KooshaPari/AgilePlus/settings/secrets
  - (Replace AgilePlus with phenotype-infrakit, heliosCLI, etc.)

- [ ] **Create secret for each repo**
  - Click "New repository secret"
  - Name: `SNYK_TOKEN`
  - Value: Your Snyk token
  - Save

- [ ] **Verify all 3 repos have secret**
  - [ ] AgilePlus
  - [ ] phenotype-infrakit
  - [ ] heliosCLI

### Verification

- [ ] **Secret is accessible in workflows**
  - Don't worry, GitHub handles this automatically
  - Just verify secret appears in repo settings

---

## Phase 1 Deployment (5 min)

### Pre-Execution Checklist

- [ ] **Token is available**
  - `echo $SNYK_TOKEN`
  - Should show your token (or *****)

- [ ] **Working directory is correct**
  - `pwd` → `/Users/kooshapari/CodeProjects/Phenotype/repos`
  - If not: `cd /Users/kooshapari/CodeProjects/Phenotype/repos`

- [ ] **Target repos exist**
  ```bash
  test -d AgilePlus && echo "✅ AgilePlus"
  test -d phenotype-infrakit && echo "✅ phenotype-infrakit"
  test -d heliosCLI && echo "✅ heliosCLI"
  ```

### Execute Deployment

- [ ] **Run deployment script**
  ```bash
  export SNYK_TOKEN="your-token-here"
  ./scripts/snyk-deploy.sh "$SNYK_TOKEN"
  ```

  Expected output:
  ```
  ✓ Pre-Flight Checks
  ✓ Snyk CLI installed
  ✓ Snyk authentication successful
  ✓ Scanning Repositories
  ✓ AgilePlus: No vulnerabilities found (or: Found N vulnerabilities)
  ✓ phenotype-infrakit: ...
  ✓ heliosCLI: ...
  ✓ Deployment Summary
  ✓ Summary report written to: .snyk-reports/snyk-deployment-TIMESTAMP.txt
  ```

### Review Reports

- [ ] **Check deployment summary**
  ```bash
  cat .snyk-reports/snyk-deployment-*.txt
  ```

  Look for:
  - Repositories Scanned: 3
  - Successful: 3
  - Failed: 0

- [ ] **Review per-repo reports**
  ```bash
  ls -la .snyk-reports/*/
  # Should have: test-TIMESTAMP.json, test-TIMESTAMP.txt, snyk.sarif
  ```

- [ ] **Check vulnerability counts**
  ```bash
  for dir in .snyk-reports/*/; do
    echo "$(basename $dir):"
    jq '.vulnerabilities | length' "$dir/test-"*.json
  done
  ```

- [ ] **Verify .snyk policy files created**
  ```bash
  ls -la AgilePlus/.snyk phenotype-infrakit/.snyk heliosCLI/.snyk
  # All should exist (-rw-r--r--)
  ```

### Fix Any Issues

- [ ] **If scan failed**
  - Check error messages in `.snyk-reports/`
  - Verify Snyk CLI is current: `snyk upgrade`
  - Verify each repo has dependencies (package.json, Cargo.toml, etc.)
  - Re-run: `./scripts/snyk-deploy.sh "$SNYK_TOKEN" FAILED_REPO`

- [ ] **If .snyk not created**
  - Generate manually: `./scripts/snyk-policy-generator.sh ./REPO`
  - Or: `./scripts/snyk-deploy.sh "$SNYK_TOKEN" REPO` again

---

## Policy Review & Commit (10 min)

### Review Policies

- [ ] **Open .snyk files for review**
  ```bash
  cat AgilePlus/.snyk
  cat phenotype-infrakit/.snyk
  cat heliosCLI/.snyk
  ```

  Should contain:
  - `version: v1.25.0`
  - `ignore: {}` section (empty or with suppressions)
  - `exclude` section with directories
  - `patch: {}` and `fix: {}`

- [ ] **Verify policy format is valid YAML**
  - No syntax errors
  - Proper indentation (2 spaces)
  - All sections present

### Edit if Needed

- [ ] **Add suppressions if required**
  - Open `.snyk` in editor
  - Add vulnerability IDs from test reports
  - Format:
    ```yaml
    ignore:
      'SNYK-VULN-ID':
        - '> package-name':
            reason: 'Why approved'
            expires: '2025-12-31T00:00:00.000Z'
    ```

- [ ] **Update exclusions if needed**
  - Add project-specific directories to skip
  - Example: `/build`, `/dist`, `/target`

### Commit Changes

- [ ] **Stage policy files**
  ```bash
  git add AgilePlus/.snyk phenotype-infrakit/.snyk heliosCLI/.snyk
  ```

- [ ] **Verify staged changes**
  ```bash
  git diff --cached --stat
  # Should show: AgilePlus/.snyk, phenotype-infrakit/.snyk, heliosCLI/.snyk
  ```

- [ ] **Create commit**
  ```bash
  git commit -m "chore(security): add Snyk policy files for Tier 1 repos

- AgilePlus: Auto-generated Snyk policy
- phenotype-infrakit: Auto-generated Snyk policy
- heliosCLI: Auto-generated Snyk policy

These policies define vulnerability suppression rules and scan exclusions."
  ```

- [ ] **Push to remote**
  ```bash
  git push origin main
  # Or your current branch
  ```

- [ ] **Verify on GitHub**
  - Go: https://github.com/KooshaPari/repos/commits/main
  - Confirm commit appears
  - Click commit to verify `.snyk` files changed

---

## GitHub Workflow Deployment (3 min)

### For Each Tier 1 Repo

#### AgilePlus

- [ ] **Copy workflow file**
  ```bash
  mkdir -p AgilePlus/.github/workflows
  cp .github/workflows/snyk-scan.yml AgilePlus/.github/workflows/
  ```

- [ ] **Stage and commit**
  ```bash
  cd AgilePlus
  git add .github/workflows/snyk-scan.yml
  git commit -m "chore(ci): add Snyk security scan workflow

- Runs on: PR open/update, main push, nightly (2 AM UTC)
- Generates SARIF for GitHub Code Scanning
- Auto-creates fix PRs for high/critical issues
- Comments on PRs with vulnerability summary"
  git push origin main
  cd ..
  ```

- [ ] **Verify on GitHub**
  - Go: https://github.com/KooshaPari/AgilePlus/blob/main/.github/workflows/snyk-scan.yml
  - Should see file contents

#### phenotype-infrakit

- [ ] **Copy workflow file**
  ```bash
  mkdir -p phenotype-infrakit/.github/workflows
  cp .github/workflows/snyk-scan.yml phenotype-infrakit/.github/workflows/
  ```

- [ ] **Stage and commit**
  ```bash
  cd phenotype-infrakit
  git add .github/workflows/snyk-scan.yml
  git commit -m "chore(ci): add Snyk security scan workflow"
  git push origin main
  cd ..
  ```

#### heliosCLI

- [ ] **Copy workflow file**
  ```bash
  mkdir -p heliosCLI/.github/workflows
  cp .github/workflows/snyk-scan.yml heliosCLI/.github/workflows/
  ```

- [ ] **Stage and commit**
  ```bash
  cd heliosCLI
  git add .github/workflows/snyk-scan.yml
  git commit -m "chore(ci): add Snyk security scan workflow"
  git push origin main
  cd ..
  ```

---

## Workflow Verification (5 min)

### Check Workflow Activation

- [ ] **Go to Actions tab for each repo**
  - https://github.com/KooshaPari/AgilePlus/actions
  - https://github.com/KooshaPari/phenotype-infrakit/actions
  - https://github.com/KooshaPari/heliosCLI/actions

- [ ] **Verify workflow appears in list**
  - Should show "Snyk Security Scan" in workflow list

- [ ] **Trigger manual workflow run** (optional but recommended)
  - Click "Snyk Security Scan" workflow
  - Click "Run workflow" dropdown
  - Select "Run workflow" button
  - Wait ~2 minutes for scan to complete

- [ ] **Check workflow results**
  - Scroll down to see job status
  - Should show: ✅ snyk-test, snyk-fix (if applicable), etc.
  - If ❌ red: Check logs for error

### First Scheduled Run

- [ ] **Wait for nightly scan** (optional)
  - Scheduled for 2 AM UTC daily
  - Or manually trigger in Actions tab
  - Check results next morning

---

## Code Scanning Dashboard Setup (2 min)

### Enable GitHub Code Scanning

- [ ] **Go to Security settings for each repo**
  - https://github.com/KooshaPari/AgilePlus/settings (Security tab)
  - Check "Code Scanning Alerts" is enabled

- [ ] **View Code Scanning dashboard**
  - Go: https://github.com/KooshaPari/AgilePlus/security/code-scanning
  - Should show vulnerabilities from Snyk SARIF uploads
  - May be empty if no vulnerabilities found

- [ ] **Repeat for all Tier 1 repos**
  - [ ] AgilePlus: https://github.com/KooshaPari/AgilePlus/security/code-scanning
  - [ ] phenotype-infrakit: https://github.com/KooshaPari/phenotype-infrakit/security/code-scanning
  - [ ] heliosCLI: https://github.com/KooshaPari/heliosCLI/security/code-scanning

---

## Post-Deployment Verification (5 min)

### Verify Local Scanning Still Works

- [ ] **Test snyk in one repo**
  ```bash
  cd AgilePlus
  snyk test
  # Should show results or "No vulnerabilities found"
  cd ..
  ```

- [ ] **Test deployment script again**
  ```bash
  ./scripts/snyk-deploy.sh "$SNYK_TOKEN" AgilePlus
  # Should complete successfully
  ```

### Monitor Dashboard

- [ ] **Check Snyk dashboard**
  - Go: https://app.snyk.io/dashboard
  - Should show Tier 1 repos being monitored
  - All 3 repos should appear in project list

- [ ] **Set up Snyk alerts** (optional)
  - Go: https://app.snyk.io/account/settings
  - Configure email notifications for new vulnerabilities

### Verify GitHub Integration

- [ ] **Check GitHub Code Scanning**
  - Go: https://github.com/KooshaPari/repos/security/code-scanning
  - Should show vulnerabilities from Snyk scans
  - Multiple repos should be visible

---

## Sign-Off & Success Criteria

### Phase 1 Complete When

- [x] **Snyk CLI installed and authenticated**
  - `snyk whoami` shows your account

- [x] **Deployment script successful**
  - `.snyk-reports/` has all 3 repos' results
  - No failed scans

- [x] **Policy files committed**
  - `.snyk` exists in all 3 Tier 1 repos
  - Committed to git and visible on GitHub

- [x] **GitHub workflow deployed**
  - `.github/workflows/snyk-scan.yml` in all 3 repos
  - Workflow appears in Actions tab
  - Optionally verified with manual run

- [x] **GitHub secrets configured**
  - `SNYK_TOKEN` in org or repo settings
  - Workflow can access it (verified via execution)

- [x] **Code Scanning active**
  - Security > Code Scanning shows Snyk results
  - All 3 repos reporting vulnerabilities (if any)

### Ready for Phase 2 When

- [ ] Phase 1 deployment stable for 1 day
- [ ] No errors in nightly scans
- [ ] GitHub Actions workflow runs successfully
- [ ] PR comments from Snyk working
- [ ] Team comfortable with process

---

## Rollback Plan (if needed)

If Phase 1 deployment has issues:

```bash
# Remove policy files (keep if repos created them)
git rm AgilePlus/.snyk phenotype-infrakit/.snyk heliosCLI/.snyk
git commit -m "chore: remove Snyk policies (rollback)"

# Or just disable workflows without removing
# Edit .github/workflows/snyk-scan.yml in each repo:
# Add "false" to first line: if: false

# Remove GitHub secret:
# Go: GitHub Org Settings > Secrets
# Click trash icon next to SNYK_TOKEN
```

After rollback, re-diagnose and retry Phase 1.

---

## Timeline

| Task | Time | Status |
|------|------|--------|
| Pre-deployment checks | 5 min | ⏳ Pending |
| Token acquisition | 5 min | ⏳ Pending |
| GitHub secrets | 3 min | ⏳ Pending |
| Phase 1 deployment | 5 min | ⏳ Pending |
| Policy review & commit | 10 min | ⏳ Pending |
| Workflow deployment | 3 min | ⏳ Pending |
| Verification | 5 min | ⏳ Pending |
| **Total** | **~40 min** | ⏳ |

---

## Support & Help

| Issue | Reference |
|-------|-----------|
| Setup guide | `docs/reference/SNYK_SETUP_GUIDE.md` |
| Quick reference | `docs/reference/SNYK_QUICK_REFERENCE.md` |
| Automation status | `SNYK_AUTOMATION_READY.md` |
| Snyk docs | https://docs.snyk.io |
| CLI help | `snyk --help` or `snyk test --help` |

---

## Sign-Off

**Phase 1 Deployment Completed By:**

- Date: _______________
- Name: _______________
- Verified by: _______________

**Phase 1 Status**: ✅ Complete

**Next Step**: Monitor for 1 day, then proceed to Phase 2 (Tier 2 repos)

---

## Notes

Use this space to record any custom settings, team preferences, or deviations from defaults:

```
_________________________________________________________________

_________________________________________________________________

_________________________________________________________________
```
