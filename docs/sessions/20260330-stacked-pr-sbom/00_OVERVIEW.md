# Session: Stacked PR delivery — CycloneDX SBOM pilot (2026-03-30)

## Goal

Land supply-chain automation as **small, reviewable PRs** (stacked / layered). **Create and open PRs first**; batch merges after review.

## Stack (merge order)

| Order | Branch | Targets | Contents |
|------:|--------|---------|----------|
| 1 | `chore/sbom-cyclonedx-pilot` | `main` | `.github/workflows/sbom.yml` only |
| 2 | `chore/docs-tooling-sbom-stack` | `chore/sbom-cyclonedx-pilot` | `docs/worklogs/DEPENDENCIES.md` (pilot documentation) |
| 3 | `chore/session-stacked-sbom-delivery` | `chore/docs-tooling-sbom-stack` | This session note |

## Opened PRs (2026-03-30)

| PR | Link |
|----|------|
| 1 | https://github.com/KooshaPari/phenotype-infrakit/pull/99 |
| 2 | https://github.com/KooshaPari/phenotype-infrakit/pull/100 |
| 3 | https://github.com/KooshaPari/phenotype-infrakit/pull/101 |

## After PR 1 merges

- Rebase or retarget PR 2 to `main` (or merge PR 2 into updated base as per GitHub stacked-PR workflow).
- Same for PR 3 after PR 2 lands.

## Commands reference

```bash
# Push stack (from repo root worktrees)
git -C worktrees/chore-sbom-cyclonedx push -u origin chore/sbom-cyclonedx-pilot
git -C worktrees/chore-docs-sbom-stack push -u origin chore/docs-tooling-sbom-stack
git -C worktrees/chore-session-sbom-stack push -u origin chore/session-stacked-sbom-delivery

gh pr create --base main --head chore/sbom-cyclonedx-pilot --title "ci(sbom): CycloneDX pilot for phenotype-error-core" --body "See docs/sessions/20260330-stacked-pr-sbom/00_OVERVIEW.md"
gh pr create --base chore/sbom-cyclonedx-pilot --head chore/docs-tooling-sbom-stack --title "docs(deps): SBOM pilot + stacked merge order" --body "Stacked on SBOM workflow PR."
gh pr create --base chore/docs-tooling-sbom-stack --head chore/session-stacked-sbom-delivery --title "docs(sessions): stacked PR delivery note for SBOM pilot" --body "Stacked on deps doc PR."
```

## Success criteria

- [x] PR 1 open (merge triggers `SBOM (CycloneDX pilot)` workflow when Actions billing allows).
- [ ] Artifact `cyclonedx-sbom-phenotype-error-core` present on workflow run.
- [x] PR 2 and PR 3 open with correct base branches.

---

_Last updated: 2026-03-30_
