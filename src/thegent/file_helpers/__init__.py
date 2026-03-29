"""File helpers for thegent.

Common file utilities.
"""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Any


def copy(src: Path, dst: Path) -> None:
    """Copy file or directory."""
    if src.is_dir():
        shutil.copytree(src, dst, dirs_exist_ok=True)
    else:
        shutil.copy2(src, dst)


def move(src: Path, dst: Path) -> None:
    """Move file or directory."""
    shutil.move(str(src), str(dst))


def remove(path: Path) -> None:
    """Remove file or directory."""
    if path.is_dir():
        shutil.rmtree(path)
    else:
        path.unlink()


def read_text(path: Path) -> str:
    """Read text file."""
    return path.read_text()


def write_text(path: Path, content: str) -> None:
    """Write text file."""
    path.write_text(content)
