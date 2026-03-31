# Security & QA Implementation — Quick Reference Card

**Print this.** Post it on your monitor. Share it with your team.

---

## 30-Second Decision

| Path | What | Cost | ROI | Time |
|------|------|------|-----|------|
| **A: GO** | Full 6-phase (SAST, Deps, Lint, Error Track, Agents, Compliance) | $1,848/yr | 38:1 | 12 wks |
| **B: GO Lite** | Phase 1 + 2 only (SAST, Deps, Snyk) | $1,500/yr | 48:1 | 4 wks |
| **C: NO-GO** | Status quo (no automation) | $0 | -39:1 | — |

**Recommend**: Path A (or B if time-constrained)

---

## Phase 1 (Weeks 1-2): SAST Foundation

### What Gets Deployed
- Semgrep (2,500+ security rules)
- trufflehog (secrets detection)
- Per-language linting (Rust, Go, Python, TypeScript)
- Pre-commit hooks (local enforcement)
- GitHub Actions workflow (11 parallel jobs)

### Get Started (30 min)
```bash
cd /path/to/test/repo
# Copy configs
cp /path/to/research/.semgrep.yml .
cp /path/to/research/.pre-commit-config.yaml .

# Install hooks locally
pip install pre-commit
pre-commit install

# Test
pre-commit run --all-files

# Deploy workflow
mkdir -p .github/workflows
cp /path/to/research/quality-gate.yml .github/workflows/

# Commit
git add . && git commit -m "chore: add Phase 1 SAST"
```

### Verify
- [ ] GitHub Actions shows 11 jobs running
- [ ] Pre-commit hooks working locally
- [ ] All checks passing on PR
- [ ] No false positives blocking merge

---

## Phase 2 (Weeks 3-4): Dependency Scanning

### What Gets Deployed
- Snyk (vulnerability scanning)
- npm audit / pip-audit / cargo-audit
- Dependabot (auto-updating deps)

### Setup (1-2 hours)
1. Create Snyk account (Team Plan, $1,500/year)
2. Setup GitHub integration
3. Deploy per-language scanners
4. Enable Dependabot

### Success =
- [ ] Snyk finding vulnerabilities
- [ ] Dependabot creating update PRs
- [ ] 80%+ of critical vulns patched
- [ ] Budget tracking active

---

## Phase 4 (Weeks 7-8): Error Tracking

### What Gets Deployed
- Sentry (error aggregation)
- Language SDKs (Python, Go, Node, etc.)
- Slack alerts
- GitHub issue creation

### Setup (2-4 hours)
1. Create Sentry account (Team Plan, $348/year)
2. Deploy SDKs to all services
3. Setup alerts
4. Enable GitHub integration

### Success =
- [ ] All services reporting errors
- [ ] Slack alerts firing on critical
- [ ] Error grouping working
- [ ] <1 min latency to Sentry

---

## Team Assignments

| Role | Phase 1 | Phase 2 | Phase 4 |
|------|---------|---------|---------|
| **Backend Engineer** | Implement SAST, workflows, pre-commit | Review Dependabot PRs | — |
| **Cloud Engineer** | Review SAST rules | Setup Snyk, audit deps | Setup Sentry SDKs |
| **DevOps** | GitHub branch protection | Dependabot config, cost tracking | Sentry infrastructure |
| **Architect** | Review phase plan | Review phase plan | Review phase plan |

---

## Key Files (Bookmark These)

1. **For decisions**: `GO_NO_GO_DECISION_FRAMEWORK.md` (10 min read)
2. **For Phase 1**: `PHASE1_IMPLEMENTATION_START.md` (step-by-step)
3. **For Snyk**: `SNYK_INTEGRATION_GUIDE.md`
4. **For Sentry**: `SENTRY_INSTRUMENTATION_GUIDE.md`
5. **For everything**: `SECURITY_QA_DEPLOYMENT_ROADMAP.md` (master plan)

**Location**: `/repos/docs/reference/`

---

## Cost Summary (Year 1)

| Item | Cost |
|------|------|
| Snyk (Team Plan) | $1,500 |
| Sentry (Team Plan) | $348 |
| Everything else | FREE |
| **TOTAL** | **$1,848** |

**Saves**: $72,000/year (manual labor)
**ROI**: 38:1
**Payback**: 3 days

---

## Success Metrics (Per Phase)

### Phase 1 ✅
- All 30 repos scanned
- <5% false positives
- 50+ linting issues caught
- Zero critical vulns found

### Phase 2 ✅
- Snyk detecting 100+ vulnerabilities
- 80%+ of critical patched
- Dependabot PRs auto-creating
- Budget tracking active

### Phase 4 ✅
- 80%+ of errors tracked in Sentry
- Slack alerts firing
- <1 min latency
- Error grouping working

---

## Common Issues & Fixes

| Issue | Fix |
|-------|-----|
| Pre-commit too slow | Cache clear: `rm -rf ~/.cache/pre-commit/` |
| Semgrep false positive | File issue, don't suppress (requires tracking) |
| Snyk budget overrun | Set hard limit at $1,500, monitor weekly |
| Sentry not capturing errors | Check SDK initialization, network logs |
| GitHub Actions failing | Check logs, verify tool versions with `pre-commit autoupdate` |

