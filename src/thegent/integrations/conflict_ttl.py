from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, UTC


@dataclass
class ConflictRecord:
    """Record of a tracked conflict with TTL and escalation state."""

    conflict_id: str
    created_at: datetime
    escalated: bool = False


class ConflictTTLManager:
    """Manages conflict TTLs with automatic escalation.

    # @trace WL-250
    """

    def __init__(
        self,
        ttl_seconds: float = 86400.0,
        escalation_seconds: float = 3600.0,
    ) -> None:
        """Initialize the conflict TTL manager.

        Args:
            ttl_seconds: Time in seconds before conflict expires (default: 24 hours)
            escalation_seconds: Time in seconds before escalation needed (default: 1 hour)
        """
        self.ttl_seconds = ttl_seconds
        self.escalation_seconds = escalation_seconds
        self._conflicts: dict[str, ConflictRecord] = {}

    def register(self, conflict_id: str) -> ConflictRecord:
        """Register a new conflict.

        Args:
            conflict_id: Unique identifier for the conflict

        Returns:
            The created ConflictRecord
        """
        record = ConflictRecord(
            conflict_id=conflict_id,
            created_at=datetime.now(UTC),
            escalated=False,
        )
        self._conflicts[conflict_id] = record
        return record

    def is_expired(self, conflict_id: str) -> bool:
        """Check if a conflict has exceeded its TTL.

        Args:
            conflict_id: The conflict ID to check

        Returns:
            True if the conflict has expired, False otherwise

        Raises:
            KeyError: If the conflict_id is not registered
        """
        record = self._conflicts[conflict_id]
        age_seconds = (datetime.now(UTC) - record.created_at).total_seconds()
        return age_seconds > self.ttl_seconds

    def needs_escalation(self, conflict_id: str) -> bool:
        """Check if a conflict needs escalation (age > escalation_seconds but not expired).

        Args:
            conflict_id: The conflict ID to check

        Returns:
            True if escalation is needed and conflict not yet expired, False otherwise

        Raises:
            KeyError: If the conflict_id is not registered
        """
        record = self._conflicts[conflict_id]
        age_seconds = (datetime.now(UTC) - record.created_at).total_seconds()
        return age_seconds > self.escalation_seconds and age_seconds <= self.ttl_seconds and not record.escalated

    def escalate(self, conflict_id: str) -> None:
        """Mark a conflict as escalated.

        Args:
            conflict_id: The conflict ID to escalate

        Raises:
            KeyError: If the conflict_id is not registered
        """
        record = self._conflicts[conflict_id]
        record.escalated = True

    def expired_ids(self) -> list[str]:
        """Get all IDs of expired conflicts.

        Returns:
            List of conflict IDs with age > ttl_seconds
        """
        now = datetime.now(UTC)
        return [
            conflict_id
            for conflict_id, record in self._conflicts.items()
            if (now - record.created_at).total_seconds() > self.ttl_seconds
        ]
