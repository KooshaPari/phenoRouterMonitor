"""Reflection decision event logging for sync operations.

# @trace WL-195
"""

from __future__ import annotations

import orjson as json
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Any

from thegent.docgen.code_annotation import CodeAnnotationGenerator


@dataclass
class ReflectionDecision:
    """A decision made during reflection phase.

    Attributes:
        wl_id: Work stream item ID.
        decision_type: Type of decision ('apply', 'skip', 'conflict').
        before_value: Value before the decision.
        after_value: Value after the decision.
        connector: Connector name.
        timestamp: ISO format timestamp when decision was made.
        cycle_id: Unique identifier for the sync cycle.
    """

    wl_id: str
    decision_type: str
    before_value: Any
    after_value: Any
    connector: str
    timestamp: str
    cycle_id: str
    direction: str = "remote_to_local"
    mutation_id: str = ""


class ReflectionEventLog:
    """Event log for reflection decisions."""

    def __init__(self, log_path: Path | str | None = None) -> None:
        """Initialize the event log.

        Args:
            log_path: Path to the JSONL event log file.
                     Defaults to docs/reference/reflection_events.jsonl
        """
        if log_path is None:
            log_path = Path("docs/reference/reflection_events.jsonl")
        else:
            log_path = Path(log_path)

        self.log_path = log_path
        self._events: list[ReflectionDecision] = []
        self._annotation_generator = CodeAnnotationGenerator(annotation_format="json")
        self._load_existing_events()

    def _load_existing_events(self) -> None:
        """Load existing events from the log file."""
        if not self.log_path.exists():
            return

        try:
            with open(self.log_path) as f:
                for line in f:
                    if line.strip():
                        event_dict = json.loads(line)
                        event_dict.pop("annotation", None)
                        event = ReflectionDecision(**event_dict)
                        self._events.append(event)
        except (json.JSONDecodeError, KeyError):
            # Silently ignore malformed entries
            pass

    def log(self, decision: ReflectionDecision) -> None:
        """Log a reflection decision.

        Args:
            decision: ReflectionDecision to log.
        """
        self._events.append(decision)

        # Ensure directory exists
        self.log_path.parent.mkdir(parents=True, exist_ok=True)

        # Append to JSONL file
        with open(self.log_path, "a") as f:
            event_dict = asdict(decision)
            event_dict["annotation"] = self._annotation_generator.format_reflection_annotation(
                {
                    "schema": "reflection-annotation-v1",
                    "wl_id": decision.wl_id,
                    "connector": decision.connector,
                    "direction": decision.direction,
                    "decision": decision.decision_type,
                    "mutation_id": decision.mutation_id or f"{decision.wl_id}:{decision.cycle_id}",
                    "timestamp": decision.timestamp,
                }
            )
            f.write(json.dumps(event_dict).decode() + "\n")

    def read_all(self) -> list[ReflectionDecision]:
        """Read all logged decisions.

        Returns:
            List of all ReflectionDecision events.
        """
        return list(self._events)

    def read_by_type(self, decision_type: str) -> list[ReflectionDecision]:
        """Read decisions of a specific type.

        Args:
            decision_type: Type filter ('apply', 'skip', 'conflict').

        Returns:
            List of matching ReflectionDecision events.
        """
        return [e for e in self._events if e.decision_type == decision_type]

    def read_since(self, dt: datetime) -> list[ReflectionDecision]:
        """Read decisions since a specific datetime.

        Args:
            dt: Cutoff datetime. Events with timestamp >= dt.isoformat() are included.

        Returns:
            List of matching ReflectionDecision events.
        """
        cutoff = dt.isoformat()
        return [e for e in self._events if e.timestamp >= cutoff]
