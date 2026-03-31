# Code Review Implementation Checklist

**Phenotype Polyrepo — Code Review Tool Setup**

---

## Phase 1: CodeRabbit Integration (Automated Review) — READY

### CodeRabbit Setup
- [x] CodeRabbit installed and configured
- [x] `.coderabbit.yaml` created at repo root
- [x] Project-specific `.coderabbit.yaml` files (AgilePlus, heliosCLI, agentapi-plusplus, portage)
- [x] Language-specific rules configured (Rust, Python)
- [x] File-pattern based review rules defined
- [x] Code review rules implemented (security, performance, architecture, testing)
- [x] Auto-merge conditions configured
- [x] Auto-summary enabled
- [x] Inline comments enabled

**Verification**:
```bash
# Verify .coderabbit.yaml is valid YAML
cd /Users/kooshapari/CodeProjects/Phenotype/repos
yamllint .coderabbit.yaml
yamllint AgilePlus/.coderabbit.yaml

# Test on next PR — should see CodeRabbit review comment
```

---

## Phase 2: GitHub Native Features (Manual Review) — READY

### Branch Protection Rules
- [ ] Go to repo Settings → Branches
- [ ] Click "Add rule" under Branch protection rules
- [ ] Configure for `main` branch:
  - [ ] Require pull request reviews before merging (1 reviewer)
  - [ ] Dismiss stale pull request approvals when new commits are pushed
  - [ ] Require code owner review
  - [ ] Require status checks to pass:
    - [ ] CodeQL (rust)
    - [ ] Cargo Audit
    - [ ] Cargo Deny
    - [ ] Gitleaks
    - [ ] OSV Scanner
    - [ ] quality-gate (if custom)
  - [ ] Require branches to be up to date before merging
  - [ ] Require conversation resolution before merging
  - [ ] Require signed commits

**For each repo**:
- [ ] phenotype-infrakit
- [ ] AgilePlus
- [ ] heliosCLI
- [ ] heliosApp
- [ ] agentapi-plusplus
- [ ] Other active projects

### CodeOwners Configuration
- [x] CODEOWNERS file exists at repo root
- [x] All file patterns covered
- [x] Primary code owners assigned (@KooshaPari)
- [ ] (Optional) Create GitHub teams for larger teams:
  - [ ] @phenotype-core
  - [ ] @phenotype-agents
  - [ ] @phenotype-infra
- [ ] Update CODEOWNERS to use teams (when ready)

**Current CODEOWNERS**:
```
* @KooshaPari
.github/workflows/ @KooshaPari
.github/hooks/ @KooshaPari
docs/ @KooshaPari
crates/ @KooshaPari
python/ @KooshaPari
```

---

## Phase 3: PR Templates — READY

### Root PR Template
- [x] `.github/PULL_REQUEST_TEMPLATE.md` created (from guide)
- [ ] Customize project-specific templates:
  - [ ] `AgilePlus/.github/PULL_REQUEST_TEMPLATE.md`
  - [ ] `heliosCLI/.github/PULL_REQUEST_TEMPLATE.md`
  - [ ] `heliosApp/.github/PULL_REQUEST_TEMPLATE.md`
  - [ ] Others as needed

**Test**:
- [ ] Create draft PR
- [ ] Verify template appears in PR description editor

---

## Phase 4: GitHub Actions Workflows — READY

### Existing Workflows (Verify Status)
- [x] `quality-gate.yml` — Lint, test, coverage checks
- [x] `security.yml` — Cargo Audit, CodeQL, Gitleaks, Python security
- [x] `codeql.yml` — CodeQL security analysis
- [x] Other workflows listed in `.github/workflows/`

### Auto-Format Workflow (Optional)
- [ ] Create `.github/workflows/auto-format.yml`
  - [ ] Runs `cargo fmt` on Rust files
  - [ ] Runs `black` on Python files (if used)
  - [ ] Commits changes back to PR
- [ ] Test on draft PR with formatting issues

### Request Human Review Workflow (Optional)
- [ ] Create `.github/workflows/request-review.yml`
  - [ ] Detects critical file changes (.github/workflows, Cargo.toml, etc.)
  - [ ] Automatically requests code owner review
  - [ ] Posts comment explaining why

### Auto-Merge Workflow (Optional)
- [ ] Create `.github/workflows/auto-merge-approved.yml`
  - [ ] Merges PR when all checks pass + approval
  - [ ] Uses `squash` merge method
  - [ ] Deletes branch after merge

---

## Phase 5: Dependabot (Dependency Updates) — READY TO ENABLE

