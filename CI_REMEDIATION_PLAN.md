# CI Remediation Plan — Quick Fix Guide

**Status:** Ready to Execute
**Estimated Time:** 20 minutes (5 min fix + 10 min CI + 5 min validation)

---

## Quick Summary

All PR merges are blocked by a single root cause: **outdated `gix` dependency (v0.62.0)** with 9 unpatched security vulnerabilities.

**Solution:** Update `gix` from 0.62 → 0.70 (or latest).

---

## Step 1: Update Gix Dependency (5 minutes)

### 1a. Verify Current Version
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
grep "^gix = " Cargo.toml  # Should show: gix = "0.62"
```

### 1b. Update Cargo.toml
**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml`

**Find (around line 50):**
```toml
[workspace.dependencies]
...
gix = "0.62"
```

**Replace with:**
```toml
[workspace.dependencies]
...
gix = "0.70"
```

**Alternative:** Use the absolute latest available (check crates.io for v0.71+)

### 1c. Regenerate Cargo.lock
```bash
rm Cargo.lock
cargo update
```

### 1d. Verify Fix
```bash
cargo audit 2>&1 | grep "error:"
# Expected: no output (0 vulnerabilities found)

cargo deny check advisories 2>&1 | grep -E "FAILED|PASSED"
# Expected: "advisories PASSED"
```

---

## Step 2: Fix CodeQL Configuration (3 minutes)

### 2a. Edit Security Workflow
**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/.github/workflows/security.yml`

**Find (lines 56-71):**
```yaml
codeql:
  name: CodeQL
  runs-on: ubuntu-latest
  permissions:
    contents: read
    security-events: write
  strategy:
    matrix:
      language: [cpp, python]  # ← PROBLEM
  steps:
    - uses: actions/checkout@v6
    - uses: github/codeql-action/init@v3
      with:
        languages: ${{ matrix.language }}
    - uses: github/codeql-action/analyze@v3
```

**Replace with (remove cpp, keep only rust via separate codeql.yml):**

**Option A: Disable CodeQL in security.yml (simplest)**
Comment out or delete the entire `codeql` job (lines 56-71). Use `.github/workflows/codeql.yml` instead (which correctly analyzes only Rust).

**Option B: Fix the matrix to omit cpp/python**
```yaml
codeql:
  name: CodeQL
  runs-on: ubuntu-latest
  # (rest unchanged, but remove matrix entirely if not needed)
  steps:
    - uses: actions/checkout@v6
    - uses: github/codeql-action/init@v3
      with:
        languages: rust  # ← Explicit, no matrix needed
    - uses: github/codeql-action/autobuild@v3
    - uses: github/codeql-action/analyze@v3
```

**Recommendation:** Use Option A — the repo already has `.github/workflows/codeql.yml` which correctly configures Rust analysis.

---

## Step 3: Commit & Push (2 minutes)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Stage changes
git add Cargo.toml Cargo.lock
git add .github/workflows/security.yml  # If you edit CodeQL

# Commit
git commit -m "fix(ci): resolve 9 gix security vulnerabilities and CodeQL false positive

- Upgrade gix from 0.62 → 0.70 (9 medium-high severity fixes)
  - Fixes path traversal (8.8/10), Windows device access (5.4/10)
  - Fixes world-writable file perms, UTF-8 validation, SHA-1 detection
- Remove redundant CodeQL C++/Python matrix in security.yml
  - Keep Rust analysis via dedicated codeql.yml
  - Eliminates spurious C++ build failure

Blocks all PRs #331, #330, #329, #327, #326, #315, #314
Fixes cargo audit, cargo deny, OSV-scanner, CodeQL checks"

# Push
git push origin main
```

---

## Step 4: Verify in CI (10 minutes)

After push, GitHub Actions will run automatically:

### 4a. Monitor Workflow
```bash
# Watch for Security workflow completion
gh run list --workflow security.yml --limit 1 -s in_progress

# Or open directly:
# https://github.com/KooshaPari/phenotype-infrakit/actions/workflows/security.yml
```

### 4b. Expected Results
- ✅ Cargo Audit: PASS
- ✅ Cargo Deny: PASS
- ✅ Gitleaks: PASS
- ✅ CodeQL (rust): PASS
- ✅ Python Security: PASS
- ✅ OSV-Scanner: PASS

### 4c. Verify All PRs Unblock
```bash
gh pr list --state open --limit 10

# All PRs should now show:
# - No red ✗ marks on security checks
# - Green ✅ marks on cargo-audit, cargo-deny, osv-scanner
```

---

## Step 5: Merge Blocked PRs (5 minutes)

Once main is green:

```bash
# For each PR that was blocked:
gh pr merge 331 --squash
gh pr merge 330 --squash
gh pr merge 329 --squash
# ... etc
```

---

## Troubleshooting

### Issue: `cargo update` fails
**Cause:** Network issue or incompatible version constraint
**Solution:**
```bash
cargo clean
cargo update -p gix
```

### Issue: Tests fail after gix update
**Cause:** API breaking changes in gix v0.70
**Solution:** Check gix release notes for migration guide, or try v0.69 instead

### Issue: CodeQL still fails after fix
**Cause:** Didn't disable/remove the problematic matrix job
**Solution:** Verify lines 56-71 of security.yml are removed or simplified

### Issue: PR still shows failing checks
**Cause:** Old CI run cached; GitHub needs to re-run
**Solution:**
```bash
# Trigger a new run on the PR branch
git commit --allow-empty -m "chore: retrigger CI"
git push
```

---

## Rollback Plan (if needed)

If the gix update breaks something critical:

```bash
# Revert the changes
git revert <commit-sha>

# Or manually revert:
git checkout HEAD~1 -- Cargo.toml Cargo.lock
git commit -m "revert: gix update pending investigation"
git push
```

---

## Files Modified

| File | Change | Lines |
|------|--------|-------|
| `Cargo.toml` | Update gix version | ~50 |
| `Cargo.lock` | Regenerated (gix deps) | ~200 |
| `.github/workflows/security.yml` | Remove CodeQL C++/Python matrix | 56-71 |

---

## Success Criteria

After executing all steps:

- [ ] `cargo audit` shows 0 vulnerabilities
- [ ] `cargo deny check` passes all checks
- [ ] GitHub Actions security workflow passes
- [ ] All 7 blocked PRs show green checks
- [ ] Can merge PRs without security gate errors

---

## Dependencies

- ✅ `cargo` (already installed)
- ✅ `cargo-audit` (built-in)
- ✅ `cargo-deny` (used in CI)
- ✅ GitHub CLI `gh` (for monitoring)

No additional tools needed.

---

**Ready to execute? Start with Step 1.**
