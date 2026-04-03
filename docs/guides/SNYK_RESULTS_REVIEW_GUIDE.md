# Snyk Results Review Guide

Complete guide to understanding, reviewing, and managing Snyk vulnerability reports.

**Time Required:** 10-20 minutes
**Complexity:** Intermediate
**Prerequisites:** Completed SNYK_LOCAL_DEPLOYMENT_GUIDE.md

---

## Overview

This guide covers:
1. Reading and understanding vulnerability reports
2. Interpreting severity levels and remediation guidance
3. Creating `.snyk` policy files for suppressions
4. Committing results to git with proper documentation
5. Creating PR description for review

---

## Part 1: Understanding the Vulnerability Report

### 1.1: Open Summary Report

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cat .snyk-reports/report.txt
```

**Report Structure:**

```
=================================================================
Snyk Security Scan Report
Generated: 2026-03-30 14:40:47
=================================================================

SUMMARY
  Total Repositories: 30
  Total Vulnerabilities: 45
    Critical:   2
    High:      8
    Medium:    18
    Low:      17

TIER 1 REPOSITORIES (Priority)
  [AgilePlus]
    Status: ✗ Vulnerabilities Found
    Vulnerabilities: 14 (1C, 4H, 6M, 3L)
    Details: .snyk-reports/AgilePlus.json

  [heliosCLI]
    Status: ✓ Clean

  [phenotype-infrakit]
    Status: ✗ Vulnerabilities Found
    Vulnerabilities: 5 (2M, 3L)
    Details: .snyk-reports/phenotype-infrakit.json

TIER 2-3 REPOSITORIES
  ... (additional repos)

CRITICAL FINDINGS REQUIRING IMMEDIATE ACTION
  1. AgilePlus: CVE-2024-XXXXX (RCE in dependency)
  2. phenotype-infrakit: CVE-2024-YYYYY (Auth bypass)
```

### 1.2: Understand Summary Abbreviations

| Abbrev | Meaning | Priority |
|--------|---------|----------|
| C | Critical | Immediate (0-7 days) |
| H | High | Soon (7-30 days) |
| M | Medium | Planned (30-90 days) |
| L | Low | Backlog (90+ days) |

---

## Part 2: Reading Individual Repository Reports

### 2.1: Open Detailed JSON Report

For a specific repository:

```bash
cat .snyk-reports/AgilePlus.json | jq .
```

Or use a JSON viewer:

```bash
# Pretty-print with syntax highlighting
cat .snyk-reports/AgilePlus.json | jq '.' | less
```

### 2.2: JSON Report Structure

The JSON contains:

```json
{
  "meta": {
    "timestamp": "2026-03-30T14:40:47.000Z",
    "api": "https://api.snyk.io/v1",
    "isPrivate": true
  },
  "vulnerabilities": [
    {
      "id": "SNYK-JS-LODASH-XXXXX",
      "title": "Prototype Pollution in lodash",
      "description": "Versions of lodash ...",
      "severity": "high",
      "cvssScore": 6.5,
      "cvss": {
        "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:R/S:U/C:N/I:H/A:N",
        "version": "3.1"
      },
      "from": ["AgilePlus@1.0.0", "express@4.17.1", "lodash@4.17.20"],
      "package": "lodash",
      "version": "4.17.20",
      "fixedIn": ["4.17.21"],
      "introduced": "0",
      "patched": false,
      "fix": "Upgrade to 4.17.21",
      "cve": "CVE-2021-23337"
    },
    // ... more vulnerabilities
  ],
  "summary": "3 vulnerabilities (1 high, 2 medium)"
}
```

### 2.3: Understanding Vulnerability Fields

| Field | Meaning | Example |
|-------|---------|---------|
| `id` | Snyk vulnerability ID | `SNYK-JS-LODASH-1234567` |
| `title` | Vulnerability name | `Prototype Pollution in lodash` |
| `severity` | Severity level | `high`, `critical`, `medium`, `low` |
| `cvssScore` | CVSS v3.1 score (0-10) | `6.5` |
| `from` | Dependency chain | `["app@1.0.0", "express", "lodash"]` |
| `package` | Vulnerable package | `lodash` |
| `version` | Vulnerable version | `4.17.20` |
| `fixedIn` | Version with fix | `["4.17.21"]` |
| `cve` | CVE identifier | `CVE-2021-23337` |

### 2.4: Example: Reading a Vulnerability

```json
{
  "id": "SNYK-JS-LODASH-1234567",
  "title": "Prototype Pollution in lodash",
  "severity": "high",
  "cvssScore": 6.5,
  "package": "lodash",
  "version": "4.17.20",
  "fixedIn": ["4.17.21"],
  "description": "A vulnerability exists in lodash versions before 4.17.21. The vulnerability allows an attacker to modify prototype objects...",
  "from": ["AgilePlus", "express@4.17.1", "lodash@4.17.20"],
  "cve": "CVE-2021-23337"
}
```

**How to Read This:**
1. **Package:** `lodash` version `4.17.20` is vulnerable
2. **Severity:** This is a `high` priority finding (CVSS 6.5)
3. **Impact:** Prototype Pollution allows attacker to modify object prototypes
4. **Fix:** Upgrade to `4.17.21` or later
5. **Location:** Included via `express` in `AgilePlus`
6. **CVE:** Tracked as CVE-2021-23337

---

## Part 3: Understanding Severity & CVSS

### 3.1: Severity Levels

```
CRITICAL (9.0-10.0 CVSS)
├─ Remote Code Execution (RCE)
├─ Authentication Bypass
└─ Data Exfiltration
   Action: Fix immediately (0-7 days)

