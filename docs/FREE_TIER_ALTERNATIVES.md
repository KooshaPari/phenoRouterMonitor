# GitHub Apps: Free-Tier Alternatives & Setup Guide

**Last Updated:** 2026-03-30  
**Repository:** phenotype-infrakit  
**Goal:** Comprehensive free alternatives to paid GitHub Apps (CodeRabbit Pro, Copilot, SonarCloud Pro, etc.)

---

## Executive Summary

This guide documents **10+ free-tier GitHub Apps and services** that provide code review, security scanning, dependency management, and AI assistance without paid tiers. All recommended tools are:
- ✓ Free forever (public repos)
- ✓ Open source or widely adopted
- ✓ Actively maintained
- ✓ Ready for immediate deployment

---

## 1. Dependabot (Built-in, Free)

### Why It's Free
- **GitHub Native:** Part of GitHub's free offering; no separate app needed
- **No Cost:** Free for all repo types (public/private)
- **No Limits:** Unlimited dependencies, unlimited update checks

### Setup Instructions

1. **Enable in repository:**
   ```bash
   mkdir -p .github
   touch .github/dependabot.yml
   ```

2. **Configure `.github/dependabot.yml`:**
   ```yaml
   version: 2
   updates:
     # NPM/TypeScript
     - package-ecosystem: "npm"
       directory: "/"
       schedule:
         interval: "weekly"
         day: "monday"
         time: "09:00"
       groups:
         development:
           dependency-type: "development"
         production:
           dependency-type: "production"
           update-types: ["patch", "minor"]
       auto-merge: true
       pull-request-branch-name:
         separator: "/"
       reviewers: ["@kooshapari"]

     # Cargo/Rust
     - package-ecosystem: "cargo"
       directory: "/"
       schedule:
         interval: "weekly"
       groups:
         dependencies:
           dependency-type: "production"
           update-types: ["patch", "minor"]
       auto-merge: true

     # Docker
     - package-ecosystem: "docker"
       directory: "/"
       schedule:
         interval: "weekly"
       auto-merge: true

     # GitHub Actions
     - package-ecosystem: "github-actions"
       directory: "/"
       schedule:
         interval: "weekly"
       auto-merge: true
   ```

3. **Configure Branch Protection (Optional):**
   - Allow auto-merge of Dependabot PRs if all checks pass
   - Settings → Branches → Branch Protection Rules → Allow auto-merge

4. **Test:**
   - Create a test dependency update
   - Verify Dependabot opens PR
   - Monitor auto-merge (if enabled)

### Free Tier Limits
- ✓ Unlimited updates
- ✓ Unlimited groups
- ✓ Unlimited scheduling rules
- ✓ Auto-merge included
- ✓ Security alerts included

### Cost: $0/month

---

## 2. GitHub CodeQL (Built-in, Free for Public)

### Why It's Free
- **Native GitHub Integration:** Part of GitHub Advanced Security (free for public repos)
- **No Separate Installation:** Enabled by default for public repos
- **Unlimited Scans:** No per-scan charges

### Setup Instructions

1. **Create `.github/workflows/codeql.yml`:**
   ```yaml
   name: CodeQL
   on:
     push:
       branches: ["main", "develop"]
     pull_request:
       branches: ["main"]
     schedule:
       - cron: "0 0 * * 0"  # Weekly on Sunday

   jobs:
     analyze:
       name: Analyze
       runs-on: ubuntu-latest
       permissions:
         security-events: write

       strategy:
         fail-fast: false
         matrix:
           language: ["python", "javascript", "go"]

       steps:
         - name: Checkout repository
           uses: actions/checkout@v4

         - name: Initialize CodeQL
           uses: github/codeql-action/init@v2
           with:
             languages: ${{ matrix.language }}
             queries: security-and-quality

         - name: Autobuild
           uses: github/codeql-action/autobuild@v2

         - name: Perform CodeQL Analysis
           uses: github/codeql-action/analyze@v2
   ```

2. **Configure Security Settings:**
   - Settings → Code Security and Analysis → CodeQL Analysis → Enable

3. **Review Results:**
   - Security tab → Code Scanning → View CodeQL alerts

4. **Fail on Severity (Optional):**
   - Create custom rule to block PRs on high/critical vulnerabilities

### Free Tier Limits
- ✓ Unlimited scans (public repos)
- ✓ All query types
- ✓ Supports: Python, JavaScript, Java, C/C++, C#, Go, Ruby, TypeScript
- ✓ Custom queries allowed
- ⚠️ Private repos: requires GitHub Enterprise (paid)

