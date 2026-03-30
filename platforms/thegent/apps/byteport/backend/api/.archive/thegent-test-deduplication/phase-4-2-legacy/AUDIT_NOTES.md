# Phase 4.2 Legacy Tests Audit

## Archived Files

### 1. legacy_auth_handlers_test.go (340 lines)
**Status**: Archived for future audit  
**Reason**: Tests legacy authentication handler implementations  
**Action**: Requires code audit to determine if auth handlers are still in use

### 2. legacy_optional_auth_middleware_uncovered_test.go (520 lines)
**Status**: Archived for future audit  
**Reason**: Tests deprecated optional auth middleware pattern  
**Action**: Requires verification that middleware code is not still in active use

## Audit Checklist

For each file, complete the following during manual review:

### legacy_auth_handlers_test.go
- [ ] Search codebase for `legacy_auth_handlers.go` or similar
- [ ] Check if `legacy_auth_handlers` package/functions are referenced
- [ ] Verify if tested functions are used in current auth flow
- [ ] Decision:
  - [ ] Keep (if still used) - move back to main
  - [ ] Merge into modern auth tests (if partially used)
  - [ ] Archive (if fully obsolete)

### legacy_optional_auth_middleware_uncovered_test.go
- [ ] Search for `optional_auth_middleware` in codebase
- [ ] Check middleware package for legacy versions
- [ ] Verify if tested code paths are still executed
- [ ] Decision:
  - [ ] Keep (if middleware still in use)
  - [ ] Merge into current middleware tests
  - [ ] Archive (if superseded by modern middleware)

## Estimated LOC Savings

- If both files are fully obsolete: 860 LOC
- If both files are partially used: 400-500 LOC
- If both files are still needed: 0 LOC

## Next Steps

1. Run codebase search for legacy function names
2. Check git history for last usage dates
3. Execute decision matrix above
4. Either restore files or keep archived
5. Document final decision in this audit file

---

**Note**: These files are archived non-destructively. They can be restored at any time if further analysis determines they should be kept or merged.
