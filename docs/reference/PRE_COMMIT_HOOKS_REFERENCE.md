# Pre-Commit Hooks Reference

**Status**: Configuration ready in `.pre-commit-config.yaml`
**Setup Time**: 10 minutes
**Runtime**: 30-80 seconds (first run), 5-15 seconds (cached)

---

## Quick Setup

```bash
# Install framework (one-time)
pip install pre-commit

# Install hooks in repo
pre-commit install
pre-commit install --hook-type commit-msg
pre-commit install --hook-type pre-push

# Test all hooks
pre-commit run --all-files

# Update hook versions
pre-commit autoupdate
```

---

## Hook Configuration

All hooks are defined in `.pre-commit-config.yaml` (root). Complete reference below:

### Tier 1: Basic File Checks

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| trailing-whitespace | Remove trailing spaces | commit | <100ms | No |
| end-of-file-fixer | Ensure newline at EOF | commit | <100ms | No |
| check-yaml | Validate YAML syntax | commit | <200ms | Yes |
| check-toml | Validate TOML syntax | commit | <200ms | Yes |
| check-json | Validate JSON syntax | commit | <200ms | No |
| check-merge-conflict | Detect merge conflict markers | commit | <100ms | Yes |
| check-added-large-files | Block files > 500KB | commit | <100ms | Yes |
| detect-private-key | Detect private keys (hardcoded) | commit | <500ms | Yes |
| mixed-line-ending | Normalize line endings | commit | <100ms | No |

**Run on**: All commits
**Total time**: ~2-3 seconds

---

### Tier 2: Commit Message Validation

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| conventional-pre-commit | Enforce conventional commits | commit-msg | <100ms | Yes |

**Format**: `type: description` where type = feat, fix, docs, style, refactor, perf, test, chore, ci, build, revert

**Examples**:
```
feat: add new quality gate
fix: resolve clippy warning in parser
docs: update README with setup instructions
```

**Run on**: Every commit message
**Total time**: <100ms

---

### Tier 3: Rust Checks

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| rustfmt | Auto-format Rust code | commit | 2-5s | Yes |
| clippy | Lint checks (warnings=errors) | commit | 10-30s | Yes |

**When it runs**: On any `.rs` file change

**What it does**:
- `rustfmt`: Automatically fixes formatting (no review needed)
- `clippy`: Detects style, performance, and correctness issues

**If it fails**:
```bash
# rustfmt failures (rare) — run nightly version
rustup install nightly
cargo +nightly fmt --all

# clippy failures — fix manually or use --fix
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --fix --allow-staged --allow-dirty
```

**Skip**: `SKIP=rustfmt,clippy git commit -m "..."`

**Total time**: 10-30 seconds

---

### Tier 4: Python Checks

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| ruff-format | Auto-format Python | commit | 1-3s | Yes |
| ruff-check | Lint checks (100+ rules) | commit | 2-5s | Yes |
| mypy | Type checking (strict mode) | commit | 5-10s | No (optional) |

**When it runs**: On any `.py` file change

**What it does**:
- `ruff format`: Auto-fixes all formatting (no review needed)
- `ruff check --fix`: Auto-fixes safe lints
- `mypy`: Detects type errors (warnings OK)

**If it fails**:
```bash
cd python

# Auto-fix formatting
uvx ruff format .

# Auto-fix lints
uvx ruff check --fix .

# Review type errors (mypy — optional)
uvx mypy src --strict

# Commit fixes
git add -A && git commit -m "style: auto-fix formatting"
```

**Skip**: `SKIP=ruff-format,ruff-check git commit -m "..."`

**Total time**: 8-18 seconds

---

### Tier 5: TypeScript/JavaScript Checks

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| prettier | Auto-format TS/JS | commit | 1-2s | Yes |
| oxlint | Fast linting | commit | 3-5s | Yes |

**When it runs**: On any `.ts`, `.tsx`, `.js`, `.jsx`, `.json`, `.css` file change (in heliosApp)

**What it does**:
- `prettier`: Auto-fixes all formatting
- `oxlint`: Detects style and performance issues

**If it fails**:
```bash
cd heliosApp

# Auto-fix formatting
bun run format

# Run linter
bun run lint || true

# Commit fixes
git add -A && git commit -m "style: auto-fix formatting"
```

**Skip**: `SKIP=prettier,oxlint git commit -m "..."`

**Total time**: 5-8 seconds

---

### Tier 6: Config & Proto Checks

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| buf-lint | Proto validation | commit | 1-2s | Yes |
| taplo-format | Auto-format TOML | commit | <500ms | Yes |

**When it runs**:
- buf-lint: On any `.proto` file change
- taplo: On any `.toml` file change

**What it does**:
- buf-lint: Validates protobuf syntax
- taplo: Auto-formats TOML (Cargo.toml, pyproject.toml, etc.)

**If it fails**:
```bash
# Proto errors — fix manually
# TOML format — auto-fixed, re-commit
```

**Total time**: 2-3 seconds

---

### Tier 7: Spell Checking

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| typos | Spell check (including code) | commit | <1s | No |

**When it runs**: On all files

**What it does**: Detects common typos (also, teh, recieve, etc.)

**If it fails**:
```bash
# Auto-fix typos
typos --write-changes

# Or review and fix manually
typos
```

**Skip**: `SKIP=typos git commit -m "..."`

**Total time**: <1 second

---

### Tier 8: Security (Pre-Push Only)

