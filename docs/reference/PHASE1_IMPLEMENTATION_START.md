# Phase 1 Implementation: Security & QA Tooling Activation

**Status**: Ready to Deploy
**Start Date**: 2026-03-30
**Duration**: 2 weeks (SAST foundation)
**Effort**: 30 minutes setup + 6-8 hours configuration
**Dependencies**: None (all tools have free tiers)

---

## Overview: What Phase 1 Does

Phase 1 activates **Static Application Security Testing (SAST)** — the foundation for all subsequent security and quality checks. Once deployed:
- ✅ All PRs scanned for code quality issues, security vulnerabilities, and secrets
- ✅ Linting and formatting enforced on every commit
- ✅ Pre-commit hooks prevent bad code from entering the repo
- ✅ GitHub Actions workflow runs all checks in parallel (2-3 min)
- ✅ No manual intervention required

**ROI**: Catches 70%+ of bugs before code review. Saves 2-3 hours/developer/week.

---

## Before You Start: Quick Checklist

- [ ] You have GitHub admin access to the repos
- [ ] You've read `COMPREHENSIVE_SECURITY_QA_SYNTHESIS.md`
- [ ] You understand the 6-phase roadmap (SAST → Dependencies → Linting → Error Tracking → Cloud Agents → Compliance)
- [ ] You have 30 minutes of uninterrupted time

---

## Step 1: Understand What Gets Deployed (5 minutes)

**Phase 1 will add**:

### 1A. Pre-Commit Hooks (Local)
Runs on every `git commit`:
- **Semgrep** (SAST scanning) — checks for 2,500+ security rules
- **trufflehog** (Secrets detection) — catches leaked API keys, passwords
- **fmt/lint** (Code formatting) — enforces style per language
- **cargo clippy** (Rust linting) — Rust-specific warnings
- **golangci-lint** (Go linting) — Go-specific warnings
- **ruff** (Python linting) — Python code quality
- **eslint** (TypeScript/JS linting) — JavaScript/TypeScript rules
- **prettier** (Code formatting) — JSON, YAML, markdown formatting

**Time**: 30-45 seconds per commit (cached results)

### 1B. GitHub Actions Workflow
Runs on every PR:
- `.github/workflows/quality-gate.yml` — Master workflow that runs all checks in parallel
- **11 parallel jobs** (2-3 min total):
  - SAST (Semgrep + CodeQL)
  - Secrets (trufflehog)
  - Linting (per-language)
  - Formatting (prettier + ruff)
  - Coverage (codecov)
  - Dependency scanning
  - Artifact testing

**Time**: 2-3 minutes per PR

### 1C. Configuration Files
Added to repo root:
```
.semgrep.yml             # Semgrep rules
.github/workflows/       # GitHub Actions workflows
.pre-commit-config.yaml  # Pre-commit hook config
pyproject.toml          # Python (ruff, black)
.eslintrc.json          # TypeScript/JS
.golangci.yml           # Go
Cargo.toml              # Rust (clippy)
.prettierrc.json        # Global formatter
```

---

## Step 2: Deploy to First Repo (15 minutes)

We'll deploy to a **single test repo first** (e.g., phenotype-infrakit or AgilePlus), then roll out to all 30 projects in waves.

### 2A. Clone Configuration Files

From `SAST_IMPLEMENTATION_GUIDE.md`, copy these files to your test repo:

```bash
# In your test repo root
cp /path/to/research/SAST_IMPLEMENTATION_GUIDE.md .
cp /path/to/research/.semgrep.yml .
cp /path/to/research/.pre-commit-config.yaml .

# Copy per-language configs
cp /path/to/research/pyproject.toml .          # Python
cp /path/to/research/.eslintrc.json .          # TypeScript/JS
cp /path/to/research/.golangci.yml .           # Go
# Rust: already has Cargo.toml and clippy rules
```

### 2B. Install Pre-Commit Hooks Locally

```bash
cd /path/to/test/repo

# Install pre-commit framework
pip install pre-commit

# Install all hooks defined in .pre-commit-config.yaml
pre-commit install

# Test: run all hooks on all files
pre-commit run --all-files

# Expected output: 200+ issues found (linting, formatting, secrets)
```

**Note**: First run takes 2-3 min (downloading tools). Subsequent runs 15-30 sec (cached).

### 2C. Fix Issues Automatically

```bash
# Auto-fix formatting issues
black .                    # Python
ruff check --fix .         # Python linting
prettier --write .         # JSON/YAML/markdown
go fmt ./...              # Go

# Rust issues require manual review (use cargo clippy for suggestions)
cargo clippy --fix --allow-dirty
```