---

## Phase 1 Rollout Order

```
Week 1, Day 1-2:
  Deploy to phenotype-infrakit (test)
  ✓ Passes verification

Week 1, Day 3-5:
  Deploy to Tier-1 (3 repos):
    ✓ phenotype-infrakit
    ✓ AgilePlus
    ✓ platforms/thegent

Week 2, Day 1-5:
  Deploy to Tier-2 (10 repos):
    ✓ heliosCLI, pheno-cli, phench, ...

Week 2, Day 5:
  Deploy to Tier-3 (17 repos)
  ✓ All 30 repos live

END OF WEEK 2:
  Phase 1 Gate Decision: PASS → Proceed to Phase 2
```

---

## Daily Standup Template

**Time**: 9am, 5 minutes

**Template**:
```
Backend Engineer:
  ✓ Completed: [Task]
  → Working on: [Task]
  ⚠ Blocker: [If any]

Cloud Engineer:
  ✓ Completed: [Task]
  → Working on: [Task]
  ⚠ Blocker: [If any]

DevOps:
  ✓ Completed: [Task]
  → Working on: [Task]
  ⚠ Blocker: [If any]

Gate: 🟢 On track / 🟡 At risk / 🔴 Blocked
```

---

## Weekly Status Report

**Time**: Friday 5pm

**Template**:
```
Week N Status: Phase X

✅ Completed:
- [Task 1]
- [Task 2]

🔄 In Progress:
- [Task 1]
- [Blocker: X]

⚠️ Risks:
- [Risk 1]: [Mitigation]

📊 Metrics:
- Repos: X/30 ✓
- Issues caught: X
- Success rate: X%

🎯 Next Week:
- [Phase task 1]
- [Phase task 2]
```

---

## Critical Decision Points

### Phase 1 Gate (End of Week 2)
```
✅ PASS if:
  ☑ All 30 repos scanned
  ☑ <5% false positives
  ☑ >80% team adoption
  ☑ Zero regressions

→ Decision: PROCEED TO PHASE 2
```

### Phase 2 Gate (End of Week 4)
```
✅ PASS if:
  ☑ Snyk live on all repos
  ☑ 80%+ of critical vulns patched
  ☑ Dependabot creating PRs
  ☑ Budget <$1,500/yr

→ Decision: PROCEED TO PHASE 3
```

---

## Emergency Contacts

| Issue | Contact | SLA |
|-------|---------|-----|
| Phase blocker | Project Lead | 24h |
| Tool question | Role owner | 2h |
| Budget alert | Finance | 4h |
| Security issue | CTO | 1h |

**Slack Channel**: #cloud-agents-dev

---

## Checklist: Before You Start

- [ ] Approved Path A, B, or C
- [ ] Assigned team roles
- [ ] Created #cloud-agents-dev channel
- [ ] Scheduled Phase 1 kickoff (tomorrow 10am)
- [ ] Read PHASE1_IMPLEMENTATION_START.md
- [ ] Verified access to repos
- [ ] Got budget approval ($1,848/year)

---

## Cheat Sheet: Commands You'll Need

```bash
# Install pre-commit
pip install pre-commit

# Setup hooks (run once per repo)
pre-commit install

# Run all checks locally
pre-commit run --all-files

# Run specific check
pre-commit run semgrep --all-files

# Auto-fix issues
black .                    # Python
ruff check --fix .         # Python linting
prettier --write .         # JSON/YAML
go fmt ./...              # Go
cargo clippy --fix        # Rust

# Update tool versions
pre-commit autoupdate

# Create GitHub Actions workflow
mkdir -p .github/workflows
cp quality-gate.yml .github/workflows/

# Deploy to GitHub
git add . && git commit -m "chore: add Phase 1 SAST"
git push origin main
```

---

## Remember

| Goal | Reality |
|------|---------|
| "This will take forever" | Phase 1 = 2 weeks, Phase 1+2 = 4 weeks |
| "This is too expensive" | $1,848/year, saves $72,000/year |
| "Pre-commit will slow me down" | 15-30 sec (cached), <2% impact |
| "This is overkill" | 70% of bugs caught before code review |
| "Who has time for this?" | 1.8 FTE for 12 weeks, then scales down |

**Bottom line**: Worth it. Do it.

---

## Next Action

1. **Read**: `GO_NO_GO_DECISION_FRAMEWORK.md` (10 min)
2. **Decide**: Path A, B, or C (you)
3. **Assign**: Team roles (you)
4. **Schedule**: Kickoff tomorrow 10am (you)
5. **Execute**: Phase 1 (Backend Engineer, starting tomorrow)
6. **Report**: Friday 5pm standup (everyone)

---

**Generated**: 2026-03-30
**Status**: Ready to print & post
**Your move**: Print this, read GO_NO_GO_DECISION_FRAMEWORK.md, approve path.

🚀 **Let's go.**
