# SAST Security Setup Guide

This document describes the Static Application Security Testing (SAST) infrastructure deployed to the Phenotype ecosystem.

## Overview

The SAST foundation includes:
- **Semgrep**: Custom rule engine for code patterns, secrets, and architecture violations
- **CodeQL**: GitHub's advanced code analysis (nightly)
- **TruffleHog**: Secret scanning (verified only)
- **Language-specific tools**: Clippy (Rust), Bandit (Python), etc.
- **Pre-commit hooks**: Local developer guards
- **Quality gate script**: CI/CD integration point

## Workflows

### sast-quick.yml (PR checks, ~3-5 min)
Runs on every PR and push to main:
- Semgrep with security-audit, owasp-top-ten, cwe-top-25 profiles
- TruffleHog secret scanning (verified findings only)
- Language-specific linting (Clippy for Rust)
- License compliance check

**Configuration**: `.semgrep.yml`, `.semgrep-rules/`

**Suppress a finding**:
```yaml
# .semgrep.yml
rules:
  - id: custom-rule
    # Add justification
    patterns: [...]
```

### sast-full.yml (Nightly, ~15-40 min)
Runs on schedule (2 AM UTC):
- CodeQL analysis (multi-language)
- Trivy repository scan
- Full Semgrep analysis with misconfigurations + CI/CD checks
- Complete secret scanning

## Local Development

### Pre-commit Setup

Install pre-commit hooks (one-time):
```bash
pre-commit install
pre-commit install --hook-type pre-push
```

Hooks run before:
- **pre-commit**: Semgrep, trailing whitespace, YAML validation, detect-private-key
- **pre-push**: TruffleHog full git history

### Run Quality Gate Locally

Before pushing:
```bash
./scripts/quality-gate.sh verify
```

This runs:
1. Semgrep scan
2. Secret scanning
3. Clippy + format checks (Rust)
4. Full test suite

### Skip Hooks (if absolutely necessary)

```bash
git commit --no-verify  # Skip pre-commit hooks
git push --no-verify    # Skip pre-push hooks
```

**Note**: CI will still run. Only use as temporary workaround.

## Handling Findings

### Semgrep Findings

**View on GitHub**: Settings → Security → Code scanning → Filter by tool:Semgrep

**Dismiss a false positive**:
1. Go to the alert on GitHub
2. Click "Dismiss" and select reason
3. Provide justification in comment
4. Alert is dismissed for that branch

**Suppress in code** (last resort):
```rust
// nosemgrep
let unsafe_value = serde_json::from_str(input)?;
```

### CodeQL Findings

**View results**: Security tab → Code scanning → CodeQL

**Fix workflow**: Fix → Commit → Alert auto-closes

### TruffleHog Secrets

If a secret is committed:
```bash
# 1. Revoke the secret immediately (on the service)
# 2. Remove from git history
git filter-branch --tree-filter 'grep -r "SECRET"' -- --all
# 3. Force push (dangerous, coordinate with team)
```

## Architecture Violations

Custom Semgrep rules detect:
- Hardcoded secrets (AWS keys, API tokens, GitHub tokens)
- Unsafe deserialization
- Unwrap without context
- SQL injection via format!()
- Circular module dependencies
- Layer violations (adapters accessing domain directly)

**Example violation**:
```rust
// ❌ BAD: Direct deserialization
let data = serde_json::from_str(&input)?;

// ✓ GOOD: With validation
let data: MyType = serde_json::from_str(&input)
    .map_err(|e| Error::InvalidJson(e.to_string()))?;
```

## CI/CD Integration

### Status Checks

PRs require:
- ✅ Semgrep (quick)
- ✅ TruffleHog
- ✅ Language-specific linting
- ✓ CodeQL (nightly, not PR-blocking)

### Auto-Issue Creation

CodeQL findings trigger auto-issues labeled `security` and `critical` on high-severity results.

### Reporting

- **Dashboard**: GitHub Security tab
- **API**: `gh api repos/:owner/:repo/code-scanning/alerts`
- **CSV export**: Use GitHub UI or API

## Tool Installation (Local)

### macOS
```bash
# Semgrep
brew install semgrep

# TruffleHog
brew tap trufflesecurity/trufflehog
brew install trufflehog

# Rust tools (included in rust-toolchain.toml)
rustup component add clippy

# Python tools
pip install bandit ruff
```

### Linux (Ubuntu)
```bash
# Semgrep
sudo apt install -y python3-pip python3-dev
pip install semgrep

# TruffleHog
wget https://github.com/trufflesecurity/trufflehog/releases/download/v3.93.6/trufflehog-3.93.6-linux-x64.tar.gz
tar xzf trufflehog-*.tar.gz

# Rust/Python tools
apt install -y rustc cargo python3-pip
cargo install clippy
```

## False Positives & Tuning

### Report False Positive

1. Verify it's actually a false positive (test it)
2. Open issue with label `sast-false-positive`
3. Include:
   - Rule ID (e.g., `hardcoded-api-key-env`)
   - Code snippet
   - Why it's a false positive
   - Suggested fix

### Adjust Rule Severity

Edit `.semgrep-rules/*.yml`:
```yaml
- id: my-rule
  severity: MEDIUM  # LOW, MEDIUM, HIGH, CRITICAL
  ...
```

Lower severity = not blocking PRs, still reported.

## Performance

### Timeout Issues

If workflows timeout (>30 min):
1. Check for large files (>50 MB binary)
2. Exclude in `.semgrep.yml`: `paths.exclude`
3. Split workflow into parallel jobs

### Caching

GitHub Actions caches:
- Rust dependencies (Swatinem/rust-cache)
- Npm/Python package managers (actions/setup-*)

Cache is automatically invalidated on `Cargo.lock`/`package-lock.json` changes.

## Escalation

**Security incident**: Reach out to security team
**Tool outage**: Check GitHub status page, open issue in respective tool repo
**Questions**: Open discussion in project or contact SAST maintainer

## References

- Semgrep docs: https://semgrep.dev/docs/
- CodeQL docs: https://codeql.github.com/
- TruffleHog docs: https://github.com/trufflesecurity/trufflehog
- GitHub Code Scanning: https://docs.github.com/en/code-security/code-scanning

---

**Last Updated**: 2026-03-30
**Tier 1 Repos**: AgilePlus, heliosCLI, phenotype-infrakit
