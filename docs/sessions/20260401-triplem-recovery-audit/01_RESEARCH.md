---
audience: [developers, agents]
---

# Research

## Git State

- branch: `main`
- last commit: `e9d8768` (`Production Draft`)
- local worktree is heavily dirty

## Damage Pattern

The diff is dominated by tracked deletions under:

- `public/*.html`
- `public/assets/css/*`
- `public/assets/fonts/*`
- `public/assets/img/*`
- `public/assets/js/*`
- `public/assets/sass/*`

Only two tracked files are modified:

- `package.json`
- `package-lock.json`

The package diff adds `@fullhuman/postcss-purgecss` to dev dependencies and corresponding lockfile
entries.

## Interpretation

The package diff looks like an experiment. The deletion volume looks like either:

- accidental removal,
- incomplete generated-asset migration,
- or a purge attempt that was never finished.

There is not enough evidence to treat the deletion set as intentional.
