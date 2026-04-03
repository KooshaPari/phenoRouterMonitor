# Spec Branch Workflow Guide

**Version:** 1.0 | **Status:** Active | **Updated:** 2026-03-30

## Overview

The `specs/main` branch is the canonical single source of truth (SSOT) for all FUNCTIONAL_REQUIREMENTS.md across the Phenotype polyrepo ecosystem. This workflow enables 50+ parallel agents to work on specifications independently of code implementation, avoiding merge conflicts and blocking relationships.

### Key Principles

- **Decoupling**: Spec changes (specs/main) are independent from code changes (main)
- **Async Merges**: Spec PR merges are non-blocking to code PR merges
- **Agent-First**: Each agent works in isolation on specs/agent-<id> branches
- **Validation**: Automated CI ensures FR uniqueness and traceability before merge
- **Audit Trail**: All spec changes are tracked separately from code changes

---

## Quick Start for Agents

### 1. Clone or Update specs/main

```bash
# If starting fresh
git clone https://github.com/KooshaPari/phenotype-infrakit.git
cd phenotype-infrakit
git checkout specs/main
git pull origin specs/main

# If already cloned
git fetch origin
git checkout specs/main
git pull origin specs/main
```

### 2. Create Your Spec Branch

Name format: `specs/agent-<YOUR_AGENT_ID>-<FEATURE_NAME>`

```bash
# Example: Adding LLM decomposition specs
git checkout -b specs/agent-llm-decomposer-phenosdk-llm

# Example: Adding event sourcing specs
git checkout -b specs/agent-event-sourcing-impl-phase2
```

### 3. Edit FUNCTIONAL_REQUIREMENTS.md

#### Adding New FRs

1. Find the appropriate `## FR-<REPO>:` section (or create it if new repo)
2. Add entry with format:

```markdown
#### FR-<REPO>-NNN: Brief Title

**Requirement:** System SHALL [specific testable requirement].
**Traces To:** E<X>.<Y> (epic reference)
**Code Location:** Path/to/implementation/file.rs
**Repository:** repo-name
**Status:** Active/Planned/Deprecated
**Test Traces:** Link to test file or description
```

#### Updating Existing FRs

If you're implementing an FR that exists:

```markdown
#### FR-AGILE-001: Feature Entity

**Requirement:** [unchanged]
**Traces To:** [unchanged]
**Code Location:** [unchanged]
**Repository:** [unchanged]
**Status:** Active  # ← Updated from Planned
**Test Traces:** Updated test location
```

#### Important Notes

- **ID Format**: `FR-<REPO>-<NNN>` where NNN is zero-padded 3 digits
- **Repo Prefix** Use canonical repo prefixes (INFRA, AGILE, HELIOS, THEGENT, WAVE, API, etc.)
- **Uniqueness**: Run validation (see below) to ensure no duplicates
- **Traceability**: Every FR must link to a PRD epic (E1.1, E2.3, etc.)
- **Status Values**: Active, Planned, Deprecated, Superseded

### 4. Validate Your Changes

#### Local Validation (recommended before push)

```bash
# Run spec verifier (if available in your setup)
spec-verifier validate --master-file FUNCTIONAL_REQUIREMENTS.md

# Or manually check:
# - No duplicate FR IDs
# - All new FRs have status and code location
# - All traces reference valid epics
```

#### Check for ID Conflicts

```bash
# Find all FR IDs in your edits
grep -E "^#### FR-.*NNN:" FUNCTIONAL_REQUIREMENTS.md

# Verify against main branch's version (if unsure)
git show main:FUNCTIONAL_REQUIREMENTS.md | grep -E "#### FR-.*:"
```

### 5. Commit and Push

```bash
# Stage your changes
git add FUNCTIONAL_REQUIREMENTS.md
git add docs/reference/SPEC_BRANCH_WORKFLOW.md  # If you updated this

# Commit with clear message
git commit -m "specs: add FR-PHENOSDK-001 through FR-PHENOSDK-008 for LLM decomposition"

# Push to your agent branch
git push origin specs/agent-<YOUR_ID>-<FEATURE>
```

### 6. Create Pull Request

```bash
# Create PR targeting specs/main (NOT main)
gh pr create \
  --title "specs: Add LLM decomposition FRs (phenosdk-decompose-llm)" \
  --body "$(cat <<'EOF'
## Summary

Adds 8 new functional requirements for LLM module decomposition in phenotype-SDK:
- FR-PHENOSDK-001 through FR-PHENOSDK-008
- Traces to Epic E8.1 (LLM contract extraction)
- All FRs have corresponding tests in phenosdk-llm/tests/

## Changed Files
- FUNCTIONAL_REQUIREMENTS.md: +62 lines, 8 new FRs

## Validation
- [x] No duplicate FR IDs (verified with spec-verifier)
- [x] All FRs trace to valid epics
- [x] Test coverage 100% (8/8 FRs have test references)
- [x] Code locations verified

## Related Issues
- Closes #123 (phenosdk-decompose-llm spec expansion)
EOF
)" \
  --base specs/main
```

