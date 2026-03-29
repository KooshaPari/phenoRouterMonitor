"""Governance override expiry event emission (WP-3003, research-governance-override-events).

Provides structured JSONL event emission when governance overrides expire,
enabling audit trails and downstream reactions.
"""

from __future__ import annotations

import json
import logging
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

from thegent.integrations.base import SerializableMixin

if TYPE_CHECKING:
    from collections.abc import Callable

_log = logging.getLogger(__name__)

_DEFAULT_EVENTS_PATH = Path("~/.thegent/governance_events.jsonl").expanduser()


@dataclass
class OverrideExpiredEvent(SerializableMixin):
    """Structured event emitted when a governance override expires."""

    override_id: str
    policy_id: str
    owner: str
    expired_at: float
    reason: str = "ttl_elapsed"
    event_type: str = field(default="governance.override.expired", init=False)


@dataclass
class OverrideActivatedEvent(SerializableMixin):
    """Structured event emitted when a governance override is activated."""

    override_id: str
    policy_id: str
    owner: str
    activated_at: float
    ttl_s: float
    expires_at: float
    event_type: str = field(default="governance.override.activated", init=False)


class OverrideEventEmitter:
    """Writes governance override lifecycle events to a JSONL audit log."""

    def __init__(self, events_path: Path | None = None) -> None:
        self._path = (events_path or _DEFAULT_EVENTS_PATH).expanduser()
        self._lock = threading.Lock()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def emit_expired(self, event: OverrideExpiredEvent) -> None:
        """Append an override-expired event to the JSONL log.

        Args:
            event: The structured expiry event to persist.
        """
        self._append(event.to_dict())
        _log.info(
            "governance.override.expired override_id=%s policy_id=%s owner=%s reason=%s",
            event.override_id,
            event.policy_id,
            event.owner,
            event.reason,
        )

    def emit_activated(
        self,
        override_id: str,
        policy_id: str,
        owner: str,
        ttl_s: float,
    ) -> None:
        """Append an override-activated event to the JSONL log.

        Args:
            override_id: Unique identifier for this override.
            policy_id: The governance policy being overridden.
            owner: The principal who applied the override.
            ttl_s: Time-to-live in seconds.
        """
        now = time.time()
        evt = OverrideActivatedEvent(
            override_id=override_id,
            policy_id=policy_id,
            owner=owner,
            activated_at=now,
            ttl_s=ttl_s,
            expires_at=now + ttl_s,
        )
        self._append(evt.to_dict())
        _log.info(
            "governance.override.activated override_id=%s policy_id=%s owner=%s ttl_s=%.1f",
            override_id,
            policy_id,
            owner,
            ttl_s,
        )

    def tail_events(self, n: int = 20) -> list[dict[str, object]]:
        """Read the last *n* events from the JSONL log.

        Args:
            n: Maximum number of events to return (most-recent last).

        Returns:
            List of event dicts, up to *n* entries.
        """
        if not self._path.exists():
            return []

        with self._lock:
            lines = self._path.read_text(encoding="utf-8").splitlines()

        results: list[dict[str, object]] = []
        for line in lines[-n:]:
            line = line.strip()
            if not line:
                continue
            try:
                results.append(json.loads(line))
            except json.JSONDecodeError:
                _log.warning("Skipping malformed event line: %.80s", line)
        return results

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    def _append(self, record: dict[str, object]) -> None:
        """Thread-safe append of a JSON record to the JSONL file."""
        self._path.parent.mkdir(parents=True, exist_ok=True)
        line = json.dumps(record, separators=(",", ":")) + "\n"
        with self._lock:
            with self._path.open("a", encoding="utf-8") as fh:
                fh.write(line)


# ---------------------------------------------------------------------------
# Override expiry monitor — background thread
# ---------------------------------------------------------------------------


@dataclass
class _Registration:
    override_id: str
    expires_at: float
    on_expire: Callable[[], None]
    policy_id: str = ""
    owner: str = ""


