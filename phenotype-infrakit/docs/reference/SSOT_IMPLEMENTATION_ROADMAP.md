# SSOT Implementation Roadmap

**Document Type**: Work Breakdown Structure (WBS) with Tasks
**Phase Scope**: Phase 1 (Specs Canonicalization) — Weeks 1-2
**Status**: Ready for execution

---

## Phase 1 Breakdown (Weeks 1-2)

### WP1: Git Infrastructure Setup (Days 1-2)

#### Task 1.1: Resolve Git Conflict Markers

**Owner**: governance-team (1 agent)
**Status**: Blocked (CLAUDE.md and AGENTS.md have merge conflict markers)
**Effort**: 1-2 hours

```bash
# Identify conflicted files
git diff --name-only --diff-filter=U

# Expected:
#   CLAUDE.md
#   AGENTS.md
#   worklog.md

# Resolution approach:
#   1. Read both sides of conflict (git show :1 CLAUDE.md, :2, :3)
#   2. Determine canonical version (prefer origin/main)
#   3. Keep canonical, discard local if diverged
#   4. Commit resolution with message:
#      "fix: resolve git merge conflict in governance files"
```

**Acceptance Criteria**:
- [ ] Zero `<<<<<<<` markers in CLAUDE.md
- [ ] Zero `<<<<<<<` markers in AGENTS.md
- [ ] Zero `<<<<<<<` markers in worklog.md
- [ ] `git status` shows clean working tree
- [ ] All three files parse correctly (no syntax errors)

**Deliverable**: Clean git history; no conflict markers

---

#### Task 1.2: Create `specs/main` Branch

**Owner**: ci-infrastructure (1 agent)
**Status**: Pending
**Effort**: 30 minutes

```bash
# Option A: Branch from current main
git checkout -b specs/main main
git push origin specs/main

# Option B: Branch from origin/main (if local is diverged)
git fetch origin
git checkout -b specs/main origin/main
git push -u origin specs/main
```

**Configuration** (GitHub):
```bash
# Protect specs/main from force-push
gh api repos/KooshaPari/phenotype-infrakit/branches/specs/main/protection \
  --input - << EOF
{
  "required_status_checks": null,
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismissal_restrictions": {},
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_linear_history": true
}
EOF
```

**Acceptance Criteria**:
- [ ] `origin/specs/main` branch exists and is visible
- [ ] Branch protection enabled (no force-push allowed)
- [ ] Branch is synced with current main
- [ ] Agents can pull from remote: `git fetch origin specs/main`

**Deliverable**: specs/main branch live on GitHub

---

#### Task 1.3: Configure CI Checks for specs/main

**Owner**: ci-infrastructure (1 agent)
**Status**: Pending
**Effort**: 1 hour

**Add to `.github/workflows/specs-validation.yml`**:

```yaml
name: Specs Validation

on:
  pull_request:
    paths:
      - 'FUNCTIONAL_REQUIREMENTS.md'
      - 'docs/reference/SPECS_REGISTRY.md'
  push:
    branches:
      - specs/main

jobs:
  validate-specs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate specs format
        run: scripts/validate-specs-format.py
      - name: Check for duplicate FR-IDs
        run: scripts/check-fr-id-collisions.py
      - name: Validate FR↔Test traceability
        run: scripts/validate-fr-test-mapping.sh

  lint-specs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Lint specs markdown
        run: vale docs/reference/SPECS_REGISTRY.md
```

**Scripts to create** (Phase 1.2):
- `scripts/validate-specs-format.py` — Validates YAML/Markdown structure
- `scripts/check-fr-id-collisions.py` — Detects duplicate FR-IDs
- `scripts/validate-fr-test-mapping.sh` — Ensures 100% FR↔Test coverage

**Acceptance Criteria**:
- [ ] Workflow runs on PR to specs/main
- [ ] All validation checks pass on current main
- [ ] Validation script can be run locally: `scripts/validate-specs-format.py`
- [ ] CI results displayed in PR checks

**Deliverable**: Automated validation for specs

---

### WP2: Specs Registry Creation (Days 2-3)

#### Task 2.1: Index All Current Specs

**Owner**: spec-indexer (1-2 agents, parallel)
**Status**: Pending
**Effort**: 4 hours

