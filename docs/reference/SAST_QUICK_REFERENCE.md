# SAST Quick Reference Guide

**What is SAST?** Static Application Security Testing — automated code scanning for vulnerabilities, secrets, and policy violations.

**What was deployed?** Semgrep + TruffleHog + License scanning to 20 repos (Tier 2/3).

---

## Quick Start: Using SAST in Your Repo

### 1. Pre-commit Hook (Run Locally Before Commit)

If your repo has `.pre-commit-config.yaml` with Semgrep entry:

```bash
cd your-repo
pre-commit run --all-files
```

This runs Semgrep and TruffleHog **before** you commit. Catches issues early.

### 2. GitHub Actions (Auto on PR)

When you push to GitHub:
- Workflow `.github/workflows/sast-quick.yml` runs automatically
- Scans on: pull requests + pushes to main
- Takes 3-9 minutes per repo

**View results:**
- Go to repo > **Security** tab > **Code scanning**
- See all Semgrep findings
- Dismiss or fix as needed

### 3. Manual Local Scan

Run Semgrep manually:

```bash
# Install Semgrep (one-time)
brew install semgrep

# Scan current directory
semgrep --config .semgrep-rules/ .

# Or use Semgrep registry (pre-made rules)
semgrep --config p/security-audit .
```

---

## What Gets Scanned

### Semgrep Rules (3 rule sets)

| Rule Set | What It Detects | Severity |
|----------|-----------------|----------|
| **secrets-detection.yml** | AWS keys, API keys, passwords, GitHub tokens, Slack webhooks | CRITICAL/HIGH |
| **architecture-violations.yml** | Forbidden imports, circular dependencies, test code in prod | MEDIUM/HIGH |
| **unsafe-patterns.yml** | SQL injection, unsafe deserialization, file operation bugs | HIGH/CRITICAL |

### TruffleHog (Secret Scanning)

Scans Git history for:
- AWS Access Keys
- GitHub tokens (PATs)
- API keys (Slack, Stripe, etc.)
- Database credentials
- Private keys

**Mode:** Verified only (high-confidence matches)

### License Scanning

Reports on:
- GPL/AGPL code (copyleft licenses)
- Proprietary/commercial dependencies
- License compliance issues

---

## Fixing Issues

### If Semgrep Finds a Problem

**Example error:**
```
secrets-detection.yml:1: Hardcoded API key detected
  api_key = "sk-1234567890abcdef"
```

**Fix options:**

1. **Remove the secret:**
   ```python
   api_key = os.getenv("API_KEY")
   ```

2. **Use environment variables:**
   ```bash
   export API_KEY="sk-..."
   ```

3. **Use a secrets manager:**
   - GitHub Secrets
   - AWS Secrets Manager
   - HashiCorp Vault

4. **Suppress if legitimate** (rare):
   ```python
   api_key = "test-key-only"  # nosemgrep: secrets-detection.hardcoded-api-key
   ```

### If TruffleHog Finds a Secret

**If it's a real secret:**
1. **Immediately rotate it** (regenerate/revoke the key)
2. Commit the removal to Git history (amend + force push, or rewrite history with BFG)
3. Alert team/security

**If it's a false positive:**
1. Add to `.semgrep-rules/false-positives.txt` (if you maintain custom rules)
2. Or ignore the alert in GitHub Security tab

### If License Check Warns

Review the dependency:
1. Check if it's a required dependency
2. Consider alternatives (permissive licenses)
3. If required, document the license compliance in `LICENSES/`

---

## Understanding the Workflow

### What Runs on Each PR/Push

**In GitHub Actions (`.github/workflows/sast-quick.yml`):**

1. **Semgrep Scan** (2-5 min)
   - Scans code against custom + pre-made rules
   - Uploads SARIF to Security tab

2. **TruffleHog Secret Scan** (1-3 min)
   - Scans entire Git history
   - Reports verified secrets only
   - Non-blocking (continues even if secrets found)

3. **Language-Specific Lint** (varies)
   - Rust: `cargo clippy` (if present)
   - Go: `golangci-lint` (if configured)
   - Python: `ruff` + `mypy` (if configured)

