# Snyk Expected Outputs & Examples

Reference guide showing expected outputs at each deployment stage, with real examples and troubleshooting guidance.

**Purpose:** Help you recognize successful execution vs. errors
**Use Case:** Validate deployment is proceeding normally
**Contains:** Example outputs, expected timings, common errors

---

## Part 1: Token Acquisition Outputs

### 1.1: Expected Login Screen (https://app.snyk.io)

**What You Should See:**
```
┌─────────────────────────────────────────┐
│         Snyk - Secure Your Code         │
│                                         │
│  Email or password                      │
│  ┌─────────────────────────────────┐   │
│  │ [your-email@example.com]        │   │
│  └─────────────────────────────────┘   │
│                                         │
│  Password                               │
│  ┌─────────────────────────────────┐   │
│  │ [●●●●●●●●●●]                   │   │
│  └─────────────────────────────────┘   │
│                                         │
│  [Sign In]  [Forgot?]                   │
│                                         │
│  Don't have an account? Sign up →       │
└─────────────────────────────────────────┘
```

**After Successful Login:**
```
┌─────────────────────────────────────────┐
│         Snyk Dashboard                  │
│                                         │
│  Welcome to Snyk, [Your Name]           │
│                                         │
│  ├─ Projects                            │
│  ├─ Reports                             │
│  ├─ Settings                            │
│  └─ ...                                 │
│                                         │
│  Top right: [Profile] [⚙️]              │
└─────────────────────────────────────────┘
```

### 1.2: Expected Auth Token Page

**URL:** https://app.snyk.io/account/...

**What You Should See:**
```
Settings > Auth Token

Current Token (if exists):
┌───────────────────────────────────────────────┐
│ ••••••••-••••-••••-••••-••••••••••••           │
│ [Copy]  [Regenerate]                          │
└───────────────────────────────────────────────┘

Expiration: Never

⚠️ Keep your token secret. Do not share it.
```

**After Regenerate:**
```
┌───────────────────────────────────────────────┐
│ 12345678-abcd-1234-abcd-1234567890ab          │
│ [Copy] ← Click this                           │
└───────────────────────────────────────────────┘

"Copied!" message appears briefly
```

### 1.3: Expected CLI Authentication

**Command:**
```bash
snyk auth 12345678-abcd-1234-abcd-1234567890ab
```

**Expected Output:**
```
Successfully authenticated
```

**If Successful:**
```bash
$ snyk auth 12345678-abcd-1234-abcd-1234567890ab
Successfully authenticated
```

**If Failed:**
```bash
$ snyk auth invalid-token-here
Error: Unauthorized
```

**Solution:** Re-generate token at https://app.snyk.io

### 1.4: Expected Environment Variable

**Command:**
```bash
export SNYK_TOKEN="12345678-abcd-1234-abcd-1234567890ab"
echo $SNYK_TOKEN
```

**Expected Output:**
```
12345678-abcd-1234-abcd-1234567890ab
```

**If Not Set:**
```
# (blank line - nothing printed)
```

**Solution:** Re-run export command

---

## Part 2: Local Deployment Outputs

### 2.1: Pre-Deployment Checks

**Command:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
./scripts/snyk-deploy.sh
```

**Expected Output (First 30 seconds):**
```
============================================================
Snyk Security Deployment Script
============================================================
Repository Root: /Users/kooshapari/CodeProjects/Phenotype/repos
Report Directory: .snyk-reports
Timestamp: 2026-03-30 14:32:15

Verifying prerequisites...
  ✓ Snyk CLI version: 1.1234.5
  ✓ Git version: 2.45.0
  ✓ SNYK_TOKEN is set
  ✓ Report directory created: .snyk-reports

