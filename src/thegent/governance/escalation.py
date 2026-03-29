"""WP-3008: Escalation SLA and governance queue (FR-028)."""

import json
import logging
import time
from dataclasses import dataclass, field

from thegent.integrations.base import SerializableMixin
from enum import StrEnum
from pathlib import Path
from typing import Any, cast

_log = logging.getLogger(__name__)


class EscalationStatus(StrEnum):
    """Status of an escalation item."""

    PENDING = "pending"
    ASSIGNED = "assigned"
    RESOLVED = "resolved"
    CANCELLED = "cancelled"
    EXPIRED = "expired"


class EscalationPriority(StrEnum):
    """Priority of an escalation item."""

    LOW = "low"
    NORMAL = "normal"
    HIGH = "high"
    URGENT = "urgent"


@dataclass
class EscalationItem(SerializableMixin):
    """An item in the escalation queue."""

    id: str
    run_id: str
    prompt: str
    reason: str
    agent: str
    priority: EscalationPriority = EscalationPriority.NORMAL
    status: EscalationStatus = EscalationStatus.PENDING
    created_at: float = field(default_factory=time.time)
    deadline: float | None = None
    assigned_to: str | None = None
    resolution: str | None = None
    snapshot_path: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


class EscalationQueue:
    """Manages the governance escalation queue."""

    def __init__(self, settings: Any = None) -> None:
        if isinstance(settings, (str | Path)):
            # Legacy/buggy caller passing session_dir Path directly
            from thegent.config import ThegentSettings

            self.settings = ThegentSettings()
            self.settings.session_dir = Path(settings)
        else:
            from thegent.config import ThegentSettings

            self.settings = settings or ThegentSettings()
        self.queue_dir = self.settings.session_dir / "escalations"
        self.queue_dir.mkdir(parents=True, exist_ok=True)

    def add(self, run_id: str, reason: str, priority: int = 2) -> str:
        """Simplified add for legacy/internal callers."""
        priority_map = {
            1: EscalationPriority.LOW,
            2: EscalationPriority.NORMAL,
            3: EscalationPriority.HIGH,
            4: EscalationPriority.URGENT,
        }
        prio = priority_map.get(priority, EscalationPriority.NORMAL)
        return self.escalate(
            run_id=run_id,
            prompt="",  # Not always available for simple block
            reason=reason,
            agent="unknown",
            priority=prio,
        )

    def escalate(
        self,
        run_id: str,
        prompt: str,
        reason: str,
        agent: str,
        priority: EscalationPriority = EscalationPriority.NORMAL,
        sla_minutes: int = 60,
        metadata: dict[str, Any | None] | None = None,
    ) -> str:
        """Add a new item to the escalation queue."""
        esc_id = f"esc-{int(time.time())}-{run_id[:8]}"
        deadline = time.time() + (sla_minutes * 60)

        item = EscalationItem(
            id=esc_id,
            run_id=run_id,
            prompt=prompt,
            reason=reason,
            agent=agent,
            priority=priority,
            deadline=deadline,
            metadata=metadata or {},
        )

        self._save_item(item)
        _log.info("Escalated task %s to queue id=%s", run_id, esc_id)
        return esc_id

    def list_items(self, status: EscalationStatus | None = None) -> list[EscalationItem]:
        """List items in the queue, optionally filtered by status."""
        items = []
        for p in self.queue_dir.glob("*.json"):
            item = self._load_and_process_item(p)
            if item and (status is None or item.status == status):
                items.append(item)

        # Sort by priority (Urgent > High > Normal > Low) and then by deadline
        priority_map = {
            EscalationPriority.URGENT: 0,
            EscalationPriority.HIGH: 1,
            EscalationPriority.NORMAL: 2,
            EscalationPriority.LOW: 3,
        }
        items.sort(key=lambda x: (priority_map[x.priority], x.deadline or float("inf")))
        return items

    def get_item(self, esc_id: str) -> EscalationItem | None:
        """Retrieve a specific escalation item."""
        p = self.queue_dir / f"{esc_id}.json"
        if not p.exists():
            return None
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            return cast("EscalationItem", EscalationItem.from_dict(data))
        except Exception as e:
            _log.error("Failed to load escalation item %s: %s", esc_id, e)
            return None

    def resolve(self, esc_id: str, resolution: str, solver: str) -> bool:
        """Mark an escalation item as resolved."""
        item = self.get_item(esc_id)
        if not item:
            return False

        item.status = EscalationStatus.RESOLVED
        item.resolution = resolution
        item.assigned_to = solver
        self._save_item(item)
        _log.info("Resolved escalation %s by %s", esc_id, solver)
        return True

    def _save_item(self, item: EscalationItem) -> None:
        """Save item to disk."""
        p = self.queue_dir / f"{item.id}.json"
        p.write_text(json.dumps(item.to_dict(), indent=2), encoding="utf-8")

    def _load_and_process_item(self, p: Path) -> EscalationItem | None:
        """Helper to load and process a single escalation item."""
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            item = cast("EscalationItem", EscalationItem.from_dict(data))

            # Auto-expire if deadline passed
            if item.status == EscalationStatus.PENDING and item.deadline and time.time() > item.deadline:
                item.status = EscalationStatus.EXPIRED
                self._save_item(item)

                # WP-3008: Move expired item to DLQ
                try:
                    from thegent.execution import DLQManager, RunMeta

                    dlq = DLQManager(self.settings.session_dir)
                    # Create a minimal RunMeta for DLQ
                    meta = RunMeta(
                        run_id=item.run_id,
                        agent=item.agent,
                        prompt=item.prompt,
                        cwd=str(Path.cwd()),
                        owner=item.assigned_to or "system",
                    )
                    dlq.enqueue(meta, f"Escalation EXPIRED: {item.reason}")
                    _log.warning("Moved expired escalation %s to DLQ", item.id)
                except Exception as e:
                    _log.error("Failed to move expired escalation %s to DLQ: %s", item.id, e)
            return item
        except Exception as e:
            _log.error("Failed to load escalation item %s: %s", p, e)
            return None
