# GitHub Workflow Deployment Guide

Complete guide to deploying Snyk GitHub Actions workflows for automated security scanning.

**Time Required:** 5-10 minutes
**Complexity:** Intermediate
**Prerequisites:**
- Snyk token acquired and verified
- Admin access to GitHub organization
- GitHub organization: KooshaPari

---

## Overview

This guide covers:
1. Adding SNYK_TOKEN to GitHub organization secrets
2. Deploying snyk-scan.yml workflow to Tier 1 repos
3. Verifying workflow execution
4. Monitoring automated nightly scans
5. Understanding workflow outputs and alerts

---

## Part 1: GitHub Organization Secrets Setup

### 1.1: Prerequisites

Verify you have:
- Admin access to KooshaPari GitHub organization
- Snyk API token (from SNYK_TOKEN_ACQUISITION_GUIDE.md)
- GitHub CLI (`gh`) installed locally

```bash
# Verify GitHub CLI
gh --version
# Output: gh version X.X.X (YYYY-MM-DD)

# Verify authentication
gh auth status
# Output: Logged in to github.com as KooshaPari (...)
```

### 1.2: Add SNYK_TOKEN to Organization Secrets

#### Option 1: Using GitHub CLI (Recommended)

```bash
# Set the secret
gh secret set SNYK_TOKEN --org KooshaPari
# When prompted, paste your token and press Enter

# Verify it was set
gh secret list --org KooshaPari | grep SNYK_TOKEN
# Output: SNYK_TOKEN    Updated 2026-03-30
```

#### Option 2: Web Interface

If you prefer the GitHub UI:

1. Go to: https://github.com/organizations/KooshaPari/settings/secrets/actions
2. Click **"New organization secret"**
3. Name: `SNYK_TOKEN`
4. Value: Paste your Snyk token
5. Repositories: Select "All repositories" (or select specific repos)
6. Click **"Add secret"**

**Important:** Select "All repositories" so the secret is available to all repos in the organization.

### 1.3: Verify Secret is Accessible

The secret is now available to all repositories. You don't need to verify at org level; we'll test it when running the workflow.

---

## Part 2: Workflow File Preparation

### 2.1: Obtain Workflow File

The workflow file should already exist in the repository:

```bash
ls -la /Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/snyk-scan.yml
```

If not, create it:

```bash
mkdir -p /Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows
```

### 2.2: Workflow File Content

The workflow file (`.github/workflows/snyk-scan.yml`) should contain:

```yaml
name: Snyk Security Scan

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    # Run at 2 AM UTC every day (9 PM EST)
    - cron: '0 2 * * *'

permissions:
  contents: read
  security-events: write

jobs:
  snyk-scan:
    runs-on: ubuntu-latest
    name: Snyk Security Scan

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run Snyk scan
        uses: snyk/actions/setup@master
        with:
          snyk-version: latest
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

      - name: Snyk test
        run: snyk test --json-file-output=snyk-report.json
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        continue-on-error: true

      - name: Upload scan results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: snyk-report.sarif
        continue-on-error: true

      - name: Publish results to PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('snyk-report.json', 'utf8'));

            let summary = `## Snyk Security Scan Results\n\n`;
            summary += `- **Vulnerabilities Found:** ${report.summary}\n`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: summary
            });
        continue-on-error: true
```

### 2.3: Workflow File Explanation

| Section | Purpose |
|---------|---------|
| `name` | Workflow display name in GitHub Actions UI |
| `on.push` | Trigger on push to main |
| `on.pull_request` | Trigger on PRs to main |
| `on.schedule` | Trigger daily at 2 AM UTC |
| `permissions` | Allow workflow to write security events |
| `jobs` | Define job (snyk-scan) |
| `steps` | Individual steps: checkout, run snyk, upload results |
| `secrets.SNYK_TOKEN` | Reference to organization secret (no need to set per-repo) |

---

## Part 3: Deploy Workflow to Tier 1 Repositories

### 3.1: Tier 1 Repositories

Tier 1 repos require immediate workflow deployment:

1. **AgilePlus**
   - Path: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
   - Remote: `https://github.com/KooshaPari/AgilePlus`

2. **heliosCLI**
   - Path: `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI`
   - Remote: `https://github.com/KooshaPari/heliosCLI`

3. **phenotype-infrakit**
   - Path: `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit`
   - Remote: `https://github.com/KooshaPari/phenotype-infrakit`

### 3.2: Deploy to AgilePlus

```bash
# Navigate to repo
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# Create workflow directory if needed
mkdir -p .github/workflows

# Copy or create workflow file
cat > .github/workflows/snyk-scan.yml << 'EOF'
name: Snyk Security Scan

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'

permissions:
  contents: read
  security-events: write

jobs:
  snyk-scan:
    runs-on: ubuntu-latest
    name: Snyk Security Scan

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run Snyk scan
        uses: snyk/actions/setup@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

      - name: Snyk test
        run: snyk test --json-file-output=snyk-report.json
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        continue-on-error: true

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: snyk-report
          path: snyk-report.json
        continue-on-error: true
EOF

# Verify file created
ls -la .github/workflows/snyk-scan.yml

# Stage and commit
git add .github/workflows/snyk-scan.yml
git commit -m "ci: add Snyk security scan workflow

Adds automated Snyk scanning on:
- Push to main
- Pull requests to main
- Daily schedule (2 AM UTC)

Uses SNYK_TOKEN organization secret (pre-configured)"

# Push to GitHub
git push origin main
```

