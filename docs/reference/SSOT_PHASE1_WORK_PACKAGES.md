# SSOT Phase 1 — Work Packages & Execution Plan

**Status:** Ready for Team Assignment
**Created:** 2026-03-31
**Sprint:** 2 weeks (2026-03-31 — 2026-04-11)
**Total Effort:** 80 hours (40h/week × 2 weeks)
**Team Size:** 5 engineers
**Health Score Target:** 42/100 → 65/100

---

## Overview

Five work packages (WP1.1 — WP1.5) decompose SSOT Phase 1 into focused, parallel streams. Each WP is owned by one engineer with clear deliverables and acceptance criteria.

**Parallel Execution:**
- WP1.1: Infrastructure setup (DevOps) — 4h Week 1
- WP1.2: Specs registry (Spec Coordinator) — 6h Week 1
- WP1.3: Auto-merge service (Infrastructure Engineer) — 12h Week 1-2
- WP1.4: CI validation (QA Engineer) — 6h Week 1-2
- WP1.5: Agent onboarding (Agent Lead) — 6h Week 1-2

**Critical Path:** WP1.1 → WP1.2 → WP1.3 → WP1.5 (sequential, 28h)
**Parallelizable:** WP1.1 + WP1.2 + WP1.4 can run simultaneously

---

## WP1.1: Branch Protection & Infrastructure Setup

**Owner:** DevOps Lead
**Duration:** Week 1, Days 1-2 (8 hours total)
**Effort:** 4h Day 1 + 4h Day 2 (buffer for testing)
**Dependencies:** None (start immediately)
**Blocks:** WP1.2, WP1.3

### Scope

- Create/verify specs/main branch on all 3 repos
- Enforce branch protection (linear history, status checks, dismissals)
- Setup GitHub Actions trigger infrastructure
- Document branch protection rules

### Deliverables

**D1.1.1: Branch Setup Script**
- File: `scripts/setup-specs-branch.sh`
- Creates/verifies specs/main on phenotype-infrakit, AgilePlus, platforms/thegent
- Sets upstream: `origin/specs/main`
- Verifies linear history (no merge commits)

**D1.1.2: Branch Protection Configuration**
- File: `.github/BRANCH_PROTECTION_SPECS_MAIN.yml` (documentation)
- Applied to all 3 repos via GitHub API
- Rules:
  - Require 1 approval (from specs-admin)
  - Require status checks: `ci-ssot-validation` pass
  - Dismiss stale PR reviews
  - Require up-to-date branches
  - Include administrators in restrictions

**D1.1.3: GitHub Actions Environment Setup**
- Secrets: `GH_BOT_TOKEN` (read/write repos)
- Environment: `SSOT` with branch and concurrency constraints
- Permissions: `contents:write`, `pull-requests:write`, `issues:write`

### Acceptance Criteria

- ✅ specs/main branch exists on all 3 repos
- ✅ Branch protection rules applied and verified
- ✅ Linear history: 0 merge commits in specs/main
- ✅ GitHub Actions environment ready (secrets configured)
- ✅ Setup script runs without errors
- ✅ All team members can push to specs/agent-* branches
- ✅ Documentation: `.github/BRANCH_PROTECTION_SPECS_MAIN.yml`

### Testing Plan

```bash
# Verify branch setup
./scripts/setup-specs-branch.sh

# Check protection
gh api repos/KooshaPari/phenotype-infrakit/branches/specs/main/protection

# Test push to agent branch (should succeed)
git checkout -b specs/agent-test-wp11
echo "test" >> test.txt
git add test.txt
git commit -m "test: verify push succeeds"
git push origin specs/agent-test-wp11

# Test direct push to specs/main (should fail if protection works)
git push origin HEAD:specs/main  # Should be rejected
```

---

## WP1.2: Specs Registry & Metadata Structure

