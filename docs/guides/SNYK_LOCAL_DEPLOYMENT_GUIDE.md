# Snyk Local Deployment Guide

Complete step-by-step guide to deploy Snyk security scanning across the Phenotype repository ecosystem.

**Time Required:** 5-10 minutes (execution time)
**Complexity:** Intermediate
**Prerequisites:**
- Snyk token acquired (see SNYK_TOKEN_ACQUISITION_GUIDE.md)
- Snyk CLI installed and authenticated
- All repos cloned locally

---

## Overview

This guide walks you through:
1. Verifying local environment and prerequisites
2. Running the Snyk deployment script
3. Understanding deployment stages and output
4. Monitoring progress in real-time
5. Locating and reviewing results

---

## Part 1: Prerequisites Checklist

Before running the deployment script, verify you have:

### 1.1: Required Software

Check each requirement:

```bash
# 1. Snyk CLI installed
snyk --version
# Expected output: Snyk CLI version X.XXX.X

# 2. Git installed
git --version
# Expected output: git version X.XX.X

# 3. Bash available
bash --version
# Expected output: GNU bash, version X.X.X

# 4. Node.js (for some scans)
node --version
# Expected output: vX.XX.X (optional, but recommended)
```

### 1.2: Authentication

Verify Snyk CLI is authenticated:

```bash
snyk auth --help
```

Or test authentication:

```bash
snyk test --dry-run
```

**Expected Output:** No "Unauthorized" error messages.

### 1.3: Repository Locations

Verify all Phenotype repos are cloned:

```bash
ls -la /Users/kooshapari/CodeProjects/Phenotype/repos | grep -E "^d"
```

**Expected Output:** Directories including:
- `AgilePlus`
- `heliosCLI`
- `phenotype-infrakit`
- Plus ~27 other projects

### 1.4: Snyk Token Available

Verify your token is accessible:

```bash
# If using environment variable
echo $SNYK_TOKEN
# Expected output: Your token (or first few chars)

# If using .env file
cat /path/to/.env | grep SNYK_TOKEN
# Expected output: SNYK_TOKEN="your-token"
```

---

## Part 2: Environment Setup

### 2.1: Set SNYK_TOKEN Environment Variable

If you haven't already, set the token in your current shell session:

```bash
export SNYK_TOKEN="your-token-here"
```

Replace `your-token-here` with your actual token from Part 1 of SNYK_TOKEN_ACQUISITION_GUIDE.md.

**Verify:**
```bash
echo $SNYK_TOKEN
# Output should show your token
```

### 2.2: Navigate to Repository Root

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
pwd
# Output should be: /Users/kooshapari/CodeProjects/Phenotype/repos
```

### 2.3: Verify Deployment Script Exists

```bash
ls -la scripts/snyk-deploy.sh
```

**Expected Output:**
```
-rwxr-xr-x  ... scripts/snyk-deploy.sh
```

If not executable, fix permissions:

```bash
chmod +x scripts/snyk-deploy.sh
```

---

## Part 3: Running the Deployment Script

### 3.1: Execute Deployment Script

```bash
export SNYK_TOKEN="your-token-here"
./scripts/snyk-deploy.sh
```

**Alternative: Pass Token Directly**

```bash
SNYK_TOKEN="your-token-here" ./scripts/snyk-deploy.sh
```

**Alternative: Source from .env File**

```bash
source .env  # If you created .env in Part 3 of token acquisition guide
./scripts/snyk-deploy.sh
```

### 3.2: Initial Output

When the script starts, you'll see:

```
============================================================
Snyk Security Deployment Script
============================================================
Repository Root: /Users/kooshapari/CodeProjects/Phenotype/repos
Report Directory: .snyk-reports
Timestamp: 2026-03-30 14:32:15

Verifying prerequisites...
  ✓ Snyk CLI version: X.XXX.X
  ✓ Git version: X.XX.X
  ✓ SNYK_TOKEN is set
  ✓ Report directory created

Starting Snyk security scans...
============================================================
```

**What This Means:**
- Script confirmed all dependencies are available
- Report directory (`.snyk-reports/`) was created
- Deployment is starting

---

## Part 4: Monitoring Deployment Progress

### 4.1: Per-Repository Scan Output

As the script runs, you'll see output for each repository:

```
Scanning: AgilePlus
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
  Type: Rust + JavaScript
  Status: In progress...

  [PASS] Manifest files detected
  [PASS] Dependencies analyzed
  Scanning manifest: Cargo.toml
  Scanning manifest: package.json
  Found 3 vulnerabilities (1 high, 2 medium)
  Saved to: .snyk-reports/AgilePlus.json