4. **License Compliance** (<1 min)
   - Checks dependencies
   - Reports GPL/AGPL/proprietary licenses
   - Non-blocking

---

## Common Issues & Fixes

### Issue: "Module not found" in workflow
**Solution:** Install dependencies before scanning
```yaml
- run: cargo build  # Rust
- run: pip install -r requirements.txt  # Python
- run: go mod download  # Go
```

### Issue: Semgrep timeout
**Solution:** Set longer timeout in workflow
```yaml
semgrep:
  timeout-minutes: 10  # Increase from 5
```

### Issue: False positive — "hardcoded API key"
**Solution:** Use meaningful test prefix
```python
# Good (Semgrep ignores)
api_key = "test_123"

# Bad (Semgrep flags)
api_key = "sk-1234567890abcdef"
```

### Issue: TruffleHog finds old secret in history
**Solution:** Rewrite Git history (BFG Repo-Cleaner)
```bash
bfg --delete-files <filename>
git reflog expire --expire-unreachable=now --all
git gc --prune=now
```

---

## Semgrep Rules Reference

### Secrets Patterns Detected

```yaml
# AWS Access Key
AKIA[0-9A-Z]{16}

# GitHub Token
ghp_[A-Za-z0-9_]{36,255}

# Slack Webhook
https://hooks.slack.com/services/[A-Z0-9/]+

# Generic API Key
api_key = "..."
API_KEY = "..."
apiKey = "..."
```

### Architecture Rules

```yaml
# Forbidden imports (example)
import test_utilities  # From prod code

# Circular dependencies
A imports B, B imports A
```

### Unsafe Patterns

```python
# SQL Injection risk
query = f"SELECT * FROM users WHERE id = {user_input}"

# Unsafe deserialization
pickle.loads(untrusted_data)

# Command injection
os.system(f"rm {filename}")
```

---

## FAQ

**Q: Can I disable SAST checks?**
A: Not recommended. SAST catches real bugs. If you have false positives, suppress with justification (e.g., `# nosemgrep: rule-id`).

**Q: What if I need to commit a test secret (e.g., mock API key)?**
A: Use test prefixes (test_*, mock_*) that don't match real secret patterns. Example: `api_key = "test_1234"` instead of `api_key = "sk_actual_key"`.

**Q: Can I customize the rules?**
A: Yes. Add custom YAML rules to `.semgrep-rules/`. See Semgrep docs for syntax.

**Q: How do I skip the secret scan?**
A: Not possible without workflow changes. If you have a legitimate need, discuss with team/security.

**Q: Does SAST catch all vulnerabilities?**
A: No. SAST finds ~30-40% of common issues. Combine with DAST (dynamic testing), code review, and security audits for comprehensive coverage.

**Q: What if the workflow fails on my PR?**
A: Check Security tab > Code scanning for specific issues. Fix or suppress with justification. All checks must pass before merge.

---

## Team Guidelines

### Before Committing:
1. Run `pre-commit run --all-files` locally
2. Fix any Semgrep violations
3. Never commit secrets

### After Pushing PR:
1. Wait for SAST checks (3-9 min)
2. Review findings in Security tab
3. Fix or dismiss with justification
4. Re-push; checks re-run

### On Main Branch:
- All SAST checks must pass
- No blocked findings allowed
- Regular review of Security tab alerts

---

## Further Reading

- **Semgrep Docs:** https://semgrep.dev/docs/
- **TruffleHog:** https://github.com/trufflesecurity/trufflehog
- **OWASP Top 10:** https://owasp.org/www-project-top-ten/
- **CWE Top 25:** https://cwe.mitre.org/top25/

---

## Support

**For questions:**
1. Check `.github/workflows/sast-quick.yml` syntax
2. Review `.semgrep-rules/` for specific rule details
3. Run `semgrep --help` for CLI options
4. Check Semgrep registry (https://semgrep.dev/r) for pre-made rules

**For emergencies:**
- TruffleHog found a real secret? **Rotate it immediately**
- Workflow broken? Check syntax in `.github/workflows/`
- Need custom rules? Create `.semgrep-rules/custom.yml`

---

**Last Updated:** 2026-03-30
**SAST Deployment:** 20/21 repos (Tier 2/3)
**Next Review:** 2026-04-30