### Cost: $0/month (public repos)

---

## 3. Renovate (Free, Self-Hosted or GitHub App)

### Why It's Free
- **Open Source:** MIT licensed, fully open source
- **No SaaS Charge:** Self-host for free, or use free tier of hosted service
- **Better Than Dependabot:** More granular grouping, better conflict resolution

### Setup Option A: GitHub App (Recommended)

1. **Install Renovate App:**
   - Visit: https://github.com/marketplace/renovate
   - Click "Install for free"
   - Select repositories to enable

2. **Create `renovate.json` in repo root:**
   ```json
   {
     "$schema": "https://docs.renovatebot.com/renovate-schema.json",
     "extends": [
       "config:base",
       ":enableVulnerabilityAlerts"
     ],
     "schedule": [
       {
         "matchUpdateTypes": ["patch", "minor"],
         "schedule": ["every weekday"]
       },
       {
         "matchUpdateTypes": ["major"],
         "schedule": ["before 3am on Monday"]
       }
     ],
     "groupName": "all dependencies",
     "groupSlug": "all-deps",
     "automerge": true,
     "automergeType": "pr",
     "automergeStrategy": "squash"
   }
   ```

3. **Configure Branch Protection:**
   - Allow Renovate to auto-merge if all checks pass

### Setup Option B: Self-Hosted (Advanced)

1. **Prerequisites:**
   - GitHub Personal Access Token with `repo`, `workflow` scopes
   - Docker or Node.js environment
   - GitHub Actions or external CI/CD

2. **Deploy Renovate Bot:**
   ```bash
   # Docker (recommended)
   docker run -e LOG_LEVEL=debug \
     -e RENOVATE_TOKEN=$GH_TOKEN \
     -e RENOVATE_AUTODISCOVER=true \
     -e RENOVATE_AUTODISCOVER_FILTER=KooshaPari/* \
     renovate/renovate:latest
   ```

3. **Or via GitHub Actions (free runner minutes):**
   ```yaml
   name: Renovate
   on:
     schedule:
       - cron: "0 0 * * *"
     workflow_dispatch:

   jobs:
     renovate:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: renovatebot/github-action@v37
           with:
             token: ${{ secrets.GITHUB_TOKEN }}
   ```

### Free Tier Limits (App)
- ✓ Unlimited repositories
- ✓ Unlimited dependency updates
- ✓ Auto-merge included
- ✓ Custom schedules
- ✓ Grouping rules
- ⚠️ Community tier may have request limits

### Cost: $0/month

---

## 4. Snyk (Free Tier for Public Repos)

### Why It's Free
- **Generous Free Tier:** Unlimited public repos, test runs, and fixes
- **Security Focus:** Specializes in vulnerability detection
- **Best-in-Class:** Often detects vulnerabilities before CVE publication

### Setup Instructions

1. **Install Snyk GitHub App:**
   - Visit: https://github.com/marketplace/snyk
   - Click "Install for free"
   - Select repositories

2. **Create `.snyk` policy file (optional):**
   ```yaml
   version: v1.25.0
   rules:
     - id: node_modules
       type: file
       pattern: node_modules/**
       action: ignore
   ```

3. **Configure GitHub Integration:**
   - Snyk dashboard (snyk.io) → Settings → Integrations
   - Connect GitHub organization
   - Enable automatic PR fixing

4. **Monitor Results:**
   - Snyk dashboard shows: vulnerabilities, dependencies, fix PRs

### Free Tier Limits
- ✓ Unlimited public repos
- ✓ Unlimited dependency scans
- ✓ Unlimited test runs
- ✓ Auto-fix PRs
- ✓ Email notifications
- ⚠️ Private repos: 5-project limit (free tier)
- ⚠️ No Docker scanning (paid only)

### Cost: $0/month (public repos)

---

## 5. SonarCloud (Free Tier for Open Source)

### Why It's Free
- **Open Source Friendly:** 100% free for public repos
- **Code Quality Focus:** Detects bugs, code smells, security hotspots
- **Excellent Metrics:** Coverage, duplication, maintainability index

### Setup Instructions

1. **Sign Up:**
   - Visit: https://sonarcloud.io
   - Click "Sign up" → Choose GitHub
   - Authorize SonarCloud

2. **Analyze Repository:**
   - SonarCloud dashboard → "+" button → Analyze new project
   - Select repository from GitHub
   - Choose analysis method

