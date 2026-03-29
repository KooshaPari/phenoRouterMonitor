"""Session/model helper facade extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

import time
from pathlib import Path
from typing import Any

import typer
from cachetools import TTLCache

from thegent.cli.commands import session_owner_helpers
from thegent.cli.services import run_model_helpers, session_id_helpers, session_path_helpers
from thegent.config import ThegentSettings

# QW-002: _resolve_cwd() cache with TTL to reduce path resolution overhead.
# Mission-Critical Rigor (G-FM-04): Use TTLCache for automatic expiration.
_CWD_CACHE: TTLCache[str, tuple[Path | None, float]] = TTLCache(maxsize=1000, ttl=10.0)


def resolve_droids_dir(cwd: Path | None, settings: ThegentSettings) -> Path:
    """Resolve droids dir: project .factory/droids first, then config."""
    if cwd and (cwd / ".factory" / "droids").exists():
        return (cwd / ".factory" / "droids").resolve()
    return settings.factory_droids_dir.expanduser().resolve()


def resolve_cwd(cd: Any) -> Path | None:
    """Resolve cwd: explicit --cd, or infer from current dir if project-like."""
    global _CWD_CACHE
    now = time.time()

    # Canonical cache key follows resolved path when available so tests can
    # consistently retrieve deterministic values and clear specific entries.
    if cd is not None:
        try:
            cache_key = str(cd.expanduser().resolve())
        except Exception:
            cache_key = str(cd)
    else:
        cache_key = f"none:{Path.cwd().resolve()}"

    if cache_key in _CWD_CACHE:
        cached_p, _ = _CWD_CACHE[cache_key]
        return cached_p

    resolved_p: Path | None = None
    if cd is not None:
        try:
            p = cd.expanduser().resolve()
        except Exception:
            p = Path(cd).resolve()
        if not p.is_dir():
            raise typer.BadParameter(f"Directory does not exist: {p}")
        resolved_p = p
    else:
        cwd = Path.cwd().resolve()

        if (cwd / ".git").exists() or (cwd / ".factory").exists() or (cwd / "pyproject.toml").exists():
            resolved_p = cwd
        elif (cwd.parent / ".factory").exists() and cwd.parent != cwd:
            resolved_p = cwd.parent
        else:
            resolved_p = None

    _CWD_CACHE[cache_key] = (resolved_p, now)
    return resolved_p


def resolve_agent_model(
    *,
    agent: str,
    model: str | None,
    mode: str,
    settings: ThegentSettings,
) -> str | None:
    return run_model_helpers.resolve_agent_model(agent=agent, model=model, mode=mode, settings=settings)


def scope_key(owner: str) -> str:
    return session_owner_helpers.scope_key(owner)


def default_owner_tag(cwd: Path | None = None, *, include_process_id: bool = False) -> str:
    return session_owner_helpers.default_owner_tag(cwd, include_process_id=include_process_id)


def compose_owner_tag(user: str, cwd: Path, scope: str = "") -> str:
    return session_owner_helpers.compose_owner_tag(user=user, cwd=cwd, scope=scope)


def session_dir(settings: ThegentSettings, owner: str) -> Path:
    return session_owner_helpers.session_dir(settings, owner)


def session_scope_dirs(base: Path, owner: str) -> list[Path]:
    return session_owner_helpers.session_scope_dirs(base, owner)


def session_paths(*, base: Path, session_id: str) -> dict[str, Path]:
    return session_path_helpers.session_paths(base=base, session_id=session_id)


def new_session_id(*, agent: str | None, owner: str) -> str:
    return session_id_helpers.new_session_id(agent=agent, owner=owner)


__all__ = [
    "compose_owner_tag",
    "default_owner_tag",
    "new_session_id",
    "resolve_agent_model",
    "resolve_cwd",
    "resolve_droids_dir",
    "scope_key",
    "session_dir",
    "session_paths",
    "session_scope_dirs",
]
