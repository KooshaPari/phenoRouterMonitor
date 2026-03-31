# Code Review and PR Automation Guide

**Phenotype Polyrepo Code Review Infrastructure**

This guide documents the code review tools, GitHub Actions workflows, CodeOwners configuration, and PR automation strategies for the Phenotype polyrepo. It covers automated review tools, branch protection, merge gates, and agent-driven review coordination.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Code Review Tool Comparison](#code-review-tool-comparison)
3. [Primary Tool: CodeRabbit](#primary-tool-coderabbit)
4. [GitHub Native Features](#github-native-features)
5. [CodeOwners Setup](#codeowners-setup)
6. [Branch Protection Configuration](#branch-protection-configuration)
7. [GitHub Actions Custom Review Workflows](#github-actions-custom-review-workflows)
8. [Automated PR Fixes](#automated-pr-fixes)
9. [PR Template Setup](#pr-template-setup)
10. [Review SLAs and Notifications](#review-slas-and-notifications)
11. [Cost Analysis](#cost-analysis)
12. [Cloud Agents as Secondary Reviewers](#cloud-agents-as-secondary-reviewers)
13. [Merge Gate Configuration](#merge-gate-configuration)
14. [Integration Guide](#integration-guide)
15. [Troubleshooting](#troubleshooting)

---

## Executive Summary

### Current State
The Phenotype polyrepo currently implements:
- **CodeRabbit** AI-powered code review (primary automated reviewer)
- **GitHub native code review** (manual review via GitHub UI)
- **CodeOwners files** (enforce approval requirements by file pattern)
- **Branch protection rules** (status checks and merge gates)
- **GitHub Actions workflows** (security scanning, quality gates, CI/CD)

### Key Metrics
- **Primary Tool**: CodeRabbit (AI, unlimited free tier, generous)
- **Review Threshold**: 70% approval by CodeOwners for auto-merge
- **Security Threshold**: 85% for critical/security changes
- **Hotfix Threshold**: 50% for urgent patches
- **Max PR Size**: 500 LOC (warn at 400 LOC)

### Architecture
```
PR Submission
    ↓
CodeRabbit AI Review (40% weight) ← Async, automated
    ↓
Security Scanning:
  - Cargo Audit (25% weight)
  - CodeQL SAST (20% weight)
  - Gitleaks Secret Detection (10% weight)
  - Custom checks (5% weight)
    ↓
Merge Gate Decision:
  - All checks pass? ✓
  - Approval threshold met? ✓
  - No conflicts? ✓
  - Branch up-to-date? ✓
    ↓
Auto-merge (if enabled) OR Manual merge by code owner

Cloud agents available for overflow / complex reviews
```

---

## Code Review Tool Comparison

### 1. CodeRabbit (PRIMARY)

**Status**: Actively configured and integrated

**Features**:
- AI-powered code review with language-specific rules (Rust, Python, Go, JavaScript)
- Detects security issues, performance problems, architecture violations
- Automatic PR summaries and inline comments
- File-pattern based review rules (Cargo.toml, .github/workflows, test files, etc.)
- Test coverage tracking and suggestions
- Auto-merge conditions with approval thresholds
- Supports custom code review rules (security, performance, design, testing, docs)

**Free Tier**:
- Unlimited PRs
- All language support
- Full AI review with suggestions
- Auto-merge capability

**Configuration**:
- Root: `/Users/kooshapari/CodeProjects/Phenotype/repos/.coderabbit.yaml`
- Project-specific: `<project>/.coderabbit.yaml`

**Pricing**:
- Free for public/private repos
- Unlimited PRs and reviews
- No per-PR charges

**Integration**:
```yaml
# CodeRabbit reviews on every PR
reviews:
  auto_summary: true
  collapse_unnecessary_comments: true
  publish_review_comment: true
  max_pr_size: 500 lines
  warn_on_size: 400 lines

language_rules:
  rust:
    focus_areas:
      - unsafe_blocks
      - error_handling
      - performance_implications
    patterns_to_watch:
      - "^unsafe "
      - "\.unwrap\(\)"
      - "\.expect\("
  python:
    focus_areas:
      - type_hints
      - error_handling
      - async_patterns
```

---

### 2. GitHub Native Code Review

**Features**:
- Free built-in PR review mechanism
- Comment on specific lines of code
- Request changes, approve, or comment
- Conversation history preserved
- Integration with branch protection

**When to Use**:
- Complex architectural decisions requiring discussion
- Breaking API changes
- Security implications needing human judgment
- Cross-team collaboration

**Configuration**:
- Done via GitHub UI or branch protection rules
- CodeOwners file specifies who must review

---

### 3. Dependabot (Free, Built-in)

**Status**: Ready to enable

**Features**:
- Automatic dependency update PRs
- Version range constraints (major, minor, patch)
- Auto-merge for patch updates (optional)
- Security updates prioritized
- Supports Cargo, Python, Go, JavaScript, Ruby, etc.

**Configuration**:
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
    allow:
      - dependency-type: "all"
    reviewers: ["@KooshaPari"]
    assignees: ["@KooshaPari"]

  - package-ecosystem: pip
    directory: "/python"
    schedule:
      interval: weekly
    allow:
      - dependency-type: "direct"
```

---

### 4. Renovate (Alternative to Dependabot)

**Status**: Not currently configured

**Features**:
- More sophisticated dependency management
- Semantic commit messages
- Automatic rebase on conflicts
- Better monorepo support (Phenotype repos)
- Supports branches, tags, digest pinning

**Comparison to Dependabot**:
| Feature | Dependabot | Renovate |
|---------|-----------|----------|
| Config Format | YAML (simple) | YAML/JS (complex) |
| Monorepo Support | Basic | Excellent |
| Semantic Messages | No | Yes |
| Auto-Rebase | No | Yes |
| Free Tier | Yes | Yes |
| Setup Effort | 15 min | 30 min |

**Recommendation**: Stick with Dependabot for simplicity; consider Renovate for Phase 2 if monorepo coordination becomes complex.

---

### 5. Snyk (Security-Focused, Free Tier)

**Status**: Not currently configured

**Features**:
- Vulnerability scanning (dependencies + code)
- License compliance checks
- Automated PR creation for fixes
- Integration with GitHub branch protection

**Pricing**:
- Free tier: unlimited public repos, limited private repo scans
- Paid: unlimited scans + advanced features

**When to Use**: If additional security scanning beyond Cargo Audit + CodeQL is needed.

---

### 6. DeepSource (Code Quality + Review, Free)

**Status**: Not currently configured

**Features**:
- SAST analysis (like CodeQL but more focused)
- Code quality metrics
- Automatic PR suggestions
- Integrates with GitHub Actions

**Pricing**: Free for open source, paid for private repos.

---

## Primary Tool: CodeRabbit

### Current Configuration

**Root Configuration**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.coderabbit.yaml`

**Key Settings**:

```yaml
reviews:
  auto_summary: true                    # Generate PR summary
  collapse_unnecessary_comments: true   # Reduce noise
  publish_review_comment: true          # Post review as comment
  max_pr_size: 500                      # Warn on large PRs
  fail_on_size_exceed: false            # Don't block large PRs

language_rules:
  rust:
    focus_areas:
      - unsafe_blocks
      - error_handling
      - performance_implications
      - api_design
      - test_coverage
    enforce_rules:
      - use_thiserror_for_errors
      - avoid_unwrap
      - proper_error_handling
      - async_await_patterns
    complexity_threshold: 15

  python:
    focus_areas:
      - type_hints
      - error_handling
      - async_patterns
      - test_coverage
    enforce_rules:
      - use_type_hints
      - proper_exception_handling
      - docstring_format
```

### File-Pattern Based Review Rules

CodeRabbit reviews different files with different rigor levels:

```yaml
file_rules:
  "**/Cargo.toml":
    review_type: "mandatory"
    focus:
      - dependency_versions
      - features
      - workspace_structure
      - deprecated_deps
    require_semver: true
    warn_on_major_version_change: true

  "review.toml":
    review_type: "mandatory"
    require_approval: true
    human_review_required: true

  ".github/workflows/**":
    review_type: "mandatory"
    focus:
      - security
      - ci_cd_best_practices
      - secret_exposure
    human_review_required: true

  "crates/phenotype-*/src/**/*.rs":
    review_type: "standard"
    focus:
      - unsafe_code
      - error_handling
      - api_design
      - test_coverage
    min_test_coverage: 0.80
    enforce_doc_comments: true

  "**/*.md":
    review_type: "light"
    focus:
      - clarity
      - formatting
      - broken_links
    allow_auto_merge: true

  ".archive/**":
    review_type: "minimal"
    allow_auto_merge: true
```

### Code Review Rules

CodeRabbit enforces structured rules across categories:

**Security Rules**:
- Detect secrets (critical severity, blocking)
- Unsafe code review (high severity)
- SQL injection checks (critical)
- Command injection checks (critical)

**Performance Rules**:
- N+1 query detection (medium)
- Large allocation detection (medium)
- Blocking operations in async contexts (high)

**Architecture Rules**:
- Circular dependency detection (high)
- API contract validation (high)
- Breaking change detection (high)

**Testing Rules**:
- Test coverage checks (80% minimum, medium severity)
- Missing test detection (medium)
- Test naming conventions (low)

**Documentation Rules**:
- Missing docstrings on public APIs (medium)
- Broken links (medium)
- Outdated examples (low, disabled by default)

### CodeRabbit Approval Configuration

```yaml
approvals:
  require_code_owner_approval: false
  dismiss_stale_reviews_on_push: true
  require_review_from_multiple_teams: false

auto_merge:
  enabled: true
  conditions:
    - all_checks_pass
    - approval_threshold_met
    - no_merge_conflicts
    - branch_up_to_date
  squash_merge: true
  delete_branch: true
  require_status_checks:
    - "CodeQL (rust)"
    - "Cargo Audit"
    - "Cargo Deny"
```

### How to Customize CodeRabbit

1. **Edit `.coderabbit.yaml`** at repo root
2. **Restart the workflow** or wait for next PR
3. **Test with draft PR** to verify rules are applied

Example: Add custom rule for new library

```yaml
code_review_rules:
  architecture:
    - rule: "forbidden_dependency_check"
      enabled: true
      severity: "high"
      message: "Forbidden dependency detected: use alternative"
      forbidden_packages:
        - "old_lib"
        - "deprecated_package"
```

---

## GitHub Native Features

### Branch Protection Rules

**Location**: GitHub repo settings → Branches → Branch protection rules

**Recommended Configuration** (for `main` branch):

```
Rule name: main

✓ Require pull request reviews before merging
  - Number of reviewers: 1
  - Dismiss stale pull request approvals when new commits are pushed
  - Require code owner review

✓ Require status checks to pass before merging
  - Require branches to be up to date before merging
  - Status checks required:
    - CodeQL (rust)
    - Cargo Audit
    - Cargo Deny
    - Gitleaks
    - OSV Scanner
    - quality-gate (if using custom quality checks)

✓ Require conversation resolution before merging

✓ Require signed commits

✓ Dismiss stale pull request approvals
```

### CodeOwners Enforcement

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/CODEOWNERS`

**Current Configuration**:

```
# Default code owner (catch-all)
* @KooshaPari

# GitHub infrastructure
.github/workflows/ @KooshaPari
.github/hooks/ @KooshaPari

# Documentation
docs/ @KooshaPari

# Rust crates
crates/ @KooshaPari

# Python code
python/ @KooshaPari
```

**To Expand** for multi-person teams:

```
# Default owner
* @KooshaPari

# Rust teams
crates/phenotype-core/ @phenotype-core-team @KooshaPari
crates/phenotype-state-machine/ @state-machine-team @KooshaPari

# Python teams
python/phenosdk/ @sdk-team @KooshaPari
python/phench/ @phench-team @KooshaPari

# Infrastructure (requires all specified owners to approve)
.github/workflows/ @KooshaPari
review.toml @KooshaPari
```

---

## CodeOwners Setup

### Purpose
CodeOwners files specify which individuals or teams must review code changes in specific directories. Combined with branch protection, they enforce approval requirements.

### Current Setup

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/CODEOWNERS`

```
* @KooshaPari

.github/workflows/ @KooshaPari
.github/hooks/ @KooshaPari
docs/ @KooshaPari
crates/ @KooshaPari
python/ @KooshaPari
```

### GitHub Teams (Recommended for Teams)

Once the Phenotype team grows, create GitHub teams:

1. Go to GitHub org settings → Teams → New team
2. Create teams like:
   - `@phenotype-core` (core infrastructure)
   - `@phenotype-agents` (agent-related code)
   - `@phenotype-infra` (infrastructure/ops)

3. Update CODEOWNERS:

```
# Phenotype Core
crates/phenotype-contracts/ @phenotype-core
crates/phenotype-error-core/ @phenotype-core
crates/phenotype-config-core/ @phenotype-core

# Agents
agileplus-agents/ @phenotype-agents
agent-wave/ @phenotype-agents

# Infrastructure
.github/workflows/ @phenotype-infra
docs/ @phenotype-infra

# Everyone else
* @KooshaPari
```

### CODEOWNERS Best Practices

1. **Keep it maintainable**: Don't exceed 20-30 lines
2. **Use teams, not individuals**: GitHub teams are easier to manage
3. **Avoid over-specification**: Only require approvals where truly needed
4. **Document the intent**: Add comments explaining why certain files need approval

```
# Critical infrastructure — requires infra team approval
.github/workflows/ @phenotype-infra
review.toml @phenotype-infra

# Core library changes — requires core team review
crates/phenotype-contracts/ @phenotype-core
crates/phenotype-error-core/ @phenotype-core

# Security-sensitive code — requires security team
# (if security team exists)
# security/ @phenotype-security

# Everything else — catch-all
* @KooshaPari
```

---

## Branch Protection Configuration

### GitHub UI Setup

1. Go to repo → Settings → Branches
2. Click "Add rule" under "Branch protection rules"
3. Configure for `main` branch:

```
✓ Require a pull request before merging
  ✓ Require approvals: 1
  ✓ Dismiss stale pull request approvals when new commits are pushed
  ✓ Require code owner review (if using CODEOWNERS)

✓ Require status checks to pass before merging
  ✓ Require branches to be up to date before merging
  Required status checks:
    - CodeQL (rust)
    - Cargo Audit
    - Cargo Deny
    - Gitleaks
    - quality-gate (optional)

✓ Require conversation resolution before merging
✓ Require signed commits
```

### Configuration via GitHub API

Alternative: Configure via `gh` CLI for documentation/automation:

```bash
# Require approvals
gh api repos/KooshaPari/phenotype-infrakit/branches/main/protection \
  -X PATCH \
  -f required_pull_request_reviews='{"dismissal_restrictions":{},"dismiss_stale_reviews":true,"require_code_owner_reviews":true,"required_approving_review_count":1}'

# Require status checks
gh api repos/KooshaPari/phenotype-infrakit/branches/main/protection/required_status_checks \
  -X PATCH \
  -f strict=true \
  -f contexts='["CodeQL (rust)","Cargo Audit","Cargo Deny","Gitleaks"]'

# Require signed commits
gh api repos/KooshaPari/phenotype-infrakit/branches/main/protection \
  -X PATCH \
  -F require_signed_commits=true
```

---

## GitHub Actions Custom Review Workflows

### Quality Gate Workflow

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/quality-gate.yml`

Runs automated quality checks on every PR:

```yaml
name: quality-gate
on: [pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run quality checks
        run: ./scripts/quality-gate.sh verify
```

**What it checks**:
- Lint (Ruff for Python, Clippy for Rust)
- Type checking (Pyright, rustc)
- Test coverage (pytest, cargo test)
- Documentation (Vale markdown linting)
- Tach boundaries (architectural constraints)

### Security Scanning Workflow

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/security.yml`

Runs on every PR and daily schedule:

```yaml
name: Security
on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2am UTC

jobs:
  cargo-audit:
    name: Cargo Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: rustsec/audit-check@v2.0.0

  cargo-deny:
    name: Cargo Deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-deny
        run: cargo install cargo-deny
      - name: Check advisories, licenses, duplicates
        run: cargo deny check

  gitleaks:
    name: Gitleaks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0
      - uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  codeql:
    name: CodeQL
    runs-on: ubuntu-latest
    strategy:
      matrix:
        language: [rust, python]
    steps:
      - uses: actions/checkout@v6
      - uses: github/codeql-action/init@v3
      - uses: github/codeql-action/autobuild@v3
      - uses: github/codeql-action/analyze@v3
```

### Custom Review-Request Workflow

Create a workflow that requests human review on complex PRs:

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/request-review.yml`

```yaml
name: Request Review

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  check-review-required:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Check if review required
        id: check
        run: |
          # Check for critical file changes
          FILES=$(git diff --name-only origin/main...HEAD)

          CRITICAL=false
          for file in $FILES; do
            if [[ "$file" =~ ^\.github/workflows/ ]] || \
               [[ "$file" == "review.toml" ]] || \
               [[ "$file" == "Cargo.lock" ]]; then
              CRITICAL=true
              break
            fi
          done

          echo "critical=$CRITICAL" >> $GITHUB_OUTPUT

      - name: Request code owner review
        if: steps.check.outputs.critical == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const codeowners = fs.readFileSync('CODEOWNERS', 'utf8');

            github.rest.pulls.requestReviewers({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number,
              reviewers: ['KooshaPari']
            });

      - name: Comment on PR
        if: steps.check.outputs.critical == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '🔍 **Critical file changes detected** — automatic human review requested.\n\nFocus areas:\n- Security implications\n- CI/CD correctness\n- Configuration correctness'
            });
```

---

## Automated PR Fixes

### Fixing Common Issues Automatically

### 1. Rust Formatting (Auto-Fix)

**Workflow**: Add a job that runs `cargo fmt` and commits changes

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/auto-format.yml`

```yaml
name: Auto Format

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  format:
    runs-on: ubuntu-latest
    if: |
      !contains(github.event.pull_request.labels.*.name, 'skip-formatting')
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.head_ref }}

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Run cargo fmt
        run: cargo fmt --all

      - name: Check formatting
        id: fmt
        run: |
          if git diff --quiet; then
            echo "formatted=false" >> $GITHUB_OUTPUT
          else
            echo "formatted=true" >> $GITHUB_OUTPUT
          fi

      - name: Commit formatting changes
        if: steps.fmt.outputs.formatted == 'true'
        uses: EndBug/add-and-commit@v9
        with:
          author_name: 'formatting-bot'
          author_email: 'bot@phenotype.local'
          message: 'style: auto-format Rust code'
          add: '*.rs'
          push: true
```

### 2. Dependency Updates (Auto-Merge)

Configure Dependabot to auto-merge patch version updates:

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/dependabot.yml`

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
    auto-merge:
      - match:
          update-types: ["minor", "patch"]
          # Don't auto-merge major versions
      dependency-type: "direct"
    reviewers: ["@KooshaPari"]

  - package-ecosystem: pip
    directory: "/python"
    schedule:
      interval: weekly
    allow:
      - dependency-type: "direct"
```

Then add auto-merge condition in CI:

```yaml
# .github/workflows/auto-merge-dependabot.yml
name: Auto Merge Dependabot

on: pull_request

jobs:
  auto-merge:
    runs-on: ubuntu-latest
    if: |
      github.actor == 'dependabot[bot]' &&
      github.event.pull_request.draft == false
    steps:
      - name: Auto-merge patch updates
        uses: actions/github-script@v6
        with:
          script: |
            const title = context.payload.pull_request.title;
            const isPatch = title.includes('patch') ||
                          title.includes('bump') && !title.includes('major');

            if (isPatch) {
              await github.rest.pulls.merge({
                owner: context.repo.owner,
                repo: context.repo.repo,
                pull_number: context.issue.number,
                merge_method: 'squash'
              });
            }
```

### 3. Markdown Linting Auto-Fix

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/auto-fix-markdown.yml`

```yaml
name: Auto Fix Markdown

on:
  pull_request:
    types: [opened, synchronize]
    paths:
      - '**.md'

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.head_ref }}

      - uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install markdownlint
        run: npm install -g markdownlint-cli

      - name: Run markdownlint
        id: lint
        run: |
          if markdownlint-cli . --fix; then
            echo "fixed=false" >> $GITHUB_OUTPUT
          else
            echo "fixed=true" >> $GITHUB_OUTPUT
          fi
        continue-on-error: true

      - name: Commit lint fixes
        if: steps.lint.outputs.fixed == 'true'
        uses: EndBug/add-and-commit@v9
        with:
          author_name: 'lint-bot'
          author_email: 'bot@phenotype.local'
          message: 'docs: fix markdown linting issues'
          add: '**.md'
          push: true
```

---

## PR Template Setup

### Root PR Template

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/PULL_REQUEST_TEMPLATE.md`

```markdown
# Pull Request

## Description
<!-- Provide a brief description of the changes -->

## Type of Change
<!-- Mark relevant options with [x] -->
- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🔧 Configuration change
- [ ] 🎨 Code style/formatting
- [ ] ♻️ Refactoring (no functional changes)
- [ ] ⚡ Performance improvement
- [ ] 🧪 Test updates
- [ ] 🔒 Security fix

## Related Issues
<!-- Link to related AgilePlus specs or GitHub issues -->
Closes #
Addresses #
Related to #

**AgilePlus Spec** (if applicable):
<!-- Reference spec format: eco-001-NNN or feature-id-NNN -->
Spec: `<feature-id>`

## Changes Made
<!-- List the main changes made in this PR -->
-
-
-

## Testing
<!-- Describe the testing performed -->
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Added new tests for new functionality

### Test Coverage
<!-- If applicable, mention test coverage changes -->
- Current coverage: %
- Coverage change: +/-%

## Security Considerations
<!-- Address any security implications -->
- [ ] No sensitive data exposed
- [ ] Authentication/authorization unchanged
- [ ] Dependencies scanned for vulnerabilities
- [ ] Input validation implemented where needed

## Performance Impact
<!-- Assess performance implications -->
- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance impact assessed and acceptable
- [ ] Performance benchmarks added

## Breaking Changes
<!-- List any breaking changes -->
-
-

## Migration Guide
<!-- If breaking changes exist, provide migration steps -->
1.
2.
3.

## Checklist
<!-- Mark completed items with [x] -->
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Code is commented where complex
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All CI checks pass
- [ ] No merge conflicts
- [ ] Commit messages follow conventional commit format
- [ ] No secrets or sensitive data in code

## Reviewer Notes
<!-- Additional notes for reviewers -->
- Focus areas:
- Known limitations:
- Future improvements:
```

### Project-Specific Templates

For specialized projects (e.g., `AgilePlus`, `heliosCLI`), customize templates:

**File**: `AgilePlus/.github/PULL_REQUEST_TEMPLATE.md`

```markdown
# Pull Request — AgilePlus

## Description
<!-- Provide a brief description of the changes -->

## Feature/Spec Reference
<!-- Link to AgilePlus spec or feature -->
Feature: `eco-NNN` or `dashboard-overhaul`
Work Package: `WP-01`, `WP-02`, etc.

## Changes Made
- [ ] Dashboard component updates
- [ ] CLI command changes
- [ ] API changes
- [ ] Configuration changes
- [ ] Documentation updates

## Work Package Status
<!-- Update AgilePlus work package status -->
```
agileplus status <feature-id> --wp <wp-id> --state implemented
```

## Checklist
- [ ] Feature spec followed
- [ ] AgilePlus work package status updated
- [ ] Tests added
- [ ] Dashboard tested (if UI changes)
- [ ] CLI tested locally
- [ ] Docs updated
```

---

## Review SLAs and Notifications

### Review SLAs

Establish clear expectations for review time:

| PR Type | Target Review Time | Max Time |
|---------|-------------------:|----------:|
| Documentation | 24 hours | 48 hours |
| Bug fix | 6 hours | 24 hours |
| Feature | 24 hours | 48 hours |
| Security fix | 2 hours | 4 hours |
| Critical infra | 1 hour | 2 hours |
| Refactoring | 24 hours | 48 hours |
| Hotfix | 30 min | 1 hour |

### Notification Strategy

**GitHub Notifications**:
1. Watch the repo for PR reviews
2. Configure notification frequency in settings
3. Use labels to filter notifications

**Email Notifications**:
- Enable "Pull request reviews" notifications
- Set to "Email on notify" or "Email on mention"

### Slack Integration (Optional)

Add Slack notifications for PR status:

**Install**: GitHub Slack Integration → Phenotype workspace → Subscribe to PR events

**Configuration**:
```
/github subscribe KooshaPari/phenotype-infrakit pulls,reviews
/github set checks auto
```

Then configure CodeRabbit to post Slack messages:

```yaml
# .coderabbit.yaml
integrations:
  slack:
    enabled: false  # Enable when Slack workspace ready
    notify_on_request: false
    notify_on_completion: true
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

---

## Cost Analysis

### Free Tools Summary

| Tool | Cost | Free Tier | Unlimited? |
|------|------|-----------|-----------|
| CodeRabbit | Free | Yes | Yes |
| GitHub Actions | Free | Yes* | Yes** |
| GitHub Code Review | Free | Yes | Yes |
| Dependabot | Free | Yes | Yes |
| CodeQL | Free | Yes | Yes |
| Cargo Audit | Free | Yes | Yes |
| Gitleaks | Free | Yes | Yes |
| OSV Scanner | Free | Yes | Yes |

**GitHub Actions Billing Notes**:
- **Free for public repos**: 3,000 minutes/month for private repos
- **Phenotype Billing Issue**: Spending limit reached on KooshaPari account
- **Workaround**: Use local runners or GitHub-hosted Linux runners (free for all)
- **Billed Runners**: macOS, Windows runners consume paid minutes (SKIPPED per policy)

### Paid Options (Not Recommended for Phenotype)

| Tool | Cost | Why Skip |
|------|------|----------|
| Snyk | $25+/user/month | CodeQL + Cargo Audit sufficient |
| DeepSource | $10-50/team/month | CodeRabbit covers code quality |
| GitHub Enterprise | $21/user/month | Not needed for current scale |

### Total Cost for Phenotype

**$0/month** (all free or already-included tools)

- CodeRabbit: $0 (free tier unlimited)
- GitHub Actions: $0 (public repo + Linux runners)
- Dependabot: $0 (GitHub-included)
- CodeQL: $0 (GitHub-included)
- GitHub Code Review: $0 (GitHub-included)

---

## Cloud Agents as Secondary Reviewers

### When to Involve Cloud Agents

Use Claude or other AI agents for secondary review when:
1. PR is complex or touches multiple systems
2. CodeRabbit's review needs expert-level validation
3. Architectural decisions need deeper analysis
4. Performance implications need profiling

### Agent-Driven Review Workflow

```
PR Submitted
    ↓
CodeRabbit AI review (40% weight)
    ↓
[If complexity > threshold]
    ↓
Launch Claude Agent for deeper review
  - Analyze architecture implications
  - Review performance impact
  - Suggest optimizations
  - Check for edge cases
    ↓
Agent posts summary comment on PR
    ↓
Human review (CodeOwners) + Agent summary
    ↓
Approve or request changes
```

### Triggering Agent Review

Create a workflow that launches agents on large PRs:

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/agent-review.yml`

```yaml
name: Agent Review (Experimental)

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  agent-review:
    runs-on: ubuntu-latest
    if: |
      github.event.pull_request.additions > 500 ||
      contains(github.event.pull_request.labels.*.name, 'needs-agent-review')
    steps:
      - uses: actions/checkout@v4

      - name: Fetch PR details
        id: pr
        uses: actions/github-script@v6
        with:
          script: |
            const pr = context.payload.pull_request;
            const diffUrl = `${{ github.server_url }}/${{ github.repository }}/pull/${{ github.event.number }}.diff`;

            core.setOutput('title', pr.title);
            core.setOutput('body', pr.body);
            core.setOutput('diff_url', diffUrl);

      - name: Request agent review (Future Integration)
        run: |
          echo "Agent Review Trigger:"
          echo "  PR Title: ${{ steps.pr.outputs.title }}"
          echo "  Additions: ${{ github.event.pull_request.additions }}"
          echo "  Deletions: ${{ github.event.pull_request.deletions }}"
          echo ""
          echo "TODO: Integrate with Claude API for agent-driven review"
```

### Manual Agent Review Process

Until automated agent integration is ready, reviewers can:

1. Comment `/analyze-agent` on PR
2. Agent (via Slack/Discord) analyzes PR diff
3. Agent posts review summary comment
4. Human reviewer uses agent summary + CodeRabbit feedback

---

## Merge Gate Configuration

### Auto-Merge Conditions

Configure merge gates in `review.toml`:

```toml
[merge_gates]
require_all_checks_pass = true
require_approval = true
require_ci_green = true
require_security_scan = true
allow_stale_reviews = false
dismiss_stale_reviews_on_push = true

[auto_merge]
enabled = true
min_reviews_required = 1  # Approval from 1 code owner
auto_squash = true
delete_branch_on_merge = true

conditions = [
  "all_checks_pass",
  "approval_threshold_met",
  "no_conflicts",
  "branch_up_to_date",
  "no_ci_failures",
]
```

### Approval Thresholds

```toml
[consensus]
approval_threshold = 0.70          # 70% approval for normal changes
critical_threshold = 0.85          # 85% for critical/security
hotfix_threshold = 0.50            # 50% for urgent hotfixes
```

### Critical File Patterns

Files requiring 85% approval (or human review):

```toml
critical_patterns = [
  "**/Cargo.toml",
  "*.lock",
  "crates/phenotype-*/src/**",
  ".github/workflows/**",
  "review.toml",
  ".coderabbit.yaml",
]

force_approve_patterns = [
  "docs/**",
  "*.md",
]
```

### Implementing Auto-Merge

**GitHub UI Method**:
1. Open PR → Enable auto-merge → Squash and merge

**GitHub API Method**:
```bash
gh pr merge <PR_NUMBER> \
  --auto \
  --squash \
  --delete-branch
```

**GitHub Actions Method**:
```yaml
# .github/workflows/auto-merge-approved.yml
name: Auto Merge

on:
  pull_request:
    types: [opened, synchronize, labeled]

jobs:
  auto-merge:
    runs-on: ubuntu-latest
    if: |
      contains(github.event.pull_request.labels.*.name, 'auto-merge') &&
      github.event.pull_request.draft == false
    steps:
      - uses: actions/github-script@v6
        with:
          script: |
            const checks = await github.rest.checks.listForRef({
              owner: context.repo.owner,
              repo: context.repo.repo,
              ref: context.payload.pull_request.head.sha
            });

            const allPass = checks.data.check_runs.every(
              check => check.conclusion === 'success'
            );

            if (allPass) {
              await github.rest.pulls.merge({
                owner: context.repo.owner,
                repo: context.repo.repo,
                pull_number: context.issue.number,
                merge_method: 'squash',
                commit_message: `merge: ${{ github.event.pull_request.title }}`
              });
            }
```

---

## Integration Guide

### Step 1: Enable CodeRabbit

1. Go to https://coderabbit.ai/
2. Sign in with GitHub
3. Install CodeRabbit app on repos (phenotype-infrakit, AgilePlus, etc.)
4. CodeRabbit will start reviewing all PRs automatically

**Verification**: Next PR should show CodeRabbit review comment

### Step 2: Configure CodeOwners

1. Create/update `/CODEOWNERS`:
```
* @KooshaPari
.github/workflows/ @KooshaPari
crates/ @KooshaPari
```

2. Enable CodeOwners enforcement in branch protection:
   - Go to Repo → Settings → Branches → main
   - Check "Require code owner review"

### Step 3: Configure Branch Protection

1. Go to Repo → Settings → Branches → Add rule
2. Apply to: `main`
3. Enable:
   - Require pull request reviews (1 reviewer)
   - Require status checks: CodeQL, Cargo Audit, Cargo Deny, Gitleaks
   - Require up-to-date branches
   - Require conversation resolution
   - Require signed commits

### Step 4: Setup PR Template

1. Create `.github/PULL_REQUEST_TEMPLATE.md` (from this guide)
2. Customize for each project if needed

### Step 5: Enable Dependabot (Optional)

1. Create `.github/dependabot.yml`
2. Configure package ecosystems (cargo, pip, npm)
3. Set auto-merge for patch updates (optional)

### Step 6: Add Custom Workflows (Optional)

1. Create `.github/workflows/auto-format.yml`
2. Create `.github/workflows/request-review.yml`
3. Create `.github/workflows/auto-merge-dependabot.yml`

---

## Troubleshooting

### CodeRabbit Not Reviewing

**Symptom**: PR submitted but no CodeRabbit review comment.

**Solutions**:
1. Check CodeRabbit is installed: GitHub Settings → Installed GitHub Apps → CodeRabbit
2. Verify `.coderabbit.yaml` exists and is valid YAML
3. Check PR is not a draft (CodeRabbit skips drafts by default)
4. Check repository is public or CodeRabbit is installed on private repo

### Reviews Not Blocking Merge

**Symptom**: PR can be merged even though reviews are "required".

**Solutions**:
1. Check branch protection is enabled: Settings → Branches → main → Require pull request reviews
2. Verify required reviewers count is set to 1+
3. Check if PR author is code owner (they can self-approve in some configs)

### Status Checks Failing

**Symptom**: CI/CD status checks failing on every PR.

**Solutions**:
1. Check Actions have permissions: Settings → Actions → Permissions → Allow all actions
2. Verify Ubuntu runners (Linux) are used (not macOS/Windows due to billing)
3. Check workflow file syntax: `gh workflow validate .github/workflows/*.yml`
4. Review CI logs: Actions → Workflow run → Job details

### Auto-Merge Not Working

**Symptom**: PR has all checks passing but won't auto-merge.

**Solutions**:
1. Enable auto-merge on PR: PR → Enable auto-merge (GitHub UI)
2. Or enable in workflow: `gh pr merge <PR> --auto --squash`
3. Verify merge conditions are met (approvals, checks, no conflicts)
4. Check if PR is draft (auto-merge disabled for drafts)

### CodeOwners Not Enforcing

**Symptom**: PRs can merge without code owner approval.

**Solutions**:
1. Verify CODEOWNERS file exists and has correct syntax
2. Check branch protection has "Require code owner review" enabled
3. Verify code owner account has proper permissions
4. Test with a small PR to specific file pattern

### Gitleaks Hanging

**Symptom**: Gitleaks job hangs indefinitely (known issue).

**Solutions**:
1. Kill hung process: `pkill -f gitleaks`
2. Switch to `trufflehog`: Use in quality-gate.sh instead
3. Remove gitleaks from workflows and use trufflehog v3.93.6

```bash
# Test locally:
trufflehog git file://. --since-commit HEAD --only-verified --fail
```

---

## Summary Table

| Need | Tool | Free? | Config |
|------|------|-------|--------|
| Automated code review | CodeRabbit | Yes | `.coderabbit.yaml` |
| Manual code review | GitHub UI | Yes | CODEOWNERS |
| Dependency updates | Dependabot | Yes | `.github/dependabot.yml` |
| Security scanning | Cargo Audit, CodeQL | Yes | `.github/workflows/security.yml` |
| Branch protection | GitHub | Yes | Repo settings |
| Auto-merge | GitHub Actions | Yes | `.github/workflows/auto-merge.yml` |
| Slack notifications | GitHub + Slack | Yes | Slack integration |
| Agent-driven review | Claude agents | TBD | `.github/workflows/agent-review.yml` |

---

## Next Steps

1. **Immediate**: Verify CodeRabbit is working on all repos
2. **Week 1**: Configure branch protection for `main` branch
3. **Week 1**: Set up PR templates for each major project
4. **Week 2**: Enable Dependabot for dependency management
5. **Week 2**: Add auto-fix workflows (formatting, markdown)
6. **Week 3**: Implement agent-driven review workflow (experimental)
7. **Ongoing**: Monitor review SLAs and adjust thresholds

---

**Last Updated**: 2026-03-30
**Author**: Code Review Infrastructure Task
**Status**: Ready for implementation
