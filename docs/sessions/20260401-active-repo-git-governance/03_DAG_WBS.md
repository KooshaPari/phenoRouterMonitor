# DAG / WBS

## Dependency Graph

1. Audit repo-local governance and live GitHub ruleset visibility.
2. Define shelf baseline and active-repo matrix.
3. Bootstrap weak repos with minimum PR-governance surfaces.
4. Reconcile shelf worklogs and session index.
5. Validate YAML and Markdown syntax and re-scan for conflict markers.

## Work Breakdown

### WP01 Audit

- enumerate active repos
- inspect `.github` surfaces
- query GitHub ruleset list and open PR status

### WP02 Shelf Baseline

- update `artifacts/active-repo-git-governance-baseline.md`
- update `docs/reference/ACTIVE_REPO_GIT_RULESETS.md`
- complete session bundle

### WP03 Repo Bootstrap

- `heliosApp`: add `CODEOWNERS`, PR template, ruleset baseline, `policy-gate`
- `forgecode`: add `CODEOWNERS`, PR template, ruleset baseline, docs CI, `policy-gate`
- `phenotype-infrakit`: keep bootstrap files in current lane as the repo-local contract

### WP04 Canonical Logging

- update `docs/sessions/index.md`
- update `docs/worklogs/WORK_LOG.md`
- fix `docs/worklogs/GOVERNANCE.md` conflict markers and keep one current entry
