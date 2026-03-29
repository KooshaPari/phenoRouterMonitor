"""Session identifier helpers extracted from CLI impl (WL-125)."""

from __future__ import annotations

import hashlib
import os
import time
from datetime import UTC, datetime


def new_session_id(agent: str | None, owner: str) -> str:
    now = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    digest = hashlib.sha1(f"{time.time_ns()}:{os.getpid()}:{owner}".encode()).hexdigest()[:8]
    agent_tag = agent or "any"
    return f"{now}-{agent_tag}-p{os.getpid()}-{digest}"