HIGH (7.0-8.9 CVSS)
├─ Privilege Escalation
├─ Significant Data Breach
└─ Denial of Service (DoS)
   Action: Fix soon (7-30 days)

MEDIUM (4.0-6.9 CVSS)
├─ Minor Data Exposure
├─ Partial Denial of Service
└─ Cross-Site Scripting (XSS)
   Action: Plan fix (30-90 days)

LOW (0.1-3.9 CVSS)
├─ Information Disclosure
├─ Best Practice Violation
└─ Weak Defaults
   Action: Backlog (90+ days)
```

### 3.2: CVSS Score Interpretation

CVSS Score: 0.0 (no impact) to 10.0 (critical)

```
0.0   = No risk
0.1-3.9 = Low
4.0-6.9 = Medium
7.0-8.9 = High
9.0-10.0 = Critical
```

**Example CVSS Scores:**
- `9.8` = Network-accessible RCE with no auth required (critical)
- `7.5` = Network-accessible DoS or data leak (high)
- `5.3` = Local privilege escalation (medium)
- `2.7` = Minor information disclosure (low)

---

## Part 4: When to Fix vs. Suppress

### 4.1: Decision Matrix

| Situation | Action | Reason |
|-----------|--------|--------|
| `severity: critical` | FIX immediately | RCE, auth bypass, data leak |
| `severity: high` + used in prod | FIX within 30 days | Exploit readily available |
| `severity: high` + dev-only | SUPPRESS + justification | Not exposed to users |
| `severity: medium` + no workaround | PLAN in roadmap | Address in next sprint |
| `severity: medium` + has workaround | SUPPRESS + document | Mitigated by other controls |
| `severity: low` | SUPPRESS or backlog | Low business impact |

### 4.2: Fix vs. Suppress Decision Flow

```
Found a vulnerability?
  │
  ├─ Is it critical? ────────────► YES ──────► FIX immediately
  │                                           (no suppression)
  ├─ Is it in production code? ──► YES ──┐
  │                                      ├─► Check if easy fix
  │                                      │
  │                                      ├─ Easy fix (dependency upgrade)?
  │                                      │  └─► FIX
  │                                      │
  │                                      └─ Hard fix (architectural)?
  │                                         └─► Create AgilePlus work item
  │                                             Add .snyk policy
  │                                             Schedule fix
  │
  ├─ Is it dev-only? ───────────► YES ──┐
  │                                      ├─► SUPPRESS
  │                                      │   (justification: dev-only)
  │                                      │
  ├─ Is there a workaround? ────► YES ──┐
  │                                      ├─► SUPPRESS
  │                                      │   (document workaround)
  │
  └─ Default: Create backlog item, SUPPRESS if not urgent
```

---

## Part 5: Creating .snyk Policy Files

### 5.1: When to Create .snyk Files

Create a `.snyk` policy file when you:
- Suppress a vulnerability (with justification)
- Accept risk after review
- Document security exceptions
- Have a remediation plan

### 5.2: .snyk File Format

Create `.snyk` in the repository root:

```bash
cat > /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.snyk << 'EOF'
# Snyk Policy as Code
# This file is used to manage Snyk suppressions and policy

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
        reason: Upgrading lodash to 4.17.21 breaks express compatibility
        expires: 2026-06-30
        created: 2026-03-30T14:40:00Z

  SNYK-RUST-OPENSSL-5678901:
    - '*':
        reason: Dev dependency only, not in production binary
        expires: 2026-12-31
        created: 2026-03-30T14:40:00Z

patch: {}
EOF
```

### 5.3: .snyk File Explained

```yaml
version: v1.19.0                        # Snyk policy version (use this)

