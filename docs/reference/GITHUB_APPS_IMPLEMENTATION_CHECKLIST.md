# GitHub Apps Implementation Checklist

**Target**: Optimize code review tooling, save $3,600/year
**Timeline**: 2 weeks
**Effort**: ~3 hours total
**Owner**: DevOps/Architecture Team

---

## Phase 1: Free Tools Installation (Week 1)

### Immediate Setup (Can be done in parallel)

#### [ ] Task 1.1: Install Stepsize
**Time**: 2 minutes | **Risk**: None | **Blocker**: No

- [ ] Go to https://github.com/apps/stepsize-io
- [ ] Click "Install"
- [ ] Grant access to your organizations
- [ ] Approve repository selection
- [ ] Verify in Settings > Integrations > GitHub Apps

**Verification**: Should see "Stepsize" in organization GitHub Apps

---

#### [ ] Task 1.2: Install Snyk via GitHub Marketplace
**Time**: 10 minutes | **Risk**: Low | **Blocker**: No

- [ ] Go to https://github.com/marketplace/snyk
- [ ] Click "Set up a plan"
- [ ] Choose "Free" plan
- [ ] Grant OAuth permissions
- [ ] Select organization
- [ ] Click "Install"
- [ ] Create Snyk account if needed

**Verification**: Snyk should appear in PR checks (after first repo scan)

---

#### [ ] Task 1.3: Set up DeepSource
**Time**: 10 minutes | **Risk**: Low | **Blocker**: No

- [ ] Go to https://deepsource.io
- [ ] Click "Start free"
- [ ] Choose "Continue with GitHub"
- [ ] Grant OAuth permissions
- [ ] Select organization
- [ ] Select repositories to analyze
- [ ] Review default configuration

**Verification**: DeepSource should show initial analysis in 5-10 minutes

---

#### [ ] Task 1.4: Set up SonarCloud
**Time**: 15 minutes | **Risk**: Low | **Blocker**: No

- [ ] Go to https://sonarcloud.io
- [ ] Click "Create organization"
- [ ] Choose "GitHub"
- [ ] Select organization
- [ ] Grant OAuth permissions
- [ ] Select up to 3 private repos (free tier limit)
- [ ] Note the `SONAR_TOKEN` (save securely)

**Verification**: SonarCloud creates initial project keys

---

### GitHub Actions Configuration

#### [ ] Task 1.5: Add Snyk to GitHub Actions (Optional, recommended)
**Time**: 15 minutes | **Risk**: Low | **Blocker**: No

**Files to Update**:
- `.github/workflows/security.yml`

**Add Job** (before `osv-scanner` job):

```yaml
snyk:
  name: Snyk Dependency Scan
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: snyk/actions/setup@master
    - name: Snyk Auth
      env:
        SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
      run: snyk auth $SNYK_TOKEN
    - name: Snyk Test
      continue-on-error: true
      run: snyk test --severity-threshold=high || true
```

**Steps**:
1. [ ] Create a Snyk API token at https://app.snyk.io/account/settings/api
2. [ ] Add as repository secret: `SNYK_TOKEN`
3. [ ] Commit `.github/workflows/security.yml`
4. [ ] Push and verify job runs

**Verification**: Job should complete with "Snyk Test" output in logs

---

#### [ ] Task 1.6: Add OWASP Dependency-Check to GitHub Actions
**Time**: 10 minutes | **Risk**: Low | **Blocker**: No

**Files to Update**:
- `.github/workflows/security.yml`

**Add Job** (after `osv-scanner` job):

```yaml
dependency-check:
  name: OWASP Dependency-Check
  runs-on: ubuntu-latest
  permissions:
    contents: read
    security-events: write
  steps:
    - uses: actions/checkout@v4
    - uses: dependency-check/Dependency-Check_Action@main
      with:
        project: 'phenotype-infrakit'
        path: '.'
        format: 'SARIF'
        args: >
          --enablePackageLevelVulnerabilities
          --enableVulnerabilityXrefs
    - name: Upload SARIF
      uses: github/codeql-action/upload-sarif@v3
      if: always()
      with:
        sarif_file: dependency-check-report.sarif
        category: dependency-check
```

**Steps**:
1. [ ] Add job to `.github/workflows/security.yml`
2. [ ] Commit and push
3. [ ] Verify job runs on next security workflow trigger

**Verification**: SARIF should upload to GitHub Code Scanning

---

## Phase 2: GitHub Actions Configuration (Week 1-2)

### SonarCloud Integration

