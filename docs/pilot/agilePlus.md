# AgilePlus Pilot Guide

AgilePlus is a private TypeScript project. The pilot uses the `--private` flag to skip registry publish steps.

## Bootstrap

```bash
cd /path/to/AgilePlus
pheno-cli bootstrap --private --lang typescript
```

The `--private` flag disables any publish or release artifact steps. All validation and local build steps still run.

## Expected Outputs

### mise.toml

```toml
[tools]
node = "20.x"
bun = "1.x"

[env]
PHENO_PRIVATE = "true"
```

### CI Workflows

`.github/workflows/ci.yml` — runs on every push/PR:
- Type-check (`tsc --noEmit`)
- Lint (`eslint`)
- Tests (`bun test`)

`.github/workflows/release.yml` — runs on version tags:
- Build only (no publish)
- Archives build artifact locally

### Git Hooks

`pre-commit`:
- Runs `tsc --noEmit`
- Runs `eslint --fix-dry-run`

`commit-msg`:
- Enforces conventional commit format

## Private Artifact Handling

Because AgilePlus is private, pheno-cli does NOT:
- Publish to npm or any public registry
- Create public GitHub releases

pheno-cli DOES:
- Build and archive the artifact locally
- Tag the version in git
- Update `package.json` version field
- Generate a changelog entry

## Validation

```bash
pheno-cli validate --repo .
```

Expected output:
```
[PASS] mise.toml exists
[PASS] CI workflows installed
[PASS] Git hooks installed
[PASS] Private mode: no publish step configured
[PASS] All checks passed
```