**Scope**: Extract all 26+ current specs from:
- `FUNCTIONAL_REQUIREMENTS.md` (root)
- `docs/reference/*.md` (scattered)
- `ADR.md` (architecture decisions)
- Project-level `PLAN.md` files
- AgilePlus `.agileplus/specs/` directory

**Process**:
```bash
# 1. Clone spec registry template
cat > docs/reference/SPECS_REGISTRY.md << 'EOF'
# Phenotype Specs Registry

**Last Updated**: 2026-03-30T22:00:00Z
**Version**: 1.0.0

## Overview

This is the authoritative registry of all Functional Requirements, Architecture
Decision Records, Plans, and User Journeys in the Phenotype ecosystem.

## Index

| ID | Title | Status | Version | Author | Created |
|----|-------|--------|---------|--------|---------|

## Specifications

EOF

# 2. For each spec:
#    - Extract ID (FR-XXX-YYY or ADR-NNN)
#    - Extract title
#    - Extract status (PROPOSED, APPROVED, IMPLEMENTED, DEPRECATED)
#    - Extract author (agent name or user)
#    - Extract creation date (ISO 8601)
#    - Extract test references
#    - Append to SPECS_REGISTRY.md

# 3. Validate:
#    - All FR-IDs unique
#    - All status values valid
#    - All dates parseable
#    - All test references exist (if provided)
```

**Output Format** (example):
```yaml
FR-001-001:
  title: "Event sourcing with SHA-256 hash chains"
  status: IMPLEMENTED
  version: "1.0.0"
  author_agent: "phase2-team"
  created: "2026-03-15T14:30:00Z"
  test_refs:
    - crates/phenotype-event-sourcing/tests/event_store_test.rs
  body: |
    # Event Sourcing with SHA-256 Hash Chains

    All events stored in append-only ledger with SHA-256 integrity chains...
```

**Acceptance Criteria**:
- [ ] SPECS_REGISTRY.md contains 26+ specs
- [ ] All FR-IDs are unique
- [ ] All status values are valid (PROPOSED | APPROVED | IMPLEMENTED | DEPRECATED)
- [ ] All dates are ISO 8601 format
- [ ] All test references have been verified to exist
- [ ] Registry is sorted by ID

**Deliverable**: SPECS_REGISTRY.md with all 26+ specs indexed

---

#### Task 2.2: Backfill Missing Specs

**Owner**: spec-analyst (1 agent)
**Status**: Pending (after Task 2.1)
**Effort**: 2 hours

**Scope**: Find "hidden" specs not yet in FUNCTIONAL_REQUIREMENTS.md

**Search strategy**:
```bash
# 1. Find all test functions with "FR" comments
grep -r "Traces to: FR-" crates/ packages/ | wc -l

# 2. Compare to SPECS_REGISTRY.md
grep "^FR-" docs/reference/SPECS_REGISTRY.md | wc -l

# 3. If counts don't match:
#    - Find tests referencing FR-IDs not in registry
#    - Create stub specs for missing FRs
#    - Status: PROPOSED (to be filled in later)

# Example:
#   Test says: // Traces to: FR-042-003
#   Registry doesn't have FR-042-003
#   → Add stub: FR-042-003 (status: PROPOSED, body: "TBD")
```

**Acceptance Criteria**:
- [ ] Count of FR-IDs in tests == count in SPECS_REGISTRY.md
- [ ] No orphaned test → FR references (100% coverage)
- [ ] All newly added specs have status PROPOSED (to be reviewed)

**Deliverable**: SPECS_REGISTRY.md with 100% FR↔Test coverage

---

### WP3: Spec Merge Service Deployment (Days 3-4)

#### Task 3.1: Build Spec Reconciliation Service

**Owner**: services-team (1-2 agents)
**Status**: Pending
**Effort**: 6 hours

**Deliverable**: `scripts/spec-reconciliation-service.py`

**Algorithm**:

