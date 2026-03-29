"""Dead-Letter Replay Engine — reprocess failed writes after connector fixes.

Orchestrates deterministic replay of queued failures with result tracking.

# @trace WL-214
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from thegent.integrations.dead_letter_queue import DeadLetterEntry, DeadLetterQueue


@dataclass
class ReplayResult:
    """Result of a single replay attempt.

    Attributes:
        entry_id: The entry ID that was replayed.
        success: True if replay succeeded.
        error: Error message if replay failed (None if success=True).
    """

    entry_id: str
    success: bool
    error: str | None = None


class DeadLetterReplayEngine:
    """Engine for replaying dead-letter entries after connector fixes."""

    def __init__(self, dlq: DeadLetterQueue) -> None:
        """Initialize replay engine.

        Args:
            dlq: The DeadLetterQueue to replay from.
        """
        self.dlq = dlq

    def replay_one(
        self,
        entry_id: str,
        handler: Callable[[DeadLetterEntry], bool],
    ) -> ReplayResult:
        """Replay a single entry by calling handler.

        Args:
            entry_id: The entry ID to replay.
            handler: Function that takes DeadLetterEntry and returns True on success.
                     Raises or returns False on failure.

        Returns:
            ReplayResult with success/error status.
        """
        all_entries = self.dlq.read_all()
        entry = None
        for e in all_entries:
            if e.entry_id == entry_id:
                entry = e
                break

        if entry is None:
            return ReplayResult(
                entry_id=entry_id,
                success=False,
                error=f"Entry {entry_id} not found",
            )

        try:
            result = handler(entry)
            if result:
                self.dlq.mark_retried(entry_id)
                return ReplayResult(entry_id=entry_id, success=True, error=None)
            return ReplayResult(
                entry_id=entry_id,
                success=False,
                error="Handler returned False",
            )
        except Exception as exc:
            return ReplayResult(
                entry_id=entry_id,
                success=False,
                error=str(exc),
            )

    def replay_all(
        self,
        handler: Callable[[DeadLetterEntry], bool],
    ) -> list[ReplayResult]:
        """Replay all pending entries.

        Args:
            handler: Function that takes DeadLetterEntry and returns True on success.
                     Raises or returns False on failure.

        Returns:
            List of ReplayResult for each entry.
        """
        results = []
        for entry in self.dlq.pending():
            result = self.replay_one(entry.entry_id, handler)
            results.append(result)
        return results

    def replay_summary(self, results: list[ReplayResult]) -> dict:
        """Generate summary of replay results.

        Args:
            results: List of ReplayResult from replay_all() or multiple replay_one() calls.

        Returns:
            Dict with total, succeeded, failed counts.
        """
        total = len(results)
        succeeded = sum(1 for r in results if r.success)
        failed = total - succeeded

        return {
            "total": total,
            "succeeded": succeeded,
            "failed": failed,
        }
