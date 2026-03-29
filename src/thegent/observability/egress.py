"""Structured SIEM egress helpers."""

from __future__ import annotations

import logging
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from typing import Any

import httpx

_log = logging.getLogger(__name__)


@dataclass
class EgressEvent:
    """Structured event payload for external egress sinks."""

    id: str
    severity: str
    event_type: str
    source: str
    payload: dict[str, Any] = field(default_factory=dict)
    timestamp: str = field(default_factory=lambda: datetime.now(UTC).isoformat())


class SIEMEgress:
    """Push governance events to an HTTP SIEM endpoint."""

    def __init__(self, endpoint_url: str) -> None:
        self.endpoint_url = endpoint_url.strip()

    def push_event(self, event: EgressEvent) -> bool:
        """Send an event to the configured SIEM endpoint."""
        if not self.endpoint_url:
            return False
        try:
            with httpx.Client(timeout=10.0) as client:
                response = client.post(self.endpoint_url, json=asdict(event))
            return response.is_success
        except Exception as exc:
            _log.warning("SIEM egress failed: %s", exc)
            return False

    def format_for_syslog(self, event: EgressEvent) -> str:
        """Render a compact syslog-style line for an event."""
        return (
            f"{event.timestamp} thegent[{event.source}] "
            f"{event.severity.upper()} {event.event_type} id={event.id} payload={event.payload}"
        )