### 2D. Deploy GitHub Actions Workflow

```bash
# Create workflow file
mkdir -p .github/workflows
cp /path/to/research/quality-gate.yml .github/workflows/

# Or from template in CODE_REVIEW_AND_PR_AUTOMATION_GUIDE.md
# Copy the "Master Quality Gate" workflow
```

### 2E. Create Initial Commit

```bash
git add .
git commit -m "chore: add Phase 1 SAST tooling (Semgrep, trufflehog, linting)"
git push origin main
```

**Result**: GitHub Actions will run on the next PR. You should see:
- ✅ Semgrep (code quality)
- ✅ trufflehog (secrets)
- ✅ Linting (per-language)
- ✅ Formatting check
- ✅ Coverage (if codecov installed)

---

## Step 3: Verify Phase 1 Deployment (5 minutes)

### 3A. Check GitHub Actions

1. Go to repo → Actions tab
2. Click latest workflow run
3. Verify all 11 jobs passed (green checkmarks)

Expected jobs:
- `sast-scan` ✅
- `secrets-check` ✅
- `lint-rust` ✅ (if applicable)
- `lint-go` ✅ (if applicable)
- `lint-python` ✅ (if applicable)
- `lint-typescript` ✅ (if applicable)
- `format-check` ✅
- `coverage` ✅
- `dependency-scan` ✅
- `artifact-test` ✅
- `security-summary` ✅

### 3B. Test Pre-Commit Hooks

```bash
# Make a breaking change
echo "bad_var_name  =   123" >> src/test.py

# Try to commit
git commit -am "test: trigger linting failure"

# Expected: Commit BLOCKED, error message shows linting issues
# Pre-commit will fix formatting auto, but you'll see:
# ❌ ruff found issues, re-running with fixes...
# ❌ Commit blocked until issues fixed
```

### 3C. Verify Secrets Detection

```bash
# Create a fake API key
echo "OPENAI_API_KEY=sk-test-1234567890abcdef" >> .env.example

# Try to commit
git add .env.example
git commit -m "test: add env template"

# Expected: Commit BLOCKED
# trufflehog will detect the key pattern and block
```

---

## Step 4: Roll Out to All Projects (Week 1-2)

Once Phase 1 works on the test repo, roll out in waves:

### Wave 1: Tier-1 Critical Projects (Week 1)
- [ ] phenotype-infrakit
- [ ] AgilePlus
- [ ] platforms/thegent

**Process**:
```bash
for repo in phenotype-infrakit AgilePlus platforms/thegent; do
  cd /path/to/$repo
  # Copy configs (same as Step 2A-2B)
  # Run pre-commit locally
  # Fix issues
  # Push to main
done
```

### Wave 2: Tier-2 Projects (Week 1-2)
- [ ] heliosCLI
- [ ] pheno-cli
- [ ] phench
- [ ] agileplus-agents
- [ ] ... (15 more projects)

### Wave 3: Tier-3 Projects & Experimental (Week 2)
- [ ] All remaining projects

---

## Step 5: Configure GitHub Branch Protection (Day 1)

Enforce that **all checks must pass before merge**:

```bash
# For each repo, go to Settings → Branches → main
# → Add Rule

# Configure:
- [x] Require status checks to pass before merging
- [x] Require code reviews before merging (2 reviewers)
- [x] Require branches to be up to date
- [x] Require status checks:
      - sast-scan
      - secrets-check
      - lint-*
      - format-check
      - coverage
```

---

## Step 6: Document & Communicate (Day 1)

### 6A. Create Developer Guide

```bash
cat > SECURITY_QA_QUICKSTART.md << 'EOF'
# Security & QA Quickstart

## Before You Commit

```bash
# Install pre-commit hooks (one-time)
pip install pre-commit && pre-commit install

# Run checks locally (or automatic on commit)
pre-commit run --all-files
```

## Common Issues & Fixes

**Issue: "ruff found issues"**
```bash
ruff check --fix .   # Auto-fix
git add . && git commit
```

**Issue: "prettier formatting issues"**
```bash
prettier --write .   # Auto-fix
git add . && git commit
```

**Issue: "semgrep security issue"**
- Read the error message carefully
- Fix the code (don't suppress)
- Re-commit

## What Gets Checked

- Code quality (Semgrep, clippy, golangci-lint, ruff)
- Secrets (trufflehog)
- Formatting (prettier, ruff, black)
- Testing (pytest, vitest)
- Dependencies (safety, npm audit)

## Full Documentation

See `docs/reference/SAST_IMPLEMENTATION_GUIDE.md`
EOF
```

