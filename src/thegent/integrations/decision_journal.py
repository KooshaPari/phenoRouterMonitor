"""Local decision journal for recording work stream decisions and replay capability.

# @trace WL-203
"""

from __future__ import annotations

import orjson as json
import uuid
from dataclasses import asdict, dataclass
from datetime import datetime, UTC
from pathlib import Path
from typing import Any


@dataclass
class JournalEntry:
    """Record of a decision made during a work stream cycle.

    Attributes:
        entry_id: Unique entry identifier.
        cycle_id: Associated cycle identifier.
        wl_id: Work stream item ID.
        decision: The decision made (e.g., 'status_changed', 'priority_updated').
        rationale: Explanation of the decision.
        before_state: State before the decision (dict).
        after_state: State after the decision (dict).
        timestamp: When the decision was recorded (ISO 8601 format).
        replayable: Whether the decision can be replayed for audit purposes.
    """

    entry_id: str
    cycle_id: str
    wl_id: str
    decision: str
    rationale: str
    before_state: dict[str, Any]
    after_state: dict[str, Any]
    timestamp: str
    replayable: bool

    @staticmethod
    def create_entry(
        cycle_id: str,
        wl_id: str,
        decision: str,
        rationale: str,
        before_state: dict[str, Any],
        after_state: dict[str, Any],
        replayable: bool = True,
    ) -> JournalEntry:
        """Factory method to create a new JournalEntry.

        Args:
            cycle_id: Associated cycle ID.
            wl_id: Work stream item ID.
            decision: Decision type/name.
            rationale: Explanation.
            before_state: State before decision.
            after_state: State after decision.
            replayable: Whether the decision can be replayed (default: True).

        Returns:
            A new JournalEntry with auto-generated ID and timestamp.
        """
        entry_id = str(uuid.uuid4())[:8]
        timestamp = datetime.now(UTC).isoformat()

        return JournalEntry(
            entry_id=entry_id,
            cycle_id=cycle_id,
            wl_id=wl_id,
            decision=decision,
            rationale=rationale,
            before_state=before_state,
            after_state=after_state,
            timestamp=timestamp,
            replayable=replayable,
        )


class DecisionJournal:
    """Journal for recording and replaying decisions made to work stream items."""

    DEFAULT_JOURNAL_FILE = Path("docs/reference/decision_journal.jsonl")

    def __init__(self, journal_file: Path | None = None) -> None:
        """Initialize the decision journal.

        Args:
            journal_file: Path to JSONL journal file. Defaults to
                         docs/reference/decision_journal.jsonl.
        """
        self._journal_file = journal_file or self.DEFAULT_JOURNAL_FILE

    def append(self, entry: JournalEntry) -> None:
        """Record a decision in the journal.

        Appends a JSON-serialized entry to the JSONL file.

        Args:
            entry: JournalEntry to record.
        """
        self._journal_file.parent.mkdir(parents=True, exist_ok=True)

        # Append as single JSON line
        entry_dict = asdict(entry)
        line = json.dumps(entry_dict).decode() + "\n"

        with self._journal_file.open("a", encoding="utf-8") as f:
            f.write(line)

    def read_all(self) -> list[JournalEntry]:
        """Read all entries from the journal.

        Returns:
            List of JournalEntry objects, in order of appearance.
        """
        if not self._journal_file.exists():
            return []

        entries: list[JournalEntry] = []
        with self._journal_file.open("r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    entry = JournalEntry(
                        entry_id=data["entry_id"],
                        cycle_id=data["cycle_id"],
                        wl_id=data["wl_id"],
                        decision=data["decision"],
                        rationale=data["rationale"],
                        before_state=data["before_state"],
                        after_state=data["after_state"],
                        timestamp=data["timestamp"],
                        replayable=data["replayable"],
                    )
                    entries.append(entry)
                except (json.JSONDecodeError, KeyError):
                    # Skip invalid entries
                    continue

        return entries

    def read_replayable(self) -> list[JournalEntry]:
        """Read only replayable entries from the journal.

        Returns:
            List of JournalEntry objects where replayable=True.
        """
        all_entries = self.read_all()
        return [entry for entry in all_entries if entry.replayable]

    def replay_entry(self, entry_id: str) -> JournalEntry:
        """Retrieve a specific entry by ID for replay.

        Args:
            entry_id: Entry ID to retrieve.

        Returns:
            The matching JournalEntry.

        Raises:
            ValueError: If entry is not found.
        """
        entries = self.read_all()
        for entry in entries:
            if entry.entry_id == entry_id:
                return entry

        raise ValueError(f"Journal entry not found: {entry_id}")
