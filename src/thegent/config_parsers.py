"""Shared parsing helpers used by :mod:`thegent.config`."""

import orjson as json
import os
from pathlib import Path

TRUE_STRINGS = {"1", "true", "yes", "on"}


def parse_retention_by_domain(value: object) -> dict[str, int]:
    """Parse retention-by-domain from JSON string or dict input."""
    if isinstance(value, str):
        try:
            parsed = json.loads(value)
            if isinstance(parsed, dict):
                return {str(k): int(val) if isinstance(val, (int | float | str)) else 0 for k, val in parsed.items()}
            return {}
        except (json.JSONDecodeError, ValueError, TypeError):
            return {}
    if isinstance(value, dict):
        return {str(k): int(val) if isinstance(val, (int | float | str)) else 0 for k, val in value.items()}
    return {}


def parse_csv_or_list(value: object, default: list[str]) -> list[str]:
    """Parse comma-separated string or list-like into a list of strings."""
    if isinstance(value, str):
        return [item.strip() for item in value.split(",") if item.strip()]
    if isinstance(value, list):
        return [str(item) for item in value]
    return list(default)


def parse_bool_or_env_flag(value: object, env_var: str) -> bool:
    """Parse bool-like values, then fall back to `ENV_VAR=="1"` semantics."""
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.lower() in TRUE_STRINGS
    return os.environ.get(env_var) == "1"


def parse_optional_path(value: object, env_var: str) -> Path | None:
    """Parse optional path-like value, or auto-detect from environment."""
    if isinstance(value, (str | Path)) and value:
        return Path(value) if isinstance(value, str) else value
    detected = os.environ.get(env_var)
    return Path(detected) if detected else None


def parse_shell_path(value: object, env_var: str, default: str) -> str:
    """Parse shell path while preserving explicit non-default values."""
    if isinstance(value, str) and value and value != default:
        return value
    shell = os.environ.get(env_var, default)
    return shell or default


def parse_first_nonempty_env(value: object, env_vars: list[str]) -> str:
    """Return explicit value first, else first non-empty env var in order."""
    if isinstance(value, str) and value:
        return value
    for env_var in env_vars:
        candidate = os.environ.get(env_var, "").strip()
        if candidate:
            return candidate
    return ""