### 3.3: Deploy to heliosCLI

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI

mkdir -p .github/workflows

# Create same workflow file
cat > .github/workflows/snyk-scan.yml << 'EOF'
name: Snyk Security Scan

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'

permissions:
  contents: read
  security-events: write

jobs:
  snyk-scan:
    runs-on: ubuntu-latest
    name: Snyk Security Scan

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run Snyk scan
        uses: snyk/actions/setup@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

      - name: Snyk test
        run: snyk test --json-file-output=snyk-report.json
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        continue-on-error: true

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: snyk-report
          path: snyk-report.json
        continue-on-error: true
EOF

git add .github/workflows/snyk-scan.yml
git commit -m "ci: add Snyk security scan workflow"
git push origin main
```

### 3.4: Deploy to phenotype-infrakit

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

mkdir -p .github/workflows

cat > .github/workflows/snyk-scan.yml << 'EOF'
name: Snyk Security Scan

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'

permissions:
  contents: read
  security-events: write

jobs:
  snyk-scan:
    runs-on: ubuntu-latest
    name: Snyk Security Scan

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run Snyk scan
        uses: snyk/actions/setup@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

      - name: Snyk test
        run: snyk test --json-file-output=snyk-report.json
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        continue-on-error: true

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: snyk-report
          path: snyk-report.json
        continue-on-error: true
EOF

git add .github/workflows/snyk-scan.yml
git commit -m "ci: add Snyk security scan workflow"
git push origin main
```

---

## Part 4: Verify Workflow Deployment

### 4.1: Check GitHub Actions Tab

For each Tier 1 repo:

1. Go to: https://github.com/KooshaPari/AgilePlus/actions
2. You should see **"Snyk Security Scan"** workflow listed
3. Refresh page if not visible (may take a few seconds)

### 4.2: Manual Trigger (Optional Test)

To test immediately without waiting for push/schedule:

```bash
# Navigate to repo
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# Create a minor change to trigger workflow
echo "# Test trigger" >> README.md
git add README.md
git commit -m "test: trigger Snyk workflow"
git push origin main
```

Then check Actions tab — workflow should start within 30 seconds.

### 4.3: Monitor First Run

After first workflow run:

1. Go to: https://github.com/KooshaPari/AgilePlus/actions
2. Click on the latest **"Snyk Security Scan"** run
3. Expand **"Snyk test"** step
4. Look for output:
   ```
   ✓ Tested X dependencies
   ✓ No known vulnerabilities found
   ```
   or
   ```
   ✗ X vulnerabilities found
   - 1 critical
   - 2 high
   ```

### 4.4: Expected Workflow Statuses

| Status | Meaning |
|--------|---------|
| ✅ **Pass** | No vulnerabilities found |
| ⚠️ **Pass (warnings)** | Vulnerabilities found but suppressed by .snyk policy |
| ❌ **Fail** | Critical/high vulnerabilities found (not suppressed) |
| ⏳ **Running** | Scan in progress |
| ⚫ **Skipped** | Workflow skipped (rare) |

**Note:** The workflow is set to `continue-on-error: true`, so it won't block merges even if vulnerabilities are found. This allows review before deciding on remediation.

---

## Part 5: Schedule Verification

### 5.1: Check Scheduled Scans

The workflow is scheduled to run daily at 2 AM UTC (9 PM EST):

```
on:
  schedule:
    - cron: '0 2 * * *'
```

**First scheduled run:** Tomorrow at 2 AM UTC

To verify scheduled jobs are configured:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
git log --oneline -5 .github/workflows/snyk-scan.yml
```

### 5.2: Viewing Scheduled Runs

In GitHub Actions UI:
1. Go to: https://github.com/KooshaPari/AgilePlus/actions
2. Filter by: "Snyk Security Scan"
3. Look for runs with source: **"Scheduled"**

You'll see these starting tomorrow.

---

## Part 6: Monitoring & Alerts

### 6.1: Setting Up Notifications

For email alerts when security issues are found:

1. Go to: https://github.com/settings/notifications
2. Enable: **"Actions"** → **"Notifications on failure"**
3. You'll get email when workflow fails (critical vulnerabilities found)

### 6.2: Creating Branch Protection Rules

To require Snyk scans pass before merging:

```bash
# For AgilePlus repo
gh api repos/KooshaPari/AgilePlus/branches/main/protection/required_status_checks \
  -X POST \
  -f contexts='["Snyk Security Scan"]'
