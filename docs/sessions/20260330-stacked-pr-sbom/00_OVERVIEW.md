# Session: Stacked PR delivery — CycloneDX SBOM pilot (2026-03-30)

## Goal

Land supply-chain automation as **small, reviewable PRs** (stacked / layered). **Create and open PRs first**; batch merges after review.

## Outcome (2026-03-30)

- Workflow `.github/workflows/sbom.yml` is on `main` from **[#95](https://github.com/KooshaPari/phenotype-infrakit/pull/95)** (earlier SBOM PR).
- Stacked PRs **[#99](https://github.com/KooshaPari/phenotype-infrakit/pull/99)**–**[#101](https://github.com/KooshaPari/phenotype-infrakit/pull/101)** were **closed without merge**; this session doc and `DEPENDENCIES.md` pilot section were not on `main` until a **consolidated follow-up PR** (`chore/sbom-docs-session`) rebased onto current `main`.
- **Consolidated PR** ([#139](https://github.com/KooshaPari/phenotype-infrakit/pull/139)) adds: `DEPENDENCIES.md` pilot documentation, this session note, and an expanded CycloneDX workflow (later superseded by script-driven generation + release assets).
- **2026-03-31 follow-up:** matrix expanded to **all** `[workspace.members]` (16 CycloneDX jobs + matching artifacts); see [#160](https://github.com/KooshaPari/phenotype-infrakit/pull/160).
- **2026-03-31 (continued):** `scripts/ci/generate-workspace-sboms.sh` replaces duplicated workflow matrix; `sbom.yml` uploads one bundle; `release.yml` attaches the same CycloneDX JSON files to **GitHub Releases** for `v*.*.*` tags; see **[#191](https://github.com/KooshaPari/phenotype-infrakit/pull/191)**.
- **Tag vs release:** `tag-automation.yml` now **only** pushes `v*` tags when `Cargo.toml` / `package.json` version changes on `main`; **`.github/workflows/release.yml`** is the sole creator of GitHub Releases (removes race with `softprops/action-gh-release`).
- **Supply-chain “next”:** `security.yml` adds **OSV-Scanner** on a generated `Cargo.lock` (SARIF → Code Scanning); `release.yml` adds **Syft** SPDX JSON next to CycloneDX assets on each version tag; single release owner in **[#225](https://github.com/KooshaPari/phenotype-infrakit/pull/225)**.
- **Worktree audit (Wave 106):** SBOM-related `worktrees/*` rows in `DEPENDENCIES.md` marked merged; operators may delete local clones when idle.

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

- [x] `sbom.yml` on `main` generates CycloneDX JSON for every workspace member; workflow runs when Actions billing allows.
- [x] CI artifact bundle `cyclonedx-sbom-workspace`; tagged releases include the same JSON files as downloadable assets.
- [x] `DEPENDENCIES.md` and session doc aligned with workflow behavior.
- [x] OSV results visible in GitHub Code Scanning when SARIF upload succeeds (org/repo policy permitting).

---

_Last updated: 2026-03-31_