### Dependabot Configuration
- [ ] Create `.github/dependabot.yml`:
  ```yaml
  version: 2
  updates:
    - package-ecosystem: cargo
      directory: "/"
      schedule:
        interval: weekly
      open-pull-requests-limit: 5
      reviewers: ["@KooshaPari"]
      auto-merge:
        - match:
            update-types: ["minor", "patch"]
  ```
- [ ] Enable for each package ecosystem:
  - [ ] Cargo (Rust)
  - [ ] Pip (Python)
  - [ ] Npm (JavaScript, if applicable)
- [ ] Test: Commit a package.json/Cargo.toml change; Dependabot should create PR

### Dependabot Auto-Merge (Optional)
- [ ] Create `.github/workflows/auto-merge-dependabot.yml`
- [ ] Auto-merge patch and minor version bumps
- [ ] Manual review for major version bumps

---

## Phase 6: Security Scanning Integration — READY

### Existing Tools (Verify)
- [x] Cargo Audit enabled (`.github/workflows/security.yml`)
- [x] CodeQL enabled (`.github/workflows/codeql.yml`)
- [x] Gitleaks enabled (`.github/workflows/security.yml`)
- [x] Cargo Deny enabled (`.github/workflows/security.yml`)
- [x] OSV Scanner enabled (`.github/workflows/security.yml`)
- [x] Python Bandit enabled (`.github/workflows/security.yml`)

### Additional Tools (Optional)
- [ ] Snyk (if needed): Create `.github/workflows/snyk.yml`
- [ ] DeepSource (if needed): Integrate via GitHub app
- [ ] OWASP Dependency-Check (if needed)

**Status Check**:
```bash
# Verify security workflows run
cd /Users/kooshapari/CodeProjects/Phenotype/repos
gh api repos/KooshaPari/phenotype-infrakit/actions/workflows \
  --jq '.workflows[] | select(.name | contains("security"))'
```

---

## Phase 7: Configuration Files Review — READY

### review.toml Validation
- [x] `review.toml` exists with merge gate configuration
- [x] Approval thresholds defined (70% normal, 85% critical, 50% hotfix)
- [x] Auto-merge conditions specified
- [x] Tool weights configured (CodeRabbit 40%, Cargo Audit 25%, etc.)
- [x] Critical file patterns listed

**Verify**:
```bash
toml-lint /Users/kooshapari/CodeProjects/Phenotype/repos/review.toml
```

### GitHub Actions Permissions
- [ ] Go to Repo → Settings → Actions → Permissions
- [ ] Verify:
  - [ ] Allow all actions and reusable workflows
  - [ ] Read/Write permissions for all workflows
  - [ ] Actions can commit changes back (for auto-fix workflows)

---

## Phase 8: Notifications and SLAs — OPTIONAL

### Email Notifications
- [ ] Configure GitHub notification preferences:
  - [ ] Watch the repo
  - [ ] Get emails on PR reviews
  - [ ] Get emails on mentions
- [ ] Set frequency (immediate, daily, custom)

### Slack Integration (When Ready)
- [ ] Install GitHub Slack app in workspace
- [ ] Subscribe to PR events:
  ```
  /github subscribe KooshaPari/phenotype-infrakit \
    pulls,reviews,commits
  ```
- [ ] Configure notification channels
- [ ] (Optional) Enable CodeRabbit Slack posting

### Review SLAs Documentation
- [ ] Document review time expectations by PR type
- [ ] Post in team communication channel
- [ ] Track metrics (average review time, merge time)

---

## Phase 9: Agent-Driven Review (Experimental) — FUTURE

### Agent Review Workflow Setup
- [ ] Create `.github/workflows/agent-review.yml`
  - [ ] Trigger on large PRs (>500 lines)
  - [ ] Or on label `needs-agent-review`
  - [ ] Post summary comment
- [ ] Integrate with Claude API (when available)
- [ ] Test with experimental PR

### Cloud Agent Coordination
- [ ] Document when to request agent review
- [ ] Setup fallback process if auto-agent fails
- [ ] Monitor agent review quality and feedback

---

## Phase 10: Documentation and Onboarding — IN PROGRESS

### Documentation Files
- [x] `CODE_REVIEW_AND_PR_AUTOMATION_GUIDE.md` (comprehensive guide)
- [x] `CODE_REVIEW_IMPLEMENTATION_CHECKLIST.md` (this file)
- [ ] `GITHUB_ACTIONS_WORKFLOW_TEMPLATES.md` (ready-to-use templates)
- [ ] Team onboarding guide (in team docs)

### Developer Onboarding
- [ ] Add code review guide to project README
- [ ] Link from main CONTRIBUTING.md
- [ ] Create quick-start guide:
  - [ ] How to open a PR
  - [ ] How to respond to reviews
  - [ ] How to trigger auto-merge
  - [ ] How to handle failed checks

---

## Testing Checklist