```

**Note:** This makes the Snyk workflow a required check. Code won't merge until scan passes.

**Optional:** You can skip this if you want scans to be informational only (not blocking).

### 6.3: Monitoring Dashboard

Create a simple monitoring script:

```bash
cat > ~/bin/check-snyk-status.sh << 'EOF'
#!/bin/bash

repos=("AgilePlus" "heliosCLI" "phenotype-infrakit")

for repo in "${repos[@]}"; do
  echo "=== $repo ==="
  gh run list -R KooshaPari/$repo -w "Snyk Security Scan" --limit 1 --json status,updatedAt,conclusion
done
EOF

chmod +x ~/bin/check-snyk-status.sh

# Run it
~/bin/check-snyk-status.sh
```

---

## Part 7: Understanding Workflow Outputs

### 7.1: Successful Scan

When the workflow completes successfully:

```
✓ Step: Checkout code
✓ Step: Run Snyk scan
✓ Step: Snyk test
  Output:
    Testing /github/workspace...
    ✓ Tested 156 dependencies
    ✓ No known vulnerabilities found

✓ Step: Upload results
✓ Workflow completed successfully
```

### 7.2: Scan with Vulnerabilities

When vulnerabilities are found:

```
✓ Step: Checkout code
✓ Step: Run Snyk scan
✓ Step: Snyk test
  Output:
    Testing /github/workspace...
    ✗ Tested 156 dependencies
    ✗ 5 vulnerabilities found
      - 1 critical
      - 2 high
      - 2 medium

    (continued on error)
✓ Step: Upload results
⚠️ Workflow completed with warnings
```

**The workflow continues** because of `continue-on-error: true`. This allows you to review findings without blocking merges.

### 7.3: Artifact Access

Scan reports are saved as artifacts:

1. Go to: https://github.com/KooshaPari/AgilePlus/actions/runs/XXXXX
2. Scroll to: **"Artifacts"** section
3. Download: `snyk-report.zip`
4. Contains: `snyk-report.json`

You can download and review the full report locally.

---

## Part 8: Troubleshooting

### Issue: "SNYK_TOKEN secret not found"

**Cause:** Secret not set in GitHub organization

**Solution:**
```bash
# Set secret again
gh secret set SNYK_TOKEN --org KooshaPari

# Verify
gh secret list --org KooshaPari
```

### Issue: Workflow Not Triggering

**Cause:** Workflow file not properly committed or workflow syntax error

**Solution:**
```bash
# Verify workflow file exists and is valid
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
ls -la .github/workflows/snyk-scan.yml

# Validate YAML syntax
cat .github/workflows/snyk-scan.yml | grep -E "^\s*(name|on|jobs):"

# Re-push if needed
git push origin main --force
```

### Issue: "snyk: command not found" in Workflow

**Cause:** Snyk CLI not installed in GitHub runner

**Solution:** Update workflow to use Snyk's official action:

```yaml
- name: Run Snyk scan
  uses: snyk/actions/setup@master  # ← This installs snyk
  with:
    snyk-version: latest
```

### Issue: Workflow Runs Slowly

**Cause:** Large dependency tree or network latency

**Solution:**
- First run is slowest (caching not yet setup)
- Subsequent runs use GitHub Actions cache
- Typical time: 2-5 minutes
- Wait 2-3 runs before investigating

### Issue: "Failed to download Snyk CLI"

**Cause:** Network issue or rate limiting

**Solution:**
```yaml
- name: Run Snyk scan
  uses: snyk/actions/setup@master
  with:
    snyk-version: latest
  retries: 3  # Retry 3 times
```

---

## Part 9: Next Steps

### Immediate
- [ ] Verify SNYK_TOKEN set in GitHub organization secrets
- [ ] Deploy workflow to all 3 Tier 1 repos
- [ ] Check Actions tab for successful first run
- [ ] Review scan results in artifacts

### Short-Term (This Week)
- [ ] Set up email notifications
- [ ] Monitor scheduled nightly scans
- [ ] Create AgilePlus work items for critical findings
- [ ] Review and adjust .snyk policy files based on findings

### Medium-Term (This Month)
- [ ] Deploy workflow to remaining repos (Tier 2-3)
- [ ] Create dashboard for organization-wide vulnerability tracking
- [ ] Establish SLA for critical/high vulnerability remediation
- [ ] Integrate Snyk reports into release process

### Long-Term (Ongoing)
- [ ] Quarterly review of vulnerability trends
- [ ] Update Snyk CLI and actions to latest versions
- [ ] Expand scanning to additional repository types
- [ ] Consider Snyk PR review app for automated PR checks

---

## Reference

- **GitHub Actions Secrets:** https://docs.github.com/en/actions/security-guides/encrypted-secrets
- **Snyk GitHub Actions:** https://github.com/snyk/actions
- **Snyk GitHub Integration:** https://docs.snyk.io/integrations/git-repository-scm-integrations/github
- **GitHub Actions Cron Syntax:** https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#schedule

---

**Last Updated:** 2026-03-30
**Status:** Ready for deployment checklist