```python
#!/usr/bin/env python3

import json
import yaml
import re
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple

class SpecReconciliationService:
    """Merges concurrent spec contributions without conflicts."""

    def __init__(self, repo_root: Path = Path(".")):
        self.repo_root = repo_root
        self.registry_path = repo_root / "docs/reference/SPECS_REGISTRY.md"
        self.audit_log_path = repo_root / "AUDIT_LOG.md"

    def detect_spec_changes(self, branch_a: str, branch_b: str) -> Tuple[List[str], List[str]]:
        """
        Detect which specs were modified on each branch.
        Returns: (specs_in_a, specs_in_b)
        """
        # Use git diff to find new FRs in FUNCTIONAL_REQUIREMENTS.md
        # Pattern: "## FR-XXX-YYY: Title"
        fr_pattern = r'^## (FR-\d+-\d+):'

        specs_a = self._extract_specs_from_branch(branch_a, fr_pattern)
        specs_b = self._extract_specs_from_branch(branch_b, fr_pattern)

        return specs_a, specs_b

    def check_collisions(self, specs_a: List[str], specs_b: List[str]) -> List[str]:
        """
        Check for FR-ID collisions (same FR defined on both branches).
        Returns: List of colliding IDs
        """
        return list(set(specs_a) & set(specs_b))

    def resolve_collisions(self, collisions: List[str]) -> Dict[str, str]:
        """
        Resolve collisions by reassigning IDs to one agent.
        Returns: Mapping of old_id → new_id
        """
        reassignments = {}
        next_id = self._find_next_fr_id()

        for collision_id in collisions:
            reassignments[collision_id] = next_id
            next_id = self._increment_fr_id(next_id)

        return reassignments

    def merge_specs(self, branch_a: str, branch_b: str, agent_a: str, agent_b: str) -> bool:
        """
        Merge two branches' spec contributions.
        Returns: True if successful, False if conflicts cannot be resolved.
        """
        specs_a, specs_b = self.detect_spec_changes(branch_a, branch_b)
        collisions = self.check_collisions(specs_a, specs_b)

        if collisions:
            reassignments = self.resolve_collisions(collisions)
            self._update_branch_fr_ids(branch_b, reassignments)
            self._log_collision_resolution(agent_a, agent_b, collisions, reassignments)
            print(f"Reassigned {len(collisions)} colliding FRs for agent {agent_b}")

        # Merge specs into registry
        self._append_to_registry(branch_a, agent_a)
        self._append_to_registry(branch_b, agent_b)

        return True

    def _extract_specs_from_branch(self, branch: str, pattern: str) -> List[str]:
        """Extract spec IDs from a git branch."""
        # Implementation using GitPython
        pass

    def _find_next_fr_id(self) -> str:
        """Find next available FR-ID in sequence."""
        # Parse existing IDs, return next in sequence (e.g., FR-001-046)
        pass

    def _increment_fr_id(self, fr_id: str) -> str:
        """Increment an FR-ID to the next sequence number."""
        # e.g., FR-001-045 → FR-001-046
        pass

    def _update_branch_fr_ids(self, branch: str, reassignments: Dict[str, str]) -> None:
        """Update FR-IDs in a branch to avoid collisions."""
        # Find and replace old IDs with new IDs in FUNCTIONAL_REQUIREMENTS.md
        pass

    def _append_to_registry(self, branch: str, agent: str) -> None:
        """Append specs from a branch to SPECS_REGISTRY.md."""
        # Extract specs from branch
        # Add to registry with metadata (author, timestamp)
        # Commit to specs/main
        pass

    def _log_collision_resolution(self, agent_a: str, agent_b: str,
                                  collisions: List[str],
                                  reassignments: Dict[str, str]) -> None:
        """Log collision resolution to AUDIT_LOG.md."""
        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "type": "spec_collision_resolved",
            "agent_a": agent_a,
            "agent_b": agent_b,
            "collisions": collisions,
            "reassignments": reassignments,
        }
        # Append to AUDIT_LOG.md
        pass


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 4:
        print("Usage: spec-reconciliation-service.py <branch-a> <branch-b> <agent-b>")
        sys.exit(1)

    branch_a, branch_b, agent_b = sys.argv[1:]
    service = SpecReconciliationService()
    success = service.merge_specs(branch_a, branch_b, "origin", agent_b)
    sys.exit(0 if success else 1)
```

**Acceptance Criteria**:
- [ ] Script detects concurrent spec changes
- [ ] Script identifies FR-ID collisions
- [ ] Script auto-reassigns colliding IDs
- [ ] Script merges specs into SPECS_REGISTRY.md
- [ ] Script logs all decisions to AUDIT_LOG.md
- [ ] Script exits 0 on success, 1 on failure
- [ ] Can be run locally: `python scripts/spec-reconciliation-service.py feat/branch1 feat/branch2 agent-name`

**Deliverable**: Executable service script

---

#### Task 3.2: Integrate Service with CI