### 7. Address Review Comments

When CI validation or human reviewers request changes:

```bash
# Make requested changes to FUNCTIONAL_REQUIREMENTS.md
vim FUNCTIONAL_REQUIREMENTS.md

# Commit and push to same branch (re-uses PR)
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "specs: address review feedback for FR-PHENOSDK-001"
git push origin specs/agent-<YOUR_ID>-<FEATURE>
```

### 8. Merge to specs/main

After approval:

```bash
# GitHub UI or CLI
gh pr merge <PR_NUMBER> --squash --base specs/main

# Or if using gh CLI from command line
gh pr merge \
  --auto \
  --squash \
  --base specs/main
```

---

## CI Validation Pipeline

### Automated Checks on specs/main

When you push to `specs/agent-*` branches, GitHub Actions runs:

1. **FR Uniqueness Check**
   - Ensures no duplicate FR IDs in the merged FUNCTIONAL_REQUIREMENTS.md
   - Blocks merge if duplicates found
   - Error message: `FR-<REPO>-<NNN> already exists in FUNCTIONAL_REQUIREMENTS.md`

2. **Traceability Validation**
   - Verifies every FR references a valid epic (E1.1, E2.3, etc.)
   - Ensures Epic references exist in master PRD.md
   - Blocks merge if trace is broken
   - Error message: `FR-<REPO>-<NNN> traces to invalid epic E99.99`

3. **Code Location Verification**
   - For Active/Deprecated FRs: code location must exist or be marked Planned
   - For Planned FRs: code location optional but accepted
   - Warns if file doesn't exist yet (acceptable for Planned status)

4. **Test Coverage Check**
   - Scans linked test files for `// Traces to: FR-<REPO>-<NNN>` comments
   - Ensures every FR has >=1 test reference
   - Fails if FR claims test traces but none found
   - Error message: `FR-<REPO>-<NNN> claims test at X but file not found`

### Merge Requirements

- ✅ All CI checks passing
- ✅ No duplicate FR IDs
- ✅ Traceability complete (all FRs trace to valid epics)
- ✅ For Pending/Superseded FRs: explicit justification in Requirement field

### After Merge to specs/main

1. Spec changes become canonical SSOT
2. All agents can pull latest specs/main for reference
3. Code implementation PRs on `main` can reference merged specs as baseline
4. No automatic sync to `main` branch (decoupled by design)

---

## Parallel Work Example

### Scenario: 5 Agents Working on Different Specs

```
Agent 1: specs/agent-phenosdk-llm-decomposer     (FR-PHENOSDK-001 through 010)
Agent 2: specs/agent-event-sourcing-impl          (FR-EVENTSRC-001 through 005)
Agent 3: specs/agent-helios-sandbox-hardening     (FR-HELIOS-010 through 015)
Agent 4: specs/agent-thegent-federation           (FR-THEGENT-020 through 030)
Agent 5: specs/agent-agileps-sync-reconciliation  (FR-AGILE-100 through 105)

All 5 agents:
1. Pull specs/main (baseline)
2. Create independent specs/agent-* branches
3. Edit FUNCTIONAL_REQUIREMENTS.md in parallel (no conflicts)
4. Push separate PRs targeting specs/main
5. Merge to specs/main independently (async, in any order)
```

Result: All specs merged in 15-30 minutes with zero conflicts, while code teams implement in parallel on `main` branch.

---

## Common Scenarios

### Scenario A: Multiple Agents Editing Same FR Section

**Problem**: Two agents adding FRs to FR-AGILE section simultaneously

**Solution**:
1. Use next available number within FR-AGILE sequence
2. Agent 1: FR-AGILE-200, FR-AGILE-201, FR-AGILE-202
3. Agent 2: FR-AGILE-210, FR-AGILE-211 (skip 203-209 to avoid conflicts)
4. Both merge to specs/main without conflict

**Prevention**: Coordinate within agent teams or use number ranges (contact @architecture-team)

### Scenario B: FR Needs Major Revision During Implementation

**Current State**: FR-AGILE-050 is Active with code location

**Change Needed**: Implementation revealed new requirements

**Steps**:
1. Create new specs/agent-* branch
2. Update FR-AGILE-050 requirement field with new text
3. Mark as `Status: Active (Revised)`
4. Add comment: `Updated 2026-03-30: Added requirement for async validation due to performance analysis`
5. Create PR with title: `specs: revise FR-AGILE-050 based on implementation findings`
6. Merge to specs/main

**Result**: History preserved in commit log, specs/main reflects latest state

### Scenario C: FR Superseded by New Approach

**Current State**: FR-AGILE-050 is Active

**New Direction**: Discovered better solution (FR-AGILE-150)

