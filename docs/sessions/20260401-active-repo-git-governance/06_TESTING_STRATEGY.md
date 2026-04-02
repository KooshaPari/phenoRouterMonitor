# Testing Strategy

## Validation Performed

- read-only audit of repo `.github` surfaces across active repos
- live `gh` queries for ruleset presence and sampled open PR status
- YAML syntax validation for new or edited workflow files
- conflict-marker scan for canonical shelf docs touched by this lane

## Follow-Up Validation

- once GitHub rulesets are updated server-side, re-query and compare against
  `docs/reference/ACTIVE_REPO_GIT_RULESETS.md`
- for repos with new policy gates, ensure the workflow job name is pinned exactly in the ruleset
- for repos where secret scanning is now hard-fail, inspect the first PR run for false positives
