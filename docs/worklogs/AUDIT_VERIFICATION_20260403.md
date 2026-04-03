# Audit Verification & Action Status - 2026-04-03

**Project:** [cross-repo]
**Status:** verified

---

## Library Integration Status (Verified)

### phenotype-infrakit Workspace

| Crate | Workspace Member | Used By Other Crates? |
|-------|-----------------|----------------------|
| phenotype-config-core | ✅ Yes (line 15) | ❌ NO - 0 imports |
| phenotype-error-core | ✅ Yes (line 18) | ❌ NO - 0 imports |
| phenotype-errors | ✅ Yes (line 19) | ❌ Used by PhenoProc only |
| phenotype-test-infra | ✅ Yes (line 30) | ❌ NO - 0 imports |
| phenotype-port-traits | ✅ Yes (line 22) | Need verification |

### Workspace Edition

```
[workspace.package]
edition = "2021"  # NOT 2024 as previously assumed
```

**Finding:** The workspace uses **edition = "2021"**, not 2024. Libraries use `edition.workspace = true` which resolves correctly. **No edition mismatch exists.**

---

## What Was Already Working

1. ✅ **TypeScript 7** - Already upgraded in heliosApp
2. ✅ **@tanstack/solid-query** - Already added to heliosApp
3. ✅ **extended_benchmark.py** - Already fixed (httpx only, no mixed imports)
4. ✅ **portage viewer** - Already modernized (single bun.lock)

---

## What Needs Manual Follow-up

### Libraries Exist But Unused (Adoption Issue)

| Library | Status | Issue |
|---------|--------|-------|
| phenotype-config-core | In workspace | Not imported by any crate |
| phenotype-error-core | In workspace | Not imported by any crate |
| phenotype-errors | In workspace | Only used by PhenoProc |
| phenotype-test-infra | In workspace | Not imported by any crate |

### Why They're Unused (Hypothesis)

1. **Missing features** - May not have all methods that domain crates need
2. **Awareness** - Developers don't know these libraries exist
3. **Migration path** - No documented process to switch from inline implementations
4. **Testing** - Need verification that adoption doesn't break tests

---

## Action Items for Next Steps

### Manual (Require Human Investigation)

1. **Audit phenotype-config-core features** - Does it have all methods needed by agileplus-domain loader?
2. **Audit phenotype-test-infra features** - What does it provide vs what projects need?
3. **Find APIClient** - The file location in docs doesn't match current codebase
4. **Adoption coordination** - Need to update crates to use these libraries

### Documentation Updates Needed

1. Update worklogs to reflect edition = "2021" (not 2024 mismatch)
2. Create adoption guides for unused libraries
3. Document the features each library provides

---

## Summary

| Category | Finding |
|----------|---------|
| Edition mismatch | ❌ **Does not exist** - workspace is 2021 |
| Unused libraries | ✅ All in workspace, just not adopted |
| Immediate code fixes | ⚠️ APIClient file not found - may be refactored |
| Documentation | 📝 Needs update to reflect actual state |