**Steps**:
1. Keep FR-AGILE-050 unchanged
2. Change Status to `Superseded`
3. Add comment: `Superseded by FR-AGILE-150 (new approach discovered during implementation)`
4. Create new FR-AGILE-150 with the revised approach
5. Merge both to specs/main

**Result**: Full audit trail preserved, new spec becomes canonical

---

## Troubleshooting

### Problem: CI Reports "FR-AGILE-001 already exists"

**Cause**: Duplicate FR ID in your edits
**Solution**:
1. Find conflicting ID: `grep "FR-AGILE-001" FUNCTIONAL_REQUIREMENTS.md`
2. Check main branch: `git show main:FUNCTIONAL_REQUIREMENTS.md | grep "FR-AGILE-001"`
3. Use next available ID (e.g., FR-AGILE-002, FR-AGILE-003)
4. Commit and push again

### Problem: "Invalid epic trace E99.99"

**Cause**: Epic doesn't exist in PRD.md
**Solution**:
1. Check valid epics in root PRD.md or repo-specific PRD
2. Use correct epic (e.g., E1.1, E2.3, E5.5)
3. If epic should exist, contact PRD owner to add it first
4. Update FR and push again

### Problem: "Code location file not found"

**Cause**: File path doesn't exist (for Active FR)
**Solution**:
1. For **Active** FRs: File must exist. Verify path is correct.
2. For **Planned** FRs: Code location optional. Change Status to Planned if appropriate.
3. Update FUNCTIONAL_REQUIREMENTS.md and push again

### Problem: PR merge conflicts with specs/main

**Cause**: Other agent merged first, changed same section
**Solution**:
```bash
git fetch origin
git rebase origin/specs/main
# Resolve conflicts manually
vim FUNCTIONAL_REQUIREMENTS.md  # Fix conflicts
git add FUNCTIONAL_REQUIREMENTS.md
git rebase --continue
git push origin specs/agent-<YOUR_ID>-<FEATURE> -f
```

---

## Best Practices

### DO

- ✅ Use descriptive branch names: `specs/agent-<purpose>-<repo>`
- ✅ Create PRs with detailed summaries of FRs being added
- ✅ Reference related code PRs: "See also: https://github.com/.../pull/123"
- ✅ Group related FRs together (same epic, same component)
- ✅ Keep FR descriptions concise but complete
- ✅ Use SHALL/MUST/SHOULD for clear requirements
- ✅ Validate locally before pushing: `grep -E "^#### FR-" FUNCTIONAL_REQUIREMENTS.md`

### DON'T

- ❌ Don't merge specs directly to main (specs/main is SSOT)
- ❌ Don't rename existing FR IDs (breaks traceability)
- ❌ Don't delete FRs (mark as Deprecated instead)
- ❌ Don't modify Traces To field without team agreement
- ❌ Don't add FRs without Status field
- ❌ Don't skip validation before pushing

---

## Integration with Code Branches

### Relationship Between specs/main and main

```
specs/main (SSOT for FRs)
  ↓
  Agents read specs/main before implementing
  ↓
main (SSOT for code)
  ↓
  Code PRs reference spec FR IDs in commit messages
  ↓
  Tests include "// Traces to: FR-<REPO>-<NNN>" comments
  ↓
  CI validates that all deployed code has FR traces
```

### Recommended Workflow

1. **Spec First**: Create specs/agent-* branch, add FRs
2. **Spec PR**: Submit specs PR, wait for CI validation + approval
3. **Spec Merge**: Merge to specs/main once approved
4. **Spec Baseline**: All agents pull specs/main
5. **Code Implementation**: Create code feature branches from main
6. **Code PR**: Submit code PR, reference spec FR IDs
7. **Code Review**: Ensure test coverage traces to FRs
8. **Code Merge**: Merge code to main

### Example Commit Messages

```
# Spec PR commit
commit: Add phenosdk-llm decomposition FRs (8 new FRs for epic E8.1)

# Code PR commit  
commit: Implement LLM contract extraction

This implements FR-PHENOSDK-001 through FR-PHENOSDK-008 from the specs/main canonical specs.

Tests added in:
- phenosdk-llm/tests/llm_contract_tests.rs (FR-PHENOSDK-001-003)
- phenosdk-llm/tests/decomposition_tests.rs (FR-PHENOSDK-004-008)
```

---

## Related Documents

| Document | Location | Purpose |
|----------|----------|---------|
| FUNCTIONAL_REQUIREMENTS.md | Root of specs/main | Master FR registry (SSOT) |
| PRD.md | Root of each repo | Product requirements with epics |
| ADR.md | /docs/adr/ | Architecture decisions tied to FRs |
| PLAN.md | Root of each repo | Implementation plan (references FRs) |
| USER_JOURNEYS.md | Root of each repo | User flows (traced to FRs) |

---

**Last Updated:** 2026-03-30
**Maintained By:** Architecture team
**Questions?** See FUNCTIONAL_REQUIREMENTS.md or contact @architecture

