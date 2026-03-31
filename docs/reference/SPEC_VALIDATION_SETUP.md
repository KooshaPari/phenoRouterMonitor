# Spec Validation Gate Setup & Configuration

This guide explains how to install, configure, and use the spec validation pre-commit hook for the phenotype-infrakit monorepo.

## Quick Start

### Installation

The validation gates are automatically available via the `.githooks/` directory. To enable them:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Configure git to use the .githooks directory
git config core.hooksPath .githooks

# Verify the hook is installed
ls -la .git/hooks/pre-commit
```

### First Commit Test

Try making a test commit to verify the gates are working:

```bash
# Create a trivial change
echo "# Test" >> README.md

# Attempt to commit (hooks will run automatically)
git add README.md
git commit -m "test: verify hooks are working"
```

Expected output:
```
════════════════════════════════════════════════════════════════
  PRE-COMMIT VALIDATION PIPELINE
  Mode: ADVISORY (warnings only)
════════════════════════════════════════════════════════════════

Gate 1️⃣  Secret Scanning (trufflehog)...
✅ No secrets detected

Gate 2️⃣  Spec Validation (FR↔Test Traceability)...
✅ Spec validation passed

════════════════════════════════════════════════════════════════
✅ PRE-COMMIT VALIDATION PASSED
════════════════════════════════════════════════════════════════
```

## Validation Modes

### Advisory Mode (Default)

In **advisory mode**, validation runs but only warns about issues without blocking commits. This is useful during development.

```bash
# Enable advisory mode (warnings only, no blocking)
git config hooks.specValidationStrict false
```

Output:
```
Gate 2️⃣  Spec Validation (FR↔Test Traceability)...
⚠️  SPEC VALIDATION WARNINGS (advisory mode)
   - Found 79 FRs without test coverage
```

**Use case**: Daily development work where FR test coverage is incomplete.

### Strict Mode

In **strict mode**, validation failures will block the commit. This mode is enforced on the main branch CI/CD pipeline.

```bash
# Enable strict mode (all gates must pass)
git config hooks.specValidationStrict true
```

Or run a single commit in strict mode:

```bash
# Override for one commit
STRICT=1 git commit -m "message"
```

**Use case**: Before merging to main or during final PR validation.

## Validation Checks

### Gate 1: Secret Scanning (trufflehog)

**What it checks**: Scans git history for exposed secrets, API keys, credentials.

**Prerequisites**:
```bash
# Install trufflehog (preferred over gitleaks)
brew install trufflehog

# Or if you prefer gitleaks
brew install gitleaks
```

**Behavior**:
- ✅ Passes if no secrets detected
- ❌ Fails if secrets found (blocking in both modes)
- ⚠️ Warns if tool not installed (non-blocking)

**Bypass** (if legitimate false positive):
```bash
git commit --no-verify -m "message"
```

### Gate 2: Spec Validation (FR↔Test Traceability)

**What it checks**:
1. **FR Uniqueness**: No duplicate FR IDs in FUNCTIONAL_REQUIREMENTS.md
2. **Test Coverage**: Every FR has ≥1 test
3. **Test Traceability**: Every test references ≥1 FR

**File scanning**:
- Scans all `.rs`, `.py`, `.ts`, `.js` files in `crates/`
- Looks for test/spec files (names containing "test" or "spec")
- Extracts FR annotations: `// Traces to: FR-XXX-YYY`

**Behavior**:
- ✅ Passes if all FRs have test coverage
- ⚠️ Warns in advisory mode if coverage gaps exist
- ❌ Blocks in strict mode if coverage gaps exist

**Example output (advisory mode)**:
```
📊 Validation Report
==================
Total FRs defined: 84
FRs with test coverage: 8
Coverage: 9%

⚠️  Warnings:
  • Found 89 test files without FR references
```

## Adding FR Annotations to Tests

When writing tests, always annotate with the FR being validated:

**Rust**:
```rust
#[test]
fn test_state_transition() {
    // Traces to: FR-DOMAIN-003, FR-DOMAIN-004
    // Verify that state transitions enforce forward-only movement
    // ... test body
}
```

**Python**:
```python
def test_feature_creation():
    # Traces to: FR-DOMAIN-001
    # Verify that Feature.create() sets initial state to CREATED
    # ... test body
```