**Owner**: ci-infrastructure (1 agent)
**Status**: Pending (after Task 3.1)
**Effort**: 2 hours

**Add to `.github/workflows/specs-merge.yml`**:

```yaml
name: Specs Merge (Phase 1)

on:
  pull_request:
    paths:
      - 'FUNCTIONAL_REQUIREMENTS.md'

jobs:
  reconcile-specs:
    runs-on: ubuntu-latest
    if: github.event.action == 'synchronize' || github.event.action == 'opened'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for merge analysis

      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Run spec reconciliation
        run: |
          python scripts/spec-reconciliation-service.py \
            origin/main \
            ${{ github.event.pull_request.head.ref }} \
            ${{ github.event.pull_request.user.login }}

      - name: Upload AUDIT_LOG changes
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: audit-log
          path: AUDIT_LOG.md
```

**Acceptance Criteria**:
- [ ] Workflow triggers on FUNCTIONAL_REQUIREMENTS.md changes
- [ ] Service runs automatically (no manual intervention)
- [ ] Collision detection works (test with synthetic PR)
- [ ] Results logged to AUDIT_LOG.md
- [ ] Agent receives notification of reassignments

**Deliverable**: CI integration for spec merge service

---

### WP4: FR↔Test Traceability Gate (Day 5)

#### Task 4.1: Build Traceability Validation Script

**Owner**: qa-team (1 agent)
**Status**: Pending
**Effort**: 3 hours

**Deliverable**: `scripts/validate-fr-test-mapping.sh`

```bash
#!/bin/bash
# Validates that all FRs have tests and all tests reference FRs

set -e

SPECS_FILE="docs/reference/SPECS_REGISTRY.md"
TEST_PATTERN="// Traces to: FR-"
ERRORS=0

# Extract all FR-IDs from SPECS_REGISTRY.md
echo "Extracting FR-IDs from $SPECS_FILE..."
FRS=$(grep "^FR-" "$SPECS_FILE" | sed 's/:.*//' | sort -u)

# For each FR, check if tests exist
echo "Validating FR↔Test mapping..."
for FR in $FRS; do
  # Count tests that reference this FR
  TEST_COUNT=$(grep -r "Traces to: $FR" crates/ packages/ 2>/dev/null | wc -l)

  if [ "$TEST_COUNT" -eq 0 ]; then
    echo "ERROR: FR-$FR has no tests"
    ((ERRORS++))
  else
    echo "✓ FR-$FR has $TEST_COUNT test(s)"
  fi
done

# Check for orphaned tests (reference non-existent FR)
echo "Checking for orphaned test references..."
ORPHANED=$(grep -r "Traces to: FR-" crates/ packages/ 2>/dev/null | \
  sed 's/.*Traces to: //' | sort -u | \
  while read FR; do
    if ! grep -q "^$FR:" "$SPECS_FILE"; then
      echo "ERROR: Test references non-existent FR: $FR"
      ((ERRORS++))
    fi
  done)

if [ "$ERRORS" -gt 0 ]; then
  echo "❌ FR↔Test traceability failed with $ERRORS errors"
  echo "Create the missing specs in FUNCTIONAL_REQUIREMENTS.md"
  exit 1
else
  echo "✅ FR↔Test traceability 100%"
  exit 0
fi
```

**Acceptance Criteria**:
- [ ] Script finds all FR-IDs in SPECS_REGISTRY.md
- [ ] Script finds all test references to FRs
- [ ] Script reports missing tests (FR → no test)
- [ ] Script reports orphaned tests (test → no FR)
- [ ] Script exits 0 when 100% coverage achieved
- [ ] Script exits 1 with error details when gaps found
- [ ] Can be run locally: `scripts/validate-fr-test-mapping.sh`

**Deliverable**: Validation script

---

#### Task 4.2: Enforce in CI

**Owner**: ci-infrastructure (1 agent)
**Status**: Pending (after Task 4.1)
**Effort**: 1 hour

**Add to `.github/workflows/specs-validation.yml`**:

```yaml
  validate-fr-test-mapping:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate FR↔Test traceability
        run: scripts/validate-fr-test-mapping.sh
      - name: Report coverage
        if: always()
        run: |
          echo "::notice::FR↔Test coverage report added to PR"
```

