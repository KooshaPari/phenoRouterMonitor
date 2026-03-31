# Polyrepo SSOT Phase 1 — Agent Workflow Guide

**Status:** Operational Guidelines
**Version:** 1.0
**Created:** 2026-03-31
**Audience:** All 50+ Phenotype Agents

---

## Overview

This guide defines how agents collaborate on specifications using the unified `specs/main` branch. Key principles:

- **Single branch for truth:** All specs converge on `specs/main`, never hand-merged
- **Concurrent authorship:** 50+ agents work on overlapping specs without conflict
- **Automatic merges:** Clean changes merge within 5 minutes; conflicts trigger review
- **Full traceability:** Every commit traces to FR/ADR/PLAN; every test traces to FR

---

## 1. Agent Branching Pattern

### Branch Naming Convention

Create feature branches following this pattern:

```
specs/agent-<agent-name>-<task-id>
```

**Components:**
- `specs/` — Spec branch prefix (reserved for specification work only)
- `agent-<name>` — Your agent identifier (lowercase, hyphens)
- `<task-id>` — Work item ID (from AgilePlus: WP-XXX-NNN, FR-XXX-NNN, or ADR-YYY)

**Examples:**

| Agent | Task | Branch |
|-------|------|--------|
| phenosdk-decomposer | FR-PHENOSDK-001 | `specs/agent-phenosdk-decomposer-fr-phenosdk-001` |
| agileplus-merger | WP-AGILE-002 | `specs/agent-agileplus-merger-wp-agile-002` |
| thegent-specs | ADR-015 | `specs/agent-thegent-specs-adr-015` |
| explorer | FR-CORE-042 | `specs/agent-explorer-fr-core-042` |

### Creating a Spec Branch

```bash
# 1. Start from specs/main (always)
git checkout specs/main
git pull origin specs/main

# 2. Create feature branch
git checkout -b specs/agent-<your-name>-<task-id>

# 3. Verify you're on correct branch
git branch -v
# Should show: * specs/agent-<name>-<task> <commit>
```

---

## 2. Commit Message Format

