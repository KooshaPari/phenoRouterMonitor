"""Manual Conflict Queue for resolving workstream conflicts.

Queues conflicts for manual resolution and tracks resolution status.

# @trace WL-205
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime


def classify_conflict(*, field: str, connector: str, wl_id: str) -> tuple[str, str, str]:
    """Classify conflict routing fields deterministically."""
    normalized_field = field.strip().lower()
    normalized_connector = connector.strip().lower()
    _ = wl_id.strip().upper()

    if normalized_field in {"status", "state", "priority"}:
        category = "state_divergence"
        severity = "high"
    elif normalized_field in {"title", "description", "body"}:
        category = "content_drift"
        severity = "medium"
    else:
        category = "schema_mismatch"
        severity = "low"

    owner_domain = "operations"
    if normalized_connector in {"github", "gitlab"}:
        owner_domain = "source-control"
    elif normalized_connector == "linear":
        owner_domain = "planning"
    return category, severity, owner_domain


@dataclass
class ConflictEntry:
    """Represents a single conflict in the queue.

    Attributes:
        conflict_id: Unique identifier for the conflict.
        wl_id: The workstream item identifier where conflict occurred.
        field: The field/property where the conflict exists.
        local_value: The local system's value for the field.
        remote_value: The remote system's value for the field.
        connector: The connector name associated with this conflict.
        created_at: Timestamp when the conflict was created.
        resolved: Whether the conflict has been marked as resolved.
    """

    conflict_id: str
    wl_id: str
    field: str
    local_value: str
    remote_value: str
    connector: str
    created_at: datetime
    resolved: bool = False
    category: str = ""
    severity: str = ""
    owner_domain: str = ""


class ConflictQueue:
    """Queue for managing conflicts requiring manual resolution.

    Maintains a FIFO queue of conflicts and tracks their resolution status.
    """

    def __init__(self) -> None:
        """Initialize an empty conflict queue."""
        self._queue: list[ConflictEntry] = []
        self._resolved_map: dict[str, ConflictEntry] = {}

    def enqueue(self, entry: ConflictEntry) -> None:
        """Add a conflict to the queue.

        Args:
            entry: The ConflictEntry to enqueue.

        Raises:
            ValueError: If entry is None or if conflict_id is empty.
        """
        if entry is None:
            raise ValueError("entry cannot be None")

        if not entry.conflict_id:
            raise ValueError("conflict_id cannot be empty")

        if not entry.category or not entry.severity or not entry.owner_domain:
            category, severity, owner_domain = classify_conflict(
                field=entry.field,
                connector=entry.connector,
                wl_id=entry.wl_id,
            )
            entry.category = category
            entry.severity = severity
            entry.owner_domain = owner_domain

        self._queue.append(entry)

    def dequeue(self) -> ConflictEntry:
        """Remove and return the first unresolved conflict from the queue.

        Returns:
            The next unresolved ConflictEntry in FIFO order.

        Raises:
            IndexError: If the queue is empty (no unresolved conflicts).
        """
        # Find first unresolved entry
        for i, entry in enumerate(self._queue):
            if not entry.resolved:
                return self._queue.pop(i)

        # No unresolved entries found
        raise IndexError("Cannot dequeue from empty queue")

    def resolve(self, conflict_id: str) -> None:
        """Mark a conflict as resolved.

        Args:
            conflict_id: The ID of the conflict to mark as resolved.

        Raises:
            KeyError: If the conflict_id is not found in the queue.
        """
        if not conflict_id:
            raise KeyError("conflict_id cannot be empty")

        # Search in active queue
        for entry in self._queue:
            if entry.conflict_id == conflict_id:
                entry.resolved = True
                return

        # Search in resolved map
        if conflict_id in self._resolved_map:
            return

        # Not found
        raise KeyError(f"Conflict with ID '{conflict_id}' not found")

    def pending(self) -> list[ConflictEntry]:
        """Get all unresolved conflicts in FIFO order.

        Returns:
            List of unresolved ConflictEntry objects.
        """
        return [entry for entry in self._queue if not entry.resolved]

    def all_entries(self) -> list[ConflictEntry]:
        """Get all conflicts (resolved and unresolved) in insertion order.

        Returns:
            List of all ConflictEntry objects.
        """
        return list(self._queue)

    def size(self) -> int:
        """Get the count of unresolved (pending) conflicts.

        Returns:
            Number of pending conflicts.
        """
        return len([entry for entry in self._queue if not entry.resolved])