### Test CodeRabbit Review
- [ ] Create draft PR with intentional issues:
  - [ ] Missing test coverage
  - [ ] Unsafe unwrap() in Rust
  - [ ] Type hint missing in Python
- [ ] Verify CodeRabbit posts review comment
- [ ] Verify CodeRabbit suggests fixes

### Test Branch Protection
- [ ] Attempt to merge PR without approval → Should be blocked
- [ ] Attempt to merge PR without passing checks → Should be blocked
- [ ] Approve PR with all checks passing → Should allow merge

### Test CodeOwners
- [ ] Modify Cargo.toml → Should request code owner review
- [ ] Modify .github/workflows → Should request code owner review
- [ ] Modify docs → Should allow merge without owner review (per config)

### Test Auto-Format Workflow
- [ ] Commit unformatted code
- [ ] Verify workflow runs `cargo fmt`
- [ ] Verify changes are committed back

### Test Dependabot
- [ ] Manually trigger Dependabot check:
  ```bash
  gh repo poke <owner/repo> --trigger-dependabot
  ```
- [ ] Verify PR is created with dependency update
- [ ] Verify auto-merge works for patch versions

### Test PR Template
- [ ] Create new PR → Should see template in description editor
- [ ] Verify all sections are present
- [ ] Check formatting is correct

---

## Metrics and Monitoring

### Metrics to Track
- [ ] Average review time (target: 24h for features, 6h for bugs)
- [ ] Number of auto-merged PRs vs. manual merges
- [ ] CodeRabbit suggestion acceptance rate
- [ ] CI/CD pass rate (target: >95%)
- [ ] Security scan findings per month
- [ ] Code owner approval time

### Dashboard Setup (Optional)
- [ ] GitHub Insights → Pull requests → Monitor trends
- [ ] GitHub Actions → Monitor workflow runs
- [ ] CodeRabbit Dashboard → Monitor review metrics

### Weekly Review
- [ ] Check PR merge metrics
- [ ] Review failed status checks
- [ ] Identify bottlenecks
- [ ] Adjust thresholds if needed

---

## Troubleshooting Reference

| Issue | Symptom | Solution |
|-------|---------|----------|
| CodeRabbit not reviewing | No review comment on PR | Check app installed, not draft PR, YAML valid |
| Branch protection not enforcing | Can merge without approval | Enable in Settings → Branches → main |
| Status checks not blocking | PR merges with failed checks | Add required checks to branch protection |
| Auto-format commits failing | Workflow runs but no commits | Check workflow has write permissions |
| Gitleaks hanging | Security workflow stuck | Kill process, switch to trufflehog |
| CodeOwners not enforcing | Can merge without owner approval | Enable "Require code owner review" |

---

## Post-Implementation Validation

### Sign-Off Checklist
- [ ] All workflows run successfully on test PR
- [ ] Branch protection enforces requirements
- [ ] CodeRabbit reviews within 5 minutes
- [ ] PR template appears in new PRs
- [ ] Dependabot creates PRs for dependency updates
- [ ] Auto-merge works for approved PRs
- [ ] Security scans pass on main branch
- [ ] Documentation is complete and clear
- [ ] Team is trained on new processes

### When to Consider "Complete"
1. CodeRabbit configured and tested on all repos
2. Branch protection enabled for `main` in all repos
3. PR templates deployed to all major projects
4. Dependabot creating and merging PRs
5. Team uses new workflows without friction
6. Metrics dashboard showing healthy review metrics

---

## Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: CodeRabbit | 30 min | READY |
| Phase 2: Branch Protection | 1 hour (per repo) | READY |
| Phase 3: PR Templates | 30 min | READY |
| Phase 4: GitHub Actions | 1-2 hours | READY |
| Phase 5: Dependabot | 30 min | READY |
| Phase 6: Security | 30 min | READY |
| Phase 7: Config Review | 30 min | READY |
| Phase 8: Notifications | 1 hour (optional) | OPTIONAL |
| Phase 9: Agent Review | 2-3 hours | FUTURE |
| Phase 10: Documentation | IN PROGRESS | IN PROGRESS |
| **Total** | **~8-10 hours** | **~70% COMPLETE** |

---

## Quick Links

- **Guide**: `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/reference/CODE_REVIEW_AND_PR_AUTOMATION_GUIDE.md`
- **CodeRabbit**: https://coderabbit.ai/
- **GitHub Docs**: https://docs.github.com/en/code-security/code-scanning/
- **Review Config**: `/Users/kooshapari/CodeProjects/Phenotype/repos/review.toml`
- **CodeRabbit Config**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.coderabbit.yaml`

---

**Last Updated**: 2026-03-30
**Status**: Ready for Phase Implementation
**Owner**: Code Review Infrastructure Task
