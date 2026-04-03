# SSOT Architecture Quick Reference

**For**: Agents working in the polyrepo
**Format**: 1-page cheat sheet + decision trees
**Updated**: 2026-03-30

---

## The Three-Branch Model (TL;DR)

```
main           → Production code (immutable, fast-forward only)
specs/main     → FR/ADR/PLAN registry (append-only, versioned)
.worktrees/*   → Your feature branch (ephemeral, force-push OK)
integration/*  → Staging for merge (service-managed, auto-cleanup)
```

**Rule**: Always work in `.worktrees/`, never directly on `main`.

---

## Creating a Feature Branch

```bash
# 1. Create feature branch from main
git checkout -b .worktrees/agileplus/feat/my-feature main

# 2. Create new functional requirements (if applicable)
cat >> FUNCTIONAL_REQUIREMENTS.md <<EOF

## FR-XXX-YYY: My Feature Title

- **Status**: PROPOSED
- **Traces To**: <AgilePlus Spec ID>
- **Tests**: <path/to/test>

Description...
EOF

# 3. Implement feature, write tests
git commit -am "feat(my-feature): implement FR-XXX-YYY"
git push origin .worktrees/agileplus/feat/my-feature

# 4. Open PR
gh pr create --head .worktrees/agileplus/feat/my-feature --base main \
  --title "feat: Implement FR-XXX-YYY"
```

---

## Spec Conflicts? (Automatic Resolution)

**Scenario**: Two agents both add new FRs to FUNCTIONAL_REQUIREMENTS.md simultaneously.

**What happens**:
1. Spec reconciliation service detects both PRs
2. Service merges specs into SPECS_REGISTRY.md (append-only)
3. Service assigns sequential FR-IDs (avoid collisions)
4. Both PRs merge without conflict ✅

**Your job**: None! Service handles it automatically.

---

## Circular Dependency Detected?

**Scenario**: Your code creates `A → B → A` dependency cycle.

**Detection timing**:
- **Pre-commit hook** (Phase 2): Catches before you commit
- **CI gate** (Phase 1): Fails PR if cycle introduced

**If detected**:
1. Service suggests refactoring: "Extract common module X"
2. You implement refactoring
3. Push updated branch
4. CI re-validates (no cycle) → merge proceeds

---

## Ready to Merge?

**Workflow**:

```
Your feature branch merged and tests pass?

YES → Reconciliation service creates integration/* branch
      ↓
      Service validates:
        ✓ No circular deps
        ✓ All tests passing
        ✓ FR↔Test traceability 100%
      ↓
      If all pass → Atomic merge to main
      If any fail → Agent notified (fix + re-push)

Result: Your feature is in production (main)
```

**Timeline**: <10 minutes from approval to main (parallel processing)

---

## Decision Tree: Resolving a Conflict

```
Git says: "CONFLICT in <file>"

IF file == FUNCTIONAL_REQUIREMENTS.md:
  → Do nothing! Spec merge service resolves automatically.
  → Your PR will merge with other agent's specs in SPECS_REGISTRY.md.

ELSE IF file == Cargo.toml or package.json:
  → Check CI for "circular dependency detected"
  → If yes: Service blocks merge, suggests refactoring
  → If no: Service auto-merges, your branch is ready

ELSE IF file == src/**/*.rs or src/**/*.ts:
  → Manual resolution required
  → Resolve conflict markers locally
  → Re-run tests: cargo test
  → Push updated branch
  → CI re-validates

ELSE IF file == *.md (docs):
  → Service attempts semantic merge (by section)
  → If sections don't overlap: Auto-merged ✅
  → If same section: Concatenate with separator
  → Otherwise: Manual merge needed
```

---

## Branches at a Glance

| Branch | Read/Write? | When to Use | How Long? |
|--------|------------|------------|-----------|
| `main` | Read (via PR) | Never directly | Forever |
| `specs/main` | Read (read CI) | Never; append-only | Forever |
| `.worktrees/*` | Read/Write | Feature/fix work | 1-30 days |
| `integration/*` | Read (service only) | Merge staging | <10 min |

---

## File Paths: Where Things Live

```
FUNCTIONAL_REQUIREMENTS.md
  → Add new FRs here while developing
  → Service moves to docs/reference/SPECS_REGISTRY.md on merge

docs/reference/SPECS_REGISTRY.md
  → Read-only canonical registry (service updates)
  → Query: "Which crate provides EventStore?" → check this file

docs/reference/DEPENDENCY_GRAPH_CANONICAL.md
  → All inter-project dependencies (Phase 2)
  → Read-only; auto-updated by service

AUDIT_LOG.md
  → Record of all merge conflicts + resolutions
  → Read-only; service appends entries
  → "Which agent caused circular dep on 2026-03-30?" → check this file

.worktrees/*/
  → Your local feature branches
  → Archived to .archive/ after merge (7-day retention)

.agileplus/specs/
  → AgilePlus spec definitions
  → Maps to FUNCTIONAL_REQUIREMENTS.md entries
```

---

## Common Mistakes & Fixes

| Mistake | Fix |
|---------|-----|
| Pushing to `main` directly | Use `.worktrees/*` instead; git will reject direct pushes |
| Feature branch lives 90+ days | Service auto-archives after 7 days; recreate if needed |
| Merging without specs | CI gate blocks (no test traces to FR) |
| Circular deps in Cargo.toml | Pre-commit hook prevents commit; refactor to extract module |
| Two agents same FR-ID | Service detects, assigns sequential IDs automatically |
| Conflict markers in FUNCTIONAL_REQUIREMENTS.md | Service resolves; don't manually fix |

---

## If Something Goes Wrong

**Service crashes during merge**:
- Recover from AUDIT_LOG.md (distributed state in Git)
- Service restarts; retries automatically
- Check Slack notification for status

**PR stuck in queue**:
- Check `gh pr status` for blocking reviews
- Check `gh pr view <number> --json checks`
- If service-caused: Post in #engineering Slack

**Test failures appearing out of nowhere**:
- Likely caused by parallel merge (integration order changed)
- Check AUDIT_LOG.md for "topological sort"
- Your tests passed locally? → Report issue to team

---

## Spec Registry Query Cheat Sheet

**What specs exist?**
```bash
grep "^FR-" docs/reference/SPECS_REGISTRY.md
```

**Find spec by title:**
```bash
grep -A 5 "Event sourcing" docs/reference/SPECS_REGISTRY.md
```

**Find tests for a spec:**
```bash
grep -r "Traces to: FR-001-001" crates/ packages/
```

**See dependency graph:**
```bash
cat docs/reference/DEPENDENCY_GRAPH_CANONICAL.md  # (Phase 2)
```

---

## Phase Milestones

| Phase | When | What's Ready |
|-------|------|------------|
| 1 | Week 2 | `specs/main`, spec merge service, FR↔Test gate |
| 2 | Week 6 | Integration branches, parallel merge, circular dep detection |
| 3 | Week 12 | Platform contracts, governance centralized, health monitoring |

**Right now** (Week 1): Phase 1 in progress. Use `.worktrees/*` branches; specs workflow coming soon.

---

## Links

- Full architecture: `docs/reference/POLYREPO_SSOT_ARCHITECTURE.md`
- Agent rules: `AGENTS.md`
- Governance: `CLAUDE.md`
- AgilePlus specs: `.agileplus/specs/`

---

**TL;DR**: Work in `.worktrees/*`, open PR, service handles merges. No manual conflict resolution needed (except code conflicts). Questions? Check `POLYREPO_SSOT_ARCHITECTURE.md` or ask in #engineering.
