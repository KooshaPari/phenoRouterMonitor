"""Process state helpers extracted from CLI impl (WL-125)."""

from __future__ import annotations

import os


def is_pid_running(pid: int) -> bool:
    if pid <= 0:
        return False
    try:
        os.kill(pid, 0)
        return True
    except OSError:
        return False