allow-licenses:                          # Allowed licenses
  - MIT
  - Apache-2.0

ignore:                                  # Suppressions
  SNYK-ID-XXXXX:                        # Vulnerability ID (from JSON report)
    - '*':                              # Applies to all paths
        reason: "Why we're suppressing" # Justification (required)
        expires: 2026-06-30             # Expiration date (required)
        created: 2026-03-30T14:40:00Z   # When created
```

### 5.4: Example: Adding a Suppression

If you have a vulnerability you want to suppress:

1. Get the vulnerability ID from the JSON report:
   ```bash
   cat .snyk-reports/AgilePlus.json | jq '.vulnerabilities[0].id'
   # Output: SNYK-JS-LODASH-1234567
   ```

2. Add to `.snyk`:
   ```yaml
   ignore:
     SNYK-JS-LODASH-1234567:
       - '*':
           reason: "Requires express upgrade to 5.0, scheduled for Q2 2026"
           expires: 2026-06-30
           created: 2026-03-30
   ```

### 5.5: Suppression Justification Examples

**Good Justifications:**

```yaml
# Dev dependency, not in production
reason: "Dev dependency only (test fixture), not in production binary"

# Workaround in place
reason: "Input validation in place, vulnerability requires direct file write access"

# Patch in progress
reason: "Upgrade to 4.17.21 in progress, see AgilePlus work item #123"

# Low risk + business constraint
reason: "Enterprise dependency, required for compliance, CVE is low impact"

# Architectural limitation
reason: "Legacy system constraint, cannot upgrade until major refactor planned for 2027"
```

**Bad Justifications:**

```yaml
# Too vague
reason: "Will fix later"

# No expiry
reason: "Known issue"  # ← Missing expiry date

# No context
reason: "Accepted risk"  # ← Why?

# Procrastination
reason: "Not my problem"
```

### 5.6: Committing .snyk Files

After creating `.snyk` files:

```bash
# Check status
git status -s

# Stage all .snyk files
git add **/.snyk

# Verify
git diff --cached

# Commit with context
git commit -m "security: add Snyk policy suppressions with justifications

- AgilePlus: Suppress lodash RCE (dev-only), expires 2026-06-30
- phenotype-infrakit: Accept medium severity TCP leak, workaround documented
- See .snyk files for full policy and reasoning"
```

---

## Part 6: Creating PR for Results Review

### 6.1: Summary Report for PR

Create a summary for code review:

```bash
cat > /tmp/snyk-pr-summary.txt << 'EOF'
## Snyk Security Scan Deployment

This PR introduces Snyk security scanning across the Phenotype ecosystem.

### Summary
- **Total Repositories Scanned:** 30
- **Total Vulnerabilities:** 45
  - Critical: 2
  - High: 8
  - Medium: 18
  - Low: 17

### Critical Findings (Require Immediate Action)
1. **AgilePlus** — CVE-2024-XXXXX (Prototype Pollution in lodash)
   - Severity: High
   - Fix: Upgrade lodash from 4.17.20 to 4.17.21
   - Status: Scheduled for sprint ending 2026-04-15

2. **phenotype-infrakit** — CVE-2024-YYYYY (Buffer overflow in crypto)
   - Severity: Medium (only in development, not in binary)
   - Fix: Upgrade to 1.5.0
   - Status: Suppressed (dev-only), expires 2026-06-30

### Tier 1 Repository Status
- ✅ **heliosCLI**: Clean (0 vulnerabilities)
- ⚠️ **AgilePlus**: 14 vulnerabilities (1 critical, 4 high, 6 medium, 3 low)
- ⚠️ **phenotype-infrakit**: 5 vulnerabilities (2 medium, 3 low)

### Deliverables
- `.snyk-reports/report.txt` — Summary of all findings
- `.snyk-reports/*.json` — Detailed per-repository reports
- `**/.snyk` — Policy files with suppressions and justifications

### Next Steps
1. ✅ Review critical findings (this PR)
2. ⏳ Create AgilePlus work items for high-severity findings
3. ⏳ Deploy GitHub Actions workflows (separate PR)
4. ⏳ Enable automated nightly scans
EOF

cat /tmp/snyk-pr-summary.txt
```

### 6.2: PR Checklist

Before creating the PR:

- [ ] All `.snyk` files created with justifications
- [ ] `.snyk-reports/report.txt` reviewed
- [ ] Critical findings understood
- [ ] No token or secrets in reports
- [ ] All policy files validated:
  ```bash
  git ls-files | grep "\.snyk$" | xargs -I {} sh -c 'echo "Validating {}"; cat {}'
  ```

### 6.3: Create PR

```bash
# Stage all changes
git add .snyk-reports/ **/.snyk

