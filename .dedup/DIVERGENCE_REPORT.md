# Test File Divergence Report

**Generated**: 2026-03-30
**Analysis**: SHA256 hash comparison of all test file duplicates

## Summary

During deduplication analysis, **18 test files** were found to have diverged content between canonical and worktree copies. These files must be reviewed manually before removal to ensure no test-specific customizations are lost.

**Action**: Keep all copies of diverged tests; only remove identical duplicates.

## Files with Diverged Content (18 Total)

### 1. test_batch_file_ops.py
- **Canonical**: `platforms/thegent/tests/test_batch_file_ops.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/thegent/utils/test_batch_file_ops.py`
- **Status**: KEEP BOTH (content differs)

### 2. test_cache.py
- **Canonical**: `platforms/thegent/tests/mesh/test_cache.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/routing/test_cache.py`
- **Status**: KEEP BOTH (content differs)

### 3. test_cli.py
- **Canonical**: `platforms/thegent/tests/docs_engine/test_cli.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/research_engine/test_cli.py`
- **Status**: KEEP BOTH (content differs)

### 4. test_cross_project.py
- **Canonical**: `platforms/thegent/tests/ipc/test_cross_project.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/registry/test_cross_project.py`
- **Status**: KEEP BOTH (content differs)

### 5. test_enterprise_compliance.py
- **Canonical**: `platforms/thegent/tests/governance/test_enterprise_compliance.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_enterprise_compliance.py`
- **Status**: KEEP BOTH (content differs)

### 6. test_git_parallelism.py
- **Canonical**: `platforms/thegent/tests/mesh/test_git_parallelism.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/unit/test_git_parallelism.py`
- **Status**: KEEP BOTH (content differs)

### 7. test_injection.py
- **Canonical**: `platforms/thegent/tests/mesh/test_injection.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/routing/guardrails/test_injection.py`
- **Status**: KEEP BOTH (content differs)

### 8. test_mcp_tools.py
- **Canonical**: `platforms/thegent/tests/docs_engine/test_mcp_tools.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/research_engine/test_mcp_tools.py`
- **Status**: KEEP BOTH (content differs)

### 9. test_observability.py
- **Canonical**: `platforms/thegent/tests/mesh/test_observability.py`
- **Diverged Copy**: `.worktrees/feat/phenosdk-decompose-core/python/pheno-core/tests/test_observability.py`
- **Status**: KEEP BOTH (content differs)

### 10. test_path_utils.py
- **Canonical**: `platforms/thegent/tests/test_path_utils.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/thegent/utils/test_path_utils.py`
- **Status**: KEEP BOTH (content differs)

### 11. test_project_registry.py
- **Canonical**: `platforms/thegent/tests/registry/test_project_registry.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_project_registry.py`
- **Status**: KEEP BOTH (content differs)

### 12. test_resilience.py (2 diverged copies)
- **Canonical**: `platforms/thegent/tests/chaos/test_resilience.py`
- **Diverged Copy 1**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_resilience.py`
- **Diverged Copy 2**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/unit/test_resilience.py`
- **Status**: KEEP ALL (content differs)

### 13. test_runner.py
- **Canonical**: `platforms/thegent/apps/byteport/scripts/test_runner.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/maif/test_runner.py`
- **Status**: KEEP BOTH (content differs)

### 14. test_schema.py (2 diverged copies)
- **Canonical**: `heliosCLI/harness/tests/test_schema.py`
- **Diverged Copy 1**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/research_engine/test_schema.py`
- **Diverged Copy 2**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_schema.py`
- **Status**: KEEP ALL (content differs)

### 15. test_session_hook.py
- **Canonical**: `platforms/thegent/tests/docs_engine/test_session_hook.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/research_engine/test_session_hook.py`
- **Status**: KEEP BOTH (content differs)

### 16. test_store.py
- **Canonical**: `phench/tests/test_store.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/research_engine/test_store.py`
- **Status**: KEEP BOTH (content differs)

### 17. test_sub_agent_dispatcher.py
- **Canonical**: `platforms/thegent/tests/agents/test_sub_agent_dispatcher.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/orchestration/test_sub_agent_dispatcher.py`
- **Status**: KEEP BOTH (content differs)

### 18. test_wl185_reflection_rollback.py
- **Canonical**: `platforms/thegent/tests/integrations/test_wl185_reflection_rollback.py`
- **Diverged Copy**: `platforms/worktrees/thegent/chore/sync-docs-security-deps/tests/test_wl185_reflection_rollback.py`
- **Status**: KEEP BOTH (content differs)

## Recommendations

1. **Investigate divergences**: Determine if diverged copies represent:
   - Test-specific customizations for worktree tasks
   - Unintended regressions or missing cherry-picks
   - Necessary branch-specific tests

2. **Merge or consolidate**: For each diverged test:
   - If divergence is intentional (worktree-specific test): keep both, document in PR
   - If divergence is a mistake: merge diverged content back to canonical
   - If worktree test is outdated: discard and use canonical

3. **Document decision**: Update this report with final decision per file

## Impact on Deduplication

- **Total identical duplicates removable**: 1,268 files - 18 diverged = ~1,250 identical copies
- **Expected savings from identical removal**: ~9.5-9.7 MB
- **Additional review needed**: 18 diverged copies (requires human judgment)

---

**Next Step**: Review divergences and update PR description accordingly.
