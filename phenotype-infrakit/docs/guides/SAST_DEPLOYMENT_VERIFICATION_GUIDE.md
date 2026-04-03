# SAST Deployment Verification Guide

**Quick Reference for Developers**

After SAST deployment, verify your repo's security configuration using this guide.

## 1-Minute Health Check

```bash
# Navigate to your repo
cd /repos/<your-repo-name>

# Verify all files exist
ls -la .github/workflows/security-guard.yml
ls -la .semgrep.yaml
ls -la .pre-commit-config.yaml

# Check pre-commit hooks
pre-commit run --all-files --dry-run
```

## Files Present?

### All 3 Files Required:

✅ `.github/workflows/security-guard.yml`
- Contains GitHub Actions workflow for CI/CD
- Triggers on PR and push

✅ `.semgrep.yaml`
- Configuration for Semgrep SAST scanner
- Includes rules and exclusions

✅ `.pre-commit-config.yaml`
- Local pre-commit hooks for developer machines
- Includes Semgrep, TruffleHog, language-specific linters

## Local Testing (5 minutes)

### Step 1: Install pre-commit framework

```bash
pip install pre-commit
```

### Step 2: Install all hooks

```bash
pre-commit install
pre-commit install --hook-type pre-push
```

### Step 3: Run security checks

```bash
# Test pre-commit stage (runs on git commit)
pre-commit run --hook-stage pre-commit --all-files

# Test pre-push stage (runs on git push)
pre-commit run --hook-stage pre-push --all-files
```

### Expected Output

Success (no issues):
```
Semgrep....................................................................Passed
TruffleHog.................................................................Passed
trailing-whitespace........................................................Passed
end-of-file-fixer...........................................................Passed
check-yaml.................................................................Passed
[eslint|ruff|cargo clippy]..................................................Passed
```

Failure (found issue):
```
Semgrep....................................................................Failed
- hook id: semgrep
- exit code: 1
- files changed
```

## Debugging Semgrep Results

### View all Semgrep findings

```bash
semgrep --config .semgrep.yaml --json
```

### Filter by severity

```bash
semgrep --config .semgrep.yaml --error  # Only ERROR level
semgrep --config .semgrep.yaml --warn   # WARNING and ERROR
```

### Test specific file

```bash
semgrep --config .semgrep.yaml src/main.rs
```

## Language-Specific Hooks

### Rust Projects

```bash
# Format check
cargo fmt -- --check

# Linting
cargo clippy --all-targets -- -D warnings
```

### Go Projects

```bash
# Install golangci-lint
brew install golangci-lint  # macOS
# or: sudo apt-get install golangci-lint  # Linux

# Run linter
golangci-lint run
```

### Python Projects

```bash
# Install ruff
pip install ruff

# Check code
ruff check .

# Format
ruff format .
```

### TypeScript/JavaScript Projects

```bash
# Install dependencies
npm install -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin

# Run ESLint
npx eslint .
```

## Handling Security Findings

### False Positive (Need to Ignore)

#### Semgrep False Positive

1. Identify the rule ID from the error (e.g., `typescript.express.security.audit.express-open-redirect`)

2. Create `.semgrep-allowlist.json`:
```json
{
  "skip_rules": [
    "typescript.express.security.audit.express-open-redirect"
  ]
}
```

3. Update `.semgrep.yaml`:
```yaml
# Add to rules section
rules:
  - id: skip-false-positives
    message: Skip allowlisted rules
    skip: ${SEMGREP_ALLOWLIST}
```

#### TruffleHog False Positive

Create `.trufflehog-allowlist.json`:
```json
{
  "skip": [
    "example-api-key-for-testing",
    "test-database-password"
  ]
}
```

### Legitimate Issue (Need to Fix)

1. Review the finding
2. Apply the recommended fix
3. Re-run: `pre-commit run --hook-stage pre-commit --all-files`

## Common Issues & Solutions

### "Semgrep command not found"

```bash
# Install Python and Semgrep
pip install semgrep

# Verify
semgrep --version
```

### "TruffleHog fails in pre-push"

```bash
# Check if git is initialized
git status

# Check remote
git remote -v

# Manually test
trufflehog git file://.
```

### "ESLint/Clippy/Ruff not found in CI"

Add to your GitHub Actions workflow (e.g., in `.github/workflows/ci.yml`):

