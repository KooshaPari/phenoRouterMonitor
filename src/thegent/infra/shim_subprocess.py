"""Shim-aware subprocess runner.

Provides shim_run that uses thegent-shims when available,
with transparent fallback to standard subprocess.

This enables gradual migration of subprocess calls to use Rust shims.
"""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path
from typing import Any


def _get_shim_path() -> str | None:
    """Get thegent-shims path if available."""
    # Check if shims are in PATH
    shim_path = shutil.which("thegent-shims")
    if shim_path:
        return shim_path

    # Check common locations
    common_paths = [
        Path("~/.local/bin/thegent-shims").expanduser(),
        "/usr/local/bin/thegent-shims",
        Path("~/.cargo/bin/thegent-shims").expanduser(),
    ]
    for path in common_paths:
        if os.path.isfile(path) and os.access(path, os.X_OK):
            return path

    return None


# Check availability at import time
SHIM_AVAILABLE = _get_shim_path() is not None


# Commands that have shims
SHIM_COMMANDS = {
    "git": "thegent-git",
    "grep": "thegent-grep",
    "find": "thegent-find",
    "rg": "thegent-grep",
    "fd": "thegent-find",
    "jq": "thegent-jq",
    "jaq": "thegent-jq",
}


def run(
    args: list[str],
    **kwargs: Any,
) -> subprocess.CompletedProcess:
    """Shim-aware shim_run.

    Uses thegent-shims for supported commands when available,
    falls back to standard subprocess.

    Args:
        args: Command arguments (e.g., ["git", "status"])
        **kwargs: Additional shim_run arguments

    Returns:
        subprocess.CompletedProcess
    """
    if not SHIM_AVAILABLE or kwargs.get("shell") or not args:
        return subprocess.run(args, **kwargs)

    cmd = args[0]
    shim_cmd = SHIM_COMMANDS.get(cmd)

    if shim_cmd and "thegent" not in cmd:
        # Use shim wrapper
        shim_args = [shim_cmd] + args[1:]
        return subprocess.run(shim_args, **kwargs)

    return subprocess.run(args, **kwargs)


def check_output(
    args: list[str],
    **kwargs: Any,
) -> str:
    """Shim-aware subprocess.check_output."""
    if not SHIM_AVAILABLE or kwargs.get("shell") or not args:
        return subprocess.check_output(args, **kwargs).decode()

    cmd = args[0]
    shim_cmd = SHIM_COMMANDS.get(cmd)

    if shim_cmd and "thegent" not in cmd:
        shim_args = [shim_cmd] + args[1:]
        return subprocess.check_output(shim_args, **kwargs).decode()

    return subprocess.check_output(args, **kwargs).decode()