```

**What Each Message Means:**

| Message | Status | Action |
|---------|--------|--------|
| `[PASS]` | ✅ Success | Continue, no action needed |
| `[WARN]` | ⚠️ Warning | Non-critical, may need review |
| `[FAIL]` | ❌ Error | Critical, may need investigation |
| `Found X vulnerabilities` | ℹ️ Info | Expected, will be reviewed later |
| `Saved to: ...json` | ✅ Success | Report saved, moving to next repo |

### 4.2: Expected Timing

Typical deployment takes 5-10 minutes:

| Stage | Duration | Progress |
|-------|----------|----------|
| Prerequisites (0.5 min) | Very fast | 5-10% |
| First 3 repos (1-2 min) | ~30 sec/repo | 10-40% |
| Middle repos (2-3 min) | ~20-30 sec/repo | 40-70% |
| Large repos (2-3 min) | ~45-60 sec/repo | 70-90% |
| Report generation (1 min) | ~1 min | 90-100% |
| **Total** | **5-10 min** | **100%** |

**Note:** Times vary based on:
- Number of manifest files per repo
- Size of dependency trees
- Network latency (Snyk API calls)
- Your machine's CPU

### 4.3: Monitoring in Real-Time

If you want to monitor progress in a separate terminal:

```bash
# Terminal 2 (while script runs in Terminal 1)
watch -n 5 'ls -lh /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/*.json | tail -5'
```

This shows newly created report files every 5 seconds.

---

## Part 5: Understanding Output Details

### 5.1: Repository Processing Examples

#### Example 1: Rust-Only Repository
```
Scanning: phenotype-infrakit
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
  Type: Rust
  Status: In progress...

  [PASS] Manifest files detected
  Scanning manifest: Cargo.toml (root)
  Scanning manifest: Cargo.toml (crates/phenotype-error-core)
  Scanning manifest: Cargo.toml (crates/phenotype-health)
  Scanning manifest: Cargo.toml (crates/phenotype-cache-adapter)
  Found 2 vulnerabilities (0 high, 2 medium)
  Saved to: .snyk-reports/phenotype-infrakit.json
```

**What This Means:**
- Repository has multiple Cargo.toml files (workspace structure)
- Snyk scanned all of them
- Found 2 medium-severity vulnerabilities
- Results saved to JSON file

#### Example 2: Multi-Language Repository
```
Scanning: AgilePlus
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
  Type: Rust + JavaScript + Go
  Status: In progress...

  [PASS] Manifest files detected
  Scanning manifest: Cargo.toml
  Scanning manifest: package.json
  Scanning manifest: go.mod
  [WARN] Some lock files missing (go.sum)
  Found 5 vulnerabilities (1 critical, 2 high, 2 medium)
  [WARN] 1 vulnerability requires manual review
  Saved to: .snyk-reports/AgilePlus.json
```

**What This Means:**
- Repository uses 3 languages
- One missing lock file (go.sum) — Snyk still scanned what it could
- Found 1 critical vulnerability (needs immediate attention)
- Manual review flag indicates special handling needed

#### Example 3: Repository with No Vulnerabilities
```
Scanning: phenotype-contracts
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-contracts
  Type: Rust
  Status: In progress...

  [PASS] Manifest files detected
  Scanning manifest: Cargo.toml
  [PASS] No known vulnerabilities detected
  Saved to: .snyk-reports/phenotype-contracts.json
```

**What This Means:**
- Repository has no known vulnerabilities
- All dependencies are up to date
- Still generates a report (for completeness)

### 5.2: Vulnerability Severity Levels

```
Found 8 vulnerabilities:
  Critical: 1   ████  Highest priority, fix immediately
  High:     2   ██    Fix within 30 days
  Medium:   3   █     Fix within 90 days
  Low:      2        Review and plan fixes
```

**Severity Definitions:**

| Severity | Definition | Timeline | Example |
|----------|-----------|----------|---------|
| **Critical** | Remote Code Execution (RCE), Auth bypass | 0-7 days | SQL injection in DB driver |
| **High** | Privilege escalation, data exposure | 7-30 days | Unvalidated API input |
| **Medium** | Denial of Service (DoS), info leak | 30-90 days | Missing rate limiting |
| **Low** | Minor security issues, best practices | 90+ days | Weak TLS cipher |

---

## Part 6: Deployment Completion

### 6.1: Final Output

When the script completes successfully:

```
============================================================
Deployment Complete
============================================================
Total Repositories Scanned: 30
Total Vulnerabilities Found: 45
  Critical:  2
  High:      8
  Medium:   18
  Low:      17