Every commit to specs/* branches **must include spec traceability**. Use this format:

```
<type>(<scope>): <subject>

<body>

Spec-Traces: <FR-XXX-NNN, ADR-YYY, PLAN-ZZZ>
Co-Authored-By: <agent-name> <agent@phenotype.local>
```

### Example Commits

#### Example 1: Adding a new FR
```
specs(functional-requirements): add FR-PHENOSDK-001 for SDK core decomposition

Add comprehensive FR definition for phenosdk-decompose-core task,
including acceptance criteria for:
- Standalone core library extraction
- Public API surface definition
- Zero-dependency constraint

Spec-Traces: FR-PHENOSDK-001
Co-Authored-By: phenosdk-decomposer <agent@phenotype.local>
```

#### Example 2: Updating ADR
```
specs(architecture): ADR-015 — crate organization and PR guidelines

Expand decision rationale for monorepo structure:
- Justified workspaces vs feature-separated crates
- Module boundary guidelines
- Dependency management rules

Spec-Traces: ADR-015
Co-Authored-By: agileplus-merger <agent@phenotype.local>
```

#### Example 3: Adding test traces
```
specs(traceability): add test traces to FR-CORE-042

Update test file with Spec-Traces comment referencing
FR-CORE-042: Cache invalidation strategies

Spec-Traces: FR-CORE-042
Co-Authored-By: explorer <agent@phenotype.local>
```

#### Example 4: Multiple specs in one commit
```
specs(platform): synchronize thegent specs across FR, ADR, PLAN

Update three documents to reflect unified multi-agent orchestration model:
- FR-THEGENT-012: Agent routing and scheduling
- ADR-023: Memory system architecture
- PLAN-2026Q2: Quarterly roadmap

Spec-Traces: FR-THEGENT-012, ADR-023, PLAN-2026Q2
Co-Authored-By: thegent-specs <agent@phenotype.local>
```

### Commit Types

Use conventional commit types:

| Type | Use Case | Example |
|------|----------|---------|
| `specs` | Spec files (FR, ADR, PLAN, UJ) | Add/update FUNCTIONAL_REQUIREMENTS.md |
| `test` | Test files (trace to FR) | Add unit test with Spec-Traces |
| `docs` | Supporting documentation | Add README, implementation guide |
| `chore` | Maintenance, not spec-related | Update .gitignore |

---

## 3. Making Spec Changes

### Step-by-Step Workflow

#### Step 1: Make Local Changes

```bash
# Edit spec files
vim FUNCTIONAL_REQUIREMENTS.md    # Add new FR section
vim ADR.md                         # Update architecture decision
vim PLAN.md                        # Add task or milestone
vim USER_JOURNEYS.md              # Document user flow
```

**Key editing guidelines:**
- FR format: Use `## FR-XXX-NNN: Title` headers
- ADR format: Decision, Rationale, Consequences sections
- PLAN format: Phase, Task, Depends-On, Status columns
- UJ format: Actor, Goal, Preconditions, Flow, Metrics

#### Step 2: Verify Markdown Structure

```bash
# Run local validation (pre-commit hook)
pre-commit run --all-files

# Expected output:
# ✅ spec-structure.py ......... PASSED
# ✅ spec-traceability.py ....... PASSED
# ✅ prettier .................. PASSED
```

If validation fails, fix issues:
```bash
# Fix formatting
prettier --write .

# Retry validation
pre-commit run --all-files
```

#### Step 3: Add & Commit with Trace

```bash
# Stage changes
git add FUNCTIONAL_REQUIREMENTS.md

# Commit with spec trace
git commit -m "specs(fr): add FR-PHENOSDK-001 for SDK decomposition

Add comprehensive functional requirement for phenosdk-decompose-core,
including acceptance criteria and test mappings.

Spec-Traces: FR-PHENOSDK-001
Co-Authored-By: phenosdk-decomposer <agent@phenotype.local>"

# Verify commit message
git show --format=%B -s
```

#### Step 4: Pre-Push Validation

**Before pushing, the pre-push hook validates:**
- ✅ All commits have Spec-Traces
- ✅ All Spec-Traces reference valid FRs/ADRs
- ✅ Markdown structure valid
- ✅ No conflicts with specs/main

```bash
# Hook runs automatically, or manually:
./scripts/hooks/pre-push

# Example output:
# ✅ 1 commit(s) to push
# ✅ All commits have Spec-Traces
# ✅ FR-PHENOSDK-001 exists in FUNCTIONAL_REQUIREMENTS.md
# ✅ No conflicts detected
# ✅ Ready to push
```

#### Step 5: Push & Auto-Merge

```bash
# Push to agent branch
git push origin specs/agent-phenosdk-decomposer-fr-phenosdk-001

# → GitHub Actions triggers automatically:
#   1. Validation gate runs (2-3 min)
#   2. If clean: Auto-merge to specs/main (within 5 min)
#   3. If conflicts: Manual review issue created

# Monitor merge status
gh pr list --state all --label specs
# or check GitHub Actions logs
gh run list --workflow auto-merge-specs.yml
```

---

## 4. Handling Merge Conflicts

### Auto-Merge Success (No Conflict)

If no conflicts detected:

```bash
# ✅ Within 5 minutes, your branch merges automatically
# ✅ GitHub Actions closes branch
# ✅ You see merge commit in specs/main

# Verify merge
git checkout specs/main
git pull origin specs/main
git log --oneline -5
# Should show your commits merged
```

### Manual Review (Conflict Detected)

If merge conflicts detected:

```bash
# ❌ Auto-merge fails
# → GitHub creates manual review issue

# Example issue:
# Title: "🔀 Merge Conflict: specs/agent-<name>-<task>"
# Body: Lists conflict details + resolution instructions
```

**To resolve conflict:**

```bash
# 1. Fetch origin
git fetch origin

# 2. Check for conflicts
git diff origin/specs/main..HEAD

# 3. Manually resolve in editor
# (Fix the conflicting sections marked with <<<<<<<, =======, >>>>>>>)
vim FUNCTIONAL_REQUIREMENTS.md

# 4. Re-commit
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "resolve: merge conflict in specs/agent-<name>-<task>"

# 5. Push again
git push origin specs/agent-<name>-<task>

# → Auto-merge retry happens automatically (within next 5 min batch)
```

### Common Conflict Scenarios

#### Scenario 1: Both agents edit same FR

**File:** FUNCTIONAL_REQUIREMENTS.md
**Conflict:** Agent A adds FR-001, Agent B adds FR-002, both edit summary

**Resolution:**
- Keep both FRs (merge both additions)
- Re-order if needed (maintain numbering sequence)
- Commit with updated Spec-Traces

#### Scenario 2: Spec version mismatch

**File:** specs/REGISTRY.md
**Conflict:** Version number incremented by both agents

**Resolution:**
- Use highest version number
- Document in CHANGELOG.md which commits incremented
- Commit as single "version bump" commit

#### Scenario 3: Test trace mismatch

**File:** tests/ (distributed across files)
**Conflict:** Multiple tests trace to same FR

**Resolution:**
- Keep all tests (more coverage is better)
- Verify all Spec-Traces point to valid FRs
- No action needed (not a real conflict)

---

## 5. FR↔Test Traceability

Every test must trace to a functional requirement. Every FR must have tests.

### Adding Spec-Traces Comment to Tests

#### Rust Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENOSDK-001
    #[test]
    fn test_sdk_core_initialization() {
        let core = SdkCore::new().expect("should initialize");
        assert!(core.is_ready());
    }

    // Traces to: FR-PHENOSDK-001, FR-PHENOSDK-002
    #[test]
    fn test_sdk_core_with_plugins() {
        let mut core = SdkCore::new().unwrap();
        core.load_plugin("my-plugin").expect("should load");
        assert_eq!(core.plugins().len(), 1);
    }
}
```

#### Python Example
```python
# Traces to: FR-CORE-042
def test_cache_invalidation_ttl():
    """Verify cache invalidates after TTL expires"""
    cache = Cache(ttl_secs=1)
    cache.set("key", "value")
    assert cache.get("key") == "value"

    time.sleep(1.1)
    assert cache.get("key") is None  # Expired


# Traces to: FR-CORE-042, ADR-023
def test_cache_memory_efficient():
    """Verify cache uses efficient memory structures"""
    cache = Cache(max_size=10_000)
    # ... test memory usage
```

#### TypeScript Example
```typescript
// Traces to: FR-THEGENT-012
describe("Agent Routing", () => {
    it("should route agent to correct handler", async () => {
        const router = new AgentRouter();
        const handler = await router.resolve("llm-handler");
        expect(handler).toBeDefined();
    });
});

// Traces to: FR-THEGENT-012, FR-THEGENT-013
describe("Agent Scheduling", () => {
    it("should schedule task with priority", async () => {
        // ... test
    });
});
```

### Running Traceability Validation

```bash
# Validate 100% FR↔Test coverage
python3 scripts/validate-fr-test-coverage.py

# Expected output:
# ✅ Found 45 FRs in FUNCTIONAL_REQUIREMENTS.md
# 🧪 Found 47 test traces in tests/
# ✅ All FRs have tests (100% coverage)
# ✅ All tests trace to valid FRs
```

---

## 6. Pre-Push Validation Checklist

Before every push, verify:

**Specification Quality**
- [ ] Markdown structure valid (headers, lists, tables)
- [ ] All new FRs follow format: `## FR-XXX-NNN: Title`
- [ ] All new ADRs have: Decision, Rationale, Consequences
- [ ] All FRs are unique (no duplicates)
- [ ] No orphan sections (all sections traced to something)

**Traceability**
- [ ] Commit message includes Spec-Traces
- [ ] Spec-Traces reference valid FRs/ADRs
- [ ] Test traces point to valid FRs
- [ ] 100% FR↔Test coverage maintained

**Git Hygiene**
- [ ] Commits squashed if >1 per task (optional; clean history preferred)
- [ ] Commit author set to agent name + email
- [ ] No local secrets or credentials in diffs
- [ ] No large files (>10MB)

**Testing**
- [ ] Ran `pre-commit run --all-files` locally
- [ ] Validation scripts passed
- [ ] No warnings or errors in output

**Conflict Check**
- [ ] Checked for conflicts: `git diff origin/specs/main..HEAD`
- [ ] If conflicts exist, documented in commit message

```bash
# Automated checklist (pre-push hook)
./scripts/hooks/pre-push

# Example output:
# 📋 Pre-Push Validation Checklist
# ✅ Markdown structure valid
# ✅ 2 commits with valid Spec-Traces
# ✅ All FRs traced in tests
# ✅ No conflicts with origin/specs/main
# ✅ Ready to push
```

---

## 7. Automatic Branch Merge Triggers

### What Triggers Auto-Merge?

Your specs/agent-* branch auto-merges to specs/main when **all** criteria met:

1. **Validation passes**
   - Markdown structure valid ✅
   - All commits have Spec-Traces ✅
   - All Spec-Traces reference valid specs ✅

2. **No conflicts**
   - Cleanly merges with specs/main ✅
   - No file deletions/renames ✅

3. **Traceability complete**
   - FR↔Test coverage 100% ✅
   - All tests trace to valid FRs ✅

4. **CI tests pass**
   - All validation workflows succeeded ✅

### Merge Timeline

```
specs/agent-<name>-<task> pushed
          ↓
     Validation runs (2-3 min)
          ↓
   All checks pass? (CI confirms)
          ↓
        YES                    NO
         ↓                      ↓
   Auto-merge          Issue created
    (<5 min)         (manual review)
         ↓                      ↓
  specs/main          Resolve & retry
   updated            (agent action)
```

### Disabling Auto-Merge (Emergency Only)

If auto-merge broken or causing issues:

```bash
# Pause auto-merge (requires admin)
gh workflow disable auto-merge-specs

# Fix issue in main branch
git checkout main
git pull origin main
# ... make fixes ...
git push origin main

# Re-enable auto-merge
gh workflow enable auto-merge-specs
```

---

## 8. PR Template for Spec Reconciliation

When your specs/agent-* branch is ready for merge, create PR with this template:

```markdown
# Agent Spec Reconciliation PR

**Agent:** phenosdk-decomposer
**Task:** FR-PHENOSDK-001: SDK Core Decomposition
**Branch:** specs/agent-phenosdk-decomposer-fr-phenosdk-001
**Related Issue:** #123

---

## Summary
Extracted SDK core module from monolithic phenosdk crate. Defined public API surface, acceptance criteria, and test mappings for standalone library usage.

## Changes
- ✅ FUNCTIONAL_REQUIREMENTS.md: Added FR-PHENOSDK-001 (47 lines)
- ✅ ADR.md: Added ADR-024 (module boundary design)
- ✅ tests/: Added 8 new tests with Spec-Traces
- ✅ Commit messages: All include Spec-Traces

## Validation Checklist
- ✅ Pre-push validation passed locally
- ✅ CI validation passed (ssot-validation workflow)
- ✅ FR↔Test coverage: 100% (all 3 FRs have tests)
- ✅ Markdown structure valid (prettier + spec-validator)
- ✅ No conflicts with specs/main (verified via git diff)
- ✅ All commits have Spec-Traces format
- ✅ No merge conflicts expected (linear merge)

## Spec Files Modified
| File | Lines Added | Type |
|------|-------------|------|
| FUNCTIONAL_REQUIREMENTS.md | +47 | FR-PHENOSDK-001 |
| ADR.md | +28 | ADR-024 decision |
| tests/test_sdk_core.rs | +63 | 8 new tests |

## FR Coverage
- ✅ FR-PHENOSDK-001: SDK Core Library Extraction
  - Test coverage: 8 tests
  - Traces: test_sdk_core_*.rs
  - Status: Ready for production

## Merge Expectation
This PR will **auto-merge** to specs/main within 5 minutes if:
- ✅ CI validation passes (in progress)
- ✅ No conflicts detected (pre-verified locally)
- ✅ FR↔Test coverage 100% (validated)

No manual review needed for clean, spec-compliant changes.

---

**Agent:** phenosdk-decomposer
**Created:** 2026-03-31
**Expected Merge:** 2026-03-31 14:45 UTC
```

---

## 9. Common Workflows

### Workflow A: Agent Updates Single FR

```bash
# 1. Start fresh
git checkout specs/main && git pull

# 2. Create branch
git checkout -b specs/agent-<name>-fr-<id>

# 3. Edit FR
vim FUNCTIONAL_REQUIREMENTS.md
# → Update acceptance criteria, add test mapping

# 4. Commit
git commit -am "specs(fr): update FR-CORE-042 acceptance criteria

Refined cache invalidation strategy acceptance criteria based on
initial implementation feedback.

Spec-Traces: FR-CORE-042
Co-Authored-By: <name> <agent@phenotype.local>"

# 5. Validate
pre-commit run --all-files

# 6. Push
git push origin specs/agent-<name>-fr-<id>

# → Auto-merge within 5 min
```

### Workflow B: Agent Adds Multiple Specs

```bash
# 1. Start fresh
git checkout specs/main && git pull

# 2. Create branch
git checkout -b specs/agent-<name>-multi-<ids>

# 3. Edit multiple files
vim FUNCTIONAL_REQUIREMENTS.md  # Add FR-XXX-001, FR-XXX-002
vim ADR.md                       # Add ADR-025
vim PLAN.md                      # Add tasks referencing FRs

# 4. Commit each logically
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "specs(fr): add FR-XXX-001 and FR-XXX-002 for feature X

Spec-Traces: FR-XXX-001, FR-XXX-002
Co-Authored-By: <name> <agent@phenotype.local>"

git add ADR.md
git commit -m "specs(adr): add ADR-025 for system design decision

Rationale: Addresses design question from FR-XXX-001 implementation

Spec-Traces: ADR-025, FR-XXX-001
Co-Authored-By: <name> <agent@phenotype.local>"

# 5. Validate
python3 scripts/validate-fr-test-coverage.py
pre-commit run --all-files

# 6. Push
git push origin specs/agent-<name>-multi-<ids>

# → Auto-merge all commits to specs/main within 5 min
```

### Workflow C: Agent Resolves Merge Conflict

```bash
# 1. Push initially (auto-merge fails)
git push origin specs/agent-<name>-<task>
# → ❌ Merge conflict detected
# → GitHub creates issue: "Merge Conflict: specs/agent-<name>-<task>"

# 2. Fetch origin and check conflict
git fetch origin
git diff origin/specs/main..HEAD | grep "^<<<<<<<" -A 20

# 3. Resolve locally
vim FUNCTIONAL_REQUIREMENTS.md
# → Fix conflict by merging both agents' changes

# 4. Re-commit and push
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "resolve: merge conflict in specs/agent-<name>-<task>

Merged both FR-XXX-001 and FR-YYY-002 additions.

Spec-Traces: FR-XXX-001, FR-YYY-002
Co-Authored-By: <name> <agent@phenotype.local>"

git push origin specs/agent-<name>-<task>

# → Auto-merge retry within next 5-min batch
```

---

## 10. Troubleshooting

### Issue: "Spec-Traces format invalid"

**Error:**
```
ERROR: Commit <hash> missing Spec-Traces in correct format
Expected: Spec-Traces: FR-XXX-NNN
```

**Fix:**
```bash
# Re-write last commit with correct format
git commit --amend

# Change message body to include:
# Spec-Traces: FR-CORE-042

# Or squash + rewrite
git rebase -i HEAD~3
# Mark commits as 'reword', add Spec-Traces to each
```

### Issue: "FR-CORE-042 not found in FUNCTIONAL_REQUIREMENTS.md"

**Error:**
```
ERROR: Spec-Traces references FR-CORE-042 but not found in FR document
```

**Fix:**
```bash
# Verify FR exists
grep -n "## FR-CORE-042" FUNCTIONAL_REQUIREMENTS.md

# If not found, add it first
vim FUNCTIONAL_REQUIREMENTS.md
# Add: ## FR-CORE-042: <Title>

# Then re-commit
git add FUNCTIONAL_REQUIREMENTS.md
git commit -m "specs(fr): add FR-CORE-042 definition

Spec-Traces: FR-CORE-042
..."

git push
```

### Issue: "Auto-merge not triggering"

**Symptoms:**
- Branch pushed 10+ minutes ago
- Still shows "pending" in GitHub Actions
- No merge commit created

**Debug:**
```bash
# Check workflow status
gh run list --workflow auto-merge-specs.yml

# View logs
gh run view <run-id> --log

# If workflow failed, check:
# 1. Branch exists: git branch -r | grep agent
# 2. No conflicts: git merge-base origin/specs/main origin/specs/agent-<name>-<task>
# 3. CI passed: gh pr list | grep agent-<name>

# Manual trigger (if stuck):
gh workflow run auto-merge-specs.yml
```

### Issue: "Merge conflict requires manual resolution"

**Symptoms:**
- GitHub issue created: "Merge Conflict: specs/agent-<name>-<task>"
- Auto-merge failed

**Resolution Steps:**
```bash
# 1. Fetch latest
git fetch origin

# 2. Check what conflicts
git diff origin/specs/main origin/specs/agent-<name>-<task> | head -50

# 3. Resolve manually (see Workflow C above)

# 4. Push re-resolved branch
git push origin specs/agent-<name>-<task>

# → Auto-merge retries automatically
```

---

## 11. Role-Specific Quick Reference

### For New Agents

1. ✅ Run `scripts/agent-setup.sh <your-name>`
2. ✅ Read this guide (Section 1-3)
3. ✅ Create first branch: `specs/agent-<name>-<task>`
4. ✅ Make changes, commit with Spec-Traces
5. ✅ Push and watch auto-merge (5 min)

### For Experienced Agents

1. ✅ Branch: `specs/agent-<name>-<task>`
2. ✅ Edit, commit with Spec-Traces
3. ✅ Run `pre-commit run --all-files`
4. ✅ Push: `git push origin specs/agent-<name>-<task>`
5. ✅ Auto-merge happens automatically

### For Spec Administrators

- Monitor health: `docs/SSOT_HEALTH_DASHBOARD.md`
- Resolve conflicts: Create PR with merged content
- Update templates: Edit `.commit-template`, `.pre-commit-config.yaml`
- Emergency: `gh workflow disable auto-merge-specs.yml`

---

## 12. Glossary

| Term | Definition |
|------|-----------|
| **specs/main** | Authoritative specification branch; never edited directly |
| **specs/agent-*** | Feature branch for agent spec work; auto-merges to specs/main |
| **Spec-Traces** | Commit message field referencing FR/ADR/PLAN (required) |
| **FR (Functional Requirement)** | Feature specification: acceptance criteria, test mapping |
| **ADR (Architecture Decision Record)** | Design decision: rationale, consequences, trade-offs |
| **PLAN** | Roadmap: phases, tasks, dependencies, timeline |
| **UJ (User Journey)** | End-to-end workflow: actor, goal, flow, metrics |
| **Auto-merge** | Automatic merge of clean specs/agent-* to specs/main (5 min) |
| **Merge Conflict** | Overlapping edits requiring manual resolution |

---

## Reference

- **Main Plan:** `/docs/reference/SSOT_PHASE1_IMPLEMENTATION_PLAN.md`
- **Spec Format Guide:** `/docs/reference/SPEC_FORMAT_STANDARDS.md`
- **Health Dashboard:** `/docs/SSOT_HEALTH_DASHBOARD.md`
- **Troubleshooting:** Section 10 above

---

**Document Owner:** Platform Team
**Version:** 1.0
**Status:** Operational
**Last Updated:** 2026-03-31

**Follow this workflow to keep specs converged and 100% traceable.**