# Create commit
git commit -m "security: deploy Snyk security scanning across repos

Introduces Snyk security scans for Tier 1 repos (AgilePlus, heliosCLI, phenotype-infrakit) and secondary repos.

Summary:
- 30 repositories scanned
- 45 vulnerabilities found (2 critical, 8 high, 18 medium, 17 low)
- Critical findings in AgilePlus and phenotype-infrakit require follow-up
- .snyk policy files created with justifications for suppressions
- Reports available in .snyk-reports/ directory

Next Steps:
- Review critical findings
- Create AgilePlus work items for high/critical severity
- Deploy GitHub Actions workflows (separate PR)

Closes #XXX (link to original Snyk spec)"

# Create PR
gh pr create \
  --title "security: deploy Snyk security scanning" \
  --body-file /tmp/snyk-pr-summary.txt \
  --base main \
  --head snyk-deployment

# View PR
gh pr view
```

---

## Part 7: Common Review Scenarios

### Scenario 1: Critical Vulnerability Found

**Finding:**
```json
{
  "id": "SNYK-RUST-OPENSSL-HEARTBLEED",
  "severity": "critical",
  "package": "openssl",
  "version": "1.0.1",
  "fixedIn": ["1.0.2", "1.1.0"],
  "description": "Heartbleed vulnerability allows memory disclosure"
}
```

**Action:**
```bash
# 1. This MUST be fixed, not suppressed
# 2. Create urgent AgilePlus work item
# 3. Schedule for immediate release

cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# Update Cargo.toml
# Find: openssl = "1.0.1"
# Replace: openssl = "1.1.0"

# Test
cargo test

# Commit
git commit -m "security: upgrade openssl from 1.0.1 (heartbleed) to 1.1.0

Fixes SNYK-RUST-OPENSSL-HEARTBLEED critical vulnerability"
```

### Scenario 2: Medium-Severity Dev-Only Dependency

**Finding:**
```json
{
  "id": "SNYK-PYTHON-PYTEST-XXXX",
  "severity": "medium",
  "package": "pytest",
  "version": "6.0.0",
  "description": "Arbitrary code execution in test fixtures"
}
```

**Action:**
```bash
# 1. This is dev-only, can be suppressed
# 2. Create .snyk policy file
# 3. Document in PR

cat > /Users/kooshapari/CodeProjects/Phenotype/repos/agileplus-agents/.snyk << 'EOF'
version: v1.19.0

ignore:
  SNYK-PYTHON-PYTEST-XXXX:
    - '*':
        reason: "Dev dependency only (test framework), not in production"
        expires: 2027-03-30
        created: 2026-03-30
EOF

git add agileplus-agents/.snyk
```

### Scenario 3: Known Vulnerability with Workaround

**Finding:**
```json
{
  "id": "SNYK-JS-SERIALIZE-JAVASCRIPT-1234",
  "severity": "high",
  "package": "serialize-javascript",
  "version": "3.0.0",
  "description": "XSS via unsafe regex"
}
```

**Action:**
```bash
# 1. Document workaround
# 2. Create suppression with expiry
# 3. Schedule upgrade for next sprint

cat > /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.snyk << 'EOF'
version: v1.19.0

ignore:
  SNYK-JS-SERIALIZE-JAVASCRIPT-1234:
    - '*':
        reason: "XSS requires malicious input in template + Node.js execution
        Mitigated by: strict input validation + CSP headers
        Fix: Upgrade to 5.0+ when Webpack 6 support available (ETA Q3 2026)"
        expires: 2026-09-30
        created: 2026-03-30
EOF

git add AgilePlus/.snyk
```

---

## Part 8: After Review

### 8.1: Merge PR

Once reviewed and approved:

```bash
gh pr merge --squash --delete-branch
```

### 8.2: Create Follow-Up Work Items

For each high/critical finding:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

agileplus specify \
  --title "security: upgrade lodash to fix prototype pollution" \
  --description "Snyk scan found CVE-2021-23337 in lodash 4.17.20. Upgrade to 4.17.21 to fix."
```

### 8.3: Proceed to GitHub Integration

Next: GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md

---

## Reference

- **Snyk Vulnerability Database:** https://snyk.io/vuln/
- **CVSS Calculator:** https://www.first.org/cvss/calculator/3.1
- **Snyk Policy as Code:** https://docs.snyk.io/cli/manage-policy-as-code
- **CVE Details:** https://nvd.nist.gov/vuln/

---

**Last Updated:** 2026-03-30
**Status:** Ready for GitHub integration
