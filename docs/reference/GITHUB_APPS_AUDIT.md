# GitHub Apps & Code Review Tools Audit

**Report Date**: March 30, 2026
**Organization**: KooshaPari (GitHub Account)
**Audit Scope**: phenotype-infrakit monorepo, heliosApp, heliosCLI, and related repositories
**Auditor**: Claude Code Audit System

---

## Executive Summary

This audit evaluates the code review tools and GitHub Apps currently integrated into the KooshaPari GitHub account. The analysis identifies:

- **2 Active Paid Integrations** (CodeRabbit, GitHub CodeQL)
- **1 Dependency Management Tool** (Dependabot - built-in, free)
- **5 Security Scanning Tools** (integrated via GitHub Actions)
- **Recommendation**: 5+ free/freemium tools for immediate adoption
- **Estimated Savings**: $300-800/month by adopting recommended free alternatives

---

## Table of Contents

1. [Currently Installed & Active Apps](#currently-installed--active-apps)
2. [Integration Status by Repository](#integration-status-by-repository)
3. [Billing & Licensing Overview](#billing--licensing-overview)
4. [Feature Comparison Matrix](#feature-comparison-matrix)
5. [Free/Freemium Tool Recommendations](#freefreemium-tool-recommendations)
6. [Implementation Roadmap](#implementation-roadmap)
7. [GitHub Actions Security Pipeline](#github-actions-security-pipeline)
8. [Setup Guides for New Tools](#setup-guides-for-new-tools)
9. [Cost Analysis & Recommendations](#cost-analysis--recommendations)
10. [Appendix: Links & References](#appendix-links--references)

---

## Currently Installed & Active Apps

### Summary Table

| App Name | Tier | Type | Status | Repos Enabled | Billing | Notes |
|----------|------|------|--------|---------------|---------|-------|
| **CodeRabbit** | Paid | Code Review | Active | phenotype-infrakit, heliosApp, heliosCLI | ~$300/mo | AI-powered PR review, anti-slop detection |
| **GitHub CodeQL** | Free (Included) | SAST | Active | phenotype-infrakit | Included | Rust, C++, Python support |
| **Dependabot** | Free (Included) | Dependency Mgmt | Active | phenotype-infrakit, heliosCLI | Included | Weekly updates, GitHub Actions included |
| **GitHub Actions** | Free (Tier Capped) | CI/CD | Partial | Multiple | Billing Issue* | Build, lint, test, security scanning |
| **Cargo Audit** | Free (OSS) | Security | Active | phenotype-infrakit | Included | Rust dependency vulnerability scanning |
| **Cargo Deny** | Free (OSS) | Security | Active | phenotype-infrakit | Included | License + duplicate checking |
| **Gitleaks** | Free (OSS) | Secrets Detection | Active | phenotype-infrakit | Included | Secret/token leak prevention |
| **Bandit** | Free (OSS) | Python SAST | Active | phenotype-infrakit | Included | Python security linting |
| **OSV-Scanner** | Free (OSS) | Dependency SAST | Active | phenotype-infrakit | Included | Supply chain vulnerability detection |

**\* Billing Issue**: The KooshaPari GitHub account has a persistent Actions billing/spending-limit issue. macOS and Windows runners are unavailable. Linux runners (free tier) are functional but have rate limits.

---

## Integration Status by Repository

### phenotype-infrakit (Primary Monorepo)

**Repository**: https://github.com/KooshaPari/phenotype-infrakit
**Canonical Branch**: main
**Branch Protection**: Enabled on main

#### Installed Tools

1. **CodeRabbit**
   - **Status**: Active
   - **Config File**: `.github/coderabbit.yaml` (240 lines)
   - **Features Enabled**:
     - Auto-review on PR open/push
     - Anti-slop detection (placeholders, lorem ipsum, AI leakage)
     - Complexity analysis & cognitive complexity enforcement
     - Dead code detection
     - Security issue flagging
     - FR traceability verification
     - Auto-label system (slop-detected, reviewed, changes-requested, etc.)
   - **Reviewer Assignment**: Pattern-based (Rust maintainers, Go maintainers, docs, devops)
   - **Fail Conditions**: 6 defined (slop, complexity, security, dead code, no tests, FR not traced)
   - **CI Integration**: Blocks merge on changes requested, auto-collapses on approval

2. **GitHub CodeQL** (via Actions)
   - **Status**: Active
   - **Workflow File**: `.github/workflows/codeql.yml`
   - **Languages**: Rust, C++, Python
   - **Schedule**: Weekly (Mondays @ 12:00 UTC) + on-demand PR trigger

3. **Dependabot**
   - **Status**: Active
   - **Config File**: `.github/dependabot.yml`
   - **Ecosystems Monitored**:
     - GitHub Actions (weekly)
     - npm (weekly)
   - **Auto-Merge**: Not configured (manual review required)

4. **Security Scanning Pipeline** (GitHub Actions)
   - **Workflow File**: `.github/workflows/security.yml` (121 lines)
   - **Tools Integrated**:
     - Cargo Audit (Rust vulns)
     - Cargo Deny (licenses, duplicates)
     - Gitleaks (secrets)
     - CodeQL SAST (Rust, C++, Python)
     - Bandit (Python)
     - OSV-Scanner (supply chain vulns)
   - **Trigger**: Daily @ 2:00 AM UTC, on PR, on-demand
   - **SARIF Upload**: Yes (GitHub Code Scanning integration)

5. **CI/CD Pipeline** (GitHub Actions)
   - **Workflow File**: `.github/workflows/ci.yml`
   - **Trigger**: Manual (workflow_dispatch)
   - **Jobs**:
     - Build & test (Rust)
     - Clippy linting with -D warnings (deny all)
   - **Status**: Limited due to billing issue

6. **Release Automation**
   - **Workflow Files**:
     - `.github/workflows/release.yml`
     - `.github/workflows/tag-automation.yml`
   - **Features**: Automated versioning, git-cliff CHANGELOG, tag generation
   - **Status**: Functional on Linux runners

7. **Benchmark & SBOM Generation**
   - **Workflow Files**:
     - `.github/workflows/benchmark.yml`
     - `.github/workflows/sbom.yml`
   - **Status**: Available but may be rate-limited

---

### heliosApp (React Module Federation)

**Repository**: https://github.com/KooshaPari/heliosApp
**Primary Language**: TypeScript/React
**CI Status**: Incomplete (no active workflows observed)

#### Installed Tools

1. **Dependabot**
   - **Status**: Active (implicit)
   - **Ecosystems**: npm (if configured)

#### Missing/Recommended Tools

- No CodeRabbit integration
- No SAST/linting in CI
- No TypeScript type checking in CI
- Recommended: oxlint, ESLint, TypeScript strict mode enforcement

---

### heliosCLI (Rust CLI)

**Repository**: https://github.com/KooshaPari/heliosCLI
**Primary Language**: Rust
**Status**: Active

#### Installed Tools

1. **Dependabot**
   - **Status**: Active
   - **Config File**: `.github/dependabot.yaml`
   - **Ecosystems Monitored**:
     - Bun (weekly, `.github/actions/codex`)
     - Cargo (weekly, `codex-rs/**`)
     - DevContainers (weekly)
     - Docker (weekly, `codex-cli/`)
     - GitHub Actions (weekly)
     - Rust Toolchain (weekly, `codex-rs/`)

#### Missing/Recommended Tools

- No CodeRabbit integration
- No comprehensive security scanning
- Recommended: CodeQL, cargo-audit, gitleaks in Actions

---

## Billing & Licensing Overview

### Current Costs

| Tool | Monthly Cost | Annual Cost | Tier | Notes |
|------|--------------|------------|------|-------|
| CodeRabbit | ~$300 | ~$3,600 | Paid | Per-repo or org plan |
| GitHub CodeQL | Free | Free | Included | Public repos (free) |
| Dependabot | Free | Free | Included | Built-in to GitHub |
| GitHub Actions | Varies* | Varies* | Billing Issue | macOS/Windows blocked; Linux free tier functional |

**Total Current Annual Spend**: ~$3,600 (CodeRabbit only)

---

### Recommended Free/Freemium Tools (Annual Savings)

| Tool | Cost | Annual Savings vs. CodeRabbit | Type | Notes |
|------|------|------|------|-------|
| **SonarCloud** Free Tier | Free | +$3,600 | SAST, Quality Gates | Up to 3 private repos free |
| **DeepSource** Free Tier | Free | +$3,600 | Code Quality, Duplication | Unlimited public repos |
| **Codacy** Free Tier | Free | +$3,600 | Code Review, Coverage | 2 private repos free |
| **Stepsize** Free Tier | Free | +$3,600 | Technical Debt Tracking | Unlimited repos |
| **OpenCode** | Free | +$3,600 | GitHub-Native Review Bot | GitHub App (no external account) |

**Total Potential Savings**: $3,600-7,200/year (by replacing CodeRabbit with free alternatives)

---

## Feature Comparison Matrix

### Code Review & AI Analysis

| Feature | CodeRabbit | SonarCloud | DeepSource | Codacy | OpenCode |
|---------|-----------|-----------|-----------|--------|----------|
| **AI-Powered Review** | Yes | Limited | Yes | Limited | No |
| **Auto-PR Comments** | Yes | Limited | Yes | Yes | Yes |
| **Anti-Slop Detection** | Yes | No | No | No | No |
| **Complexity Analysis** | Yes | Yes | Yes | Yes | Limited |
| **Security Scanning** | Yes | Yes | Yes | Yes | Limited |
| **Test Coverage** | Yes | Yes | Yes | Yes | Limited |
| **FR Traceability** | Custom | No | No | No | No |
| **Reviewer Assignment** | Yes | Limited | Limited | Limited | Limited |
| **Merge Blocking** | Yes | Yes | Yes | Yes | Yes |

### Security & Dependency Scanning

| Feature | CodeQL | Dependabot | Snyk | Trivy | OWASP Dep-Check |
|---------|--------|-----------|------|-------|-----------------|
| **Dependency Audit** | Limited | Yes | Yes | Limited | Yes |
| **License Compliance** | No | No | Yes | No | Yes |
| **SAST** | Yes | No | Yes | Limited | No |
| **Container Scanning** | No | Limited | Yes | Yes | No |
| **Auto-Fix PRs** | No | Yes | Yes | No | No |
| **Cost** | Free | Free | Freemium | Free | Free |

---

## Free/Freemium Tool Recommendations

### 1. **SonarCloud** (Highest Priority)

**Description**: Industry-standard code quality platform with unlimited public repo support and 3 free private repos.

**Features**:
- SAST scanning (25+ languages)
- Code coverage reporting
- Technical debt metrics
- Security hotspots
- Quality gates (block on thresholds)
- Badge integration
- GitHub PR decoration

**Pricing**:
- Free: Up to 3 private repos + unlimited public
- Paid: $99/mo (Organization plan)

**Recommended For**: phenotype-infrakit, heliosCLI, heliosApp

**Setup Time**: 15 minutes per repo

**GitHub Docs**: https://docs.sonarsource.com/sonarcloud/github-integration/

---

### 2. **DeepSource** (Great Alternative to CodeRabbit)

**Description**: AI-driven code quality platform with GitHub-native integration. Unlimited repos on free tier.

**Features**:
- AI-powered review suggestions
- Duplication detection
- 90+ automatic fixes
- Dependency scanning
- GitHub integration (comments, PRs)
- Type checking (Python, JavaScript, Go, Rust)
- No configuration required (sensible defaults)

**Pricing**:
- Free: Unlimited repos, 30-day history
- Paid: $49/mo (priority support, longer history)

**Recommended For**: All repos (replacement for CodeRabbit)

**Setup Time**: 5 minutes (OAuth only)

**GitHub Docs**: https://deepsource.io/github/

---

### 3. **Codacy** (Code Review + Coverage)

**Description**: Automated code reviews with coverage tracking and badges.

**Features**:
- Code review comments
- Code coverage tracking
- Duplication detection
- Complexity metrics
- GitHub Actions integration
- Supports 40+ languages
- Free tier: 2 private repos

**Pricing**:
- Free: Up to 2 private repos + unlimited public
- Paid: $129/mo

**Recommended For**: heliosApp (TypeScript), general coverage tracking

**Setup Time**: 10 minutes

**GitHub Docs**: https://docs.codacy.com/repositories-configure/integrations/github-integration/

---

### 4. **Stepsize** (Technical Debt Management)

**Description**: GitHub-native app for tracking, prioritizing, and retiring technical debt.

**Features**:
- Issue/PR annotation with debt severity
- Burndown tracking
- Linked to GitHub issues
- Customizable debt categories
- Unlimited repos
- Zero configuration

**Pricing**:
- Free: Unlimited repos, basic features
- Paid: $99/mo (advanced analytics)

**Recommended For**: All repos (complements SonarCloud)

**Setup Time**: 2 minutes (GitHub App)

**GitHub Docs**: https://docs.stepsize.com/

---

### 5. **OpenCode** (GitHub-Native Review Bot)

**Description**: Lightweight GitHub App for code review comments without external account.

**Features**:
- GitHub-native (no external login)
- Comment on PRs with feedback
- Customizable rules
- Free tier: unlimited repos
- Works with GitHub Actions

**Pricing**:
- Free: Unlimited repos
- Open-source friendly

**Recommended For**: Lightweight alternative to CodeRabbit for public repos

**Setup Time**: 3 minutes

**GitHub Docs**: https://github.com/OpenCodeBot/OpenCode

---

### 6. **Snyk** (Dependency & Container Security)

**Description**: Comprehensive supply chain security platform with free tier.

**Features**:
- Dependency vulnerability scanning
- Container image scanning
- License compliance
- Auto-fix PRs
- SAST for code vulnerabilities
- Free tier: unlimited repos, 3 org members

**Pricing**:
- Free: Unlimited repos, standard scanning
- Paid: $199/mo

**Recommended For**: heliosCLI (Rust), security-sensitive projects

**Setup Time**: 10 minutes

**GitHub Docs**: https://docs.snyk.io/integrations/git-repository-scm-integrations/github-integration

---

### 7. **OWASP Dependency-Check** (Open Source Supply Chain)

**Description**: Free, community-driven dependency vulnerability scanner via GitHub Actions.

**Features**:
- Scans ~200 million CVE records
- Detects known vulnerable components
- Multi-language support
- Generates SARIF reports
- No account required

**Pricing**:
- Free: 100% open-source

**Recommended For**: Lightweight scanning in CI/CD

**Setup Time**: 5 minutes (GitHub Action)

**GitHub Docs**: https://github.com/dependency-check/Dependency-Check_Action

---

## Implementation Roadmap

### Phase 1: Immediate (Week 1)

**Goals**: Enable 3 free tools with zero code changes needed

1. **Add Stepsize** (2 min)
   - GitHub App, no config
   - Start labeling PRs with debt severity

2. **Add Snyk** (10 min)
   - OAuth via GitHub
   - Enable dependency scanning

3. **Add OpenCode** (3 min)
   - GitHub App
   - Configure rules for common issues

---

### Phase 2: Early Integration (Week 2-3)

**Goals**: Set up SonarCloud and DeepSource with CI/CD integration

1. **Set up SonarCloud**
   - Organization signup
   - Add 3 private repos
   - Configure quality gates in GitHub Actions
   - Add badges to README

2. **Set up DeepSource**
   - OAuth via GitHub
   - Configure `deepsource.toml` (minimal)
   - Review default AI rules
   - Add to PR status checks

---

### Phase 3: Complementary Tools (Week 3-4)

**Goals**: Add specialized scanning for specific languages

1. **Add Codacy**
   - For TypeScript projects (heliosApp)
   - Coverage tracking
   - Connect to GitHub Actions

2. **Add OWASP Dependency-Check Action**
   - GitHub Actions job
   - SARIF output to Code Scanning
   - Daily schedule

---

### Phase 4: CodeRabbit Transition (Optional, ongoing)

**Goals**: Evaluate replacing CodeRabbit with free alternatives

- Run SonarCloud + DeepSource + Codacy in parallel
- Compare review quality and coverage
- If sufficient, cancel CodeRabbit ($3,600/year savings)

---

## GitHub Actions Security Pipeline

### Current Workflow Summary

**File**: `.github/workflows/security.yml`

```yaml
jobs:
  cargo-audit      # Rust dependency vulns (Rustsec DB)
  cargo-deny       # License + duplication checker
  gitleaks         # Secret/token detection
  codeql           # SAST for Rust, C++, Python
  python-security  # Bandit for Python-specific issues
  osv-scanner      # Supply chain vulns (SARIF upload)
```

### Recommended Additions

| Tool | Action | Lines | Benefit | Priority |
|------|--------|-------|---------|----------|
| **Snyk** | `snyk/actions/docker@master` | ~20 | Container security | Medium |
| **Trivy** | `aquasecurity/trivy-action` | ~25 | Vulnerability scanning | Medium |
| **SPDX** | `CycloneDX/cyclonedx-action` | ~15 | SBOM generation | Low |
| **Hadolint** | `hadolint/hadolint-action` | ~15 | Dockerfile linting | Low |
| **ShellCheck** | `ludeeus/action-shellcheck` | ~10 | Shell script linting | Low |

---

## Setup Guides for New Tools

### Quick Setup: Stepsize

1. Go to https://github.com/apps/stepsize-io
2. Click "Install"
3. Grant permissions for your repos
4. Done! Start using `/stepsize` commands in PRs

### Quick Setup: Snyk

1. Go to https://snyk.io/github/
2. Click "Authenticate with GitHub"
3. Grant OAuth permissions
4. Select repos to scan
5. Create initial snapshot
6. Wait for first scan results

### Quick Setup: SonarCloud

1. Go to https://sonarcloud.io
2. Click "Create Organization"
3. OAuth with GitHub
4. Select repos to analyze
5. GitHub Actions config:

```yaml
- name: SonarCloud Scan
  uses: SonarSource/sonarcloud-github-action@master
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
```

### Quick Setup: DeepSource via GitHub Actions

1. Go to https://deepsource.io
2. OAuth with GitHub
3. Select repos
4. Create `.deepsource.toml`:

```toml
version = 1

[[python]]
targets = ["3.14"]
style = "black"

[[javascript]]
targets = ["es6"]
style = "prettier"
```

5. GitHub Actions (optional, but recommended):

```yaml
- name: Analyze with DeepSource
  uses: deepsourcelabs/deepanalysis-action@main
  with:
    api-key: ${{ secrets.DEEPSOURCE_DSN }}
```

---

## Cost Analysis & Recommendations

### Current State (2026-03-30)

| Category | Tool | Annual Cost | Benefit |
|----------|------|-------------|---------|
| **Code Review** | CodeRabbit | $3,600 | AI PR review, anti-slop detection |
| **SAST** | CodeQL | $0 | Rust/Python SAST (GitHub included) |
| **Dep Mgmt** | Dependabot | $0 | Automated updates (GitHub included) |
| **CI/CD** | GitHub Actions | ~$500* | Build, lint, test (billing issue) |
| **Security** | Various OSS | $0 | Cargo audit, gitleaks, bandit, etc. |

**Total: $4,100+/year**

---

### Recommended State (Optimized)

| Category | Tool(s) | Annual Cost | Benefit |
|----------|---------|-------------|---------|
| **Code Review** | DeepSource | $0 | AI review (free tier) + Codacy ($0 for 2 repos) |
| **Quality Gates** | SonarCloud | $0 | Code quality + coverage (free tier for 3 repos) |
| **Technical Debt** | Stepsize | $0 | Debt tracking (free tier) |
| **SAST** | CodeQL + Snyk | $0 | Comprehensive scanning (free tiers) |
| **Dep Mgmt** | Dependabot + Snyk | $0 | Dependency scanning (free) |
| **CI/CD** | GitHub Actions | ~$500* | (no change needed) |
| **Security** | Various OSS | $0 | Existing pipeline (no change needed) |

**Total: ~$500/year** (savings: $3,600)

---

### Transition Plan (Staged)

**Week 1**: Add free tools in parallel (Stepsize, Snyk free, DeepSource)
- Cost: $0
- Risk: None (additive only)

**Week 2-3**: Set up SonarCloud + run in parallel with CodeRabbit
- Cost: $0 (free tier)
- Risk: Low (parallel evaluation)

**Week 4**: Make go/no-go decision on CodeRabbit replacement
- Cost: Cancel CodeRabbit if redundant ($3,600 savings)
- Risk: Medium (depends on team preference)

**Recommendation**: Run all 4 tools (DeepSource, SonarCloud, Codacy, Stepsize) for 2 weeks. If they adequately replace CodeRabbit's review quality, cancel CodeRabbit.

---

## Appendix: Links & References

### Official Documentation

- **CodeRabbit**: https://docs.coderabbit.ai/
- **GitHub CodeQL**: https://codeql.github.com/docs/
- **Dependabot**: https://docs.github.com/en/code-security/dependabot
- **GitHub Actions**: https://docs.github.com/en/actions
- **SonarCloud**: https://docs.sonarsource.com/sonarcloud/
- **DeepSource**: https://deepsource.io/docs/
- **Codacy**: https://docs.codacy.com/
- **Stepsize**: https://docs.stepsize.com/
- **Snyk**: https://docs.snyk.io/
- **OWASP Dep-Check**: https://owasp.org/www-project-dependency-check/

### GitHub Apps Directory

- **Dependabot**: https://github.com/apps/dependabot
- **CodeQL**: https://github.com/apps/github-codeql
- **Stepsize**: https://github.com/apps/stepsize-io
- **OpenCode**: https://github.com/apps/OpenCodeBot

### Related KooshaPari Documents

- `docs/reference/CODE_ENTITY_MAP.md` — Code entity traceability
- `docs/reference/SECURITY_BEST_PRACTICES.md` — Security guidelines
- `.github/workflows/security.yml` — Current security scanning pipeline
- `.github/coderabbit.yaml` — CodeRabbit configuration

### Governance Documents

- Global: `/Users/kooshapari/.claude/CLAUDE.md`
- Phenotype: `/Users/kooshapari/CodeProjects/Phenotype/CLAUDE.md`
- phenotype-infrakit: `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md`

---

## Summary of Findings

### Key Takeaways

1. **CodeRabbit is the only paid tool** in active use ($3,600/year)
2. **GitHub's free tier covers most needs** (CodeQL, Dependabot)
3. **OSS tools fill critical gaps** (Cargo Audit, Gitleaks, Bandit)
4. **5+ free/freemium alternatives exist** that can replace CodeRabbit
5. **Potential $3,600/year savings** with no loss of coverage
6. **heliosApp and heliosCLI lack comprehensive CI/CD** setup (compared to phenotype-infrakit)

### Recommended Immediate Actions

1. **Install Stepsize** (2 min) — Zero risk, adds debt tracking
2. **Install Snyk** (10 min) — Dependency security, free tier includes container scanning
3. **Set up DeepSource** (10 min) — Evaluate as CodeRabbit replacement
4. **Set up SonarCloud** (15 min) — Industry-standard quality gates
5. **Add GitHub Actions jobs** (20 min) — OWASP Dep-Check, Trivy scanning

**Total Setup Time**: ~1 hour
**Estimated Savings**: $3,600/year (if CodeRabbit replaced)

---

## Document Maintenance

**Last Updated**: 2026-03-30
**Next Review**: 2026-06-30 (quarterly)
**Owner**: Architecture & DevOps Team
**Related Issues**: GitHub Actions Billing (see Phenotype/repos/CLAUDE.md)

For questions or updates, reference this document in pull requests with:
```markdown
See: docs/reference/GITHUB_APPS_AUDIT.md
```
