# Snyk Deployment Checklist

Complete checklist for deploying Snyk security scanning across the Phenotype ecosystem.

**Total Time:** 25-30 minutes
**Complexity:** Intermediate
**Prerequisites:** Admin GitHub access, familiar with command line

---

## Pre-Deployment Phase (5 minutes)

### Environment Verification

- [ ] **Snyk CLI installed**
  ```bash
  snyk --version
  # Expected: Snyk CLI version X.XXX.X
  ```

- [ ] **GitHub CLI installed**
  ```bash
  gh --version
  # Expected: gh version X.X.X
  ```

- [ ] **Git installed**
  ```bash
  git --version
  # Expected: git version X.XX.X
  ```

- [ ] **All Phenotype repos cloned locally**
  ```bash
  ls /Users/kooshapari/CodeProjects/Phenotype/repos | wc -l
  # Expected: 30 directories
  ```

- [ ] **GitHub authentication verified**
  ```bash
  gh auth status
  # Expected: Logged in to github.com as KooshaPari
  ```

### Token Acquisition

- [ ] **Snyk account active**
  - Visited https://app.snyk.io
  - Successfully logged in or created account

- [ ] **Snyk API token generated**
  - Navigated to Settings → Auth Token
  - Clicked "Regenerate token"
  - Token copied to clipboard

- [ ] **Token stored securely (NOT committed to git)**
  - Option 1: `export SNYK_TOKEN="your-token"`
  - Option 2: Temporary `.env` file
  - Option 3: Secure password manager

- [ ] **No token in shell history**
  - Commands started with space (if using export)
  - Checked `~/.bash_history` or `~/.zsh_history`

### Local Authentication

- [ ] **Snyk CLI authenticated**
  ```bash
  snyk auth $SNYK_TOKEN
  # Expected: Successfully authenticated
  ```

- [ ] **Authentication verified**
  ```bash
  snyk test --dry-run
  # Expected: No "Unauthorized" errors
  ```

---

## Token Acquisition Phase (5 minutes)

Follow: **SNYK_TOKEN_ACQUISITION_GUIDE.md**

- [ ] **Completed Part 1:** Snyk Account Access
- [ ] **Completed Part 2:** Token Generation
- [ ] **Completed Part 3:** Secure Token Storage
- [ ] **Completed Part 4:** Token Verification
- [ ] **Passed Part 5:** Troubleshooting (if needed)

**Sign-Off:**
- [ ] Token is valid and accessible
- [ ] `snyk auth <token>` succeeds
- [ ] No token visible in shell history

---

## Local Deployment Phase (5-10 minutes execution)

Follow: **SNYK_LOCAL_DEPLOYMENT_GUIDE.md**

### Environment Setup

- [ ] **SNYK_TOKEN exported**
  ```bash
  export SNYK_TOKEN="your-token-here"
  echo $SNYK_TOKEN  # Verify it's set
  ```

- [ ] **Repository root navigated**
  ```bash
  cd /Users/kooshapari/CodeProjects/Phenotype/repos
  pwd  # Should be /Users/kooshapari/CodeProjects/Phenotype/repos
  ```

- [ ] **Deployment script exists and is executable**
  ```bash
  ls -la scripts/snyk-deploy.sh
  chmod +x scripts/snyk-deploy.sh
  ```

### Script Execution

- [ ] **Deployment script started**
  ```bash
  ./scripts/snyk-deploy.sh
  # Expect: Initial output with version info and prereq check
  ```

- [ ] **Monitored progress** (5-10 minutes)
  - Watched for "Scanning: [repo]" messages
  - Noted vulnerability counts per repo
  - Script completed without hanging

- [ ] **Deployment completed successfully**
  - Expected final message: "Deployment Complete"
  - Expected execution time: 5-10 minutes
  - No critical errors in output

### Results Verification

- [ ] **Report directory created**
  ```bash
  ls -la .snyk-reports/
  # Expected: .snyk-reports directory with .json files
  ```

- [ ] **Summary report readable**
  ```bash
  cat .snyk-reports/report.txt | head -30
  # Expected: Snyk Security Scan Report with summary stats
  ```