Starting Snyk security scans...
============================================================
```

**What Each Line Means:**
| Output | Status | Action |
|--------|--------|--------|
| `✓ Snyk CLI version` | ✅ OK | CLI installed correctly |
| `✓ Git version` | ✅ OK | Git installed correctly |
| `✓ SNYK_TOKEN is set` | ✅ OK | Token accessible |
| `✓ Report directory created` | ✅ OK | Ready to save reports |

### 2.2: Scanning Progress Output

**Expected Output (1-2 minutes into deployment):**
```
Scanning: AgilePlus
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
  Type: Rust + JavaScript
  Status: In progress...

  [PASS] Manifest files detected (3 found)
    - Cargo.toml (root)
    - package.json
    - crates/agileplus-cli/Cargo.toml
  [PASS] Dependencies analyzed
  Scanning manifest: Cargo.toml
    Testing Rust dependencies: 245 packages
    ✓ Tested 245 packages
    ✗ Found 3 vulnerabilities (1 high, 2 medium)
    Saving to: .snyk-reports/AgilePlus.json

Scanning: heliosCLI
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI
  Type: Rust
  Status: In progress...

  [PASS] Manifest files detected (1 found)
  [PASS] No known vulnerabilities detected
  Saving to: .snyk-reports/heliosCLI.json
```

**What This Means:**
- `[PASS]` = Step succeeded, continue
- `✓ Tested 245 packages` = Successfully analyzed dependencies
- `✗ Found 3 vulnerabilities` = Vulnerabilities detected (expected, will review later)
- `Saving to:` = Report file being written

### 2.3: Mid-Deployment Progress (After 5 minutes)

**Expected Output (Progress through ~50% of repos):**
```
Scanning: phenotype-infrakit
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
  Type: Rust
  Status: In progress...

  [PASS] Manifest files detected (15 found)
    - Cargo.toml (root workspace)
    - crates/phenotype-error-core/Cargo.toml
    - crates/phenotype-health/Cargo.toml
    - ... (12 more)
  [PASS] Dependencies analyzed
  Testing 850 dependencies across 15 crates...
  ✓ Tested 850 dependencies
  ✗ Found 2 vulnerabilities (2 medium)
  Saving to: .snyk-reports/phenotype-infrakit.json

Scanning: pheno-cli
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/pheno-cli
  Type: Go
  Status: In progress...

  [WARN] go.sum file missing (go.mod present)
  Scanning manifest: go.mod
    Testing Go dependencies: 32 packages
    ✓ Tested 32 packages
    ✓ No known vulnerabilities detected
  [WARN] Some lock files missing - results may be incomplete
  Saving to: .snyk-reports/pheno-cli.json
```

**Note:** `[WARN]` messages are normal — Snyk still scans what it can

### 2.4: Deployment Completion

**Expected Output (After 7-10 minutes total):**
```
Scanning: zen
  Repository: /Users/kooshapari/CodeProjects/Phenotype/repos/zen
  Type: Rust + Python
  Status: In progress...

  [PASS] Manifest files detected (2 found)
  Scanning manifest: Cargo.toml
    ✓ Tested 156 dependencies
    ✗ Found 1 vulnerability (1 low)
  Scanning manifest: requirements.txt
    ✓ Tested 45 dependencies
    ✓ No known vulnerabilities detected
  Saving to: .snyk-reports/zen.json

============================================================
Deployment Complete
============================================================
Total Repositories Scanned: 30
Total Vulnerabilities Found: 45
  Critical:  2
  High:      8
  Medium:    18
  Low:       17

