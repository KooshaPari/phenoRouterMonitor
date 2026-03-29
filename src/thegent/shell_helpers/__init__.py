"""Shell helpers for thegent.

Common shell utilities.
"""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any


def run(cmd: str, shell: bool = True, cwd: Path | None = None) -> tuple[int, str, str]:
    """Run shell command."""
    result = subprocess.run(
        cmd,
        shell=shell,
        cwd=cwd,
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout, result.stderr


def exists(cmd: str) -> bool:
    """Check if command exists."""
    code, _, _ = run(f"which {cmd}")
    return code == 0


def ensure_dir(path: Path) -> None:
    """Ensure directory exists."""
    path.mkdir(parents=True, exist_ok=True)


def file_exists(path: Path) -> bool:
    """Check if file exists."""
    return path.exists()
