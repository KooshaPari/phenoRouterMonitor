# Phench Phase 1: Foundation Consolidation

## Summary

Foundation phase for Phench consolidation: model unification, legacy code cleanup, error standardization. Eliminates duplicate model definitions, deletes legacy git_ops.py, and standardizes error handling across the codebase.

## Goals

- Unify model definitions (resolve dual-definition issue)
- Delete legacy git_ops.py module
- Standardize error handling patterns
- Establish foundation for Phase 2 architecture refactoring

## Success Criteria

- ✓ Models unified: single definition in domain/models.py
- ✓ git_ops.py deleted, all callers migrated to SubprocessGitAdapter
- ✓ 50+ ValueError calls replaced with domain exceptions
- ✓ All tests pass (1,112 LOC test suite)
- ✓ No compiler/clippy warnings

## Scope

**LOC Savings Target**: 251-281 LOC
- Model unification: 88 LOC
- Delete git_ops.py: 113 LOC
- Error standardization: 50-80 LOC

**Files Affected**: ~25 files

## Work Packages

### WP01: Analyze Model Duplication
**Effort**: 1-2 min | **Status**: planned
- Identify all differences between root models.py and domain/models.py
- Document RepoSelection variant fields
- Map all import statements across codebase
- Create consolidation strategy document

### WP02: Merge Model Definitions
**Effort**: 3-4 min | **Status**: planned | **Depends on**: WP01
- Consolidate root models.py into domain/models.py
- Handle RepoSelection field variants
- Update all 20+ import statements
- Add backwards-compat re-exports in root __init__.py
- Run tests

### WP03: Delete git_ops.py
**Effort**: 3-4 min | **Status**: planned | **Depends on**: WP02
- Migrate pure utility functions (sanitize_repo_id, etc.) to infrastructure/git_util.py
- Update 15+ callers to use SubprocessGitAdapter
- Verify no code duplication with infrastructure/git.py
- Run tests

### WP04: Standardize Error Handling
**Effort**: 4-5 min | **Status**: planned | **Depends on**: WP03
- Create ValidationHelper class in domain/exceptions.py
- Replace 50+ raise ValueError(...) calls with domain exceptions
- Create adapter functions in infrastructure/ to convert stdlib errors
- Update error propagation patterns
- Run tests

### WP05: Integration Testing & Verification
**Effort**: 2-3 min | **Status**: planned | **Depends on**: WP04
- Run full test suite (1,112 LOC)
- Verify no performance regression
- Check for uncaught warnings
- Validate hexagonal architecture still intact

## Risks & Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Import churn | MEDIUM | Map all callers before refactoring, feature branch |
| Test failures | MEDIUM | Run tests after each WP, incremental approach |
| Backwards compatibility | LOW | Re-exports in root __init__.py for 1 release |

## Timeline

- **Duration**: 1 week
- **Effort**: 5-8 min active agent work
- **Effort Units**: ~15 tool calls

## Related

- `docs/worklogs/PHENCH_COMPREHENSIVE_LOC_AUDIT_2026-03-29.md` — Full audit report
- `docs/worklogs/LOC_AUDIT_DEEP_FINDINGS_2026-03-29.md` — Consolidated findings
- Phench Phase 2: Architecture Refactoring (depends on Phase 1)

## Notes

- Phench has well-structured hexagonal architecture; consolidation maintains this pattern
- Low technical debt; high structural redundancy
- Phase 1 is low-risk, high-impact foundation for Phase 2
