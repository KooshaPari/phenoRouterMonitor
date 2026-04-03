# Snyk Security Automation Setup Guide

Complete guide for deploying Snyk vulnerability scanning across the Phenotype ecosystem.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites)
3. [Getting Snyk Token](#getting-snyk-token)
4. [Local Setup](#local-setup)
5. [Automated Deployment](#automated-deployment)
6. [GitHub Integration](#github-integration)
7. [Policy Configuration](#policy-configuration)
8. [Running Scans](#running-scans)
9. [Interpreting Results](#interpreting-results)
10. [Troubleshooting](#troubleshooting)
11. [CI/CD Integration](#cicd-integration)

## Quick Start

**For immediate deployment (requires Snyk token):**

```bash
# 1. Get Snyk token (see: Getting Snyk Token section)
export SNYK_TOKEN="your-token-here"

# 2. Run deployment script for Tier 1 repos
cd /Users/kooshapari/CodeProjects/Phenotype/repos
./scripts/snyk-deploy.sh "$SNYK_TOKEN" AgilePlus phenotype-infrakit heliosCLI

# 3. Review reports in .snyk-reports directory
ls -la .snyk-reports/

# 4. Commit .snyk policy files
git add AgilePlus/.snyk phenotype-infrakit/.snyk heliosCLI/.snyk
git commit -m "chore(security): add Snyk policy files"
```

## Prerequisites

### Required Software

- **Snyk CLI**: Install via npm or Homebrew
- **Node.js 16+**: Required by Snyk CLI
- **jq**: JSON parser for report processing
- **Git**: For policy file management

### Installation

```bash
# macOS
brew install snyk jq

# Linux
npm install -g snyk
apt-get install jq  # or: yum install jq

# Verify installation
snyk --version
jq --version
```

## Getting Snyk Token

### Step 1: Create Snyk Account

1. Go to https://app.snyk.io/auth/register
2. Sign up with GitHub or email
3. Complete account setup

### Step 2: Generate API Token

1. Navigate to Account Settings: https://app.snyk.io/account/settings
2. Scroll to **API Token** section
3. Click "Generate Token"
4. Copy the token (keep it private!)

### Step 3: Secure Token Storage

**Option A: Environment Variable (Local Development)**

```bash
# Add to ~/.bashrc, ~/.zshrc, or ~/.bash_profile
export SNYK_TOKEN="your-token-here"

# Reload shell
source ~/.bashrc
```

**Option B: GitHub Organization Secret**

1. Go to: https://github.com/KooshaPari/repos/settings/secrets
2. Click "New repository secret" (or "New organization secret" for all repos)
3. Name: `SNYK_TOKEN`
4. Value: Paste your Snyk API token
5. Click "Add secret"

**Option C: GitHub Repository Secrets (per-repo)**

For individual repositories:

1. Go to: https://github.com/KooshaPari/REPO/settings/secrets
2. Click "New repository secret"
3. Name: `SNYK_TOKEN`
4. Value: Paste your token
5. Save

## Local Setup

### Authenticate Locally

```bash
# Using environment variable
export SNYK_TOKEN="your-token-here"
snyk auth $SNYK_TOKEN

# Or authenticate interactively
snyk auth

# Verify authentication
snyk whoami
```

### Test Authentication

```bash
# This should show your Snyk account info
snyk whoami

# Expected output:
# Your account name
# your-email@example.com
```

## Automated Deployment

### Using snyk-deploy.sh

The deployment script automates scanning across multiple repositories.

**Syntax:**

```bash
./scripts/snyk-deploy.sh <snyk-token> [repo1] [repo2] ...
```

**Examples:**

```bash
# Scan Tier 1 repos (default)
./scripts/snyk-deploy.sh $SNYK_TOKEN

# Scan specific repos
./scripts/snyk-deploy.sh $SNYK_TOKEN AgilePlus phenotype-infrakit heliosCLI

# Scan all repos with explicit token
SNYK_TOKEN="token-here" ./scripts/snyk-deploy.sh "" AgilePlus phenotype-infrakit agentapi-plusplus
```

### Script Features

- **Non-blocking**: Continues if one repo fails
- **Policy generation**: Creates `.snyk` file automatically
- **JSON reports**: Generates machine-readable reports
- **SARIF output**: GitHub Code Scanning compatible
- **Detailed summary**: Aggregates results across repos

### Deployment Output

Reports are generated in `.snyk-reports/`:

```
.snyk-reports/
├── snyk-deployment-20240330_102030.txt    # Summary report
├── AgilePlus/
│   ├── test-20240330_102030.json         # Raw test results
│   ├── test-20240330_102030.txt          # Human-readable report
│   └── snyk.sarif                        # GitHub Code Scanning format
├── phenotype-infrakit/
│   ├── test-20240330_102030.json
│   ├── test-20240330_102030.txt
│   └── snyk.sarif
└── heliosCLI/
    ├── test-20240330_102030.json
    ├── test-20240330_102030.txt
    └── snyk.sarif
```

## GitHub Integration

### Enable Snyk in GitHub Settings

1. **For Organization** (all repos):
   - Go: https://github.com/organizations/KooshaPari/settings/secrets
   - Add `SNYK_TOKEN` secret
   - All repos inherit this secret

2. **For Single Repository**:
   - Go: https://github.com/KooshaPari/REPO/settings/secrets
   - Add `SNYK_TOKEN` secret
   - Only this repo can access it

### Deploy GitHub Actions Workflow

The workflow file is pre-configured at `.github/workflows/snyk-scan.yml`:

**Features:**
- Triggers on: PRs, main push, nightly schedule
- Generates SARIF reports for Code Scanning
- Auto-comments on PRs with results
- Creates fix PRs for Tier 1 repos
- Fails on critical vulnerabilities

**Deploy to all repos:**

```bash
# Copy workflow to each repository
for repo in AgilePlus phenotype-infrakit heliosCLI; do
  mkdir -p "$repo/.github/workflows"
  cp .github/workflows/snyk-scan.yml "$repo/.github/workflows/"
  cd "$repo"
  git add .github/workflows/snyk-scan.yml
  git commit -m "chore(ci): add Snyk security scan workflow"
  cd ..
done
```

## Policy Configuration

### Create .snyk Policy Files

**Method 1: Auto-generate (Recommended)**

```bash
# Use policy generator script
./scripts/snyk-policy-generator.sh ./AgilePlus
./scripts/snyk-policy-generator.sh ./phenotype-infrakit --type=rust
./scripts/snyk-policy-generator.sh ./heliosCLI --type=node
```

**Method 2: Manual policy generation**

```bash
# For Rust projects
./scripts/snyk-policy-generator.sh ./AgilePlus --type=rust

# For Node.js projects
./scripts/snyk-policy-generator.sh ./heliosApp --type=node

# For Python projects
./scripts/snyk-policy-generator.sh ./phench --type=python

# For Go projects
./scripts/snyk-policy-generator.sh ./thegent --type=go
```

**Method 3: Generate with current suppressions**

```bash
# Scans for vulnerabilities and auto-suppresses them (with 1-year expiry)
./scripts/snyk-policy-generator.sh ./AgilePlus --suppress-all
```

### .snyk Policy File Format

**Location:** `.snyk` in project root

**Example:**

```yaml
version: v1.25.0

# Suppress specific vulnerabilities
ignore:
  'SNYK-JS-LODASH-1234567':
    - '> lodash':
        reason: 'Used safely, not vulnerable'
        expires: '2025-03-30T00:00:00.000Z'

# Exclude directories from scanning
exclude:
  global:
    - /node_modules
    - /vendor
    - /dist
    - /.git

patch: {}
fix: {}

cli:
  checkForUpdates: false
```

### Edit Policy Files

1. Run `snyk test` locally to find vulnerability IDs
2. Add entries to `.snyk`:

```yaml
ignore:
  'VULN-ID-HERE':
    - '> package-name':
        reason: 'Why this is approved'
        expires: '2025-12-31T00:00:00.000Z'
```

3. Commit policy file:

```bash
git add .snyk
git commit -m "chore(security): update Snyk suppressions for VULN-ID"
```

## Running Scans

### Local Scanning

**Basic test:**

```bash
cd /path/to/repo
snyk test
```

**With JSON output:**

```bash
snyk test --json > snyk-report.json
```

**With severity threshold:**

```bash
snyk test --severity-threshold=high
```

**Monitor for continuous updates:**

```bash
snyk monitor
```

### Batch Scanning

Using deployment script:

```bash
# Scan all Tier 1 repos
./scripts/snyk-deploy.sh $SNYK_TOKEN

# Scan specific subset
./scripts/snyk-deploy.sh $SNYK_TOKEN AgilePlus heliosCLI

# Scan Tier 2 repos
./scripts/snyk-deploy.sh $SNYK_TOKEN agent-wave agentapi-plusplus KaskMan
```

## Interpreting Results

### Severity Levels

| Level | Description | Risk |
|-------|-------------|------|
| **Critical** | Exploitable vulnerability with public exploit | Block deployment |
| **High** | Exploitable vulnerability in common scenarios | Requires review |
| **Medium** | Vulnerability requires specific conditions | Can defer 30 days |
| **Low** | Difficult to exploit; minimal impact | Can defer 90 days |

### Understanding Reports

**JSON Report Structure:**

```json
{
  "vulnerabilities": [
    {
      "id": "SNYK-JS-LODASH-1234567",
      "title": "Prototype Pollution",
      "severity": "high",
      "description": "...",
      "fixedIn": ["4.17.21"],
      "from": ["lodash@4.17.20"]
    }
  ]
}
```

**Key Fields:**
- `id`: Snyk vulnerability identifier
- `severity`: Critical, High, Medium, Low
- `fixedIn`: Versions where fix is available
- `from`: Dependency chain leading to vulnerability

### Fix Options

1. **Update**: Upgrade to patched version
2. **Suppress**: Acknowledge and document risk
3. **Fix PR**: Use `snyk fix` to auto-patch

## Troubleshooting

### Snyk CLI Not Found

```bash
# Install via npm (global)
npm install -g snyk

# Or Homebrew (macOS)
brew install snyk

# Verify
which snyk
snyk --version
```

### Authentication Failed

```bash
# Check token validity
snyk whoami

# Re-authenticate
snyk auth  # Interactive prompt
# OR
snyk auth $SNYK_TOKEN
```

### No Vulnerabilities Found (but expected some)

```bash
# Run with verbose output
snyk test --debug

# Check if .snyk policy is suppressing them
cat .snyk
```

### SARIF Upload Fails

```bash
# Verify SARIF file exists
ls -la snyk.sarif

# Check file format
jq . snyk.sarif | head

# Manually upload in GitHub UI
# Settings > Code Scanning > Upload SARIF
```

### Timeouts on Large Repos

```bash
# Increase timeout
snyk test --timeout 30m

# Or scan specific paths
snyk test ./src

# Skip specific directories
snyk test --exclude=node_modules,vendor
```

## CI/CD Integration

### GitHub Actions

Workflow is pre-configured in `.github/workflows/snyk-scan.yml`.

**Manual deployment to repo:**

```bash
# For single repo
cd /path/to/repo
mkdir -p .github/workflows
cp /Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/snyk-scan.yml .github/workflows/
git add .github/workflows/snyk-scan.yml
git commit -m "chore(ci): add Snyk security scan"
git push
```

### Workflow Triggers

- **Pull Requests**: On PR open/update (test only)
- **Main Branch**: On push to main (test + optional fix PR)
- **Nightly**: Daily at 2 AM UTC
- **Manual**: Via workflow_dispatch

### Monitoring Scan Results

1. **In GitHub**:
   - Security tab > Code Scanning
   - Shows all vulnerabilities across repos
   - Comments appear on PRs

2. **In Snyk Dashboard**:
   - https://app.snyk.io/dashboard
   - Monitor project health
   - Set up alerts

3. **Via API**:

```bash
# Get Snyk test results programmatically
curl -X GET "https://api.snyk.io/v1/test" \
  -H "Authorization: token $SNYK_TOKEN"
```

## Deployment Timeline

### Phase 1 (Week 1): Tier 1 Repos

- AgilePlus
- phenotype-infrakit
- heliosCLI

**Steps:**
1. Generate tokens
2. Run `snyk-deploy.sh` for each repo
3. Review and commit `.snyk` files
4. Deploy GitHub Actions workflow
5. Monitor initial scan results

### Phase 2 (Week 2): Tier 2 Repos

- agent-wave
- agentapi-plusplus
- KaskMan

### Phase 3 (Week 3): Tier 3 Repos

- forgecode
- zen
- vibeproxy

### Phase 4+: Monitoring & Maintenance

- Daily scan results
- Fix PRs for critical issues
- Policy updates
- Alert configuration

## Support & Documentation

- **Snyk Docs**: https://docs.snyk.io
- **CLI Reference**: https://docs.snyk.io/snyk-cli/cli-reference
- **Policy Files**: https://docs.snyk.io/policies/the-snyk-policy-file
- **GitHub Integration**: https://docs.snyk.io/integrations/git-repositories/github
- **Issues/Questions**: Create issue in repos with `[snyk]` tag

## Quick Reference

| Task | Command |
|------|---------|
| Authenticate | `snyk auth $SNYK_TOKEN` |
| Test local repo | `snyk test` |
| Generate policy | `./scripts/snyk-policy-generator.sh ./REPO` |
| Deploy to repos | `./scripts/snyk-deploy.sh $SNYK_TOKEN` |
| Deploy workflow | Copy `.github/workflows/snyk-scan.yml` to repo |
| Monitor in Snyk | `snyk monitor` |
| Auto-fix vulnerabilities | `snyk fix --force` |
| View results | GitHub: Security tab > Code Scanning |
| Check token | `snyk whoami` |