- [ ] **Per-repository reports created**
  ```bash
  ls .snyk-reports/*.json | wc -l
  # Expected: 30 JSON files (one per repo)
  ```

- [ ] **Key findings noted**
  - [ ] Total vulnerabilities count
  - [ ] Critical vulnerabilities (if any)
  - [ ] High vulnerabilities (if any)
  - [ ] Tier 1 repos status

**Sign-Off:**
- [ ] All 30 repositories scanned
- [ ] Reports generated without errors
- [ ] Summary report is readable

---

## Results Review Phase (10-15 minutes)

Follow: **SNYK_RESULTS_REVIEW_GUIDE.md**

### Understanding Reports

- [ ] **Summary report reviewed**
  ```bash
  cat .snyk-reports/report.txt
  ```
  - [ ] Total vulnerability count understood
  - [ ] Severity distribution noted (C/H/M/L)
  - [ ] Tier 1 repos (AgilePlus, heliosCLI, phenotype-infrakit) status understood

- [ ] **Critical findings identified** (if any)
  ```bash
  cat .snyk-reports/report.txt | grep -A 5 "CRITICAL"
  ```
  - [ ] List of critical CVEs noted
  - [ ] Affected repositories identified
  - [ ] Remediation path understood

- [ ] **High findings reviewed**
  ```bash
  cat .snyk-reports/*.json | jq '.vulnerabilities[] | select(.severity=="high")'
  ```
  - [ ] Count of high-severity findings
  - [ ] Decision made: fix vs. suppress each finding

### Policy File Creation

For each finding that will be suppressed:

- [ ] **Decision documented** (Fix vs. Suppress)
  - [ ] Critical: FIX (no suppression)
  - [ ] High (prod): FIX or plan in AgilePlus
  - [ ] High (dev-only): SUPPRESS
  - [ ] Medium/Low: SUPPRESS or backlog

- [ ] **.snyk policy files created** (for suppressions)
  ```bash
  # For each repo with suppressions:
  cat > [repo-path]/.snyk << 'EOF'
  version: v1.19.0
  ignore:
    SNYK-ID-XXXXX:
      - '*':
          reason: "Justification here"
          expires: 2026-06-30
          created: 2026-03-30
  EOF
  ```

- [ ] **Justifications documented** (for each suppression)
  - [ ] Reason is specific and actionable
  - [ ] Expiration date is set (max 90 days)
  - [ ] Examples:
    - "Dev-only dependency, not in production binary"
    - "Requires breaking upgrade, scheduled for Q2 2026"
    - "Input validation in place, vulnerability requires direct access"

- [ ] **Policy files validated**
  ```bash
  # Check YAML syntax
  git ls-files | grep "\.snyk$" | xargs -I {} sh -c 'echo "Validating {}"; cat {}'
  ```

### Commitment Preparation

- [ ] **Changes staged**
  ```bash
  git add .snyk-reports/ **/.snyk
  git status -s
  ```

- [ ] **No sensitive data in commits**
  - [ ] No SNYK_TOKEN in any files
  - [ ] No credentials in reports
  - [ ] Only policy files and reports staged

- [ ] **Commit message prepared**
  ```bash
  git commit -m "security: deploy Snyk security scanning

  Introduces Snyk security scanning for 30 repos across ecosystem.

  Summary:
  - [N] vulnerabilities found ([C] critical, [H] high, [M] medium, [L] low)
  - .snyk policy files created with justifications
  - Results in .snyk-reports/ directory

  Next Steps:
  - Review critical/high findings
  - Create AgilePlus work items for remediation
  - Deploy GitHub Actions workflows

  Closes #XXX"
  ```

- [ ] **Commit created**
  ```bash
  git commit -m "..."  # Use prepared message above
  ```

**Sign-Off:**
- [ ] All suppressions documented with justification
- [ ] Expiration dates set on all suppressions
- [ ] Changes committed with clear message
- [ ] No sensitive data in commit

---

## GitHub Integration Phase (5 minutes)

Follow: **GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md**