**Acceptance Criteria**:
- [ ] Gate blocks merges with <100% traceability
- [ ] Gate provides clear error message with missing specs
- [ ] Gate passes when all FRs have tests

**Deliverable**: CI enforcement gate

---

### WP5: Agent Training & Documentation (Days 5-6)

#### Task 5.1: Update AGENTS.md with specs/main Workflow

**Owner**: documentation (1 agent)
**Status**: Pending (after Tasks 1-4)
**Effort**: 2 hours

**Add to AGENTS.md**:

```markdown
## Specs/Main Workflow (Phase 1)

### Creating a New Specification

1. **Check if spec already exists**:
   ```bash
   grep "My Feature Title" docs/reference/SPECS_REGISTRY.md
   ```

2. **Add to FUNCTIONAL_REQUIREMENTS.md**:
   ```markdown
   ## FR-001-NNN: My Feature Title

   - **Status**: PROPOSED
   - **Traces To**: <AgilePlus Spec ID or WP>
   - **Tests**: path/to/test/file

   Description of behavior...
   ```
   (Don't worry about the ID; spec merge service assigns it)

3. **Commit with FR reference**:
   ```bash
   git commit -am "feat: implement FR-001-NNN (spec: my-feature)"
   ```

4. **Open PR** and let the service handle the rest.

### Resolving Spec Conflicts

You may see messages like:
```
Your spec assigned ID FR-001-045 (was FR-001-022)
Branch has been updated automatically
```

This is normal! The spec merge service detected a collision with another agent's PR
and reassigned your ID to avoid duplication. Your branch is updated automatically.

### Understanding the Service

- **When**: Service runs when you push FUNCTIONAL_REQUIREMENTS.md changes
- **What**: Detects ID collisions, auto-reassigns, merges specs
- **Time**: Usually <1 minute
- **Result**: Check AUDIT_LOG.md to see what happened

### Rules

- Never manually edit SPECS_REGISTRY.md (service manages it)
- Always reference specs with FR-ID in tests: `// Traces to: FR-001-NNN`
- Always add specs to FUNCTIONAL_REQUIREMENTS.md (not SPECS_REGISTRY.md)
- Keep specs/main clean (no direct edits by agents)
```

**Acceptance Criteria**:
- [ ] AGENTS.md updated with specs/main workflow
- [ ] Examples provided (spec creation, test reference)
- [ ] Common scenarios documented (collision, orphaned test)
- [ ] Links to full architecture doc (POLYREPO_SSOT_ARCHITECTURE.md)

**Deliverable**: Updated AGENTS.md

---

#### Task 5.2: Create FAQ & Troubleshooting

**Owner**: documentation (1 agent)
**Status**: Pending (after Tasks 1-4)
**Effort**: 1 hour

**Deliverable**: `docs/reference/SPECS_WORKFLOW_FAQ.md`

```markdown
# Specs Workflow FAQ

## Q: What if two agents create the same FR-ID?
A: The spec merge service detects the collision and auto-reassigns one ID.
   Check your branch; it will be updated automatically with the new ID.
   See AUDIT_LOG.md for details.

## Q: Can I manually edit SPECS_REGISTRY.md?
A: No. The service manages the registry. Always edit FUNCTIONAL_REQUIREMENTS.md instead.
   On merge, the service moves your spec to the registry automatically.

## Q: My test references FR-XXX-YYY but CI complains "orphaned test".
A: The FR-ID doesn't exist in SPECS_REGISTRY.md yet.
   Add the spec to FUNCTIONAL_REQUIREMENTS.md with that ID.
   Or, if the ID is wrong, update your test comment to match an existing FR.

## Q: How do I check the status of my spec while my PR is open?
A: Look at the GitHub PR checks: "Specs Validation" will show any issues.
   Or run locally: `scripts/validate-specs-format.py`

## Q: What does "FR-001-NNN" format mean?
A: FR-001 = project group (001 = phenotype-infrakit)
   NNN = sequence number (incrementing, e.g., 001, 002, 003)
   Full ID example: FR-001-045

## Q: Can I delete a spec?
A: No. Deprecated specs are marked DEPRECATED in SPECS_REGISTRY.md, not deleted.
   This preserves audit history. Rationale: Tests may still reference old specs.
