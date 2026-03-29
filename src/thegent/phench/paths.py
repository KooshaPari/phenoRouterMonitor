from __future__ import annotations

import os
import re
from fnmatch import fnmatch
from pathlib import Path

_IDENTIFIER_RE = re.compile(r"^[A-Za-z0-9._-]+$")


def phenotype_root() -> Path:
    configured = os.environ.get("THGENT_PHENOTYPE_ROOT")
    if configured:
        return Path(configured).expanduser().resolve()
    return (Path.home() / "CodeProjects" / "Phenotype").resolve()


def projects_root() -> Path:
    return phenotype_root() / "projects"


def phenotype_repos_root() -> Path:
    """Return the parent directory that usually contains sibling project checkouts."""
    env_root = os.environ.get("THGENT_PHENOTYPE_REPOS_ROOT")
    if env_root:
        return Path(env_root).expanduser().resolve()
    return _derive_default_repos_root()


def _derive_default_repos_root() -> Path:
    cwd = Path.cwd().resolve()
    for current in [cwd] + list(cwd.parents):
        if current.name == "repos":
            return current
    return phenotype_root() / "repos"


def should_include_repo(
    repo_name: str,
    include_patterns: list[str] | None = None,
    exclude_patterns: list[str] | None = None,
) -> bool:
    """Match repository name against optional include/exclude wildcard patterns."""
    if include_patterns:
        if not any(fnmatch(repo_name, pattern) for pattern in include_patterns):
            return False
    if exclude_patterns:
        if any(fnmatch(repo_name, pattern) for pattern in exclude_patterns):
            return False
    return True


def home_mirror_root() -> Path:
    configured = os.environ.get("THGENT_PHENCH_HOME_ROOT")
    if configured:
        return Path(configured).expanduser().resolve()
    return Path.home() / "phench"


def validate_target_name(target: str) -> str:
    return _validate_identifier(target, _IDENTIFIER_RE, "target name")


def validate_family_name(family: str) -> str:
    return _validate_identifier(family, _IDENTIFIER_RE, "family name")


def _validate_identifier(
    value: str,
    pattern: re.Pattern[str],
    label: str,
) -> str:
    candidate = value.strip()
    if not candidate:
        raise ValueError(f"{label} cannot be empty")
    if "/" in candidate or "\\" in candidate or ".." in candidate:
        raise ValueError(f"invalid {label}: {candidate}")
    if not pattern.fullmatch(candidate):
        raise ValueError(f"invalid {label}: {candidate}")
    return candidate


def target_root(target: str, family: str | None = None) -> Path:
    target_name = validate_target_name(target)
    if family is None:
        return projects_root() / target_name
    return projects_root() / validate_family_name(family) / target_name


def target_repos_root(target: str, family: str | None = None) -> Path:
    return target_root(target, family=family) / "repos"


def target_state_root(target: str, family: str | None = None) -> Path:
    return target_root(target, family=family) / ".phench"


def mirror_target_state_root(target: str, family: str | None = None) -> Path:
    if family is None:
        return home_mirror_root() / target / ".phench"
    return home_mirror_root() / validate_family_name(family) / target / ".phench"


def projects_modules_root() -> Path:
    return projects_root() / "modules"


def module_manifest_root(module: str) -> Path:
    normalized = validate_target_name(module)
    return projects_modules_root() / normalized


def module_manifest_path(module: str) -> Path:
    return module_manifest_root(module) / "manifest.json"


def repository_root_candidates() -> list[Path]:
    """Return candidate repository roots for shared module scanning."""
    base = phenotype_root() / "repos"
    if not base.exists():
        return []
    return sorted([path for path in base.iterdir() if path.is_dir() and not path.name.startswith(".")])


def _load_json_file(path: Path) -> dict:
    """Load a JSON file, handling orjson/json differences."""
    import json as _json
    from pathlib import Path as _Path
    text = path.read_text(encoding="utf-8")
    try:
        import orjson
        return orjson.loads(text)
    except Exception:
        return _json.loads(text)


def module_manifests_root() -> Path:
    """Return the root directory for all module manifests."""
    return projects_root() / "modules"