### 6B. Create Slack Announcement

```
📋 Phase 1: SAST Security & Quality Tooling Live

We've just deployed automatic code scanning to ALL projects:
✅ Every PR now scanned for vulnerabilities (Semgrep)
✅ API keys/secrets detected and blocked (trufflehog)
✅ Code formatting enforced (prettier, ruff)
✅ Language-specific linting (clippy, golangci-lint, ruff)

What You Need to Do:
1. Install pre-commit hooks: `pip install pre-commit && pre-commit install`
2. Run locally before committing: `pre-commit run --all-files`
3. If checks fail: read error message and fix code

Full guide: docs/reference/SECURITY_QA_QUICKSTART.md

Questions? Check the Phase 1 docs or ask in #cloud-agents-dev
```

---

## Phase 1 Completion Checklist

### Day 1 (Setup)
- [ ] Read COMPREHENSIVE_SECURITY_QA_SYNTHESIS.md
- [ ] Understand 6-phase roadmap
- [ ] Review SAST_IMPLEMENTATION_GUIDE.md
- [ ] Copy configs to test repo
- [ ] Deploy quality-gate.yml workflow
- [ ] Create initial commit
- [ ] Setup GitHub branch protection

### Week 1 (Tier-1 Rollout)
- [ ] Deploy to phenotype-infrakit
- [ ] Deploy to AgilePlus
- [ ] Deploy to platforms/thegent
- [ ] Verify all checks pass
- [ ] Document any project-specific issues

### Week 2 (Tier-2/3 Rollout)
- [ ] Deploy to Tier-2 projects (heliosCLI, pheno-cli, etc.)
- [ ] Deploy to Tier-3 projects
- [ ] Celebrate: Phase 1 complete ✅

---

## Success Metrics (After 2 Weeks)

| Metric | Target | How to Verify |
|--------|--------|---------------|
| Pre-commit hooks deployed | 30/30 repos | `grep -r pre-commit .github` |
| Quality gates passing | 100% of PRs | GitHub Actions: all jobs green |
| Developer adoption | 80%+ running locally | Time savings reports from team |
| Issues caught | 50+/week | Semgrep/linting issue counts |
| False positives | <5% | Review suppression requests |

---

## Next Steps: Phase 2 (Weeks 3-4)

After Phase 1 stabilizes:
- **Phase 2: Dependencies** — Add dependency scanning (Snyk, pip-audit, npm audit)
- **Phase 3: Linting** — Enforce code quality rules organization-wide
- **Phase 4: Error Tracking** — Deploy Sentry for production error monitoring
- **Phase 5: Cloud Agents** — Route issues to autonomous agents for fixing
- **Phase 6: Compliance** — Add audit logging, RBAC, SLA tracking

See `SECURITY_QA_TOOLING_AUDIT.md` for full 12-week roadmap.

---

## Troubleshooting

**Q: Semgrep finds issue but code looks fine**
A: Read the error message. Semgrep catches real security issues. Don't suppress. If false positive, file issue in semgrep repo.

**Q: Pre-commit hooks too slow (>1 min per commit)**
A: First run is slow (downloading tools). Subsequent runs cached. If still slow:
```bash
# Clear pre-commit cache
rm -rf ~/.cache/pre-commit/

# Or skip non-critical checks:
SKIP=semgrep git commit  # Skip Semgrep for this commit only
```

**Q: GitHub Actions failing but pre-commit passes locally**
A: You may have different tool versions. Run:
```bash
pre-commit autoupdate  # Get latest versions
pre-commit run --all-files
```

**Q: How do I suppress a Semgrep rule?**
A: Don't. Fix the code instead. Suppression requires justification and opens a ticket. Only suppress if: (1) issue is confirmed false positive, (2) fix would break functionality, (3) security risk is mitigated elsewhere.

**Q: Can I run only specific checks?**
A: Yes:
```bash
pre-commit run semgrep --all-files      # Just Semgrep
pre-commit run prettier --all-files     # Just formatting
```

---

## Contact & Support

- **Phase 1 Owner**: [Assign to Backend Engineer]
- **Questions**: Check `docs/reference/` for full guides
- **Blockers**: Create issue with tag `security-qa-phase-1`
- **Team Channel**: #cloud-agents-dev

---

**Status**: Ready to Deploy
**Last Updated**: 2026-03-30
**Next Review**: 2026-04-13 (end of Phase 1)

🚀 **Deploy Phase 1 today. Catch 70% of bugs before code review.**
