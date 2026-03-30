# Session: Stacked PR delivery — CycloneDX SBOM pilot (2026-03-30)

## Goal

Land supply-chain automation as **small, reviewable PRs** (stacked / layered). **Create and open PRs first**; batch merges after review.

## Outcome (2026-03-30)

- Workflow `.github/workflows/sbom.yml` is on `main` from **[#95](https://github.com/KooshaPari/phenotype-infrakit/pull/95)** (earlier SBOM PR).
- Stacked PRs **[#99](https://github.com/KooshaPari/phenotype-infrakit/pull/99)**–**[#101](https://github.com/KooshaPari/phenotype-infrakit/pull/101)** were **closed without merge**; this session doc and `DEPENDENCIES.md` pilot section were not on `main` until a **consolidated follow-up PR** (`chore/sbom-docs-session`) rebased onto current `main`.
- **Consolidated PR** adds: `DEPENDENCIES.md` pilot documentation, this session note, and a **matrix** of CycloneDX jobs for seven workspace members (replacing the single-crate job while keeping the same workflow file).

## Stack (original plan — historical)

| Order | Branch | Targets | Contents |
|------:|--------|---------|----------|
| 1 | `chore/sbom-cyclonedx-pilot` | `main` | `.github/workflows/sbom.yml` only |
| 2 | `chore/docs-tooling-sbom-stack` | `chore/sbom-cyclonedx-pilot` | `docs/worklogs/DEPENDENCIES.md` (pilot documentation) |
| 3 | `chore/session-stacked-sbom-delivery` | `chore/docs-tooling-sbom-stack` | This session note |

## Commands reference

```bash
git fetch origin
git checkout -B chore/sbom-docs-session origin/main
git cherry-pick <docs-commit> <session-commits>
# Then amend workflow matrix + push

git push -u origin chore/sbom-docs-session
gh pr create --base main --head chore/sbom-docs-session --fill
```

## Success criteria

- [x] `sbom.yml` on `main` generates CycloneDX JSON (matrix); workflow runs when Actions billing allows.
- [x] Artifacts `cyclonedx-sbom-<crate-id>` for each matrix row.
- [x] `DEPENDENCIES.md` and session doc aligned with workflow behavior.

---

_Last updated: 2026-03-30_
