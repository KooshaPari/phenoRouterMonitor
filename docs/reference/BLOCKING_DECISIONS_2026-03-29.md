# Blocking Decisions & Next Steps (2026-03-29)

**Status**: Waiting on user clarification for 4 critical git state decisions + architectural direction

---

## 4 Critical Blocking Decisions

### 1. phenoSDK Packages (`packages/pheno-{core,llm,resilience}`)

**Question**: Are these new shared modules or temporary/experimental directories?

**Current State**:
- Located in canonical `repos/phenotype-infrakit/packages/`
- Three packages exist: pheno-core, pheno-llm, pheno-resilience
- Mentioned in memory as potential shared extraction targets
- No releases, no published packages yet

**Implications**:
- **If shared**: Need to publish to GitHub Packages, add to @phenotype scope, document interfaces
- **If temporary**: Should be moved to `.archive/`, worktree, or deleted
- **If forked**: Need to determine upstream and sync strategy

**Next Steps**: Need user to clarify the intent.

---

### 2. vibe-kanban Repository (2,524 changes on main)

**Question**: Intentional upstream sync or accidental checkin?

**Current State**:
- Repository is **archived** (read-only on GitHub)
- 2,524 uncommitted changes on main
- Cannot push or merge PRs to archived repo
- Was part of fork strategy (vibeproxy, zen, aizen, vibe-kanban, ccusage)

**Implications**:
- **If intentional**: Must unarchive repo + merge changes as PR + push
- **If accidental**: Should reset to last good commit
- **If obsolete**: Archive intentional; clean up local changes and forget

**Next Steps**: Need user to clarify status of vibe-kanban in current portfolio.

---

### 3. zen-wtrees Branch (386 commits)

**Question**: Keep (merge as PR) or discard (reset)?

**Current State**:
- Local worktree branch with 386 commits ahead of main
- Likely WIP from previous agent sessions
- Unsynced with upstream zen-mcp-server fork
- No active PR

**Implications**:
- **If keep**: Create PR, review changes, merge to main + push
- **If discard**: `git reset --hard origin/main` to clean state
- **If archive**: Move to `.archive/zen-wtrees-backup` with git-bundle for history recovery

**Next Steps**: Need user to review branch and decide.

---

### 4. 4sgm Archive Directory

**Question**: Active ongoing work or historical cleanup?

**Current State**:
- Embedded in `.archive/` directory
- Contains git repositories (embedded, not submodules)
- Not in main worktree
- Unknown usage

**Implications**:
- **If active**: Extract to separate repo + manage properly
- **If historical**: Document the archive intent and verify no missing dependencies
- **If data loss risk**: Create backups + export as git-bundles

**Next Steps**: Need user to clarify purpose and retention policy.

---

## Versioning Strategy Decision

**Question**: Approve Hybrid Epoch+SemVer strategy?

**Proposed Strategy** (from ab81250 agent):
- **Manifest versioning**: `YYYY.QN.PATCH` (e.g., `v2026.03A.0` for Q1 2026)
- **Crate/package versioning**: Independent SemVer per package (e.g., phenotype-shared@1.5.0)
- **Release channels**: alpha, beta, stable branches
- **Tooling**: git-cliff for CHANGELOG automation

**Status**: CHANGELOG.md already created for 14+ repos using this strategy.

**Approval needed**: User should confirm this is acceptable or request modifications.

---

## Architecture Decision

**Question**: Which pattern(s) to adopt first?

**Options**:
- **Option A** (Foundation First): Chassis + Intent-Driven Registry
- **Option B** (UI First): Module Federation + Health Monitoring
- **Option C** (Incremental): Sidecar pattern for auth/logging consistency

**Recommendation**: Sequence A → B → C

**Next Steps**: User should review `/docs/reference/ARCHITECTURAL_PATTERNS_APPLICATION.md` and select priority.

---

## Completed Work (Previous Agent Sessions)

✅ **Versioning Strategy**: Hybrid Epoch+SemVer designed (ab81250)
✅ **CHANGELOG Generation**: 6 repos created (a5a678d)
✅ **Release-Drafter Configs**: 16 repos updated (a639696)
✅ **Governance Audit**: 86.7% compliance established (a4f748d)
✅ **AgilePlus Specs**: PRD/ADR/PLAN/FR/USER_JOURNEYS confirmed in main (a36c97c)
✅ **GitHub Releases**: 9 orphaned releases published (a304466)
✅ **Worklogs Consolidation**: Merged to main (a26fb98)

**Remaining PRs to Merge**:
- phenotype-shared, phenotype-nexus, phenotype-gauge, cliproxyapi-plusplus, ccusage, aizen, heliosCLI
- Expect 10-15 more PRs from governance audit

---

## Immediate Next Actions

**Priority 1** (Blocking): Get user input on 4 critical decisions above
**Priority 2** (High): Approve or modify versioning strategy
**Priority 3** (High): Review architectural patterns and select implementation priority
**Priority 4** (Medium): Merge pending CHANGELOG/governance PRs once decisions confirmed
**Priority 5** (Medium): Deploy @phenotype/docs to GitHub Packages and integrate into all consumer projects

---

## Timeline Estimate

Once user provides input on blocking decisions:
- Resolve zen-wtrees, vibe-kanban, phenoSDK, 4sgm: 1-2 hours (cleanup agents)
- Merge pending PRs: 30 minutes (gh pr merge automation)
- Implement Phase 1 (Chassis): 4-6 hours (documentation + interface design)
- Publish @phenotype/docs: 2-3 hours (workflow + GitHub Packages setup)
- Integrate into 8+ consumer projects: 6-8 hours (parallel agents, 3-4 per batch)

**Total for MVP** (Phase 1 + Publishing + 1st batch): ~12-15 hours wall clock

---

## See Also

- `docs/reference/ARCHITECTURAL_PATTERNS_APPLICATION.md` — Complete pattern analysis
- `VERSIONING_STRATEGY_SUMMARY.md` — Versioning details
- Original audit reports from agents a36c97c, a617f74, a36a7aa