3. **Method A: Automatic Analysis (GitHub Actions)**
   ```yaml
   name: SonarCloud
   on:
     push:
       branches: ["main", "develop"]
     pull_request:
       types: [opened, synchronize, reopened]

   jobs:
     sonarcloud:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
           with:
             fetch-depth: 0  # Full history for better analysis
         - name: SonarCloud Scan
           uses: SonarSource/sonarcloud-github-action@master
           env:
             GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
             SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
   ```

4. **Method B: GitHub Actions Analysis (via SonarScanner)**
   ```yaml
   name: SonarCloud Analysis
   on:
     push:
       branches: ["main"]
     pull_request:

   jobs:
     scan:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
           with:
             fetch-depth: 0
         - name: Run tests
           run: npm test -- --coverage
         - uses: SonarSource/sonarcloud-github-action@master
           env:
             GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
             SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
   ```

5. **Create `sonar-project.properties` (if needed):**
   ```properties
   sonar.projectKey=KooshaPari_phenotype-infrakit
   sonar.organization=kooshapari
   sonar.sources=src
   sonar.tests=tests
   sonar.exclusions=**/test/**,**/node_modules/**
   sonar.javascript.lcov.reportPaths=coverage/lcov.info
   sonar.python.coverage.reportPath=coverage.xml
   ```

6. **View Results:**
   - Pull requests show SonarCloud quality gates
   - Dashboard shows metrics over time

### Free Tier Limits
- ✓ Unlimited public repos
- ✓ Unlimited scans
- ✓ All languages supported
- ✓ Quality gates
- ✓ Custom rules
- ⚠️ Private repos: 7-day free trial, then paid

### Cost: $0/month (public repos)

---

## 6. DeepSource (Free Tier for Open Source)

### Why It's Free
- **Open Source Priority:** Free for public repos
- **Autofix Enabled:** Automatically creates fix PRs
- **Multiple Languages:** Supports 15+ languages

### Setup Instructions

1. **Sign Up:**
   - Visit: https://deepsource.io
   - Sign in with GitHub
   - Authorize DeepSource

2. **Activate Repository:**
   - Dashboard → "Activate Repository"
   - Select from GitHub
   - Grant necessary permissions

3. **Create `.deepsource.yaml` (optional):**
   ```yaml
   version: 1
   python_targets:
     - 3.9
   test_patterns:
     - tests/**/*.py
     - test_*.py
   exclude_patterns:
     - __pycache__/**
     - .venv/**
   ```

4. **Monitor Results:**
   - DeepSource PR comments show issues
   - Dashboard shows overall health

### Free Tier Limits
- ✓ Unlimited public repos
- ✓ Unlimited issues
- ✓ Autofix PRs
- ✓ All languages (15+)
- ⚠️ Private repos: limited to 1 (paid tier for more)

### Cost: $0/month (public repos)

---

## 7. Gitpod (Free Tier for Workspace Review)

### Why It's Free
- **Developer Environment:** Cloud-based VS Code in browser
- **No Setup:** No local dependency installation needed
- **Collaborative Review:** Share workspace URLs for code review

### Setup Instructions

1. **Create `.gitpod.yml`:**
   ```yaml
   image: gitpod/workspace-full:latest

   tasks:
     - init: |
         rustup default stable
         cargo build --release
     - command: cargo test

   ports:
     - port: 8080
       onOpen: open-browser

   vscode:
     extensions:
       - rust-lang.rust-analyzer
       - GitHub.copilot
   ```

2. **Enable Gitpod Button:**
   - Add to README:
     ```markdown
     [![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/KooshaPari/phenotype-infrakit)
     ```

3. **Share Workspace:**
   - Open PR or branch in Gitpod
   - Click "Share" to generate shareable URL
   - Share with reviewers for collaborative review

### Free Tier Limits
- ✓ 50 hours/month
- ✓ Up to 4 parallel workspaces
- ✓ 30GB storage
- ✓ Unlimited snapshots
- ⚠️ Higher limits available on paid tier

### Cost: $0/month (50 hours/month free tier)

---

## 8. GitHub Copilot Free (Limited, Future Release)

### Status: Experimental
- GitHub is testing limited free Copilot X tier
- May include basic completions without chat
- Availability: TBD (2026 roadmap)

### How to Get Free Copilot Now:
1. **Students:** GitHub Student Developer Pack (free Copilot)
2. **OSS Maintainers:** Verified maintainer program
3. **Limited Public Preview:** Check https://github.com/features/copilot

### Cost: $0/month (students/OSS maintainers)

---

## 9. AutoPR (Free, GitHub Action)