#### [ ] Task 2.1: Add SonarCloud to GitHub Actions
**Time**: 20 minutes | **Risk**: Low | **Blocker**: No

**Files to Create/Update**:
- `.github/workflows/quality.yml` (new file)

**Workflow Content**:

```yaml
name: Quality Gate (SonarCloud)

on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main

jobs:
  sonarcloud:
    name: SonarCloud Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Fetch all history for analysis

      - uses: actions/setup-python@v5
        with:
          python-version: '3.14'

      - uses: SonarSource/sonarcloud-github-action@master
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
```

**Steps**:
1. [ ] Get `SONAR_TOKEN` from SonarCloud (saved in Task 1.4)
2. [ ] Add as repository secret: `SONAR_TOKEN`
3. [ ] Create `.github/workflows/quality.yml`
4. [ ] Create `sonar-project.properties` in repo root:

```properties
sonar.projectKey=KooshaPari_phenotype-infrakit
sonar.projectName=phenotype-infrakit
sonar.sources=crates,platforms,python
sonar.exclusions=**/node_modules/**,**/target/**,**/.archive/**
sonar.coverage.exclusions=**/*test*,**/*mock*
```

5. [ ] Commit both files
6. [ ] Verify workflow runs on next PR

**Verification**: SonarCloud badge should appear in PR checks

---

### TypeScript/JavaScript Linting

#### [ ] Task 2.2: Add oxlint to heliosApp CI (Optional)
**Time**: 15 minutes | **Risk**: Low | **Blocker**: No

**File**: `heliosApp/.github/workflows/lint.yml` (create new)

```yaml
name: Lint

on:
  push:
    branches: [main]
  pull_request:

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oxc-project/oxlint-action@main
        with:
          cwd: .
```

**Steps**:
1. [ ] Create `heliosApp/.github/workflows/lint.yml`
2. [ ] Commit and push
3. [ ] Verify on next PR to heliosApp

---

## Phase 3: Parallel Evaluation (Week 2-3)

### CodeRabbit vs Alternatives

#### [ ] Task 3.1: Run DeepSource, SonarCloud, Codacy in parallel with CodeRabbit
**Time**: Passive (monitoring) | **Risk**: None | **Blocker**: No

**What to Monitor**:
- [ ] DeepSource: Review quality, false positives, actionable suggestions
- [ ] SonarCloud: Coverage metrics, quality gates, severity distribution
- [ ] Codacy: Comment quality, language support (especially TypeScript)
- [ ] CodeRabbit: Current baseline for comparison

**Evaluation Criteria**:
1. **Review Coverage**: % of issues caught
2. **False Positive Rate**: % of non-issues flagged
3. **Actionability**: % of suggestions that are implemented
4. **User Experience**: Ease of understanding recommendations
5. **Configuration**: Effort to customize/tune

**Timeline**: 2 weeks of parallel observation

**Verification Document**: Create `docs/GITHUB_TOOLS_EVALUATION.md`

---

#### [ ] Task 3.2: Document findings
**Time**: 30 minutes | **Risk**: None | **Blocker**: No

**Create file**: `docs/GITHUB_TOOLS_EVALUATION.md`

**Contents**:
- Comparison table (criteria × tools)
- Pros/cons for each tool
- Recommendation with rationale
- Cost-benefit analysis

---

## Phase 4: Decision & Cleanup (Week 4)

### CodeRabbit Cancellation (If Recommended)

#### [ ] Task 4.1: Make go/no-go decision
**Time**: 10 minutes | **Risk**: Medium | **Blocker**: Go-gate for next tasks

**Decision Criteria**:
- [ ] DeepSource + SonarCloud coverage adequate? (>80% parity with CodeRabbit)
- [ ] False positive rate acceptable? (<10%)
- [ ] Team agrees to replacement?
- [ ] Cost savings ($3,600/year) justify any minor gaps?

**If YES**: Proceed to 4.2
**If NO**: Keep CodeRabbit, use free tools as supplements

---

#### [ ] Task 4.2: Cancel CodeRabbit Subscription
**Time**: 5 minutes | **Risk**: Low | **Blocker**: No

**Steps**:
1. [ ] Go to CodeRabbit dashboard
2. [ ] Navigate to Billing
3. [ ] Click "Cancel Subscription"
4. [ ] Select reason: "Using alternative tools"
5. [ ] Confirm cancellation
6. [ ] Document in decision ticket

**Verification**: Subscription should show "Cancelled" status

---