class OverrideExpiryMonitor:
    """Background thread that fires callbacks when registered overrides expire.

    Usage::

        emitter = OverrideEventEmitter()
        monitor = OverrideExpiryMonitor(emitter=emitter)
        monitor.start()

        monitor.register("ovr-001", time.time() + 10, lambda: print("expired!"))
        ...
        monitor.stop()
    """

    def __init__(
        self,
        emitter: OverrideEventEmitter | None = None,
        poll_interval_s: float = 1.0,
    ) -> None:
        self._emitter = emitter or OverrideEventEmitter()
        self._poll_interval = poll_interval_s
        self._registrations: dict[str, _Registration] = {}
        self._lock = threading.Lock()
        self._stop_event = threading.Event()
        self._thread: threading.Thread | None = None

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def start(self) -> None:
        """Start the background polling thread."""
        if self._thread is not None and self._thread.is_alive():
            return
        self._stop_event.clear()
        self._thread = threading.Thread(
            target=self._run,
            name="override-expiry-monitor",
            daemon=True,
        )
        self._thread.start()
        _log.debug("OverrideExpiryMonitor started")

    def stop(self, timeout_s: float = 5.0) -> None:
        """Signal the background thread to stop and wait for it.

        Args:
            timeout_s: Maximum seconds to wait for clean stop.
        """
        self._stop_event.set()
        if self._thread is not None:
            self._thread.join(timeout=timeout_s)
            self._thread = None
        _log.debug("OverrideExpiryMonitor stopped")

    # ------------------------------------------------------------------
    # Registration
    # ------------------------------------------------------------------

    def register(
        self,
        override_id: str,
        expires_at: float,
        on_expire: Callable[[], None],
        policy_id: str = "",
        owner: str = "",
    ) -> None:
        """Register an override for expiry monitoring.

        Args:
            override_id: Unique identifier for the override.
            expires_at: Unix timestamp when the override expires.
            on_expire: Zero-arg callback invoked on expiry.
            policy_id: Policy the override applies to (for event metadata).
            owner: Who applied the override (for event metadata).
        """
        reg = _Registration(
            override_id=override_id,
            expires_at=expires_at,
            on_expire=on_expire,
            policy_id=policy_id,
            owner=owner,
        )
        with self._lock:
            self._registrations[override_id] = reg
        _log.debug(
            "Registered override %s expiring at %.3f (policy=%s owner=%s)",
            override_id,
            expires_at,
            policy_id,
            owner,
        )

    def unregister(self, override_id: str) -> None:
        """Remove an override from monitoring (e.g. if manually revoked).

        Args:
            override_id: The override to remove.
        """
        with self._lock:
            self._registrations.pop(override_id, None)

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    def _run(self) -> None:
        """Main polling loop — checks for expired overrides every poll interval."""
        while not self._stop_event.wait(self._poll_interval):
            self._check_expirations()

    def _check_expirations(self) -> None:
        now = time.time()
        with self._lock:
            expired = [r for r in self._registrations.values() if now >= r.expires_at]
            for reg in expired:
                del self._registrations[reg.override_id]

        for reg in expired:
            _log.info("Override %s expired at %.3f", reg.override_id, reg.expires_at)
            evt = OverrideExpiredEvent(
                override_id=reg.override_id,
                policy_id=reg.policy_id,
                owner=reg.owner,
                expired_at=reg.expires_at,
                reason="ttl_elapsed",
            )
            self._safe_emit(evt)
            self._safe_callback(reg)

    def _safe_emit(self, evt: OverrideExpiredEvent) -> None:
        try:
            self._emitter.emit_expired(evt)
        except OSError:
            _log.exception("IO error emitting expired event for %s", evt.override_id)
        except ValueError:
            _log.exception("Serialization error for event %s", evt.override_id)

    def _safe_callback(self, reg: _Registration) -> None:
        try:
            reg.on_expire()
        except Exception as exc:
            _log.error(
                "on_expire callback raised for %s: %s: %s",
                reg.override_id,
                type(exc).__name__,
                exc,
            )