### Why It's Free
- **GitHub Action:** Runs in GitHub Actions (free runner minutes)
- **Open Source:** Apache 2.0 licensed
- **Smart PRs:** Generates pull requests based on issues

### Setup Instructions

1. **Create `.github/workflows/auto-pr.yml`:**
   ```yaml
   name: AutoPR
   on:
     issues:
       types: [opened, labeled]

   jobs:
     auto-pr:
       runs-on: ubuntu-latest
       if: contains(github.event.issue.labels.*.name, 'auto-fix')
       steps:
         - uses: actions/checkout@v4
         - name: Generate PR
           uses: gitautos/github-action-auto-commit@v0
           with:
             github_token: ${{ secrets.GITHUB_TOKEN }}
             commit_message: "Auto-fix: ${{ github.event.issue.title }}"
   ```

### Free Tier Limits
- ✓ Unlimited PRs
- ✓ Limited by GitHub Actions free tier (2,000 minutes/month)

### Cost: $0/month (uses GitHub Actions quota)

---

## 10. Gitguardian (Free Tier, Secret Detection)

### Why It's Free
- **Secret Detection:** Scans for API keys, tokens, credentials
- **GitHub Integration:** Automatic PR checks
- **Open Source Friendly:** Free tier available

### Setup Instructions

1. **Install GitHub App:**
   - Visit: https://github.com/marketplace/gitguardian
   - Click "Install for free"
   - Select repositories

2. **Create `.gitguardian.yml` (optional):**
   ```yaml
   version: 1
   matches-ignore:
     - name: example-api-key
       match: "^AKIA.*"
   ```

3. **Monitor Results:**
   - Gitguardian dashboard shows detected secrets
   - Automatic comments on PRs with potential leaks

### Free Tier Limits
- ✓ Unlimited public repos
- ✓ Unlimited scans
- ✓ Real-time detection
- ⚠️ Private repos: limited scans (paid tier for unlimited)

### Cost: $0/month (public repos)

---

## 11. Trufflehog (Free, Open Source Secret Scanning)

### Why It's Free
- **Open Source:** MIT licensed
- **Standalone:** No SaaS platform required
- **Self-Hosted:** Run in your own CI/CD

### Setup Instructions

1. **Install via GitHub Actions:**
   ```yaml
   name: TruffleHog Secret Scan
   on:
     push:
       branches: [main, develop]
     pull_request:

   jobs:
     truffleHog:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
           with:
             fetch-depth: 0
         - name: TruffleHog Secret Scan
           uses: trufflesecurity/trufflehog@main
           with:
             path: ./
             base: ${{ github.event.repository.default_branch }}
             head: HEAD
             extra_args: --only-verified
   ```

2. **Or Install Locally:**
   ```bash
   # macOS
   brew install trufflesecurity/trufflehog/trufflehog

   # Linux
   curl -sSfL https://raw.githubusercontent.com/trufflesecurity/trufflehog/main/install.sh | sh -s -- -b /usr/local/bin

   # Run
   trufflehog git file://. --since-commit HEAD --only-verified
   ```

### Free Tier Limits
- ✓ Unlimited scans
- ✓ Unlimited repositories
- ✓ All verification backends
- ✓ Self-hosted, no SaaS costs

### Cost: $0/month

---

## 12. Scorecards (Free, Security Scoring)

### Why It's Free
- **Open Source:** CNCF project, fully open source
- **Comprehensive:** Scores across 16 security categories
- **GitHub Integration:** Badge and workflow support

### Setup Instructions

1. **Create `.github/workflows/scorecards.yml`:**
   ```yaml
   name: Scorecards supply-chain security
   on:
     branch_protection_rule:
     schedule:
       - cron: "20 6 * * 0"
     push:
       branches: [main]

   jobs:
     analysis:
       name: Scorecards analysis
       runs-on: ubuntu-latest
       permissions:
         security-events: write
         id-token: write

       steps:
         - name: Checkout code
           uses: actions/checkout@v4

         - name: Run analysis
           uses: ossf/scorecard-action@v2
           with:
             results_file: results.sarif
             results_format: sarif
             publish_results: true

         - name: Upload SARIF
           uses: github/codeql-action/upload-sarif@v2
           with:
             sarif_file: results.sarif
   ```

2. **View Results:**
   - Security tab → Scorecards
   - GitHub score badge available

### Free Tier Limits
- ✓ Unlimited repos
- ✓ Unlimited scans
- ✓ All categories
- ✓ Public dashboard

### Cost: $0/month

---

## Free Stack Recommendation

### Recommended Configuration (All Free)