See `docs/reference/FR_ANNOTATION_GUIDE.md` for complete annotation examples and best practices.

## Manual Validation

To check FR coverage without committing:

```bash
# Run spec validator directly
bash .githooks/spec-validator

# Check coverage percentage
bash .githooks/spec-validator 2>&1 | grep "Coverage:"
```

## Troubleshooting

### Hook not running on commit

**Problem**: Pre-commit hook isn't executing

**Solution**:
```bash
# Verify git hooks path is configured
git config core.hooksPath
# Should output: .githooks

# If not set, configure it
git config core.hooksPath .githooks

# Verify hook is executable
ls -la .git/hooks/pre-commit
# Should show: -rwxr-xr-x (executable)
```

### "spec-validator: command not found"

**Problem**: Hook can't find spec-validator script

**Solution**:
```bash
# Verify spec-validator exists
ls -la .githooks/spec-validator

# Make sure it's executable
chmod +x .githooks/spec-validator

# Test manually
bash .githooks/spec-validator
```

### False positives from secret scanning

**Problem**: trufflehog reports false positives (e.g., test data, placeholder values)

**Solution**:
1. Use `.gitignore` to exclude test files with fake data
2. Bypass with `git commit --no-verify`
3. Mark as legitimate in `.trufflehohconfig` (if using config file)

### FR coverage gaps blocking commits in strict mode

**Problem**: You need to commit but FRs don't have tests yet

**Solution**:
```bash
# Option 1: Bypass strict mode for this commit
STRICT=0 git commit -m "WIP: adding new FRs"

# Option 2: Use --no-verify
git commit --no-verify -m "WIP: adding new FRs"

# Option 3: Switch back to advisory mode temporarily
git config hooks.specValidationStrict false
git commit -m "message"
git config hooks.specValidationStrict true
```

## CI/CD Integration

On the main branch, the CI/CD pipeline enforces **strict mode**:

```yaml
# Example GitHub Actions workflow
- name: Run pre-commit validation (strict)
  env:
    STRICT: "1"
  run: bash .githooks/pre-commit
```

All FRs must have test coverage before merging to main.

## Configuration

### Per-repository settings

```bash
# Enable strict mode for this repository
git config hooks.specValidationStrict true

# Disable strict mode
git config hooks.specValidationStrict false

# Check current setting
git config --get hooks.specValidationStrict
```

### Global settings (all repos)

```bash
# Set globally
git config --global hooks.specValidationStrict true

# Repository setting overrides global
```

### Environment variable override

```bash
# Override for single command
STRICT=1 git commit -m "message"
STRICT=0 git commit -m "message"
```

## Maintenance

### Updating the validation gates

The validation gate consists of:

| File | Purpose |
|------|---------|
| `.githooks/pre-commit` | Main validation pipeline |
| `.githooks/spec-validator` | FR↔Test coverage checking |
| `docs/reference/FR_ANNOTATION_GUIDE.md` | Annotation guidelines |
| `FUNCTIONAL_REQUIREMENTS.md` | Source of truth for FRs |

To update validation rules:
1. Edit `.githooks/spec-validator`
2. Test locally: `bash .githooks/spec-validator`
3. Commit changes to `.githooks/`

### Monitoring coverage trends

Track FR test coverage over time:

```bash
# Extract coverage percentage
bash .githooks/spec-validator 2>&1 | grep "Coverage:" | awk '{print $NF}'

# Log to file for historical tracking
echo "$(date): $(bash .githooks/spec-validator 2>&1 | grep 'Coverage:' | awk '{print $NF}')" >> coverage-history.txt
```

## Related Documentation

- `docs/reference/FR_ANNOTATION_GUIDE.md` — How to annotate tests with FR references
- `FUNCTIONAL_REQUIREMENTS.md` — Master list of all FRs
- `.githooks/pre-commit` — Validation pipeline script
- `.githooks/spec-validator` — FR coverage checking script

## Questions or Issues?

1. Check the Troubleshooting section above
2. Run `bash .githooks/spec-validator` manually to debug
3. Refer to `FR_ANNOTATION_GUIDE.md` for annotation syntax
4. Review `FUNCTIONAL_REQUIREMENTS.md` for available FRs to reference

---

**Last Updated**: 2026-03-30
**Related Components**: `FR_ANNOTATION_GUIDE.md`, `FUNCTIONAL_REQUIREMENTS.md`
