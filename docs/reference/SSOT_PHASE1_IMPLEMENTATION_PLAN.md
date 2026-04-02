# Polyrepo SSOT Phase 1 — Implementation Plan
## Specs Canonicalization (2-Week Sprint)

**Status:** Ready for Execution
**Created:** 2026-03-31
**Sprint Duration:** 2 weeks (10 working days)
**Target Completion:** 2026-04-11
**Health Score Target:** 42/100 → 65/100
**Effort Total:** 80 hours (5 engineers × 2 weeks)

---

## Executive Summary

This plan establishes the **Single Source of Truth (SSOT)** for Phenotype ecosystem specifications through:
- **specs/main** — Authoritative FR/ADR registry with linear history
- **Auto-merge orchestration** — Concurrent spec branches → specs/main (5-min batches)
- **FR↔Test traceability gate** — CI validation ensuring 100% coverage
- **Agent workflow standardization** — 50+ agents commit with traceability

**Result:** Unified spec governance, zero conflicts, 100% FR test coverage, automatic merges.

---

## Week 1: Foundation & Infrastructure (40 hours)

### Day 1: Specs/Main Branch Infrastructure Setup

**Goal:** Create authoritative specs/main branch with merge strategy, commit hooks, CI validation.

#### Task 1.1: Branch Protection & Merge Strategy (4h)
**Owner:** DevOps Lead

**Deliverables:**
- [ ] Verify specs/main exists on origin with linear history (no merge commits)
- [ ] Branch protection rules:
  - Require 1 approval (from specs-admin role)
  - Require status checks: `ci-ssot-validation` passing
  - Dismiss stale PR reviews
  - Require branches up-to-date
  - Include administrators in restrictions
- [ ] Merge strategy: Squash + Rebase (linear history)
- [ ] Auto-delete head branches on merge
- [ ] Create `.github/BRANCH_PROTECTION_SPECS_MAIN.yml` documenting rules

**Git Commands:**
```bash
# Verify specs/main branch
git checkout specs/main
git log --oneline | head -5  # Should show linear history

# Protect via GitHub API
gh api repos/KooshaPari/phenotype-infrakit/branches/specs/main/protection \
  -X PUT -f required_pull_request_reviews.dismiss_stale_reviews=true
```

**Success Criteria:**
- ✅ specs/main protected on all repos (phenotype-infrakit, AgilePlus, platforms/thegent)
- ✅ Linear history verified (no merge commits)
- ✅ CI validation gate configured

---

#### Task 1.2: FR/ADR/PLAN/USER_JOURNEYS Master Registry (6h)
**Owner:** Spec Coordinator

**Deliverables:**
- [ ] Create `/SPECS_REGISTRY.md` at repo root (master index)
- [ ] Structure:
  ```markdown
  # Specs Registry — Single Source of Truth

  ## Central Registry (specs/main branch)

  ### Canonical Specs (Deployed)
  - phenotype-infrakit/FUNCTIONAL_REQUIREMENTS.md (180 lines, v2.1)
  - phenotype-infrakit/ADR.md (144 lines, v1.0)
  - phenotype-infrakit/PLAN.md (25 lines, v1.0)
  - phenotype-infrakit/USER_JOURNEYS.md (262 lines, v1.0)
  - ...

  ### Spec Versions & Approval Status
  | Repo | FR | ADR | PLAN | UJ | Status | Last Updated |
  |------|-------|------|------|-----|--------|--------------|
  | phenotype-infrakit | v2.1 | v1.0 | v1.0 | v1.0 | ✅ Deployed | 2026-03-31 |
  | AgilePlus | v2.0 | v0.9 | v1.1 | TBD | ⏳ Review | 2026-03-30 |
  | platforms/thegent | v1.9 | v2.1 | v2.0 | TBD | ⏳ Review | 2026-03-29 |

  ### Spec Synchronization Schedule
  - specs/main merges: Every 5 minutes (batched)
  - Manual reviews: Daily 10am UTC
  - Release cut: Weekly (Mondays)
  ```

- [ ] Create `/specs/REGISTRY_SCHEMA.json` — Spec metadata validation schema
  ```json
  {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "Spec Registry Entry",
    "type": "object",
    "properties": {
      "repo": { "type": "string" },
      "spec_type": { "enum": ["FR", "ADR", "PLAN", "UJ"] },
      "version": { "type": "string", "pattern": "^\\d+\\.\\d+$" },
      "lines": { "type": "integer" },
      "frs_covered": { "type": "array", "items": { "type": "string" } },
      "last_updated": { "type": "string", "format": "date-time" },
      "status": { "enum": ["draft", "review", "deployed"] }
    },
    "required": ["repo", "spec_type", "version", "status"]
  }
  ```

