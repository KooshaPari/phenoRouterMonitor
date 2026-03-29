"""WP-2005: Poison pill detection for runaway agent output.

PoisonPillDetector scans agent output chunks for patterns that indicate:
- Infinite loops (same chunk repeated > 5 times in 10s)
- Output overflow (single chunk > 100 KB)
- Runaway tool use (> 200 tool_use occurrences in a single session)

When a poison pill is detected:
1. A 'poison_pill_detected' event is emitted to the governance log.
2. PoisonPillError is raised immediately — fail fast, fail loudly.

# @trace WL-039 WP-2005
"""

from __future__ import annotations

import logging
import time
from collections import deque
from dataclasses import dataclass, field
from typing import Final

_log = logging.getLogger(__name__)

# Thresholds
CHUNK_SIZE_LIMIT_BYTES: Final[int] = 100 * 1024  # 100 KB
REPEAT_WINDOW_SEC: Final[float] = 10.0
REPEAT_COUNT_LIMIT: Final[int] = 5
TOOL_USE_LIMIT: Final[int] = 200


class PoisonPillError(Exception):
    """Raised when a poison pill pattern is detected in agent output.

    Callers MUST NOT swallow this. The agent session should be aborted.
    """

    def __init__(self, reason: str, kind: str) -> None:
        self.reason = reason
        self.kind = kind
        super().__init__(f"PoisonPill[{kind}]: {reason}")


@dataclass
class _ChunkRecord:
    """Internal record of a seen chunk for repeat detection."""

    content: str
    timestamp: float


@dataclass
class PoisonPillDetector:
    """Stateful detector for poison pill patterns in a single agent session.

    Create one instance per agent session. Feed output chunks via scan_chunk().
    Feed tool-use events via record_tool_use().

    The detector is NOT thread-safe within a single session (sessions are
    single-threaded streams). Use a separate instance per session.
    """

    _recent_chunks: deque[_ChunkRecord] = field(default_factory=lambda: deque(maxlen=200))
    _tool_use_count: int = field(default=0)
    _governance_log: list[dict[str, object]] = field(default_factory=list)

    def scan_chunk(self, chunk: str) -> None:
        """Check a single output chunk for poison pill patterns.

        Args:
            chunk: A string output chunk from the agent.

        Raises:
            PoisonPillError: immediately if any pattern is triggered.
        """
        self._check_chunk_size(chunk)
        self._check_repeat(chunk)
        # Record after checks
        self._recent_chunks.append(_ChunkRecord(content=chunk, timestamp=time.monotonic()))

    def record_tool_use(self) -> None:
        """Increment the tool-use counter and check against TOOL_USE_LIMIT.

        Raises:
            PoisonPillError: when tool_use count exceeds TOOL_USE_LIMIT.
        """
        self._tool_use_count += 1
        if self._tool_use_count > TOOL_USE_LIMIT:
            self._emit_event("tool_use_overflow", f"tool_use count={self._tool_use_count}")
            raise PoisonPillError(
                reason=f"tool_use count {self._tool_use_count} exceeds limit {TOOL_USE_LIMIT}",
                kind="tool_use_overflow",
            )

    @property
    def tool_use_count(self) -> int:
        """Return the current tool-use count for this session."""
        return self._tool_use_count

    @property
    def governance_log(self) -> list[dict[str, object]]:
        """Return a copy of all emitted governance events."""
        return list(self._governance_log)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _check_chunk_size(self, chunk: str) -> None:
        size = len(chunk.encode("utf-8"))
        if size > CHUNK_SIZE_LIMIT_BYTES:
            self._emit_event("chunk_overflow", f"chunk_size={size} bytes")
            raise PoisonPillError(
                reason=f"Single output chunk is {size} bytes (limit {CHUNK_SIZE_LIMIT_BYTES})",
                kind="chunk_overflow",
            )

    def _check_repeat(self, chunk: str) -> None:
        now = time.monotonic()
        window_start = now - REPEAT_WINDOW_SEC
        recent_same = sum(1 for r in self._recent_chunks if r.timestamp >= window_start and r.content == chunk)
        if recent_same >= REPEAT_COUNT_LIMIT:
            self._emit_event(
                "repeat_chunk",
                f"chunk repeated {recent_same + 1} times in {REPEAT_WINDOW_SEC}s",
            )
            raise PoisonPillError(
                reason=(
                    f"Same chunk seen {recent_same + 1} times within {REPEAT_WINDOW_SEC}s (limit {REPEAT_COUNT_LIMIT})"
                ),
                kind="repeat_chunk",
            )

    def _emit_event(self, kind: str, detail: str) -> None:
        event: dict[str, object] = {
            "event": "poison_pill_detected",
            "kind": kind,
            "detail": detail,
            "tool_use_count": self._tool_use_count,
            "timestamp": time.time(),
        }
        self._governance_log.append(event)
        _log.critical(
            "POISON_PILL_DETECTED kind=%s detail=%s tool_use_count=%d",
            kind,
            detail,
            self._tool_use_count,
        )