Report Files Generated: 30
  Location: /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/
  Summary: .snyk-reports/report.txt
  Detailed: .snyk-reports/*.json

Execution Time: 7 minutes 32 seconds
Next Steps: Review .snyk-reports/report.txt
============================================================
```

### 6.2: Verify Reports Created

```bash
ls -la /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/
```

**Expected Output:**
```
drwx------ ... .snyk-reports
-rw-r--r-- ... report.txt
-rw-r--r-- ... AgilePlus.json
-rw-r--r-- ... heliosCLI.json
-rw-r--r-- ... phenotype-infrakit.json
... (28 more .json files)
```

### 6.3: Quick Report Summary

View the summary report:

```bash
cat /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/report.txt
```

**Expected Output (first 50 lines):**
```
=================================================================
Snyk Security Scan Report
Generated: 2026-03-30 14:40:47
=================================================================

SUMMARY
-------
Total Repositories: 30
Repositories Scanned: 30 (100%)

Vulnerability Summary:
  Total Vulnerabilities: 45
    Critical:   2  (AgilePlus, phenotype-infrakit)
    High:       8  (AgilePlus x4, heliosCLI x3, pheno-cli x1)
    Medium:    18
    Low:       17

TIER 1 REPOSITORIES (Priority Scan)
-----------------------------------

1. AgilePlus
   Status: ✗ (Vulnerabilities Found)
   Vulnerabilities: 14 (1 critical, 4 high, 6 medium, 3 low)
   Details: .snyk-reports/AgilePlus.json
   Action: REQUIRED - Critical vulnerability detected
   ...
```

---

## Part 7: Common Issues & Troubleshooting

### Issue: "SNYK_TOKEN environment variable not set"

**Cause:** Token not exported before running script

**Solution:**
```bash
export SNYK_TOKEN="your-token-here"
./scripts/snyk-deploy.sh
```

### Issue: "snyk: command not found"

**Cause:** Snyk CLI not installed or not in PATH

**Solution:**
```bash
# Install with Homebrew
brew install snyk

# Or npm
npm install -g snyk

# Verify
snyk --version
```

### Issue: "Unauthorized" or "Invalid token" errors

**Cause:** Token is invalid or expired

**Solution:**
1. Regenerate token at https://app.snyk.io
2. Export the new token
3. Re-run the script

```bash
export SNYK_TOKEN="new-token-here"
./scripts/snyk-deploy.sh
```

### Issue: Script Times Out

**Cause:** Network issues or large repositories

**Solution:**
```bash
# Run with extended timeout (example: 30 minutes)
timeout 1800 ./scripts/snyk-deploy.sh

# Or manually scan one repository
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
snyk test --json > ../snyk-reports/AgilePlus.json
```

### Issue: Report Directory Permission Denied

**Cause:** `.snyk-reports/` directory not writable

**Solution:**
```bash
# Check permissions
ls -la /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports

# Fix permissions
chmod -R 755 /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports

# Or remove and let script recreate
rm -rf /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports
./scripts/snyk-deploy.sh
```

### Issue: Some Repositories Fail, Others Succeed

**Cause:** Repository-specific issues (missing manifests, corrupted files)

**Solution:**
1. Script will skip failing repos and continue
2. Review error messages for each failed repo
3. Run individual scans on problem repos:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
snyk test --json-file-output=snyk-report.json
```

---

## Part 8: Next Steps

After successful deployment:

### Immediate (Before Proceeding)
1. ✅ Verify report files exist: `ls -la .snyk-reports/`
2. ✅ Review summary: `cat .snyk-reports/report.txt`
3. ✅ Keep SNYK_TOKEN for reference (you may need it for GitHub Secrets)

### Short-Term (Next Guide)
1. Proceed to **SNYK_RESULTS_REVIEW_GUIDE.md**
2. Understand vulnerability findings
3. Review and suppress findings with justification
4. Create `.snyk` policy files

### Medium-Term (GitHub Integration)
1. Add SNYK_TOKEN to GitHub organization secrets
2. Deploy GitHub Actions workflows (GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md)
3. Enable automated nightly scans

### Long-Term (Ongoing)
1. Monitor reports for new vulnerabilities
2. Plan remediation for critical/high findings
3. Track fixes in AgilePlus work items
4. Review scan trends quarterly

---

## Reference

- **Snyk CLI Commands:** https://docs.snyk.io/cli
- **Snyk Test Options:** https://docs.snyk.io/cli/commands/test
- **Snyk Policy Files:** https://docs.snyk.io/cli/manage-policy-as-code
- **Severity Definitions:** https://docs.snyk.io/getting-started/severity-levels

---

**Last Updated:** 2026-03-30
**Status:** Ready for results review
