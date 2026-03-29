"""Session path helper services extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

from pathlib import Path


def session_paths(base: Path, session_id: str) -> dict[str, Path]:
    """Build canonical file paths for a session under a scope directory."""
    return {
        "meta": base / f"{session_id}.json",
        "stdout": base / f"{session_id}.stdout.log",
        "stderr": base / f"{session_id}.stderr.log",
        "rc": base / f"{session_id}.rc",
        "in": base / f"{session_id}.in",
    }


__all__ = ["session_paths"]