**Owner:** Spec Coordinator
**Duration:** Week 1, Days 2-3 (12 hours total)
**Effort:** 6h Day 2 + 6h Day 3
**Dependencies:** Blocks on WP1.1 (needs specs/main branch)
**Blocks:** WP1.5 (agent workflows reference registry)

### Scope

- Create master specs registry (index of all specs across repos)
- Design & implement spec metadata schema
- Build registry validation & auto-update logic
- Document registry structure and usage

### Deliverables

**D1.2.1: Master Specs Registry**
- File: `/SPECS_REGISTRY.md` (at repo root)
- Contents:
  - Central registry table (repo, spec_type, version, status, last_updated)
  - Spec synchronization schedule
  - Version tracking across 3 repos
  - Human-readable, machine-parseable

**D1.2.2: Registry Schema (JSON Schema)**
- File: `/specs/REGISTRY_SCHEMA.json`
- Validates spec metadata format
- Properties: repo, spec_type, version, lines, frs_covered, status
- Required fields: repo, spec_type, version, status

**D1.2.3: Registry Maintenance Script**
- File: `scripts/update-specs-registry.py`
- Runs after every specs/main merge
- Scans FUNCTIONAL_REQUIREMENTS.md, ADR.md, PLAN.md, USER_JOURNEYS.md
- Updates registry with version, line counts, FR counts
- Validates against REGISTRY_SCHEMA.json
- Output: Updated SPECS_REGISTRY.md

**D1.2.4: Registry Usage Documentation**
- File: `/docs/reference/SPECS_REGISTRY_GUIDE.md`
- How to read the registry
- How to add new specs
- How to update version numbers
- Examples for 3+ repos

### Acceptance Criteria

- ✅ SPECS_REGISTRY.md exists with all repos listed
- ✅ Registry table accurate (cross-checked against HEAD)
- ✅ Schema validates all 3 repos' specs
- ✅ Registry auto-updates on specs/main merge
- ✅ Documentation complete with examples
- ✅ Script runs without errors: `python3 scripts/update-specs-registry.py`
- ✅ All team members can read and understand registry

### Testing Plan

```bash
# Validate schema
python3 -c "import json; json.load(open('specs/REGISTRY_SCHEMA.json'))"

# Run update script
python3 scripts/update-specs-registry.py

# Verify registry updated
grep "phenotype-infrakit" SPECS_REGISTRY.md

# Validate against schema (if jsonschema available)
python3 scripts/validate-registry-schema.py
```

---

## WP1.3: Auto-Merge Service Implementation

**Owner:** Infrastructure Engineer
**Duration:** Week 1-2, Days 4-10 (20 hours total)
**Effort:** 12h Week 1 + 8h Week 2
**Dependencies:** Blocks on WP1.1 (needs branch setup)
**Blocks:** Final deployment

### Scope

- Design & implement batch-merger Rust service
- Build git2-rs merge logic (3-way merge, conflict detection)
- Implement GitHub Actions workflow integration
- Create GitHub issue generation for conflicts
- Test with real branches (5+ scenarios)

### Deliverables

**D1.3.1: Batch-Merger Crate**
- Crate: `libs/phenotype-batch-merger/`
- Cargo.toml: Dependencies (git2, tokio, serde, tracing)
- src/lib.rs: Public API
  - `list_agent_branches()` → Vec<Branch>
  - `validate_branch()` → ValidationResult
  - `attempt_merge()` → MergeResult
  - `process_batch()` → BatchResult

**D1.3.2: Merge Logic Implementation**
- 3-way merge using git2-rs (base, branch, main)
- Conflict detection: index.has_conflicts()
- Conflict detail extraction (file, line ranges)
- Merge commit creation with proper parent references
- Push to origin + branch deletion

**D1.3.3: GitHub Actions Workflow**
- File: `.github/workflows/auto-merge-specs.yml`
- Schedule: Every 5 minutes (business hours)
- Event trigger: Push to specs/agent-*
- Manual trigger: workflow_dispatch (for testing)
- Outputs: successful, conflicts, failed counts

