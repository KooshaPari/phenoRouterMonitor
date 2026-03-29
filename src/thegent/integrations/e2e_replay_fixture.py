"""End-to-end replay fixture.

Implements event recording and replay for end-to-end testing, enabling
deterministic test execution with recorded event sequences.

# @trace WL-198
"""

from __future__ import annotations

import logging
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

logger = logging.getLogger(__name__)


@dataclass
class ReplayEvent:
    """A recorded event with metadata for replay.

    Attributes:
        event_id: Unique identifier for the event.
        event_type: Type/category of the event.
        payload: Arbitrary data associated with the event.
    """

    event_id: str
    event_type: str
    payload: dict[str, Any]


class E2EReplayFixture:
    """Records and replays events for end-to-end testing."""

    def __init__(self) -> None:
        """Initialize the replay fixture."""
        self._events: list[ReplayEvent] = []
        self._event_counter = 0
        logger.debug("Initialized E2E replay fixture")

    def record(self, event_type: str, payload: dict[str, Any]) -> ReplayEvent:
        """Record an event for later replay.

        Args:
            event_type: Type/category of the event.
            payload: Data associated with the event.

        Returns:
            The recorded ReplayEvent.

        Raises:
            ValueError: If event_type is empty.
        """
        if not event_type or not event_type.strip():
            raise ValueError("event_type cannot be empty")

        self._event_counter += 1
        event_id = f"event_{self._event_counter}"
        event = ReplayEvent(event_id=event_id, event_type=event_type, payload=payload or {})
        self._events.append(event)

        logger.debug(f"Recorded event: {event_id} ({event_type})")

        return event

    def replay(self, handler: Callable[[ReplayEvent], None]) -> int:
        """Replay all recorded events through a handler.

        Args:
            handler: Callable that processes each ReplayEvent.

        Returns:
            Number of events replayed.
        """
        count = 0

        for event in self._events:
            try:
                handler(event)
                count += 1
            except Exception as e:
                logger.error(f"Error replaying event {event.event_id}: {e}")
                raise

        logger.debug(f"Replayed {count} events")

        return count

    def events(self) -> list[ReplayEvent]:
        """Get all recorded events.

        Returns:
            List of ReplayEvent objects in recording order.
        """
        return list(self._events)

    def clear(self) -> None:
        """Clear all recorded events."""
        self._events.clear()
        self._event_counter = 0
        logger.debug("Cleared all recorded events")