### Organization Secrets Setup

- [ ] **SNYK_TOKEN added to GitHub organization secrets**
  ```bash
  gh secret set SNYK_TOKEN --org KooshaPari
  # Paste token when prompted
  ```

- [ ] **Secret verified**
  ```bash
  gh secret list --org KooshaPari | grep SNYK_TOKEN
  # Expected: SNYK_TOKEN    Updated [timestamp]
  ```

### Workflow Deployment to Tier 1 Repos

#### AgilePlus

- [ ] **Workflow file created**
  ```bash
  cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
  mkdir -p .github/workflows
  cat > .github/workflows/snyk-scan.yml << 'EOF'
  [see GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md for full content]
  EOF
  ```

- [ ] **Workflow committed and pushed**
  ```bash
  git add .github/workflows/snyk-scan.yml
  git commit -m "ci: add Snyk security scan workflow"
  git push origin main
  ```

- [ ] **Workflow visible on GitHub**
  - Visited: https://github.com/KooshaPari/AgilePlus/actions
  - Confirmed: "Snyk Security Scan" workflow listed

#### heliosCLI

- [ ] **Workflow file created**
  ```bash
  cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI
  mkdir -p .github/workflows
  cat > .github/workflows/snyk-scan.yml << 'EOF'
  [same as AgilePlus]
  EOF
  ```

- [ ] **Workflow committed and pushed**
  ```bash
  git add .github/workflows/snyk-scan.yml
  git commit -m "ci: add Snyk security scan workflow"
  git push origin main
  ```

- [ ] **Workflow visible on GitHub**
  - Visited: https://github.com/KooshaPari/heliosCLI/actions
  - Confirmed: "Snyk Security Scan" workflow listed

#### phenotype-infrakit

- [ ] **Workflow file created**
  ```bash
  cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
  mkdir -p .github/workflows
  cat > .github/workflows/snyk-scan.yml << 'EOF'
  [same as AgilePlus]
  EOF
  ```

- [ ] **Workflow committed and pushed**
  ```bash
  git add .github/workflows/snyk-scan.yml
  git commit -m "ci: add Snyk security scan workflow"
  git push origin main
  ```

- [ ] **Workflow visible on GitHub**
  - Visited: https://github.com/KooshaPari/phenotype-infrakit/actions
  - Confirmed: "Snyk Security Scan" workflow listed

### Workflow Verification

- [ ] **All 3 workflows triggered** (via push)
  - [ ] AgilePlus: Workflow started
  - [ ] heliosCLI: Workflow started
  - [ ] phenotype-infrakit: Workflow started

- [ ] **First runs completed successfully**
  - [ ] Each workflow completed within 5 minutes
  - [ ] No "Unauthorized" errors
  - [ ] Results artifact available for download

- [ ] **Scheduled scans configured**
  - [ ] Cron job set to: `0 2 * * *` (2 AM UTC daily)
  - [ ] First scheduled run expected: Tomorrow at 2 AM UTC

**Sign-Off:**
- [ ] Organization secret (SNYK_TOKEN) set
- [ ] All 3 Tier 1 workflows deployed
- [ ] All 3 workflows executed successfully
- [ ] No "Unauthorized" errors in any run

---

## Post-Deployment Phase (5 minutes)

### Cleanup & Documentation

- [ ] **Temporary token storage cleaned up**
  ```bash
  # If using .env file:
  rm -f /Users/kooshapari/CodeProjects/Phenotype/repos/.env

  # Clear shell history (optional):
  history -c

  # Or selectively remove SNYK_TOKEN from history
  ```

- [ ] **Verification checklist completed**
  - [ ] Token acquisition ✓
  - [ ] Local deployment ✓
  - [ ] Results review ✓
  - [ ] GitHub integration ✓

- [ ] **Next steps documented**
  - [ ] Create AgilePlus work items for critical/high findings
  - [ ] Schedule vulnerability remediation
  - [ ] Plan Phase 2 (deploy to remaining repos)
  - [ ] Set up team notifications

### Final Verification

