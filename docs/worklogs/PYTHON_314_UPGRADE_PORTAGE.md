# Python 3.14 Upgrade Plan - portage

**Project:** portage
**Status:** planning
**Target:** Python 3.14

---

## Current State

| Item | Current | Notes |
|------|---------|-------|
| `requires-python` | `>=3.12` | pyproject.toml |
| `.python-version` | `3.13` | Current local version |
| `uv.lock` | Present | Using uv for package management |
| CI runners | Python 3.12 | GitHub Actions matrix |

## Upgrade Checklist

### Phase 1: Pre-upgrade (Before 3.14 release)
- [ ] Monitor Python 3.14 release schedule
- [ ] Verify all dependencies support 3.14
- [ ] Check CI runner availability (GitHub Actions)

### Phase 2: Preparation (When 3.14 available)
- [ ] Update `requires-python = ">=3.14"` in pyproject.toml
- [ ] Update `.python-version` to `3.14`
- [ ] Run `uv python pin 3.14` to update lockfile
- [ ] Test with `uv sync --python 3.14`

### Phase 3: CI Updates
- [ ] Update `pytest.yml` matrix to include 3.14
- [ ] Update `ruff-format.yml` Python version
- [ ] Update any other workflow pins
- [ ] Verify all jobs pass with 3.14

### Phase 4: Validation
- [ ] Run full test suite with 3.14
- [ ] Run type checks (pyright/mypy)
- [ ] Run linting (ruff)
- [ ] Verify no deprecation warnings

## Dependencies to Verify

Key dependencies that need 3.14 support confirmation:
- `pydantic>=2.11.7` - Should support
- `typer>=0.16.0` - Should support  
- `httpx>=0.28.0` - Should support
- `fastapi>=0.128.0` - Should support
- `litellm>=1.79.0` - Check compatibility
- `tenacity>=9.1.2` - Should support

## Risk Assessment

| Risk | Level | Mitigation |
|------|-------|------------|
| Dependency incompatibility | MEDIUM | Test early, report issues |
| CI runner not available | LOW | Use self-hosted or wait |
| Type check failures | LOW | Fix incrementally |

## Timeline

- **Python 3.14 release**: Expected mid-2026
- **Portage upgrade**: 1-2 weeks after 3.14 stable
- **Full validation**: 1 week

---

_Last updated: 2026-04-03_