#### [ ] Task 4.3: Remove CodeRabbit from branch protection (Optional)
**Time**: 10 minutes | **Risk**: Medium | **Blocker**: No

**Only if CodeRabbit is cancelled**

**Steps**:
1. [ ] Go to repository Settings > Branches
2. [ ] Click on main branch protection rule
3. [ ] Scroll to "Require status checks to pass before merging"
4. [ ] Uncheck "CodeRabbit" (if present)
5. [ ] Save

**Note**: If CodeRabbit is set as **required**, removing it requires admin action (may need to adjust branch protection rules)

---

#### [ ] Task 4.4: Update documentation
**Time**: 15 minutes | **Risk**: None | **Blocker**: No

**Files to Update**:
- [ ] `docs/reference/GITHUB_APPS_AUDIT.md` → Add "Decision Made: [Date]"
- [ ] `docs/reference/GITHUB_APPS_SUMMARY.md` → Update cost table
- [ ] `CLAUDE.md` → Add note about tool migration
- [ ] `README.md` → Update CI/CD section if needed

---

## Phase 5: Repository-Specific Setup (Weeks 2-3)

### heliosCLI Enhancement

#### [ ] Task 5.1: Add security scanning to heliosCLI CI
**Time**: 20 minutes | **Risk**: Low | **Blocker**: No

**Files to Create**: `.github/workflows/security.yml`

**Include**:
- Cargo audit
- Cargo deny
- CodeQL (Rust)
- Gitleaks

```yaml
name: Security

on:
  push:
    branches: [main]
  pull_request:

jobs:
  cargo-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

---

### heliosApp Enhancement

#### [ ] Task 5.2: Add CI/CD to heliosApp
**Time**: 30 minutes | **Risk**: Low | **Blocker**: No

**Create 2 workflows**:

1. **`.github/workflows/lint.yml`** - Already created in Task 2.2
2. **`.github/workflows/build.yml`** - New

```yaml
name: Build & Test

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v1
      - run: bun install
      - run: bun run build
      - run: bun run test
      - run: bun run type-check
```

---

## Rollback Plan (If Needed)

If any tool causes issues:

### [ ] Disable Tool Without Uninstalling
**For GitHub Apps** (Stepsize, Snyk, etc.):
1. Go to Settings > Integrations > GitHub Apps
2. Click the app
3. Click "Suspend"
4. (Can re-enable later without re-installing)

**For GitHub Actions**:
1. Comment out the job in the workflow file
2. Commit and push
3. Job will not run

**For Branch Protection**:
1. Go to Settings > Branches > Protection
2. Uncheck the required status check
3. Save

---

## Success Criteria

- [ ] All 4 free tools installed and active (Phase 1)
- [ ] GitHub Actions workflows updated with new jobs (Phase 2)
- [ ] SonarCloud integration complete with quality gates (Phase 2)
- [ ] DeepSource vs CodeRabbit evaluation completed (Phase 3)
- [ ] Decision document created (Phase 3)
- [ ] heliosCLI and heliosApp CI/CD complete (Phase 5)
- [ ] All documentation updated (Phase 4)

---

## Estimated Time Breakdown

| Phase | Task | Time | Total |
|-------|------|------|-------|
| **Phase 1** | Install 4 tools + configs | 2+10+10+15+15+10 min | 62 min |
| **Phase 2** | Add SonarCloud + oxlint | 20+15 min | 35 min |
| **Phase 3** | Monitor & document | Passive + 30 min | 30 min |
| **Phase 4** | Decision + cleanup | 5+10+15 min | 30 min |
| **Phase 5** | heliosCLI + heliosApp | 20+30 min | 50 min |
| **TOTAL** | | | **207 minutes** (~3.5 hours) |

---

## Sign-Off

**Owner**: DevOps/Architecture
**Start Date**: [To be filled]
**Target Completion**: [Start + 2 weeks]
**Actual Completion**: [To be filled]

**Approver**: [Name]
**Date**: [To be filled]

---

## Related Documents

- **Full Audit**: `docs/reference/GITHUB_APPS_AUDIT.md`
- **Quick Summary**: `docs/reference/GITHUB_APPS_SUMMARY.md`
- **Evaluation Results**: `docs/GITHUB_TOOLS_EVALUATION.md` (create in Phase 3)
- **Branch Protection Config**: `Settings > Branches`
- **Secrets Management**: `Settings > Secrets and variables`

---

Generated: 2026-03-30
Last Updated: 2026-03-30
Owner: Claude Code Audit System
