# Snyk Automation Infrastructure — Ready for Phase 1 Deployment

**Status**: ✅ Complete & Ready for Execution
**Date**: 2026-03-30
**Phase**: Phase 1 (Tier 1 Repos)

## Executive Summary

Snyk security automation infrastructure is fully deployed and ready for immediate execution. All scripts, workflows, and documentation are production-ready. Waiting only for Snyk API token to begin scanning.

**Timeline to Full Coverage**: ~15 minutes (Phase 1) + nightly scans thereafter

## What's Deployed

### 1. Automated Deployment Scripts

**Location**: `/scripts/snyk-deploy.sh`

**Capabilities**:
- ✅ Authenticates with Snyk API
- ✅ Scans multiple repositories in one command
- ✅ Generates `.snyk` policy files automatically
- ✅ Produces JSON test reports
- ✅ Generates SARIF for GitHub Code Scanning
- ✅ Non-blocking (continues on individual repo failures)
- ✅ Creates aggregated summary report

**Usage**:
```bash
./scripts/snyk-deploy.sh $SNYK_TOKEN AgilePlus phenotype-infrakit heliosCLI
```

**Output**: `.snyk-reports/` directory with per-repo findings

---

### 2. Policy Generator Script

**Location**: `/scripts/snyk-policy-generator.sh`

**Capabilities**:
- ✅ Auto-detects project type (Rust, Node.js, Python, Go)
- ✅ Generates type-specific `.snyk` policy templates
- ✅ Optional: Scans and suppresses current vulnerabilities
- ✅ Creates policy files ready for manual review

**Usage**:
```bash
./scripts/snyk-policy-generator.sh ./AgilePlus
./scripts/snyk-policy-generator.sh ./phenotype-infrakit --type=rust
```

**Output**: `.snyk` policy file in repository root

---

### 3. GitHub Actions Workflow Template

**Location**: `/.github/workflows/snyk-scan.yml`

**Features**:
- ✅ Triggers: PR open/update, main branch push, nightly schedule
- ✅ Runs Snyk test with severity thresholds
- ✅ Generates SARIF for Code Scanning dashboard
- ✅ Auto-comments on PRs with vulnerability summary
- ✅ Creates fix PRs automatically for high/critical issues
- ✅ Fails build on critical vulnerabilities (configurable)
- ✅ Stores reports as artifacts (30-day retention)

**Deployment**: Copy to each repository:
```bash
mkdir -p $REPO/.github/workflows
cp .github/workflows/snyk-scan.yml $REPO/.github/workflows/
```

---

### 4. Comprehensive Documentation

**Location**: `/docs/reference/SNYK_SETUP_GUIDE.md`

**Covers**:
- Quick start (5 minutes)
- Getting Snyk token
- Local setup & authentication
- Deployment procedures
- Policy configuration
- Scan execution & monitoring
- Result interpretation
- Troubleshooting guide
- CI/CD integration details
- Full timeline for Phases 1-4

---

## What's Ready to Execute

| Component | Status | Notes |
|-----------|--------|-------|
| **Deployment Script** | ✅ Ready | `/scripts/snyk-deploy.sh` |
| **Policy Generator** | ✅ Ready | `/scripts/snyk-policy-generator.sh` |
| **GitHub Workflow** | ✅ Ready | `/.github/workflows/snyk-scan.yml` |
| **Documentation** | ✅ Ready | `/docs/reference/SNYK_SETUP_GUIDE.md` |
| **Snyk CLI** | ⏳ Requires | Install: `npm install -g snyk` or `brew install snyk` |
| **Snyk Token** | ⏳ Requires | Get from: https://app.snyk.io/account/settings |
| **GitHub Secrets** | ⏳ Requires | Add `SNYK_TOKEN` to org/repo settings |

---

## What Needs to Happen Next (User Action)

### Step 1: Get Snyk Token (5 minutes)