| Hook | Purpose | Stage | Speed | Critical? |
|------|---------|-------|-------|-----------|
| trufflehog | Secret scanning | pre-push | 5-15s | Yes |

**When it runs**: On push (not commit)

**What it does**: Scans git history for exposed secrets (API keys, passwords, etc.)

**If it fails**:
```bash
# Remove secret from code
# Rewrite commit history (if needed)
git rebase -i HEAD~1  # Edit history
git push --force-with-lease origin main
```

**Skip** (dangerous): `SKIP=trufflehog git push`

**Total time**: 5-15 seconds

---

## Hook Execution Flow

```
git commit -m "fix: something"
  ├─ [~2s] Basic file checks (whitespace, merge conflicts, large files)
  ├─ [<1s] TOML format check
  ├─ [2-5s] Rust format + clippy
  ├─ [2-5s] Python ruff format + lint
  ├─ [5-8s] TypeScript prettier + oxlint
  ├─ [<1s] Typos
  └─ [commit-msg] Conventional commit validation
      └─ Commit succeeds if all hooks pass

git push origin main
  ├─ [5-15s] trufflehog secret scanning
  └─ Push succeeds if no secrets found
```

**Total time**: 30-80 seconds (first run), 5-15 seconds (cached)

---

## Skipping Hooks

### Skip Specific Hook

```bash
SKIP=hook-id git commit -m "..."
SKIP=clippy,ruff-format git commit -m "..."
```

### Skip All Hooks (Dangerous)

```bash
git commit --no-verify -m "..."
```

### Skip Pre-Push Only

```bash
git push --no-verify
```

---

## Customization

### Disable Hook Globally

Edit `.pre-commit-config.yaml`:

```yaml
- id: rustfmt
  name: rustfmt
  ...
  stages: [push]  # Move from commit to push
```

### Change Stage (commit → push)

Some hooks are expensive (clippy). Move to pre-push:

```yaml
- id: clippy
  ...
  stages: [push]  # Only on push, not commit
```

### Add Custom Hook

```yaml
- repo: local
  hooks:
    - id: custom-check
      name: My Custom Check
      entry: bash -c 'your-command'
      language: system
      files: '\.rs$'
      stages: [commit]
```

---

## Troubleshooting

### Hooks not running

**Problem**: Installed but not triggering
**Fix**:
```bash
pre-commit install --hook-type commit-msg
pre-commit install --hook-type pre-push
pre-commit run --all-files  # Verify
```

### Hook fails on first run

**Problem**: Large codebase, reformatting needed
**Fix**:
```bash
# Let hooks fix everything
pre-commit run --all-files

# Commit the fixes
git add -A
git commit -m "style: auto-fix formatting"
```

### Hook times out

**Problem**: Clippy on large workspace takes > 30s
**Fix**:
```bash
# Move clippy to pre-push only
# Edit .pre-commit-config.yaml:
# stages: [push]
```

### "command not found"

**Problem**: Hook tries to run tool that isn't installed
**Fix**:
```bash
# Install the tool
rustup install nightly
pip install uv
```

### Pre-commit not respecting .gitignore

**Problem**: Hook checks ignored files
**Fix**: Pre-commit ignores `.gitignore` by design. Use `exclude` in hook config:

```yaml
- id: mypy
  exclude: 'tests/'
```

---

## Performance Tips

### Cache Aggressive

Rust compilation is cached per workspace. Subsequent runs are fast.

### Run in Parallel (Default)

Pre-commit already runs hooks in parallel where possible.

### Move Expensive Checks to Push

```yaml
- id: clippy
  stages: [push]  # Not on every commit
```

### Skip on Merge Commits

```bash
[[ $(git rev-parse --abbrev-ref HEAD) == "main" ]] && SKIP=<hooks> || true
```

---

## Integration with CI

**Local (pre-commit)**: Catches 99% of issues before push
**CI (GitHub Actions)**: Final verification, security scans

**Philosophy**:
- What passes locally should pass CI
- CI should add nothing new (except coverage reporting)

---

## FAQ

**Q: Can I commit without running hooks?**
A: Yes, but not recommended: `git commit --no-verify`. Better: fix locally first.

**Q: Do hooks work with `git merge`?**
A: No, only on normal commits.

**Q: Can hooks auto-commit fixes?**
A: No, pre-commit doesn't re-commit. But hooks CAN modify files. Just re-run the commit.

**Q: What if I'm on `main` and a hook fails?**
A: Fix locally, re-commit, force-push (not recommended). Better: use branches.

**Q: How do I add a hook to all projects?**
A: Symlink `.pre-commit-config.yaml` from central location, or copy to each repo.

---

## Reference

| Phase | Hook | Time | Critical |
|-------|------|------|----------|
| 1 | Basic checks (trailing, EOL, etc) | 1-2s | Yes |
| 2 | Rust (fmt + clippy) | 10-30s | Yes |
| 3 | Python (ruff format + check) | 5-10s | Yes |
| 4 | TypeScript (prettier + oxlint) | 5-8s | Yes |
| 5 | Config (TOML, proto) | 2-3s | Yes |
| 6 | Spell checking | <1s | No |
| 7 | Security (trufflehog) | 5-15s | Yes (pre-push) |

**Total**: 30-80s first run, 5-15s cached

---

**Status**: ✅ Ready to use
**Last Updated**: 2026-03-30
**Maintained By**: Phenotype Quality Engineering