- [ ] Document in `/docs/reference/SPECS_REGISTRY_GUIDE.md` (how to use/maintain)

**Success Criteria:**
- ✅ SPECS_REGISTRY.md exists with all repos listed
- ✅ Version tracking accurate (cross-checked against HEAD)
- ✅ Schema validates all existing specs
- ✅ Registry updates auto-triggered on specs/main merge (via GHA)

---

#### Task 1.3: Commit Message Format & FR Traceability (4h)
**Owner:** QA Lead

**Deliverables:**
- [ ] Create `.commit-template` at repo root
  ```
  <type>(<scope>): <subject>

  <body>

  Spec-Traces: <FR-XXX, ADR-YYY, ...>
  Related-Issues: #123, #456
  Co-Authored-By: <agent-name> <agent@phenotype.local>
  ```

- [ ] Git config (auto-applied in CI):
  ```bash
  git config commit.template .commit-template
  ```

- [ ] Create pre-commit hook: `scripts/hooks/validate-spec-traces.sh`
  ```bash
  #!/bin/bash
  # Validate that commit message references at least one FR/ADR/PLAN spec

  MESSAGE="$(cat "$1")"

  if ! echo "$MESSAGE" | grep -qE "Spec-Traces:.*FR-|Spec-Traces:.*ADR-|Spec-Traces:.*PLAN-"; then
    echo "ERROR: Commit must include Spec-Traces referencing FR/ADR/PLAN"
    echo "Format: Spec-Traces: FR-XXX-NNN, ADR-YYY"
    exit 1
  fi

  exit 0
  ```

- [ ] Hook installed via pre-commit framework (see Task 1.4)
- [ ] Document in `/docs/reference/COMMIT_MESSAGE_FORMAT.md`

**Success Criteria:**
- ✅ .commit-template enforced on all agent commits
- ✅ 100% of specs/main commits have Spec-Traces
- ✅ CI validates traceability (no untraced commits merge)

---

#### Task 1.4: Pre-Push Validation & CI Gate (6h)
**Owner:** CI Engineer

