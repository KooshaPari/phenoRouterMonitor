"""CLI helpers for thegent.

Common CLI utilities.
"""

from __future__ import annotations

import sys
from typing import Any


def print_error(msg: str) -> None:
    """Print error to stderr."""


def print_warning(msg: str) -> None:
    """Print warning to stdout."""


def confirm(prompt: str, default: bool = False) -> bool:
    """Ask for confirmation."""
    suffix = " [Y/n]" if default else "[y/N]"
    response = input(f"{prompt}{suffix}: ").strip().lower()
    if not response:
        return default
    return response in ("y", "yes")


def progress_bar(current: int, total: int, width: int = 40) -> str:
    """Create a progress bar string."""
    percent = current / total
    filled = int(width * percent)
    bar = "=" * filled + "-" * (width - filled)
    return f"[{bar}] {int(percent * 100)}%"
