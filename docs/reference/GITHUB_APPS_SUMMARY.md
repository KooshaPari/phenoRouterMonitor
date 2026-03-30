# GitHub Apps & Code Review Tools - Quick Reference

**Report Date**: March 30, 2026
**Quick Link**: See full audit at `docs/reference/GITHUB_APPS_AUDIT.md`

---

## Installed Tools Summary Table

| App Name | Tier | Type | Status | Primary Repo | Monthly Cost | Key Features |
|----------|------|------|--------|--------------|--------------|--------------|
| CodeRabbit | Paid | Code Review | ✅ Active | phenotype-infrakit | $300 | AI review, anti-slop, FR tracing |
| CodeQL | Free | SAST | ✅ Active | phenotype-infrakit | $0 | Rust/C++/Python analysis |
| Dependabot | Free | Dep Mgmt | ✅ Active | All repos | $0 | Auto-update PRs |
| Cargo Audit | Free (OSS) | Security | ✅ Active | phenotype-infrakit | $0 | Rust vulns |
| Cargo Deny | Free (OSS) | Security | ✅ Active | phenotype-infrakit | $0 | License check |
| Gitleaks | Free (OSS) | Secrets | ✅ Active | phenotype-infrakit | $0 | Secret detection |
| Bandit | Free (OSS) | Python SAST | ✅ Active | phenotype-infrakit | $0 | Python security |
| OSV-Scanner | Free (OSS) | Supply Chain | ✅ Active | phenotype-infrakit | $0 | Vuln scanning |

---

## Recommended Free Tools (Ranked by Priority)

| Priority | Tool | Cost | Setup | Replaces | Annual Savings |
|----------|------|------|-------|----------|-----------------|
| 🔴 1 | **DeepSource** | Free | 5 min | CodeRabbit | $3,600 |
| 🟠 2 | **SonarCloud** | Free (3 repos) | 15 min | Manual review | $99-300 |
| 🟡 3 | **Codacy** | Free (2 repos) | 10 min | Partial CodeRabbit | $129-300 |
| 🟢 4 | **Stepsize** | Free | 2 min | Manual debt tracking | $99 |
| 🔵 5 | **Snyk** | Free (unlimited) | 10 min | Partial security coverage | $199-500 |

---

## By Repository Status

### phenotype-infrakit ✅ Well-Configured
- CodeRabbit: Active, comprehensive config (240 lines)
- Security pipeline: 8 tools integrated
- CI/CD: GitHub Actions (functional)
- **Gaps**: None identified

### heliosCLI ⚠️ Partial Setup
- Dependabot: Active (7 ecosystems)
- Security: Missing (recommend: CodeQL, cargo-audit)
- CodeRabbit: Not enabled
- **Recommendation**: Add 2 GitHub Actions jobs (30 min)

### heliosApp ⚠️ Minimal Setup
- Dependabot: Active (implicit)
- Security: Missing
- CI/CD: No active workflows
- **Recommendation**: Add linting, type-check, CodeQL (1 hour)

---

## Cost-Benefit Analysis

### Current (2026-03-30)
```
CodeRabbit:      $3,600/year
GitHub Actions:  ~$500/year
TOTAL:           ~$4,100/year
```

### Optimized (After Recommendations)
```
DeepSource:      $0/year (free tier)
SonarCloud:      $0/year (free tier for 3 repos)
Codacy:          $0/year (free tier for 2 repos)
Stepsize:        $0/year (free tier)
Snyk:            $0/year (free tier for unlimited)
GitHub Actions:  ~$500/year (no change)
TOTAL:           ~$500/year

SAVINGS:         $3,600/year 🎉
```

---

## Quick Actions (This Week)

- [ ] **Day 1**: Install Stepsize (2 min) — Start labeling technical debt
- [ ] **Day 1**: Install Snyk (10 min) — Enable dependency scanning
- [ ] **Day 2**: Set up DeepSource (10 min) — Parallel CodeRabbit evaluation
- [ ] **Day 3**: Set up SonarCloud (15 min) — Quality gates
- [ ] **Day 5**: Add OWASP Dep-Check GitHub Action (5 min) — Supply chain scanning
- [ ] **Day 10**: Review results from all 4 tools + make CodeRabbit decision

---

## Links

| Tool | Main | GitHub | Docs |
|------|------|--------|------|
| DeepSource | https://deepsource.io | GitHub Marketplace | https://deepsource.io/docs/ |
| SonarCloud | https://sonarcloud.io | GitHub Marketplace | https://docs.sonarsource.com/sonarcloud/ |
| Codacy | https://codacy.com | GitHub Marketplace | https://docs.codacy.com/ |
| Stepsize | https://stepsize.com | https://github.com/apps/stepsize-io | https://docs.stepsize.com/ |
| Snyk | https://snyk.io | GitHub Marketplace | https://docs.snyk.io/ |

---

## Workflows to Update

### Add to all repositories:
1. **Security scanning** (OWASP Dep-Check):
   - File: `.github/workflows/security.yml`
   - Action: `dependency-check/Dependency-Check_Action`
   - Time: 5 min

2. **SAST for TypeScript** (oxlint):
   - File: `.github/workflows/lint.yml`
   - Action: `oxlint/oxlint-action`
   - Time: 5 min

3. **Coverage gates** (SonarCloud via Actions):
   - File: `.github/workflows/quality.yml`
   - Action: `SonarSource/sonarcloud-github-action`
   - Time: 10 min

---

## FAQ

**Q: Will removing CodeRabbit break anything?**
A: No. DeepSource, SonarCloud, and Codacy provide comparable or better coverage. Run both for 2 weeks to verify.

**Q: Do these free tools require external accounts?**
A: Yes, all except Stepsize require signup. But signup is free and takes <5 min via GitHub OAuth.

**Q: Can we use all of them together?**
A: Yes! They complement each other:
- DeepSource: AI-powered review suggestions
- SonarCloud: Quality gates + metrics
- Codacy: Coverage tracking (for TypeScript projects)
- Stepsize: Debt categorization
- Snyk: Dependency security

**Q: Do these work with GitHub Actions?**
A: Yes. DeepSource, SonarCloud, Codacy, and Snyk all have Actions. Stepsize is a GitHub App (works automatically).

**Q: What about heliosApp and heliosCLI?**
A: They need minimal setup (<1 hour each). See implementation roadmap in full audit.

---

## Next Steps

1. Read full audit: `docs/reference/GITHUB_APPS_AUDIT.md`
2. Install Stepsize + Snyk this week
3. Evaluate DeepSource vs CodeRabbit for 2 weeks
4. Make CodeRabbit cancellation decision by end of month
5. Plan heliosApp/heliosCLI CI/CD setup for next sprint

---

Generated: 2026-03-30
Last Updated: 2026-03-30
Owner: Claude Code Audit System