**D1.3.4: Batch Processor**
- Async processing: 1-50 branches per batch
- Parallel validation & merge attempts (tokio)
- Comprehensive error handling
  - Transient: Retry with backoff
  - Permanent: Create GitHub issues
- Batch result reporting (success/conflict/failed counts)

**D1.3.5: GitHub Issue Creator**
- Generate issues for conflicts (title, body with fix steps)
- Generate issues for validation failures
- Generate issues for merge failures
- Include actionable resolution instructions

### Acceptance Criteria

- ✅ Crate compiles cleanly: `cargo build --release`
- ✅ All tests pass: `cargo test --package phenotype-batch-merger`
- ✅ Handles 50 concurrent branches (load test)
- ✅ Conflict detection works (test with 5+ conflict scenarios)
- ✅ GitHub issues created correctly
- ✅ GitHub Actions workflow runs on schedule
- ✅ Merge completes in <5 minutes
- ✅ All commits + pushes traceable (git log shows merges)
- ✅ Documentation: AUTO_MERGE_SERVICE_ARCHITECTURE.md

### Testing Plan

```bash
# Unit tests
cargo test --package phenotype-batch-merger

# Build release binary
cargo build --release --package phenotype-batch-merger

# Integration test: Real repo
cd /tmp/test-repo
git clone https://github.com/KooshaPari/phenotype-infrakit.git test
cd test
git checkout specs/main

# Create test branch with clean changes
git checkout -b specs/agent-test-wp13-clean
echo "## FR-TEST-001: Test FR" >> FUNCTIONAL_REQUIREMENTS.md
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "specs: add FR-TEST-001\n\nSpec-Traces: FR-TEST-001\nCo-Authored-By: test <test@phenotype.local>"
git push origin specs/agent-test-wp13-clean

# Trigger auto-merge (manual for testing)
gh workflow run auto-merge-specs.yml

# Verify merge
git fetch origin
git log origin/specs/main --oneline | grep "Merge specs/agent-test"

# Test 2: Conflict scenario
git checkout -b specs/agent-test-wp13-conflict
# Edit same line as another branch
git push origin specs/agent-test-wp13-conflict

# Verify issue created
gh issue list | grep "Merge Conflict"
```

### Effort Breakdown

- Day 4: Design, API definition, stub implementation (4h)
- Day 5: Merge logic, conflict detection, error handling (4h)
- Day 6: GitHub Actions integration, issue creation (4h)
- Day 7-8: Testing, bug fixes, edge case handling (4h)
- Day 9-10: Load testing, documentation, deployment prep (4h)

---

## WP1.4: CI Validation Gate & FR↔Test Coverage

**Owner:** QA Engineer
**Duration:** Week 1-2, Days 3-9 (12 hours total)
**Effort:** 6h Week 1 + 6h Week 2
**Dependencies:** None (can start immediately)
**Blocks:** Final CI configuration

### Scope

- Design FR↔Test traceability system
- Implement validation scripts (markdown, FRs, coverage)
- Create GitHub Actions validation workflow
- Build traceability matrix generator
- Document validation rules and requirements

### Deliverables

**D1.4.1: Spec Validation Script**
- File: `scripts/validate-spec-structure.py`
- Checks:
  - Markdown structure valid (no syntax errors)
  - All FR headers match format: `## FR-XXX-NNN: Title`
  - ADR sections present: Decision, Rationale, Consequences
  - No orphan FRs (all referenced in tests or journeys)
- Output: Pass/fail + list of issues

**D1.4.2: FR↔Test Coverage Validator**
- File: `scripts/validate-fr-test-coverage.py`
- Extracts all FRs from FUNCTIONAL_REQUIREMENTS.md
- Extracts all test traces (grep "Traces to: FR-" in tests/)
- Validates:
  - Every FR has ≥1 test
  - Every test traces to ≥1 valid FR
  - 100% coverage (no orphans)