**Rust:**
```yaml
- uses: dtolnay/rust-toolchain@stable
```

**Go:**
```yaml
- uses: golangci/golangci-lint-action@v3
```

**Python:**
```yaml
- uses: actions/setup-python@v5
- run: pip install ruff
```

**TypeScript:**
```yaml
- uses: actions/setup-node@v4
- run: npm ci
```

### "Pre-commit hook stage mismatch"

```bash
# List all hooks and their stages
pre-commit validate-manifest

# Install specific stage
pre-commit install --hook-type commit-msg
pre-commit install --hook-type pre-push
```

## Running Security Checks Manually

### Quick scan (30 seconds)

```bash
semgrep --quick --config .semgrep.yaml
```

### Deep scan (2-5 minutes)

```bash
semgrep --config .semgrep.yaml --deep
```

### Scan specific directory

```bash
semgrep --config .semgrep.yaml src/ tests/
```

## GitHub Actions Integration

### Where to Find Results

1. Navigate to your PR on GitHub
2. Scroll to "Checks" section
3. Click "Security Guard" workflow
4. View detailed logs

### Interpreting Results

✅ **Passed:** All security checks clean
- No Semgrep findings
- No secrets detected
- All language-specific checks pass

❌ **Failed:** Security issue found
- Click "Details" to see full output
- Review Semgrep findings with line numbers
- Fix in your branch and push again

## Deployment Configuration Details

### Workflow Triggers

- Runs on every push
- Runs on every PR (opened, updated, reopened)
- Runs on all branches

### Exclusions (Won't Be Scanned)

```
tests/
**/*_test.rs
**/*_test.go
**/*.test.ts
**/*.test.js
**/__pycache__
node_modules/
target/
build/
dist/
.archive/
```

### Tools Included

| Tool | Version | Purpose | Stage |
|------|---------|---------|-------|
| Semgrep | v1.72.0 | Multi-language SAST | pre-commit |
| TruffleHog | v3.93.6 | Secret detection | pre-push |
| Conventional Commit | v4.0.0 | Message validation | commit-msg |
| Language-specific | varies | Format + lint | pre-commit |

## Getting Help

### Resources

- **Semgrep Rules:** https://semgrep.dev/explore
- **TruffleHog Docs:** https://github.com/trufflesecurity/trufflehog
- **Pre-commit Framework:** https://pre-commit.com/

### Common Questions

**Q: Can I skip security checks for my commit?**
A: Not recommended, but possible:
```bash
git commit --no-verify  # Dangerous!
# Better: Fix the issue first, then commit normally
```

**Q: How do I update Semgrep rules?**
A: Create custom rules in `.semgrep-rules/` directory:
```bash
mkdir -p .semgrep-rules
# Create .semgrep-rules/custom.yaml with org-specific rules
```

**Q: Why is my test file flagged by Semgrep?**
A: Tests are usually excluded. Check `.semgrep.yaml` for your file pattern:
```yaml
exclude:
  - "**/*_test.rs"
  - "**/*.test.ts"
```

**Q: Can I use different hooks per environment?**
A: Yes, use pre-commit stages:
```bash
# Only pre-commit hooks (local dev)
pre-commit run --hook-stage pre-commit --all-files

# Only pre-push hooks (before push)
pre-commit run --hook-stage pre-push --all-files
```

## Validation Checklist

- [ ] All 3 files present in repo
- [ ] `pre-commit run --all-files` passes locally
- [ ] Security-guard workflow appears in GitHub Actions
- [ ] No false positives blocking commits
- [ ] Team members notified of new security checks
- [ ] CI/CD environment has all required tools

## Next Steps

1. **Verify locally** (5 min): Run `pre-commit run --all-files`
2. **Test in PR** (10 min): Create test PR and verify workflow runs
3. **Fix any issues** (varies): Address Semgrep findings or update allowlists
4. **Document in README** (5 min): Add security checks to repo documentation
5. **Train team** (15 min): Share this guide with developers

---

**Need Help?**
- Check detailed report: `docs/reports/TIER2_3_DEPLOYMENT_REPORT.md`
- Review deployment logs: `scripts/sast-deployment-*.log`
- Consult Semgrep docs: https://semgrep.dev/docs/

**Last Updated:** 2026-03-31