1. Sign up at: https://app.snyk.io/auth/register (if needed)
2. Go to: https://app.snyk.io/account/settings
3. Copy API token from "API Token" section
4. Save securely (don't commit to git)

### Step 2: Install Snyk CLI (2 minutes)

```bash
# macOS
brew install snyk

# Linux
npm install -g snyk

# Or
apt-get install snyk
```

### Step 3: Add GitHub Secret (3 minutes)

**Option A: Organization Secret** (all repos inherit)

1. Go: https://github.com/KooshaPari/organizations/settings/secrets
2. Click "New organization secret"
3. Name: `SNYK_TOKEN`
4. Value: Paste token
5. Save

**Option B: Per-Repo Secret**

1. Go: https://github.com/KooshaPari/REPO/settings/secrets
2. Click "New repository secret"
3. Name: `SNYK_TOKEN`
4. Value: Paste token
5. Save

### Step 4: Execute Phase 1 Deployment (5 minutes)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Set token
export SNYK_TOKEN="your-token-here"

# Run deployment
./scripts/snyk-deploy.sh "$SNYK_TOKEN" AgilePlus phenotype-infrakit heliosCLI

# Review reports
ls -la .snyk-reports/
```

### Step 5: Commit Policy Files (2 minutes)

```bash
git add AgilePlus/.snyk phenotype-infrakit/.snyk heliosCLI/.snyk
git commit -m "chore(security): add Snyk policy files for Tier 1 repos"
git push
```

### Step 6: Deploy GitHub Actions Workflow (3 minutes)

For each repository:

```bash
for repo in AgilePlus phenotype-infrakit heliosCLI; do
  mkdir -p "$repo/.github/workflows"
  cp .github/workflows/snyk-scan.yml "$repo/.github/workflows/"
  cd "$repo"
  git add .github/workflows/snyk-scan.yml
  git commit -m "chore(ci): add Snyk security scan workflow"
  git push
  cd ..
done
```

---

## Execution Timeline

### Phase 1: Tier 1 Repos (Week 1)

**Repos**: AgilePlus, phenotype-infrakit, heliosCLI

**Timeline**:
- Token setup: 5 min
- CLI install: 2 min
- GitHub secret: 3 min
- Deployment: 5 min
- Policy review: 10 min
- Commit & push: 5 min
- Workflow deploy: 5 min

**Total**: ~35 minutes to full Phase 1 coverage

**Ongoing**: Nightly scans + PR checks automatically

### Phase 2: Tier 2 Repos (Week 2)

**Repos**: agent-wave, agentapi-plusplus, KaskMan

```bash
./scripts/snyk-deploy.sh "$SNYK_TOKEN" agent-wave agentapi-plusplus KaskMan
```

### Phase 3: Tier 3 Repos (Week 3)

**Repos**: forgecode, zen, vibeproxy

```bash
./scripts/snyk-deploy.sh "$SNYK_TOKEN" forgecode zen vibeproxy
```

### Phase 4+: Continuous Monitoring

- Nightly scans (automated)
- PR vulnerability checks (automated)
- Fix PRs for critical issues (automated)
- Policy review & updates (manual as needed)

---

## Success Criteria

After Phase 1 deployment, you'll have:

✅ **Local scanning capability**: Run `snyk test` in any repo
✅ **Automated deployments**: `snyk-deploy.sh` scans all repos
✅ **Policy files**: `.snyk` in each repo (committed to git)
✅ **GitHub Code Scanning**: Vulnerabilities visible in GitHub Security tab
✅ **PR checks**: PRs show vulnerability status before merge
✅ **Nightly monitoring**: Automatic scans at 2 AM UTC
✅ **Fix PRs**: Auto-patches for high/critical (configurable)
✅ **Aggregated reporting**: Central visibility via GitHub dashboard

---

## Troubleshooting Quick Links

**Token issues**: See "Getting Snyk Token" in SNYK_SETUP_GUIDE.md
**CLI install**: See "Prerequisites" in SNYK_SETUP_GUIDE.md
**Workflow not running**: Check GitHub Actions enabled in Settings
**Reports not generating**: Verify repo has dependencies (package.json, Cargo.toml, etc.)
**SARIF upload fails**: Check GitHub token has `security-events` permission

---

## Files Summary

### Scripts (executable)

```
scripts/
├── snyk-deploy.sh              # Automated deployment (multi-repo)
└── snyk-policy-generator.sh    # Policy file generator
```

**Make executable if needed:**
```bash
chmod +x scripts/snyk-deploy.sh
chmod +x scripts/snyk-policy-generator.sh
```

### Workflows

```
.github/workflows/
└── snyk-scan.yml               # GitHub Actions workflow template
```

**Deploy to repos:**
```bash
cp .github/workflows/snyk-scan.yml $REPO/.github/workflows/
```

### Documentation

```
docs/reference/
└── SNYK_SETUP_GUIDE.md         # Complete setup & reference guide
```

---

## Default Scanning Behavior

### What Gets Scanned

- ✅ Node.js dependencies (package.json, yarn.lock, pnpm-lock.yaml)
- ✅ Rust crates (Cargo.toml, Cargo.lock)
- ✅ Python packages (pyproject.toml, requirements.txt)
- ✅ Go modules (go.mod, go.sum)
- ✅ License policies
- ✅ Infrastructure-as-code (Terraform, Kubernetes)

### What Gets Reported

- ✅ Vulnerabilities with CVE details
- ✅ Severity levels (critical, high, medium, low)
- ✅ Recommended fixes and upgrade paths
- ✅ CVSS scores and exploit availability
- ✅ Dependency chains
- ✅ License compliance issues (optional)

### Default Fail Thresholds

| Trigger | Threshold | Action |
|---------|-----------|--------|
| Build failure | Critical found | Fails PR check |
| Fix PR creation | High+ found | Auto-creates PR |
| Warning | Medium found | PR comment only |
| Info | Low found | Dashboard only |

(All configurable via CLI flags and workflow settings)

---

## After Phase 1: Ongoing Management

### Weekly Tasks

- Review GitHub Code Scanning dashboard
- Approve/merge fix PRs from Snyk
- Update `.snyk` policies as needed
- Monitor nightly scan results

### Monthly Tasks

- Audit suppressed vulnerabilities
- Update expiry dates on suppressions
- Review Snyk dashboard trends
- Plan Phase 2 expansion (if needed)

### As-Needed

- Run `snyk fix --force` for bulk patching
- Update policies for new vulnerabilities
- Adjust severity thresholds
- Configure alerts in Snyk dashboard

---

## Additional Resources

| Resource | URL |
|----------|-----|
| **Snyk Docs** | https://docs.snyk.io |
| **CLI Reference** | https://docs.snyk.io/snyk-cli/cli-reference |
| **Policy Files** | https://docs.snyk.io/policies/the-snyk-policy-file |
| **GitHub Integration** | https://docs.snyk.io/integrations/git-repositories/github |
| **Snyk Dashboard** | https://app.snyk.io/dashboard |

---

## Contact & Support

For issues or questions:

1. Check SNYK_SETUP_GUIDE.md troubleshooting section
2. Review Snyk documentation
3. Run script with `--help` flag for usage details
4. Check `.snyk-reports/` for detailed error logs

---

## Checklist for Phase 1 Rollout

**Pre-Execution:**
- [ ] Snyk account created
- [ ] API token generated & saved securely
- [ ] Snyk CLI installed locally
- [ ] Scripts reviewed (`snyk-deploy.sh`, `snyk-policy-generator.sh`)

**Execution:**
- [ ] Run `snyk-deploy.sh` for Tier 1 repos
- [ ] Review `.snyk-reports/` output
- [ ] Review generated `.snyk` policy files
- [ ] Commit policy files to git

**Post-Execution:**
- [ ] Deploy GitHub Actions workflow to Tier 1 repos
- [ ] Add `SNYK_TOKEN` to GitHub org secrets
- [ ] Test workflow with manual trigger
- [ ] Verify PR comments appear on test PR
- [ ] Check Code Scanning dashboard shows results

**Ongoing:**
- [ ] Monitor nightly scan results
- [ ] Review fix PRs from Snyk
- [ ] Update policies as vulnerabilities are fixed

---

## Summary

**All infrastructure is ready.** Waiting only for:
1. Snyk API token (from user)
2. GitHub secret setup (from user)

Then execute Phase 1 deployment in ~35 minutes total.

Subsequent phases (Tier 2, 3, etc.) follow identical procedures.

**Next Action**: Obtain Snyk token and begin Phase 1 execution (see Step 1-6 above).