```
Primary Tools:
├── Dependabot (dependency updates)
├── GitHub CodeQL (security scanning)
├── Snyk (vulnerability detection)
├── SonarCloud (code quality)
└── Trufflehog (secret detection)

Optional Enhancements:
├── Renovate (better Dependabot alternative)
├── DeepSource (autofix PRs)
├── Gitpod (collaborative review)
└── Scorecards (supply chain security)

NOT RECOMMENDED (Paid):
├── CodeRabbit Pro ($20-50/month) → Use SonarCloud instead
├── Copilot ($10/month individual) → Use GitHub Copilot free tier when available
└── SonarCloud Pro → Free tier covers 95% of needs
```

### Implementation Timeline

**Week 1 (Immediate):**
- [ ] Enable Dependabot if not already active
- [ ] Enable GitHub CodeQL workflow
- [ ] Install Snyk for open source repos
- [ ] Install Gitguardian for secret detection

**Week 2 (Recommended):**
- [ ] Set up SonarCloud for code quality
- [ ] Configure Renovate as Dependabot alternative
- [ ] Add Trufflehog to GitHub Actions

**Week 3 (Optional):**
- [ ] Set up DeepSource for autofix capabilities
- [ ] Create Gitpod configuration for collaborative review
- [ ] Add Scorecards for supply chain security

---

## Cost Comparison

| Tool | Free Tier | Cost (Paid) | Recommendation |
|------|-----------|------------|-----------------|
| Dependabot | ✓ All features | N/A | Use free tier ✓ |
| GitHub CodeQL | ✓ Public repos | Paid for private | Use free tier ✓ |
| Snyk | ✓ Public repos | $35-500/mo | Use free tier ✓ |
| SonarCloud | ✓ Public repos | Paid for private | Use free tier ✓ |
| DeepSource | ✓ Public repos | Paid for private | Use free tier ✓ |
| Renovate | ✓ All features | N/A | Use free tier ✓ |
| Gitguardian | ✓ Public repos | $99-299/mo | Use free tier ✓ |
| Trufflehog | ✓ All features | N/A | Use free tier ✓ |
| CodeRabbit | Limited | $20-50/mo | Skip, use SonarCloud |
| Copilot | Limited | $10-21/mo | Skip for now |

### Total Annual Cost (Recommended Stack): **$0**

---

## Migration from Paid Apps

### From CodeRabbit to SonarCloud
1. **Keep:** CodeRabbit free tier (can still use)
2. **Add:** SonarCloud (free for public)
3. **Result:** CodeRabbit + SonarCloud covers same ground as CodeRabbit Pro

### From Copilot to Free Alternatives
1. **Wait for:** GitHub Copilot free tier (2026 roadmap)
2. **Or use:** Gitpod ($0 free tier, up to 50 hrs/month)
3. **Cost savings:** $10-21/month per user

### From Private SonarCloud to Free Tier
1. **Action:** Open-source repo (make public)
2. **Or migrate:** Analysis to self-hosted SonarQube (free, open source)

---

## Self-Hosting Alternatives (Advanced)

For maximum cost savings, self-host these tools:

| Tool | Install | Cost | Setup Time |
|------|---------|------|-----------|
| SonarQube | Docker | $0 | 30 min |
| Gitea | Docker | $0 | 30 min |
| Forgejo | Docker/Binary | $0 | 20 min |
| Mattermost | Docker | $0 | 45 min |
| Woodpecker CI | Docker | $0 | 45 min |

---

## References

- [GitHub Marketplace](https://github.com/marketplace?type=apps)
- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [Renovate Documentation](https://docs.renovatebot.com/)
- [Snyk Documentation](https://docs.snyk.io/)
- [SonarCloud Documentation](https://docs.sonarcloud.io/)
- [GitHub CodeQL](https://codeql.github.com/docs/)
- [Gitpod Documentation](https://www.gitpod.io/docs)
- [Trufflehog Documentation](https://github.com/trufflesecurity/trufflehog)

---

## Quick-Start Checklist

- [ ] Enable Dependabot (if not already)
- [ ] Create GitHub CodeQL workflow
- [ ] Install Snyk GitHub App
- [ ] Install Gitguardian GitHub App
- [ ] Set up SonarCloud
- [ ] Add Trufflehog to GitHub Actions
- [ ] Document all tools in README
- [ ] Create team runbook for using each tool
- [ ] Monitor costs monthly
- [ ] Review and update quarterly

---

**Last Updated:** 2026-03-30  
**Maintained by:** phenotype-infrakit team  
**Next Review:** 2026-06-30