- Output: Coverage report + issues

**D1.4.3: Traceability Matrix Generator**
- File: `scripts/generate-traceability-matrix.py`
- Inputs: FUNCTIONAL_REQUIREMENTS.md, test files
- Outputs: `docs/TRACEABILITY_MATRIX.md`
- Table format: FR | Test File | Test Count | Status
- Updated weekly (via scheduled workflow)

**D1.4.4: GitHub Actions Validation Workflow**
- File: `.github/workflows/ssot-validation.yml`
- Trigger: Push to specs/*, PR to specs/main
- Jobs:
  - Validate spec structure
  - Validate FR↔Test coverage
  - Generate traceability matrix
  - Block merge if coverage <100%

**D1.4.5: Commit Message Hook Validation**
- File: `scripts/hooks/validate-spec-traces.sh`
- Runs pre-commit: Check Spec-Traces format
- Validates references: All FRs in Spec-Traces exist
- Blocks if invalid format or missing FRs

**D1.4.6: Documentation**
- File: `/docs/reference/FR_TEST_TRACEABILITY_GUIDE.md`
- How to add "Traces to: FR-XXX-NNN" comments
- Examples: Rust, Python, Go, TypeScript
- Troubleshooting common issues

### Acceptance Criteria

- ✅ All validation scripts run without errors
- ✅ Markdown validation catches 5+ common issues
- ✅ FR↔Test coverage 100% on all 3 repos
- ✅ Traceability matrix auto-generated weekly
- ✅ GitHub Actions workflow blocks bad PRs
- ✅ Commit hook installed and working
- ✅ Documentation complete with examples
- ✅ Validation passes on current main branches

### Testing Plan

```bash
# Test validation scripts
python3 scripts/validate-spec-structure.py
python3 scripts/validate-fr-test-coverage.py

# Generate traceability matrix
python3 scripts/generate-traceability-matrix.py
cat docs/TRACEABILITY_MATRIX.md

# Test with invalid specs
echo "## INVALID-HEADER" >> FUNCTIONAL_REQUIREMENTS.md
python3 scripts/validate-spec-structure.py  # Should fail

# Test with untraced FR
echo "## FR-TEST-999: Untraced" >> FUNCTIONAL_REQUIREMENTS.md
python3 scripts/validate-fr-test-coverage.py  # Should fail

# Test commit hook
pre-commit run validate-spec-traces --all-files
```

### Effort Breakdown

- Day 3: Design traceability system (2h)
- Day 4-5: Implement validation scripts (4h)
- Day 6: GitHub Actions workflow (2h)
- Day 7: Traceability matrix generator (2h)
- Day 8-9: Testing, documentation (2h)

---

## WP1.5: Agent Onboarding & Documentation

**Owner:** Spec Coordinator + Tech Writer
**Duration:** Week 1-2, Days 5-10 (12 hours total)
**Effort:** 6h Week 1 + 6h Week 2
**Dependencies:** Blocks on WP1.1, WP1.2, WP1.3 (needs infrastructure ready)
**Blocks:** Final agent launch

### Scope

- Write comprehensive agent workflow guide
- Create onboarding scripts & templates
- Develop training materials (quickstart, examples)
- Onboard 5+ pilot agents, collect feedback
- Finalize documentation based on feedback

### Deliverables

**D1.5.1: Agent Workflow Guide**
- File: `/docs/reference/SSOT_PHASE1_AGENT_WORKFLOW.md` (already created)
- Sections:
  - Branching pattern (specs/agent-<name>-<task-id>)
  - Commit message format with examples
  - Step-by-step workflow (create, edit, validate, push)
  - Conflict handling (3+ scenarios)
  - FR↔Test traceability (how to add traces)
  - Pre-push validation checklist
  - Auto-merge triggers & timeline
  - PR template for spec reconciliation
  - Common workflows (single FR, multiple specs, conflict resolution)
  - Troubleshooting (5+ issues)
  - Glossary

**D1.5.2: Agent Setup Script**
- File: `scripts/agent-setup.sh`
- One-time setup for each agent
- Configures: git user, commit template, pre-commit hooks
- Installs: pre-commit framework, validation hooks
- Output: "✅ Agent <name> setup complete"

**D1.5.3: Onboarding Guide (Quickstart)**
- File: `/docs/guides/SSOT_PHASE1_QUICKSTART.md`
- 5-minute read (500 words)
- Overview, 3 common workflows, troubleshooting
- Links to full documentation

**D1.5.4: Spec Format Standards**
- File: `/docs/reference/SPEC_FORMAT_STANDARDS.md`
- Format for each spec type: FR, ADR, PLAN, UJ
- Example headers, sections, examples
- From 3+ canonical repos

**D1.5.5: Training Materials**
- Video walkthrough (5 min): Create branch → Commit → Merge
- Presentation slides: SSOT overview, agent roles, workflows
- Example branches with real commits
- Q&A document

**D1.5.6: Pilot Onboarding**
- Recruit 5 pilot agents (from different domains)
- Walk through setup script
- Collect feedback (what's confusing? what's missing?)
- Iterate on documentation
- Publish final version

### Acceptance Criteria

- ✅ Agent workflow guide complete (1,500+ lines, 5+ examples)
- ✅ Setup script works for 5+ agents
- ✅ Quickstart guide <500 words, <5 min read
- ✅ Spec format standards cover all 4 types
- ✅ Video recorded (optional but recommended)
- ✅ 5 pilot agents successfully onboarded
- ✅ Feedback incorporated into final docs
- ✅ All team members confident in workflows

### Testing Plan

```bash
# Setup script test
bash scripts/agent-setup.sh test-agent-1

# Verify setup
git config user.name  # Should be "test-agent-1"
git config commit.template  # Should be .commit-template

# Pilot workflow
git checkout -b specs/agent-test-agent-1-wp15-pilot
echo "## FR-TEST-101: Pilot test" >> FUNCTIONAL_REQUIREMENTS.md
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "specs: add FR-TEST-101

Spec-Traces: FR-TEST-101
Co-Authored-By: test-agent-1 <agent@phenotype.local>"

pre-commit run --all-files  # Should pass
git push origin specs/agent-test-agent-1-wp15-pilot  # Should succeed

# Verify auto-merge
git checkout specs/main && git pull origin specs/main
git log --oneline -5 | grep "Merge specs/agent"  # Should show merge
```

### Effort Breakdown

- Day 5: Write agent workflow guide (4h)
- Day 6: Create onboarding script & quickstart (2h)
- Day 7: Write spec format standards (2h)
- Day 8: Recruit & onboard 5 pilot agents (4h)
- Day 9: Collect feedback & iterate (2h)
- Day 10: Finalize all documentation (2h)

---

## WP1.6: Operational Monitoring & Final Deployment (Bonus)

**Owner:** Platform Engineer
**Duration:** Week 2, Days 9-10 (bonus, if time permits)
**Effort:** 8 hours
**Dependencies:** Blocks on WP1.1-1.5

### Scope (Optional, May Defer to Phase 2)

- Deploy health dashboard
- Configure daily health checks
- Deploy alerting (score below 50)
- Create rollback procedures
- Execute final deployment to all repos

### Deliverables

- Health score calculator script
- Daily health check workflow
- Alerting GitHub issues
- Deployment playbook
- Rollback procedures

### Note

This WP is **optional** and can be deferred to Phase 2 if timeline tight. Core SSOT Phase 1 complete with WP1.1-1.5.

---

## Dependency Graph (DAG)

```
WP1.1 (Branch Setup)
  ├─→ WP1.2 (Registry)
  │   └─→ WP1.5 (Onboarding)
  │
  ├─→ WP1.3 (Auto-Merge Service)
  │   └─→ WP1.5 (Onboarding)
  │
  └─→ WP1.4 (CI Validation)
      └─→ WP1.5 (Onboarding)

WP1.5 (Onboarding) ← Final blocker before agent launch
```

**Critical Path:** WP1.1 → WP1.2 → WP1.5 (sequential)
**Parallel Streams:** WP1.3 + WP1.4 can run during WP1.2

**Optimized Timeline:**
- Week 1, Days 1-2: WP1.1 (4h)
- Week 1, Days 2-3: WP1.2 (6h) + WP1.3 start (4h) + WP1.4 (4h) in parallel
- Week 1, Days 4-5: WP1.3 (8h) + WP1.4 (2h) continued
- Week 2, Days 6-10: WP1.5 (12h) + WP1.3/1.4 cleanup (4h) + Testing (4h)

---

## Team Assignment & Responsibilities

| WP | Owner | Role | Skills Required |
|----|-------|------|-----------------|
| WP1.1 | DevOps Lead | Infrastructure | Git, GitHub API, branch protection, CI/CD |
| WP1.2 | Spec Coordinator | Governance | Specifications, technical writing, automation |
| WP1.3 | Infrastructure Engineer | Services | Rust, git2-rs, GitHub Actions, tokio |
| WP1.4 | QA Engineer | Testing | Python, validation, test coverage, CI |
| WP1.5 | Spec Coordinator + Tech Writer | Training | Technical writing, training, agent workflows |

---

## Success Metrics (End of Phase 1)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Specs completeness | 95/100 | SPECS_REGISTRY.md |
| Health score | 65/100 | SSOT_HEALTH_DASHBOARD.md |
| Auto-merge success rate | 95%+ | GitHub Actions logs |
| FR↔Test coverage | 100% | validate-fr-test-coverage.py |
| Agent adoption | 80%+ | Agents using specs/* branches |
| Merge conflict rate | <5% | GitHub issues created |
| Avg merge time | <5 min | Batch results |
| Documentation completeness | 100% | All 5 docs finished |

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Merge conflicts flood system | Medium | High | Batch size limit (50), conflict queue, manual review |
| Validation too strict | High | Medium | Start warnings, escalate blocks in Phase 2 |
| Agent adoption slow | Medium | Medium | Training, examples, automation, peer support |
| CI performance issues | Low | High | Caching, parallel processing, monitoring |
| Git2-rs bugs | Low | Critical | Comprehensive testing, fallback to git CLI |

---

## Rollback Plan

If critical issues discovered:

1. **Disable auto-merge:** `gh workflow disable auto-merge-specs.yml`
2. **Revert branch protection:** Remove specs/main rules
3. **Return to manual merges:** Clear communication to team
4. **Root cause analysis:** What went wrong?
5. **Fix & re-deploy:** Address issues, test thoroughly

---

## Phase 1 Completion Criteria

All WP1.1-1.5 complete **and**:

- ✅ specs/main protected on all 3 repos
- ✅ 50+ agent branches test-merged successfully
- ✅ 100% FR↔Test coverage on all 3 repos
- ✅ Health score: 65/100 (from 42/100 baseline)
- ✅ Documentation complete & team trained
- ✅ Zero critical issues in production
- ✅ Agent workflow validated with 5+ pilot agents

---

## Next Steps: Phase 2 Preparation

After Phase 1 complete:
1. Retrospective: What worked? What needs improvement?
2. Plan Phase 2: Ecosystem consolidation, cross-repo traceability
3. Expand agent adoption: Roll out to all 50+ agents
4. Integrate with AgilePlus: Link specs → work packages → executions

---

**Document Owner:** Project Coordinator
**Created:** 2026-03-31
**Status:** Ready for Team Assignment
**Review Date:** 2026-04-11 (end of sprint)

**Assign work packages to team members and begin Week 1 on 2026-03-31.**
