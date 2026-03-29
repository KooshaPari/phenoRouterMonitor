"""Persistent backlog management for AgilePlus cycles.

Maintains a JSONL queue of known issues that could not be resolved in a single
cycle, enabling carry-over across cycles and audit trail of all findings.
"""

import orjson as json
import logging
import uuid
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path

from pydantic import BaseModel, Field

_log = logging.getLogger(__name__)


class BacklogStatus(StrEnum):
    """Lifecycle status for backlog items."""

    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    RESOLVED = "resolved"
    DEFERRED = "deferred"


class BacklogItem(BaseModel):
    """A single backlog entry tracked across AgilePlus cycles."""

    item_id: str = Field(default_factory=lambda: uuid.uuid4().hex[:12])
    finding_id: str
    dimension: str
    severity: float
    description: str
    attempts: int = 0
    last_attempted_at: str | None = None
    created_at: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    status: BacklogStatus = BacklogStatus.PENDING
    deferred_reason: str | None = None


class BacklogManager:
    """Manages a persistent JSONL backlog of unresolved findings.

    Items persist across AgilePlus cycles. Resolved items remain in the file
    for audit trail but are excluded from get_pending() results.
    """

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = session_dir
        self._ensure_dir()

    @property
    def backlog_path(self) -> Path:
        return self.session_dir / "agileplus" / "backlog.jsonl"

    def _ensure_dir(self) -> None:
        """Create the agileplus subdirectory if it does not exist."""
        self.backlog_path.parent.mkdir(parents=True, exist_ok=True)

    def _read_all(self) -> list[BacklogItem]:
        """Read all backlog items from the JSONL file."""
        if not self.backlog_path.exists():
            return []
        items: list[BacklogItem] = []
        with self.backlog_path.open("r", encoding="utf-8") as f:
            for line in f:
                stripped = line.strip()
                if not stripped:
                    continue
                items.append(BacklogItem.model_validate(json.loads(stripped)))
        return items

    def _write_all(self, items: list[BacklogItem]) -> None:
        """Rewrite the entire backlog JSONL (used for in-place updates)."""
        with self.backlog_path.open("w", encoding="utf-8") as f:
            for item in items:
                f.write(item.model_dump_json() + "\n")

    def _find_item(self, items: list[BacklogItem], item_id: str) -> BacklogItem:
        """Find an item by ID or raise ValueError."""
        for item in items:
            if item.item_id == item_id:
                return item
        msg = f"Backlog item not found: {item_id}"
        raise ValueError(msg)

    def add(
        self,
        finding_id: str,
        dimension: str,
        severity: float,
        description: str,
    ) -> BacklogItem:
        """Add a finding to the backlog as a new pending item."""
        item = BacklogItem(
            finding_id=finding_id,
            dimension=dimension,
            severity=severity,
            description=description,
        )
        with self.backlog_path.open("a", encoding="utf-8") as f:
            f.write(item.model_dump_json() + "\n")
        _log.debug("Added backlog item %s for finding %s", item.item_id, finding_id)
        return item

    def update_status(
        self,
        item_id: str,
        status: BacklogStatus,
        reason: str | None = None,
    ) -> None:
        """Update the status of a backlog item."""
        items = self._read_all()
        item = self._find_item(items, item_id)
        item.status = status
        if reason is not None:
            item.deferred_reason = reason
        self._write_all(items)
        _log.debug("Updated backlog item %s to status %s", item_id, status)

    def get_pending(self) -> list[BacklogItem]:
        """Return pending items sorted by severity descending, then attempts ascending."""
        items = self._read_all()
        pending = [i for i in items if i.status == BacklogStatus.PENDING]
        pending.sort(key=lambda i: (-i.severity, i.attempts))
        return pending

    def increment_attempt(self, item_id: str) -> None:
        """Increment the attempt counter and update the last_attempted_at timestamp."""
        items = self._read_all()
        item = self._find_item(items, item_id)
        item.attempts += 1
        item.last_attempted_at = datetime.now(UTC).isoformat()
        self._write_all(items)
        _log.debug("Incremented attempt for %s (now %d)", item_id, item.attempts)

    def resolve(self, item_id: str) -> None:
        """Mark a backlog item as resolved."""
        self.update_status(item_id, BacklogStatus.RESOLVED)

    def defer(self, item_id: str, reason: str) -> None:
        """Mark a backlog item as deferred with a reason."""
        self.update_status(item_id, BacklogStatus.DEFERRED, reason=reason)

    def get_all(self) -> list[BacklogItem]:
        """Return all backlog items (including resolved/deferred) for audit trail."""
        return self._read_all()
