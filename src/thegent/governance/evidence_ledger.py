"""Hash-chained JSONL evidence ledger for AgilePlus cycles.

Records evidence events with cryptographic hash chaining for tamper detection,
following the RunRegistry pattern from execution.py.
"""

import hashlib
import json
import logging
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from pydantic import BaseModel, Field

from thegent.governance.evidence_graph import EvidenceGraph

_log = logging.getLogger(__name__)

# Event type constants
CYCLE_STARTED = "cycle_started"
SCAN_COMPLETED = "scan_completed"
PLAN_CREATED = "plan_created"
TASK_DISPATCHED = "task_dispatched"
TASK_COMPLETED = "task_completed"
VERIFICATION_COMPLETED = "verification_completed"
CYCLE_COMPLETED = "cycle_completed"


class EvidenceEvent(BaseModel):
    """A single evidence event in the hash-chained ledger."""

    event_type: str
    timestamp: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    cycle_id: str
    payload: dict[str, Any]
    prev_hash: str | None = None
    hash: str | None = None


class EvidenceLedger:
    """Hash-chained JSONL evidence ledger for AgilePlus cycles.

    Follows the RunRegistry pattern: each record contains a prev_hash linking
    to the previous record's hash, forming a tamper-evident chain.
    """

    SCHEMA_VERSION = 1

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = session_dir
        self._ensure_dir()
        self._ensure_version_marker()

    @property
    def ledger_path(self) -> Path:
        return self.session_dir / "agileplus" / "evidence_ledger.jsonl"

    def _ensure_dir(self) -> None:
        """Create the agileplus subdirectory if it does not exist."""
        self.ledger_path.parent.mkdir(parents=True, exist_ok=True)

    def _ensure_version_marker(self) -> None:
        """Write a schema version marker if the ledger file is new."""
        if not self.ledger_path.exists():
            marker = {
                "event": "schema_version",
                "version": self.SCHEMA_VERSION,
                "timestamp": datetime.now(UTC).isoformat(),
            }
            marker["hash"] = self._calculate_hash(marker)
            with self.ledger_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(marker) + "\n")

    def _get_last_hash(self) -> str | None:
        """Return the hash of the last record in the ledger."""
        if not self.ledger_path.exists():
            return None

        try:
            with self.ledger_path.open("r", encoding="utf-8") as f:
                last_line = None
                for line in f:
                    if line.strip():
                        last_line = line
                if last_line:
                    data = json.loads(last_line)
                    return data.get("hash")
        except Exception:
            _log.exception("Failed to read last hash from ledger")
        return None

    def _calculate_hash(self, data: dict[str, Any]) -> str:
        """Calculate a stable SHA-256 hash for a record, excluding the hash field itself."""
        d = {k: v for k, v in data.items() if k != "hash"}
        body = json.dumps(d, sort_keys=True, separators=(",", ":"))
        return hashlib.sha256(body.encode()).hexdigest()

    def record(self, event_type: str, cycle_id: str, payload: dict[str, Any] | None = None, **kwargs: Any) -> str:
        """Record an evidence event with hash chaining.

        Returns the hash of the newly recorded event.

        Args:
            event_type: Type of event being recorded
            cycle_id: ID of the cycle
            payload: Optional dict payload (if not provided, kwargs are used)
            **kwargs: Additional fields to include in payload
        """
        # Merge payload and kwargs
        if payload is None:
            payload = kwargs
        elif kwargs:
            payload = {**payload, **kwargs}

        event = EvidenceEvent(
            event_type=event_type,
            cycle_id=cycle_id,
            payload=payload,
        )
        event.prev_hash = self._get_last_hash()
        data = event.model_dump()
        event.hash = self._calculate_hash(data)
        with self.ledger_path.open("a", encoding="utf-8") as f:
            f.write(event.model_dump_json() + "\n")
        _log.debug("Recorded evidence event %s for cycle %s", event_type, cycle_id)
        return event.hash

    def query(
        self,
        cycle_id: str | None = None,
        event_type: str | None = None,
    ) -> list[EvidenceEvent]:
        """Query evidence events, optionally filtering by cycle_id and/or event_type."""
        if not self.ledger_path.exists():
            return []

        results: list[EvidenceEvent] = []
        with self.ledger_path.open("r", encoding="utf-8") as f:
            for line in f:
                stripped = line.strip()
                if not stripped:
                    continue
                data = json.loads(stripped)
                if data.get("event") == "schema_version":
                    continue
                if cycle_id is not None and data.get("cycle_id") != cycle_id:
                    continue
                if event_type is not None and data.get("event_type") != event_type:
                    continue
                results.append(EvidenceEvent.model_validate(data))
        return results

    def verify_chain(self) -> bool:
        """Verify the integrity of the hash chain.

        Returns True if every record's hash is correct and prev_hash links
        form an unbroken chain. Returns False on any inconsistency.
        """
        if not self.ledger_path.exists():
            return True

        prev_hash: str | None = None
        with self.ledger_path.open("r", encoding="utf-8") as f:
            for line in f:
                stripped = line.strip()
                if not stripped:
                    continue
                data = json.loads(stripped)
                recorded_hash = data.get("hash")
                if recorded_hash is None:
                    _log.warning("Record missing hash field")
                    return False
                expected_hash = self._calculate_hash(data)
                if recorded_hash != expected_hash:
                    _log.warning("Hash mismatch: recorded=%s expected=%s", recorded_hash, expected_hash)
                    return False
                recorded_prev = data.get("prev_hash")
                if recorded_prev != prev_hash:
                    _log.warning(
                        "Chain break: recorded prev_hash=%s expected=%s",
                        recorded_prev,
                        prev_hash,
                    )
                    return False
                prev_hash = recorded_hash
        return True

    def link_to_graph(self, graph: EvidenceGraph, event_hash: str, artifact_id: str) -> None:
        """Link an evidence event to an artifact in the EvidenceGraph."""
        graph.add_link(event_hash, artifact_id)