- [ ] **All 3 Tier 1 repos have active workflows**
  ```bash
  for repo in AgilePlus heliosCLI phenotype-infrakit; do
    echo "=== $repo ==="
    gh run list -R KooshaPari/$repo -w "Snyk Security Scan" --limit 1
  done
  ```

- [ ] **Organization secret is set**
  ```bash
  gh secret list --org KooshaPari | grep SNYK_TOKEN
  ```

- [ ] **No security issues introduced**
  - [ ] No token in any git commits
  - [ ] No credentials in artifacts
  - [ ] All policy files validated

---

## Deployment Sign-Off

### Completed By

**Name:** ___________________________
**Date:** ___________________________
**Time:** ___________________________

### Phase Completion Summary

| Phase | Status | Time | Notes |
|-------|--------|------|-------|
| Pre-Deployment | ✓ Pass | 5 min | All prerequisites verified |
| Token Acquisition | ✓ Pass | 5 min | Token generated and tested |
| Local Deployment | ✓ Pass | 5-10 min | 30 repos scanned, reports generated |
| Results Review | ✓ Pass | 10-15 min | Findings reviewed, suppressions documented |
| GitHub Integration | ✓ Pass | 5 min | Workflows deployed to 3 Tier 1 repos |
| Post-Deployment | ✓ Pass | 5 min | Cleanup and verification complete |
| **TOTAL** | **✓ PASS** | **25-30 min** | **Ready for production** |

### Findings Summary

- **Total Repositories Scanned:** 30
- **Total Vulnerabilities:** _______
  - Critical: _______
  - High: _______
  - Medium: _______
  - Low: _______
- **Critical Findings:** _______
- **High Findings:** _______
- **Suppressions Created:** _______

### Follow-Up Actions

- [ ] **Create AgilePlus work items** for critical/high findings
  - Count: _______
  - Ticket IDs: _______________________

- [ ] **Schedule security review** with team
  - Date: _______________________
  - Attendees: _______________________

- [ ] **Plan Phase 2** (deploy to remaining repos)
  - Timeline: _______________________
  - Owner: _______________________

### Approval

- [ ] **Deployment approved for production**
- [ ] **GitHub workflows active and monitored**
- [ ] **Team notified of new security scans**
- [ ] **Next review scheduled**

---

## Quick Reference: Commands by Phase

### Token Acquisition
```bash
# Visit https://app.snyk.io
# Generate token → Copy to clipboard
export SNYK_TOKEN="your-token-here"
snyk auth $SNYK_TOKEN
snyk test --dry-run
```

### Local Deployment
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
export SNYK_TOKEN="your-token-here"
./scripts/snyk-deploy.sh
cat .snyk-reports/report.txt
```

### Results Review
```bash
cat .snyk-reports/report.txt
cat .snyk-reports/AgilePlus.json | jq '.vulnerabilities[0]'
# Create .snyk files with suppressions
git add .snyk-reports/ **/.snyk
git commit -m "security: deploy Snyk scanning"
```

### GitHub Integration
```bash
gh secret set SNYK_TOKEN --org KooshaPari
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
cat > .github/workflows/snyk-scan.yml << 'EOF'
[workflow content]
EOF
git add .github/workflows/snyk-scan.yml
git commit -m "ci: add Snyk security scan workflow"
git push origin main
```

---

## Support & Troubleshooting

**Common Issues:**

1. **"SNYK_TOKEN not set"**
   - Solution: `export SNYK_TOKEN="your-token"`

2. **"snyk: command not found"**
   - Solution: `brew install snyk`

3. **Workflow shows "Unauthorized"**
   - Solution: Re-run `gh secret set SNYK_TOKEN --org KooshaPari`

4. **Workflow not triggering**
   - Solution: Check `.github/workflows/snyk-scan.yml` is committed

**For more details, see:**
- SNYK_TOKEN_ACQUISITION_GUIDE.md
- SNYK_LOCAL_DEPLOYMENT_GUIDE.md
- SNYK_RESULTS_REVIEW_GUIDE.md
- GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md

---

**Last Updated:** 2026-03-30
**Status:** Ready for execution
