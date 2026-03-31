# thegent Pilot Guide

thegent is a Python project targeting PyPI for alpha pre-release publishing.

## Bootstrap

```bash
cd /path/to/thegent
pheno-cli bootstrap --lang python --registry pypi
```

## Expected Outputs

### mise.toml

```toml
[tools]
python = "3.12"

[env]
PYPI_REGISTRY = "pypi"
```

### CI Workflows

`.github/workflows/ci.yml` — runs on every push/PR:
- `pytest`
- `ruff check`
- `mypy`

`.github/workflows/release.yml` — runs on version tags matching `v*a*`:
- `pytest`
- `python -m build`
- `twine upload dist/*` to PyPI

### Git Hooks

`pre-commit`:
- Runs `ruff check --fix`
- Runs `mypy`

`commit-msg`:
- Enforces conventional commit format

## Alpha Publish Steps

1. Bump version to alpha:
   ```bash
   pheno-cli version bump --channel alpha --increment 1
   # Sets pyproject.toml version to: 0.2.0a1
   ```

2. Commit and tag:
   ```bash
   git add pyproject.toml
   git commit -m "chore: bump version to 0.2.0a1"
   git tag v0.2.0a1
   git push origin main --tags
   ```

3. CI picks up the tag and publishes to PyPI.

## Expected Pre-release Format

```
0.2.0a1
```

PyPI uses PEP 440 pre-release syntax. pheno-cli generates this format when registry is `pypi` and channel is `alpha`.

Format mapping:
- `alpha` → `aN`
- `beta` → `bN`
- `rc` → `rcN`
- `dev` / `canary` → `.devN`

## Validation

```bash
pheno-cli validate --repo .
```

Expected output:
```
[PASS] mise.toml exists
[PASS] CI workflows installed
[PASS] Git hooks installed
[PASS] pyproject.toml version format: 0.2.0a1
[PASS] All checks passed
```
