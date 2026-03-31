# SAST Tier 1 Quick Start

**Status**: ✅ Deployed to AgilePlus, heliosCLI
**Files**: `docs/reports/PHASE1_SAST_DEPLOYMENT_COMPLETION.md` (full report)

## What Was Deployed?

### 1. Semgrep + 16 Custom Security Rules
- Secrets detection (AWS keys, API tokens, passwords)
- Unsafe patterns (SQL injection, unwrap without context)
- Architecture violations (circular deps, layer violations)

### 2. GitHub Actions Workflows
- **sast-quick.yml**: Every PR (~3-5 min, Semgrep + TruffleHog + Clippy)
- **sast-full.yml**: Nightly (~15-40 min, CodeQL + Trivy + full Semgrep)
- **rust-security.yml**: On Rust file changes (Cargo Audit + Clippy)

### 3. Pre-commit Hooks
- Semgrep (pre-commit stage)
- TruffleHog (pre-push stage)
- Standard checks (trailing whitespace, private keys)

### 4. Quality Gate Script
- Run: `./scripts/quality-gate.sh verify`
- Checks: Semgrep, secrets, clippy, format, tests

## Get Started (5 mins)

### Step 1: Install tools
```bash
brew install semgrep trufflehog  # macOS
# or use apt-get on Linux
```

### Step 2: Enable pre-commit hooks
```bash
cd AgilePlus  # or heliosCLI
pre-commit install && pre-commit install --hook-type pre-push
```

### Step 3: Test quality gate locally
```bash
./scripts/quality-gate.sh verify
```

Expected output:
```
=== Phenotype SAST Quality Gate ===
[1/4] Semgrep scan...
✓ Semgrep passed
[2/4] TruffleHog secret scan...
✓ Secret scanning passed
[3/4] Cargo checks...
✓ Clippy passed
✓ Format passed
[4/4] Tests...
✓ Tests passed
✓ All quality gates passed!
```

### Step 4: Enable GitHub CodeQL (owner/maintainer only)
1. Go to repo Settings → Security & analysis
2. Scroll to "Code scanning"
3. Click "Enable" next to CodeQL
4. Confirm + workflows start automatically

## Daily Development

### Making a commit
```bash
# Your changes...
git add .
git commit -m "feat: add new feature"

# Pre-commit hooks run automatically
# If fails: fix issues, stage again, commit again
```

### Before pushing
```bash
git push
# Pre-push hooks run automatically (TruffleHog)
# Prevents accidental secret commits
```

### On PR
- Semgrep + TruffleHog + Clippy run automatically
- Results show in PR checks
- If fails: fix code, push again, checks re-run

## Handling Issues

### Semgrep finding too strict?
1. Run locally: `semgrep --config=.semgrep.yml --error .`
2. If false positive:
   - Open issue labeled `sast-false-positive`
   - Include rule ID + code snippet
3. To suppress (rare):
   ```rust
   // nosemgrep: rule-id
   let unsafe_code = ...;
   ```

### Secret accidentally committed?
1. **Immediately**: Revoke the secret (API key, password, token)
2. Remove from git history:
   ```bash
   git filter-branch --tree-filter 'grep -r "SECRET"' -- --all
   git push --force-with-lease
   ```
3. Report to security team

### Skip pre-push hooks (emergency only)
```bash
git push --no-verify
```
Note: GitHub Actions will still run. Only use as temporary workaround.

## Documentation

Full setup guide: `SAST_SETUP.md` in each repo
- Installation for macOS/Linux
- Workflow descriptions
- False positive handling
- Tool configuration

## Current Repos

### ✅ AgilePlus
```
✓ All components deployed
✓ Pre-commit hooks ready
✓ Quality gate script active
```

### ✅ heliosCLI
```
✓ All components deployed
✓ Pre-commit hooks ready
⏳ Quality gate script (needs chmod)
```

### ⏳ phenotype-infrakit
```
Verification needed (minimal repo contents)
Contact maintainer for deployment status
```

## Slack/Chat Notifications

Workflows can be configured to post to Slack (Phase 2):
```yaml
- name: Notify Slack
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'SAST check ${{ job.status }}: ${{ github.repository }}'
```

## FAQ

**Q: My tool not installed, can I skip pre-commit hooks?**
A: Yes, hooks skip gracefully if tool missing. Install when convenient.

**Q: Does this slow down development?**
A: Semgrep is ~30 sec. TruffleHog on push is ~1 min. Tests dominate runtime.

**Q: What if CodeQL finds issues on main?**
A: Create issue, add `security` label, plan remediation. Not a blocker for PRs.

**Q: Can I contribute to custom rules?**
A: Yes! Open PR to `.semgrep-rules/` with new rule + tests.

## Next: Tier 2 Rollout

Remaining 27 repos will get SAST deployed in Week 2:
- pheno-cli, bifrost-extensions, agent-wave, phenotype-design, etc.
- Same setup, automated via deployment script

---

For detailed info: `docs/reports/PHASE1_SAST_DEPLOYMENT_COMPLETION.md`
For setup help: `SAST_SETUP.md` (in each repo)