**Deliverables:**
- [ ] Create `.github/workflows/ssot-validation.yml` (runs on every push to specs/*)
  ```yaml
  name: SSOT Validation Gate

  on:
    push:
      branches:
        - specs/main
        - specs/agent-*
    pull_request:
      branches:
        - specs/main

  jobs:
    validate-specs:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Validate FR/ADR/PLAN/UJ files
          run: |
            # Check all spec files exist
            [ -f FUNCTIONAL_REQUIREMENTS.md ] || exit 1
            [ -f ADR.md ] || exit 1

            # Validate markdown structure
            scripts/validate-spec-structure.py

            # Check FR coverage (no orphan FRs)
            scripts/validate-fr-coverage.py

        - name: Validate commit messages
          run: |
            # Check all commits in PR have Spec-Traces
            for commit in $(git rev-list origin/main..HEAD); do
              msg=$(git show -s --format=%B $commit)
              if ! echo "$msg" | grep -qE "Spec-Traces:"; then
                echo "Commit $commit missing Spec-Traces"
                exit 1
              fi
            done

        - name: Check for conflicts
          run: |
            # Verify no merge conflicts in specs
            if grep -r "<<<<<<< HEAD" *.md; then
              echo "Merge conflicts found in spec files"
              exit 1
            fi

        - name: Publish validation report
          if: always()
          run: |
            # Generate validation report (see Task 1.5)
            scripts/generate-ssot-report.py > /tmp/ssot-report.md
            # Comment on PR or publish as artifact
  ```

- [ ] Create validation scripts:
  - `scripts/validate-spec-structure.py` — Check markdown headers, FR format
  - `scripts/validate-fr-coverage.py` — Ensure 100% FR→Test traceability
  - `scripts/generate-ssot-report.py` — Create human-readable validation report

- [ ] Hook into pre-push (local validation before remote):
  ```bash
  #!/bin/bash
  # .git/hooks/pre-push

  # Run local validation before push
  python3 scripts/validate-spec-structure.py
  if [ $? -ne 0 ]; then
    echo "Local validation failed. Fix issues before pushing."
    exit 1
  fi
  ```

**Success Criteria:**
- ✅ All specs/agent-* PRs require CI validation
- ✅ No unvalidated commits merge to specs/main
- ✅ Validation report auto-generated on every push
- ✅ <2min validation runtime

---

### Day 2-3: Agent Commit Hooks & Traceability

#### Task 1.5: Agent Onboarding & Commit Hook Installation (8h)
**Owner:** Agent Lead

**Deliverables:**
- [ ] Create `scripts/agent-setup.sh` — One-time agent setup
  ```bash
  #!/bin/bash
  # Agent onboarding: Configure git hooks, commit template, etc.

  echo "🤖 Phenotype Agent Setup"

  # 1. Configure commit template
  git config user.name "agent-$1"  # $1 = agent name
  git config user.email "agent-$1@phenotype.local"
  git config commit.template .commit-template

  # 2. Install pre-commit hooks
  pip install pre-commit
  pre-commit install --install-hooks

  # 3. Install pre-push hooks
  cp scripts/hooks/pre-push .git/hooks/
  chmod +x .git/hooks/pre-push

  # 4. Validate setup
  echo "✅ Agent setup complete"
  git log --oneline -3
  ```

- [ ] Create `scripts/hooks/pre-push` — Validate before agent push
  - Check commit messages have Spec-Traces
  - Run local validation (no remote calls)
  - Warn if pushing >5 commits (batch size too large)

- [ ] Create `AGENT_WORKFLOW.md` (see separate document)
  - How to create feature branches: `specs/agent-<name>-<task-id>`
  - Commit message format with examples
  - How to handle conflicts
  - How to merge (auto or manual review)

- [ ] Document agent personas & FR mappings:
  ```yaml
  agents:
    - name: phenosdk-decomposer
      role: SDK decomposition & extraction
      domain: phenotype-infrakit, phenosdk
      fr_prefix: FR-PHENOSDK

    - name: agileplus-merger
      role: AgilePlus spec consolidation
      domain: AgilePlus
      fr_prefix: FR-AGILE

    - name: thegent-specs
      role: thegent multi-agent architecture
      domain: platforms/thegent
      fr_prefix: FR-THEGENT
  ```

**Success Criteria:**
- ✅ Agent setup script tested with 3+ agents
- ✅ All agents have pre-commit + pre-push hooks
- ✅ First 50 agent commits have valid Spec-Traces
- ✅ AGENT_WORKFLOW.md documented and followed

---

### Day 4: Auto-Merge Service Architecture

#### Task 1.6: Design Auto-Merge Orchestration Service (8h)
**Owner:** Infrastructure Architect

**Deliverables:**
- [ ] Create `/docs/reference/AUTO_MERGE_SERVICE_ARCHITECTURE.md`
  ```
  # Auto-Merge Service Architecture

  ## Overview
  Watch for commits to specs/agent-* branches → Validate → Auto-merge to specs/main

  ## Components
  1. **GitHub Event Listener** (GitHub Actions)
     - Trigger: Push to specs/agent-* branches
     - Action: Invoke merge orchestrator

  2. **Merge Orchestrator** (Rust microservice)
     - Input: Branch name, commit range
     - Logic: Validate, check conflicts, attempt merge
     - Output: Success or manual review request

  3. **Conflict Handler** (GitHub Actions)
     - Trigger: Merge orchestrator returns "conflict"
     - Action: Create GitHub issue for manual review

  4. **Batch Processor** (Cron job)
     - Every 5 minutes: Batch pending specs/agent-* branches
     - Attempt simultaneous merges

  ## Data Flow

  specs/agent-<name>-<task>  → GitHub Event → Merge Orchestrator
                                ↓
                         ┌──────┴──────┐
                         ↓             ↓
                   No Conflicts   Conflicts
                         ↓             ↓
                  Auto-merge    Issue Created
                         ↓             ↓
                    specs/main   Manual Review

  ## Success Criteria
  - 100% of agent commits merge within 5 min (no conflicts)
  - <1% conflict rate (issues created for review)
  - Zero manual intervention for clean merges
  ```

- [ ] Create batch merge orchestrator skeleton:
  ```rust
  // libs/phenotype-batch-merger/src/lib.rs

  use git2::Repository;
  use tokio::time::{interval, Duration};

  pub struct MergeOrchestrator {
      repo: Repository,
      batch_interval: Duration,
  }

  impl MergeOrchestrator {
      /// Watch for agent branches and auto-merge
      pub async fn run(&self) -> Result<()> {
          let mut ticker = interval(Duration::from_secs(300)); // 5 min

          loop {
              ticker.tick().await;

              // 1. List all specs/agent-* branches
              let branches = self.list_agent_branches()?;

              // 2. For each branch, attempt merge
              for branch in branches {
                  match self.attempt_merge(&branch) {
                      Ok(_) => println!("✅ Merged {}", branch),
                      Err(ConflictError) => {
                          self.create_manual_review_issue(&branch)?;
                      }
                      Err(e) => return Err(e),
                  }
              }
          }
      }

      fn attempt_merge(&self, branch: &str) -> Result<(), MergeError> {
          // Implementation in Task 1.7
          todo!()
      }
  }
  ```

- [ ] Create GitHub Actions trigger:
  ```yaml
  # .github/workflows/auto-merge-specs.yml

  name: Auto-Merge Specs Agent Branches

  on:
    schedule:
      - cron: '*/5 * * * *'  # Every 5 minutes
    push:
      branches:
        - specs/agent-*

  jobs:
    batch-merge:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
          with:
            fetch-depth: 0

        - name: Run merge orchestrator
          run: |
            cargo run --release --bin batch-merger \
              --repo-path . \
              --target-branch specs/main
  ```

**Success Criteria:**
- ✅ Merge orchestrator design documented
- ✅ Batch processing strategy clear (5-min windows)
- ✅ Conflict handling strategy defined (GitHub issues)
- ✅ GitHub Actions trigger configured

---

### Day 5: Validation Gate & Traceability

#### Task 1.7: FR↔Test Traceability CI Gate (8h)
**Owner:** QA Engineer

**Deliverables:**
- [ ] Create `scripts/validate-fr-test-coverage.py`
  ```python
  #!/usr/bin/env python3
  """
  Validate FR↔Test traceability: Every FR has ≥1 test, every test traces to ≥1 FR
  """

  import re
  import sys
  from pathlib import Path

  def extract_frs(md_file: Path) -> set[str]:
      """Extract all FR-XXX-NNN references from markdown"""
      content = md_file.read_text()
      return set(re.findall(r'FR-[A-Z]+-\d{3}', content))

  def extract_test_traces(test_dir: Path) -> dict[str, set[str]]:
      """Extract Spec-Traces from all test files"""
      traces = {}
      for test_file in test_dir.rglob('*.rs'):
          content = test_file.read_text()
          # Pattern: // Traces to: FR-XXX-NNN
          frs = set(re.findall(r'Traces to:.*?(FR-[A-Z]+-\d{3})', content))
          if frs:
              traces[str(test_file)] = frs
      return traces

  def validate(repo_root: Path) -> bool:
      """Validate 100% FR↔Test coverage"""

      # 1. Extract FRs from FUNCTIONAL_REQUIREMENTS.md
      fr_file = repo_root / "FUNCTIONAL_REQUIREMENTS.md"
      if not fr_file.exists():
          print("❌ FUNCTIONAL_REQUIREMENTS.md not found")
          return False

      frs = extract_frs(fr_file)
      print(f"📋 Found {len(frs)} FRs: {sorted(frs)}")

      # 2. Extract test traces
      test_dir = repo_root / "tests"
      test_traces = extract_test_traces(test_dir)
      print(f"🧪 Found {len(test_traces)} test files with traces")

      # 3. Check coverage: every FR traced, every test traces
      traced_frs = set()
      for test_file, frs_in_test in test_traces.items():
          for fr in frs_in_test:
              if fr not in frs:
                  print(f"❌ Test {test_file} traces to unknown FR {fr}")
                  return False
              traced_frs.add(fr)

      # 4. Check for orphan FRs (no tests)
      orphans = frs - traced_frs
      if orphans:
          print(f"❌ Orphan FRs (no tests): {sorted(orphans)}")
          return False

      print("✅ 100% FR↔Test coverage verified")
      return True

  if __name__ == "__main__":
      repo_root = Path.cwd()
      success = validate(repo_root)
      sys.exit(0 if success else 1)
  ```

- [ ] Integrate into CI workflow:
  ```yaml
  - name: Validate FR↔Test coverage
    run: python3 scripts/validate-fr-test-coverage.py
  ```

- [ ] Create traceability matrix generator:
  ```python
  # scripts/generate-traceability-matrix.py
  # Output: docs/TRACEABILITY_MATRIX.md
  # Table: FR | Test File | Test Count | Status
  ```

- [ ] Document in `/docs/reference/FR_TEST_TRACEABILITY_GUIDE.md`
  - How to add "Traces to: FR-XXX-NNN" comment to tests
  - Examples for Rust, Python, Go, TypeScript
  - Validation workflow

**Success Criteria:**
- ✅ All FRs in FUNCTIONAL_REQUIREMENTS.md have ≥1 test
- ✅ All tests in test suite trace to ≥1 FR
- ✅ Traceability matrix auto-generated weekly
- ✅ CI blocks merge if coverage <100%

---

## Week 2: Agent Workflow & Operational Setup (40 hours)

### Day 6: Auto-Merge Implementation & Testing

#### Task 2.1: Implement Batch Merge Orchestrator (10h)
**Owner:** Infrastructure Engineer

**Deliverables:**
- [ ] Complete implementation of `libs/phenotype-batch-merger/src/lib.rs`:
  ```rust
  pub async fn attempt_merge(
      &self,
      repo: &Repository,
      branch: &str,
  ) -> Result<MergeResult, MergeError> {
      // 1. Fetch latest from origin
      repo.find_remote("origin")?
          .fetch(&[branch], None, None)?;

      // 2. Get branch tip commit
      let branch_ref = repo.find_reference(branch)?;
      let branch_oid = branch_ref.target().ok_or(MergeError::NoTarget)?;

      // 3. Check for conflicts with target
      let target_oid = repo.find_reference("refs/heads/specs/main")?
          .target()
          .ok_or(MergeError::NoTarget)?;

      let base_oid = self.find_merge_base(&repo, branch_oid, target_oid)?;

      // 4. Attempt merge (no-commit)
      let mut index = repo.merge_trees(
          &repo.find_tree(base_oid)?,
          &repo.find_tree(branch_oid)?,
          &repo.find_tree(target_oid)?,
          None,
      )?;

      if index.has_conflicts() {
          return Err(MergeError::Conflict(self.extract_conflict_details(&index)));
      }

      // 5. Create merge commit
      let sig = repo.signature()?;
      let message = format!("Merge {} into specs/main\n\nAuto-merged by phenotype-batch-merger", branch);

      index.write_tree()?;
      // ... complete merge

      Ok(MergeResult::Success { branch, commit_hash: "..." })
  }
  ```

- [ ] Add error handling for:
  - Network failures (retry up to 3×)
  - Merge conflicts (create GitHub issue)
  - Invalid branch names
  - Authentication errors

- [ ] Create GitHub integration:
  ```rust
  pub struct GitHubIssueCreator {
      client: reqwest::Client,
      token: String,
      repo: String,
  }

  impl GitHubIssueCreator {
      pub async fn create_merge_conflict_issue(
          &self,
          branch: &str,
          conflict_details: &str,
      ) -> Result<String> {
          // Create issue with conflict details, ask for manual review
          todo!()
      }
  }
  ```

- [ ] Test suite:
  ```rust
  #[tokio::test]
  async fn test_merge_no_conflicts() {
      // Create test repo, branch, and test merge
  }

  #[tokio::test]
  async fn test_merge_with_conflicts() {
      // Create conflicting branch, verify error handling
  }

  #[tokio::test]
  async fn test_batch_merge_multiple_branches() {
      // Test simultaneous merges of 5+ branches
  }
  ```

**Success Criteria:**
- ✅ Orchestrator builds cleanly (no warnings)
- ✅ All 3 test cases pass
- ✅ Orchestrator handles 100+ concurrent merge attempts
- ✅ Error recovery automatic (retry on transient failures)

---

#### Task 2.2: Deploy GitHub Actions Workflows (6h)
**Owner:** DevOps Engineer

**Deliverables:**
- [ ] Deploy `.github/workflows/auto-merge-specs.yml` to all repos
  - Schedule: Every 5 minutes
  - Trigger: Push to specs/agent-*
  - Action: Run batch-merger binary

- [ ] Deploy `.github/workflows/ssot-validation.yml` (from Task 1.4)
  - Trigger: Push to specs/* branches
  - Action: Run validation scripts

- [ ] Deploy `.github/workflows/fr-test-coverage.yml`
  - Trigger: Pull request to specs/main
  - Action: Run FR↔Test coverage validation
  - Block merge if coverage <100%

- [ ] Create GitHub Actions secrets:
  ```bash
  gh secret set GH_BOT_TOKEN -b "$(gh auth token)"
  ```

- [ ] Test workflows:
  ```bash
  # Trigger manually
  gh workflow run auto-merge-specs.yml

  # Monitor
  gh run list --workflow auto-merge-specs.yml
  ```

**Success Criteria:**
- ✅ All 3 workflows deploy successfully
- ✅ Workflows run automatically on schedule
- ✅ Manual trigger works (for testing)
- ✅ Logs readable and actionable

---

### Day 7-8: Agent Onboarding & Documentation

#### Task 2.3: Agent Workflow Standardization (8h)
**Owner:** Spec Coordinator (see separate document)

**Deliverables:**
- [ ] Create `/docs/reference/SSOT_PHASE1_AGENT_WORKFLOW.md`
  - Agent branching pattern
  - Commit message format with examples
  - Pre-push validation checklist
  - Automatic merge triggers
  - PR template for spec reconciliation
  - (Full document separate)

- [ ] Create GitHub PR template for agent branches:
  ```markdown
  # Agent Spec Reconciliation PR

  **Agent Name:** [e.g., phenosdk-decomposer]
  **Task ID:** [e.g., FR-PHENOSDK-001]
  **Branch:** specs/agent-<name>-<task-id>

  ## Changes
  - [ ] FUNCTIONAL_REQUIREMENTS.md updated
  - [ ] ADR.md updated (if applicable)
  - [ ] Tests added/updated
  - [ ] All commits traced to FR

  ## Validation Checklist
  - [ ] Pre-push validation passed locally
  - [ ] CI validation passed (specs-validation)
  - [ ] FR↔Test coverage 100%
  - [ ] No conflicts with specs/main

  ## Approval
  This PR will auto-merge to specs/main if:
  - [ ] CI validation passes
  - [ ] No merge conflicts
  - [ ] 100% FR↔Test coverage
  ```

- [ ] Update AGENT_WORKFLOW.md with examples:
  ```bash
  # Example: Agent creates spec branch
  git checkout -b specs/agent-phenosdk-decomposer-fr001

  # Edit specs
  echo "## FR-PHENOSDK-001: Decompose SDK Core" >> FUNCTIONAL_REQUIREMENTS.md

  # Commit with trace
  git commit -am "specs: add FR-PHENOSDK-001

  Spec-Traces: FR-PHENOSDK-001
  Co-Authored-By: phenosdk-decomposer <agent@phenotype.local>"

  # Pre-push validation
  git push origin specs/agent-phenosdk-decomposer-fr001

  # → CI validation runs automatically
  # → If clean: Auto-merged to specs/main within 5 min
  # → If conflict: Manual review issue created
  ```

**Success Criteria:**
- ✅ Agent workflow documented with 5+ examples
- ✅ All agents onboarded and tested
- ✅ First 10 spec branches merge successfully
- ✅ Zero manual intervention needed for clean merges

---

#### Task 2.4: Spec Documentation & Training (6h)
**Owner:** Tech Writer

**Deliverables:**
- [ ] Create training guide: `/docs/guides/SSOT_PHASE1_QUICKSTART.md`
  - 5-minute overview of specs/main architecture
  - Common workflows (create branch, commit, merge)
  - Troubleshooting (conflicts, failed validation)
  - FAQ

- [ ] Create spec format guide: `/docs/reference/SPEC_FORMAT_STANDARDS.md`
  - FUNCTIONAL_REQUIREMENTS.md structure (FR-XXX-NNN headers)
  - ADR.md format (decision, rationale, consequences)
  - PLAN.md layout (phases, tasks, dependencies)
  - USER_JOURNEYS.md pattern (actor, flow, metrics)
  - Examples from 3+ canonical repos

- [ ] Create video walkthrough (optional):
  - Record: Creating spec branch → Commit → Merge
  - Duration: 5-10 minutes
  - Upload to team wiki

- [ ] Update CLAUDE.md with SSOT references:
  ```markdown
  ## SSOT Phase 1 Governance

  All specs live on `specs/main` branch:
  - Push to `specs/agent-<name>-<task>` branches only
  - Commit format: Include `Spec-Traces: FR-XXX-NNN`
  - Auto-merge: Clean branches merge within 5 minutes
  - Manual review: Conflicts trigger GitHub issue

  See: docs/reference/SSOT_PHASE1_AGENT_WORKFLOW.md
  ```

**Success Criteria:**
- ✅ Quickstart guide <500 words, <5 min read
- ✅ All 4 spec types have format examples
- ✅ Video walkthrough recorded (optional)
- ✅ Team trained and confident

---

### Day 9: Operational Monitoring & Metrics

#### Task 2.5: Deploy SSOT Monitoring & Health Dashboard (8h)
**Owner:** Platform Engineer

**Deliverables:**
- [ ] Create `.github/workflows/ssot-health-check.yml`
  - Runs daily at 9am UTC
  - Checks:
    - specs/main branch clean (no unmerged commits >24h)
    - All agent branches either merged or with active issues
    - FR↔Test coverage 100%
    - Spec versions up-to-date
  - Output: Health score (42/100 → 65/100 target)

- [ ] Create health score calculation:
  ```python
  # scripts/calculate-ssot-health.py

  def health_score():
      score = 0

      # 1. Specs completeness (25 points)
      # phenotype-infrakit: all 4 specs ✅ (10 pts)
      # AgilePlus: all 4 specs ✅ (10 pts)
      # platforms/thegent: all 4 specs ✅ (5 pts)
      score += 25

      # 2. Traceability (20 points)
      # FR↔Test: 100% coverage → 20 pts
      score += 20 * (test_coverage / 100)

      # 3. Merge health (15 points)
      # Auto-merge success rate: 100% → 15 pts
      score += 15 * (merge_success_rate / 100)

      # 4. Agent adoption (20 points)
      # Agents using specs/main: 80%+ → 20 pts
      num_agents = 50
      adopting = count_agents_using_ssot()
      score += 20 * (adopting / num_agents)

      # 5. Documentation (20 points)
      # All workflows documented, examples present
      score += check_documentation_completeness() * 20

      return score
  ```

- [ ] Create dashboard: `docs/SSOT_HEALTH_DASHBOARD.md`
  - Auto-updated daily
  - Shows:
    - Current health score (target: 65/100)
    - Specs completeness by repo
    - FR↔Test coverage %
    - Merge success rate %
    - Active agent branches
    - Recent issues/conflicts
  - Example:
    ```
    # SSOT Health Dashboard (Updated 2026-03-31 09:00 UTC)

    ## Overall Score: 42/100 → Target: 65/100

    ### Specs Completeness
    | Repo | FR | ADR | PLAN | UJ | Score |
    |------|----|----|------|-----|-------|
    | phenotype-infrakit | ✅ | ✅ | ✅ | ✅ | 25/25 |
    | AgilePlus | ✅ | ✅ | ✅ | TBD | 18/25 |
    | platforms/thegent | ✅ | ✅ | ✅ | TBD | 18/25 |

    ### FR↔Test Coverage: 92% (target: 100%)

    ### Merge Health (Last 7 days)
    - Auto-merges: 124 success, 3 conflicts (97.6% success)
    - Avg merge time: 2.3 min (target: 5 min)

    ### Active Agent Branches
    - specs/agent-phenosdk-decomposer-fr001: ⏳ In merge (2 min)
    - specs/agent-agileplus-merger-wp13: ✅ Merged
    ```

- [ ] Create alerting:
  ```yaml
  # .github/workflows/ssot-alerts.yml

  on:
    schedule:
      - cron: '0 9 * * *'  # Daily at 9am

  jobs:
    check-health:
      runs-on: ubuntu-latest
      steps:
        - name: Calculate health score
          run: python3 scripts/calculate-ssot-health.py > /tmp/score.txt

        - name: Alert if score dropping
          run: |
            CURRENT=$(cat /tmp/score.txt | grep "Score:" | awk '{print $2}')
            if [ $CURRENT -lt 50 ]; then
              # Create issue or Slack alert
              gh issue create \
                --title "⚠️ SSOT Health Score Below 50: $CURRENT/100" \
                --body "See SSOT_HEALTH_DASHBOARD.md for details"
            fi
  ```

**Success Criteria:**
- ✅ Health dashboard auto-updates daily
- ✅ Health score currently 42/100 (baseline)
- ✅ Projected 65/100 after Phase 1 complete
- ✅ Alerts fire when score drops below 50

---

### Day 10: Deployment & Final Verification

#### Task 2.6: Deployment to All Repos & Final QA (10h)
**Owner:** Release Manager

**Deliverables:**
- [ ] Deployment checklist:
  ```markdown
  # SSOT Phase 1 Deployment Checklist

  ## Pre-Deployment (Day 1)
  - [ ] All code merged to main
  - [ ] All workflows tested in staging
  - [ ] Documentation reviewed and finalized
  - [ ] Team trained on new workflows

  ## Deployment (Day 10)

  ### phenotype-infrakit
  - [ ] Copy .commit-template, .pre-commit-config.yaml
  - [ ] Copy scripts/ (validation, hooks, orchestrator)
  - [ ] Deploy .github/workflows/ (ssot-validation, auto-merge-specs, fr-test-coverage)
  - [ ] Set up branch protection on specs/main
  - [ ] Test: Create specs/agent-test-* branch, verify auto-merge

  ### AgilePlus
  - [ ] (Same as above)

  ### platforms/thegent
  - [ ] (Same as above)

  ### Verify Deployment
  - [ ] Health score: 42/100 → 65/100 target
  - [ ] All 3 repos specs/main protected
  - [ ] All workflows running on schedule
  - [ ] No merge conflicts in past 24h
  - [ ] Documentation accessible

  ## Post-Deployment (Week 3)
  - [ ] Team retrospective on SSOT Phase 1
  - [ ] Update health score targets (Phase 2)
  - [ ] Plan Phase 2 (ecosystem consolidation)
  ```

- [ ] Create rollback plan (in case of issues):
  - Disable auto-merge workflows
  - Revert branch protection rules
  - Return to manual merge process
  - Communicate to team
  - Root cause analysis

- [ ] Create success metrics report:
  ```markdown
  # SSOT Phase 1 Success Metrics (Target)

  | Metric | Baseline | Target | Status |
  |--------|----------|--------|--------|
  | Specs completeness | 78/100 | 95/100 | 🎯 |
  | Health score | 42/100 | 65/100 | 🎯 |
  | Auto-merge success rate | N/A | 95%+ | 🎯 |
  | FR↔Test coverage | 85% | 100% | 🎯 |
  | Agent adoption | 0% | 80%+ | 🎯 |
  | Merge conflict rate | N/A | <5% | 🎯 |
  | Avg merge time | N/A | <5 min | 🎯 |
  | Documentation completeness | 0% | 100% | 🎯 |
  ```

**Success Criteria:**
- ✅ All 3 repos deployed and operational
- ✅ Health score reaches 65/100
- ✅ No critical issues post-deployment
- ✅ Team confident in new workflows

---

## Critical Path Analysis

**Longest dependency chain:**
1. Task 1.1 (Branch protection setup) — 4h
2. Task 1.2 (Registry structure) — 6h
3. Task 1.3 (Commit messages) — 4h
4. Task 1.4 (CI validation) — 6h
5. Task 1.5 (Agent hooks) — 8h
6. Task 2.1 (Merge orchestrator) — 10h
7. Task 2.6 (Final deployment) — 10h

**Total critical path:** 48 hours (sequential)
**Parallelizable:** Tasks 1.4, 1.5, 1.6 (can run during 1.2-1.3)
**Optimized timeline:** 40 hours (5 parallel streams)

---

## Risk Mitigation Strategies

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Merge conflicts flood system | Medium | High | Implement conflict batching; limit concurrent branches to 10 |
| Validation too strict | High | Medium | Start with warnings, escalate to blocks in Phase 2 |
| Agent adoption slow | Medium | Medium | Provide examples, templates, training; automate setup |
| Auto-merge fails silently | Low | Critical | Comprehensive logging; alert on every failure |
| Spec drift from code | High | Medium | Enforce FR↔Test validation; block merges if coverage <100% |
| CI performance degrades | Medium | High | Cache validation results; batch processing every 5 min |

---

## Success Criteria (End of Phase 1)

- ✅ **Infrastructure:** specs/main protected, auto-merge deployed to all 3 repos
- ✅ **Process:** 50+ agents using specs/* branches with zero manual merge intervention
- ✅ **Quality:** FR↔Test coverage 100%, zero orphan FRs
- ✅ **Metrics:** Health score 42/100 → 65/100
- ✅ **Documentation:** SSOT_PHASE1_AGENT_WORKFLOW.md complete, team trained
- ✅ **Monitoring:** Health dashboard deployed, daily checks automated

---

## Next Steps: Phase 2

After Phase 1 completes:
1. **Expand specs completeness** (95/100 → 98/100)
2. **Cross-repo traceability** (link phenotype-infrakit → heliosCLI → thegent)
3. **Evidence bundle integration** (tests, Playwright, CI/CD links in specs)
4. **Agent autonomy** (auto-create specs from ADR, auto-run tests)
5. **Enterprise readiness** (audit logging, RBAC, SLA tracking)

---

**Document Owner:** Platform Architect
**Created:** 2026-03-31
**Status:** Ready for Execution
**Next Review:** 2026-04-11 (end of sprint)

**🚀 Deploy Phase 1 to reach 65/100 health score within 2 weeks.**