Report Files Generated: 30
  Location: /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/
  Summary: .snyk-reports/report.txt
  Detailed: .snyk-reports/*.json (30 files)

Execution Time: 7 minutes 32 seconds
Next Steps: Review .snyk-reports/report.txt

============================================================
```

**Sign of Success:**
- [ ] Total time: 5-10 minutes
- [ ] All 30 repos scanned
- [ ] Vulnerability counts shown
- [ ] Reports created in `.snyk-reports/`
- [ ] No critical errors (warnings OK)

### 2.5: Report Directory Created

**Command:**
```bash
ls -lh /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/
```

**Expected Output:**
```
total 520K
-rw-r--r--  1 user  staff  12K Mar 30 14:40 report.txt
-rw-r--r--  1 user  staff  2.1K Mar 30 14:32 AgilePlus.json
-rw-r--r--  1 user  staff  1.8K Mar 30 14:33 agileplus-agents.json
-rw-r--r--  1 user  staff  896B  Mar 30 14:34 agileplus-dashboard.json
... (27 more .json files)
```

**What This Means:**
- 30 `.json` files = 30 repos scanned
- File sizes vary based on vulnerability count
- `report.txt` is summary

---

## Part 3: Results Review Outputs

### 3.1: Summary Report Content

**Command:**
```bash
cat /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/report.txt
```

**Expected Output (First 50 lines):**
```
=================================================================
Snyk Security Scan Report
Generated: 2026-03-30 14:40:47
Repository Root: /Users/kooshapari/CodeProjects/Phenotype/repos
=================================================================

SUMMARY
-------
Total Repositories: 30
Repositories Scanned: 30 (100%)
Scan Duration: 7 minutes 32 seconds

Vulnerability Summary:
  Total Vulnerabilities: 45
    Critical:   2  ██████████
    High:       8  ████████████████████
    Medium:    18  ██████████████████████████████████████
    Low:       17  ████████████████████████████

Risk Assessment:
  Critical Risk (2 findings)     ────────────────────── IMMEDIATE ACTION
  High Risk (8 findings)         ────────────────────── 30 DAYS
  Medium Risk (18 findings)      ────────────────────── 90 DAYS
  Low Risk (17 findings)         ────────────────────── BACKLOG

=================================================================
TIER 1 REPOSITORIES (Priority Scan)
=================================================================

1. AgilePlus
   ├─ Status: ✗ VULNERABILITIES FOUND
   ├─ Vulnerabilities: 14 (1 critical, 4 high, 6 medium, 3 low)
   ├─ Key Finding: CVE-2021-23337 (Prototype Pollution - lodash)
   ├─ Report: .snyk-reports/AgilePlus.json
   └─ Action: REQUIRED - Critical vulnerability needs immediate fix

2. heliosCLI
   ├─ Status: ✓ CLEAN
   ├─ Vulnerabilities: 0
   ├─ Dependencies: 345 tested
   └─ Action: None - Keep as baseline

3. phenotype-infrakit
   ├─ Status: ⚠ MEDIUM FINDINGS
   ├─ Vulnerabilities: 5 (2 medium, 3 low)
   ├─ Key Finding: CVE-2024-XXXXX (TCP socket leak)
   ├─ Report: .snyk-reports/phenotype-infrakit.json
   └─ Action: PLANNED - Schedule for next sprint

=================================================================
CRITICAL FINDINGS REQUIRING IMMEDIATE ACTION
=================================================================

1. CVE-2021-23337 (Prototype Pollution in lodash)
   ├─ Repository: AgilePlus
   ├─ Severity: CRITICAL
   ├─ CVSS Score: 8.2
   ├─ Affected Version: 4.17.20
   ├─ Fixed Version: 4.17.21
   ├─ Dependency Chain: AgilePlus → express@4.17.1 → lodash@4.17.20
   └─ Recommendation: Upgrade lodash to 4.17.21 immediately

2. CVE-2024-YYYYY (Buffer Overflow in crypto)
   ├─ Repository: phenotype-infrakit
   ├─ Severity: CRITICAL
   ├─ CVSS Score: 9.1
   ├─ Affected Version: openssl@1.0.1
   ├─ Fixed Version: 1.1.0
   └─ Recommendation: Upgrade openssl immediately

=================================================================
TIER 2 REPOSITORIES
=================================================================

... (additional repos)

=================================================================
NEXT STEPS
=================================================================

1. Immediate (Today)
   - Review critical findings above
   - Create AgilePlus work items for fixes
   - Assign owners for remediation

2. Short-term (This Week)
   - Deploy GitHub Actions workflows
   - Enable automated nightly scans
   - Set up team notifications

3. Medium-term (This Month)
   - Remediate high-severity findings
   - Create .snyk policy files for suppressions
   - Deploy to Tier 2 repos

=================================================================
```

### 3.2: Detailed JSON Report

**Command:**
```bash
cat /Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/AgilePlus.json | jq '.vulnerabilities[0]'
```

**Expected Output (Single Vulnerability Example):**
```json
{
  "id": "SNYK-JS-LODASH-1234567",
  "title": "Prototype Pollution in lodash",
  "description": "Versions of lodash lower than 4.17.21 are vulnerable to prototype pollution attacks via _.template() function.",
  "severity": "high",
  "cvssScore": 6.5,
  "cvss": {
    "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:R/S:U/C:N/I:H/A:N",
    "version": "3.1"
  },
  "from": [
    "AgilePlus@1.0.0",
    "express@4.17.1",
    "lodash@4.17.20"
  ],
  "package": "lodash",
  "version": "4.17.20",
  "fixedIn": [
    "4.17.21"
  ],
  "introduced": "0",
  "patched": false,
  "upgrade": "lodash@4.17.21",
  "upgradePath": [
    false,
    "express@4.17.1",
    "lodash@4.17.21"
  ],
  "patches": [],
  "isUpgradable": true,
  "isPatchable": false,
  "isFixable": true,
  "fixAvailable": true,
  "severityPerVersion": {
    "4.17.20": "high"
  },
  "publicationTime": "2021-02-15T10:20:00Z",
  "disclosureTime": "2021-02-15T10:20:00Z",
  "firstPatchedVersion": "4.17.21",
  "references": [
    {
      "url": "https://nvd.nist.gov/vuln/detail/CVE-2021-23337",
      "title": "CVE-2021-23337"
    }
  ],
  "cve": "CVE-2021-23337"
}
```

**Key Fields to Note:**
- `severity: "high"` = Priority
- `package: "lodash"` = What's vulnerable
- `version: "4.17.20"` = Vulnerable version
- `fixedIn: ["4.17.21"]` = Solution
- `isFixable: true` = Can be fixed with upgrade

### 3.3: Creating .snyk Policy File

**Command:**
```bash
cat > /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.snyk << 'EOF'
version: v1.19.0
allow-licenses:
  - MIT
  - Apache-2.0
  - BSD-2-Clause
  - BSD-3-Clause
  - ISC
ignore:
  SNYK-JS-LODASH-1234567:
    - '*':
        reason: "Upgrading lodash requires express upgrade to 5.0, scheduled for Q2 2026"
        expires: 2026-06-30
        created: 2026-03-30
patch: {}
EOF
```

**Expected Output (No visible output if successful):**
```
(File is created silently)
```

**Verify it was created:**
```bash
cat /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.snyk
```

**Expected Output:**
```yaml
version: v1.19.0
allow-licenses:
  - MIT
  - Apache-2.0
  - BSD-2-Clause
  - BSD-3-Clause
  - ISC
ignore:
  SNYK-JS-LODASH-1234567:
    - '*':
        reason: "Upgrading lodash requires express upgrade to 5.0, scheduled for Q2 2026"
        expires: 2026-06-30
        created: 2026-03-30
patch: {}
```

---

## Part 4: GitHub Integration Outputs

### 4.1: Setting Organization Secret

**Command:**
```bash
gh secret set SNYK_TOKEN --org KooshaPari
```

**Expected Output (Interactive Prompt):**
```
? Paste your secret (it will be hidden) [? for help]
```

**After Pasting Token and Pressing Enter:**
```
✓ Set organization secret SNYK_TOKEN for KooshaPari
```

**If Already Exists:**
```
✓ Set organization secret SNYK_TOKEN for KooshaPari (updated)
```

### 4.2: Verifying Secret is Set

**Command:**
```bash
gh secret list --org KooshaPari
```

**Expected Output:**
```
NAME              UPDATED
SNYK_TOKEN        2026-03-30 14:42:13 +0000 UTC
```

### 4.3: Deploying Workflow File

**Command:**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
ls -la .github/workflows/snyk-scan.yml
```

**Expected Output:**
```
-rw-r--r--  1 user  staff  1.2K Mar 30 14:43 .github/workflows/snyk-scan.yml
```

### 4.4: Committing Workflow

**Command:**
```bash
git add .github/workflows/snyk-scan.yml
git commit -m "ci: add Snyk security scan workflow"
git push origin main
```

**Expected Output:**
```
[main 8a7c4b2] ci: add Snyk security scan workflow
 1 file changed, 35 insertions(+)
 create mode 100644 .github/workflows/snyk-scan.yml

Enumerating objects: 5, done.
Counting objects: 100% (5/5), done.
Delta compression using delta objects: 100% (100/5), done.
Compressing objects: 100% (2/2), done.
Writing objects to 100% (3/3), 346 bytes | 346.00 KiB/s, done.
Total 3 (delta 2), reused 0 (delta 0), writing 2 (delta 2)
remote: Resolving deltas: 100% (2/2), completed with 2 remote deltas.
To https://github.com/KooshaPari/AgilePlus.git
   a3b4c5d..8a7c4b2  main -> main
```

### 4.5: Workflow Triggering

**Expected Output (Check GitHub Actions UI):**
1. Go to: https://github.com/KooshaPari/AgilePlus/actions
2. Look for: **"Snyk Security Scan"** workflow
3. Status should show: **"In Progress"** or **"Completed"**

**Example Workflow Run Page:**
```
Snyk Security Scan

Workflow file: .github/workflows/snyk-scan.yml
Event: push
Triggered: 2 minutes ago
Duration: 3 min 45 sec
Conclusion: Success ✓

Jobs:
  snyk-scan                                ✓ 3m45s
    ├─ Checkout code                       ✓ 12s
    ├─ Run Snyk scan                       ✓ 15s
    ├─ Snyk test                           ✓ 2m30s
    ├─ Upload results                      ✓ 48s
    └─ Publish results to PR                (skipped - not a PR)
```

### 4.6: Workflow Output Sample

**Expected Output (From "Snyk test" step):**
```
Testing /github/workspace...

Testing Cargo.toml...
Tested 245 dependencies for known vulnerabilities
Found 3 vulnerabilities

Tested Rust packages

✓ Tested 245 packages
✗ 3 vulnerabilities found:
  - 1 high severity
  - 2 medium severity

JSON report saved to snyk-report.json
```

### 4.7: Downloading Artifacts

**From GitHub Actions UI:**
1. Go to: https://github.com/KooshaPari/AgilePlus/actions/runs/[run-id]
2. Scroll to: **"Artifacts"** section
3. Download: `snyk-report` (.zip file)
4. Contains: `snyk-report.json`

**Command-line Alternative:**
```bash
gh run download [run-id] -D snyk-reports/
```

---

## Part 5: Common Errors & Troubleshooting

### Error 1: "SNYK_TOKEN environment variable not set"

**What You See:**
```
Error: SNYK_TOKEN environment variable not set
Please set SNYK_TOKEN before running this script.
```

**Why It Happens:**
- Token not exported in current shell session

**How to Fix:**
```bash
export SNYK_TOKEN="your-token-here"
echo $SNYK_TOKEN  # Verify
./scripts/snyk-deploy.sh
```

### Error 2: "Unauthorized"

**What You See:**
```
Error: Unauthorized
Invalid or expired token
```

**Why It Happens:**
- Token is invalid or expired
- Token was regenerated at https://app.snyk.io

**How to Fix:**
```bash
# Regenerate at https://app.snyk.io
# Then:
export SNYK_TOKEN="new-token-here"
snyk auth $SNYK_TOKEN
./scripts/snyk-deploy.sh
```

### Error 3: "snyk: command not found"

**What You See:**
```
-bash: snyk: command not found
```

**Why It Happens:**
- Snyk CLI not installed

**How to Fix:**
```bash
brew install snyk  # macOS/Linux
# OR
npm install -g snyk

# Verify
snyk --version
```

### Error 4: GitHub Workflow "Unauthorized"

**What You See (In GitHub Actions):**
```
Run: snyk test --json-file-output=snyk-report.json
Error: Unauthorized
Please check your SNYK_TOKEN
```

**Why It Happens:**
- GitHub organization secret not set
- or secret name is wrong

**How to Fix:**
```bash
# Verify secret is set
gh secret list --org KooshaPari | grep SNYK_TOKEN

# If missing, set it
gh secret set SNYK_TOKEN --org KooshaPari

# Re-run workflow (make a small commit or trigger manually)
```

### Error 5: "go.sum file missing" Warning

**What You See:**
```
[WARN] go.sum file missing (go.mod present)
```

**Why It Happens:**
- Go project missing lock file (normal for some repos)

**Is It Critical:**
- ❌ No - Snyk continues with what it has

**Action:**
- ℹ️ Informational only, no fix needed

---

## Part 6: Success Indicators Checklist

### Token Acquisition Phase ✓

- [ ] Snyk login successful
- [ ] Token generated at https://app.snyk.io
- [ ] Token copied to clipboard
- [ ] `snyk auth <token>` outputs "Successfully authenticated"
- [ ] `snyk test --dry-run` works without errors

### Local Deployment Phase ✓

- [ ] Script starts with "Snyk Security Deployment Script"
- [ ] All prerequisites check marks appear: `✓ Snyk CLI`, `✓ Git`, `✓ Token`, etc.
- [ ] All 30 repos scanned (watch for "Scanning: [repo]" messages)
- [ ] Final output shows: "Deployment Complete"
- [ ] `.snyk-reports/` directory created with 30+ JSON files
- [ ] `report.txt` is readable and shows summary

### Results Review Phase ✓

- [ ] Summary report shows vulnerability counts
- [ ] Critical findings identified (if any)
- [ ] `.snyk` policy files created for suppressions
- [ ] All suppressions have expiration dates
- [ ] Git changes staged and committed

### GitHub Integration Phase ✓

- [ ] `SNYK_TOKEN` secret set at org level
- [ ] Workflow files committed to all 3 Tier 1 repos
- [ ] Workflows visible in GitHub Actions tab
- [ ] First run completed (check status: ✓ or ⚠)
- [ ] Artifacts available for download
- [ ] No "Unauthorized" errors in workflow output

---

## Part 7: Performance Expectations

### Timing by Phase

| Phase | Min | Max | Typical |
|-------|-----|-----|---------|
| Token acquisition | 5 min | 15 min | 8 min |
| Local deployment | 5 min | 15 min | 8 min |
| Results review | 10 min | 30 min | 15 min |
| GitHub setup | 5 min | 10 min | 7 min |
| **Total** | **25 min** | **70 min** | **38 min** |

### Repo Scanning Times

| Repo Type | Avg Time | Example |
|-----------|----------|---------|
| Small Rust | 15-20 sec | phenotype-contracts |
| Medium Rust | 30-45 sec | AgilePlus |
| Large Rust workspace | 60-90 sec | phenotype-infrakit |
| JavaScript/Node | 20-40 sec | helios-ui |
| Go | 20-30 sec | pheno-cli |
| Python | 15-25 sec | phench |
| Mixed (2-3 langs) | 45-90 sec | AgilePlus |

### Workflow Execution Times

| Step | Min | Max | Typical |
|------|-----|-----|---------|
| Checkout | 10 sec | 30 sec | 15 sec |
| Setup Snyk | 10 sec | 20 sec | 12 sec |
| Run scan | 1 min | 5 min | 2:30 min |
| Upload artifacts | 20 sec | 60 sec | 45 sec |
| **Total workflow** | **2 min** | **6 min** | **4 min** |

---

## Reference

- **Snyk CLI Output:** https://docs.snyk.io/cli/output
- **Snyk JSON Report Format:** https://docs.snyk.io/features/scanning-and-fixing/snyk-findings/understanding-snyk-findings
- **CVSS Severity Guide:** https://www.first.org/cvss/v3.1/specification-document

---

**Last Updated:** 2026-03-30
**Status:** Ready as reference guide