```

**Acceptance Criteria**:
- [ ] FAQ covers most common questions
- [ ] FAQ links to full architecture doc
- [ ] Troubleshooting steps are clear and actionable

**Deliverable**: FAQ documentation

---

### WP6: Soft Launch & Validation (Days 1-7, Weeks 1-2)

#### Task 6.1: Soft Launch Spec Service (Log-Only Mode)

**Owner**: services-team (1 agent)
**Status**: Pending (after Task 3)
**Effort**: 2 hours

**What this means**:
- Service runs, detects collisions, **logs them**
- Service does **NOT** modify branches or merge specs
- Agents still manually handle conflicts (current behavior)
- **Benefit**: Validates service correctness before full rollout

**Configuration** (env var in CI):
```bash
SPEC_SERVICE_MODE=log-only  # Don't auto-merge; just log
```

**Duration**: 3-4 days (Week 1)

**Success metrics**:
- [ ] Service correctly detects all collisions (0 false negatives)
- [ ] Service correctly ignores non-overlapping specs (0 false positives)
- [ ] Logs in AUDIT_LOG.md are accurate and complete

**Acceptance Criteria**:
- [ ] Log-only mode deployed in CI
- [ ] At least 3-5 PRs processed in log-only mode
- [ ] Zero errors in service logs
- [ ] AUDIT_LOG.md shows comprehensive conflict detection

**Deliverable**: Service validation data

---

#### Task 6.2: Full Rollout (Auto-Merge Mode)

**Owner**: services-team (1 agent)
**Status**: Pending (after Task 6.1)
**Effort**: 1 hour

**Configuration** (env var in CI):
```bash
SPEC_SERVICE_MODE=auto-merge  # Full service (merge specs, reassign IDs)
```

**Rollout strategy**:
- Enable for new PRs only (don't re-process old ones)
- Monitor for 3-4 days (Week 2)
- If issues: Revert to log-only mode

**Success metrics**:
- [ ] Auto-merge mode processes all new PRs without errors
- [ ] All agents successfully using specs/main workflow
- [ ] 100% of FUNCTIONAL_REQUIREMENTS.md merged to SPECS_REGISTRY.md
- [ ] Zero merge failures

**Acceptance Criteria**:
- [ ] Full rollout enabled in CI
- [ ] At least 5-10 PRs processed in auto-merge mode
- [ ] All merges successful (0 failures)
- [ ] AUDIT_LOG.md shows clean collision resolutions

**Deliverable**: Phase 1 complete (all tasks delivered)

---

## Resource Requirements

### Agents Needed
- **governance-team**: 1 agent (Task 1.1 — conflict resolution)
- **ci-infrastructure**: 2 agents (Tasks 1.2, 1.3, 3.2, 4.2)
- **spec-indexer**: 2 agents (Tasks 2.1, 2.2 — parallel)
- **services-team**: 2 agents (Tasks 3.1, 6.1, 6.2)
- **qa-team**: 1 agent (Task 4.1)
- **documentation**: 2 agents (Tasks 5.1, 5.2 — parallel)

**Total**: 10 agents, 80-100 tool calls, ~2 weeks wall-clock

### Infrastructure
- GitHub Actions runners (standard Linux, <$100 total usage)
- No additional cloud services needed (all git-based)
- Local tools: GitPython, PyYAML, Bash (free/included)

---

## Success Criteria (Phase 1 Complete)

- [ ] All git conflict markers resolved
- [ ] `specs/main` branch created and protected
- [ ] SPECS_REGISTRY.md created with 26+ specs
- [ ] Spec merge service deployed and tested (log-only mode)
- [ ] Full rollout of auto-merge mode
- [ ] FR↔Test traceability gate enforced (100% coverage)
- [ ] AGENTS.md updated with specs/main workflow
- [ ] FAQ and troubleshooting documentation complete
- [ ] AUDIT_LOG.md shows clean conflict history (>10 entries)
- [ ] All agents trained and confident in new workflow

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Service has bugs | Log-only mode validates before auto-merge |
| Agents don't adopt workflow | CI gate blocks non-compliant PRs; mandatory training |
| Conflict detection false positives | Soft launch with validation |
| Merge commits become too granular | Squash commits in integration phase (Phase 2) |

---

## Next Phase

Once Phase 1 is complete:
- Proceed to Phase 2: Dependency Reconciliation (Weeks 3-6)
- Reuse spec merge service pattern for dependency reconciliation
- Build on success from Phase 1

---

**Document Control**

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-03-30 | Claude Code | Phase 1 implementation roadmap |

