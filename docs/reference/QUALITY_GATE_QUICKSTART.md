# Quality Gate Quick Start

**TL;DR** — 4 commands to get linting working in any Tier 1 repo:

```bash
# 1. Install pre-commit framework (one-time, global)
pip install pre-commit

# 2. Install hooks in this repo
cd <repo>
pre-commit install
pre-commit install --hook-type pre-push

# 3. Run quality gate before pushing
./scripts/quality-gate.sh

# 4. Auto-fix formatting (if needed)
./scripts/quality-gate.sh --fix
```

---

## What Gets Checked?

### Before Every Commit (Pre-Commit Hooks)
- ✅ Trailing whitespace & line endings
- ✅ YAML/TOML/JSON syntax
- ✅ No private keys committed
- ✅ No commits to main/master by accident
- ✅ Rust formatting (rustfmt)
- ✅ Commit message format (conventional-commit)

### Before Every Push (Pre-Push Hooks)
- ✅ All pre-commit checks (above)
- ✅ Rust linting (clippy)
- ✅ Python linting (ruff)
- ✅ Type checking (mypy, cargo check)
- ✅ Secret scanning (trufflehog)

### On GitHub (CI/CD)
- ✅ Format check (all languages)
- ✅ Linting (all languages)
- ✅ Type checking (strict mode)
- ✅ Security scanning (CodeQL, semgrep)

---

## Repo-Specific Setup

### heliosCLI (Rust + Python)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI

# Install hooks
pre-commit install
pre-commit install --hook-type pre-push

# Test quality gate
../../scripts/quality-gate.sh

# If it fails, auto-fix formatting
../../scripts/quality-gate.sh --fix
```

**Note**: heliosCLI has broken dependencies (phenotype-shared refs). Fix by:
1. Commenting out missing deps in `crates/harness_pyo3/Cargo.toml`
2. Running `cargo clippy --all-targets` to see all lint issues
3. Fixing issues manually or with `./scripts/quality-gate.sh --fix`

### platforms/thegent (Go + Python + Rust)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent

# Install hooks (if not already done)
pre-commit install
pre-commit install --hook-type pre-push

# Test quality gate
../../scripts/quality-gate.sh
```

**Note**: thegent already has comprehensive pre-commit config. Hooks should work out of the box.

### phenotype-infrakit (Rust - Scaffold)

Currently a scaffold (README only). When crates are added:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

# Copy .pre-commit-config.yaml template from guide
cp <path-to-template> .pre-commit-config.yaml

# Install hooks
pre-commit install
pre-commit install --hook-type pre-push

# Test quality gate
../../scripts/quality-gate.sh
```

---

## Troubleshooting

### "Pre-commit not found"
```bash
pip install pre-commit
```

### "Hooks not running"
```bash
# Reinstall
pre-commit install
pre-commit install --hook-type pre-push

# Test manually
pre-commit run --all-files
```

### "Formatting errors on commit"
```bash
# Auto-fix
./scripts/quality-gate.sh --fix

# Or manually
cargo fmt
ruff format .
gofmt -w .

# Re-stage and commit
git add .
git commit -m "fix: format code"
```

### "Bypass hooks (emergency only)"
```bash
git commit --no-verify
git push --no-verify
```

⚠️ Never bypass on main/master

---

## Files & Locations

| File | Purpose | Location |
|------|---------|----------|
| LINTING_AND_QUALITY_SETUP.md | Complete guide (4 layers, all tools) | `/docs/reference/` |
| QUALITY_BASELINE_REPORT.md | Baseline status & phase 1 completion | `/docs/reference/` |
| quality-gate.sh | Universal quality gate script | `/scripts/` |
| .pre-commit-config.yaml | Hook config (language-specific) | Per-repo root |
| rustfmt.toml | Rust formatting rules | Per-repo root (Rust projects) |
| pyproject.toml | Python tool config | Per-repo root (Python projects) |

---

## Next: Full Implementation

See **LINTING_AND_QUALITY_SETUP.md** for:
- All 4 quality gate layers (pre-commit, pre-push, GitHub Actions, branch protection)
- Language-specific tool configs
- Detailed troubleshooting
- Auto-fix guidelines
- Suppression policy

