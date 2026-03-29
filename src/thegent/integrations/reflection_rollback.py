"""Reflection rollback command.

# @trace WL-185
"""

from __future__ import annotations

from collections import OrderedDict
from dataclasses import dataclass
from datetime import datetime, UTC
from pathlib import Path
from typing import ClassVar
from uuid import uuid4
import orjson as json
from typing import Any


@dataclass
class RollbackEntry:
    """Represents a single rollback entry snapshot."""

    entry_id: str
    timestamp: datetime
    snapshot: dict[str, Any]


class ReflectionRollbackStore:
    """Store for managing rollback entries."""

    def __init__(self) -> None:
        """Initialize the rollback store."""
        self._entries: dict[str, RollbackEntry] = {}

    def record(self, entry_id: str, snapshot: dict[str, Any]) -> RollbackEntry:
        """Record a new rollback entry.

        Args:
            entry_id: Unique identifier for this rollback entry.
            snapshot: Dictionary containing the snapshot data.

        Returns:
            The created RollbackEntry.
        """
        entry = RollbackEntry(
            entry_id=entry_id,
            timestamp=datetime.now(UTC),
            snapshot=snapshot.copy(),
        )
        self._entries[entry_id] = entry
        return entry

    def rollback_to(self, entry_id: str) -> dict[str, Any]:
        """Rollback to a previously recorded entry.

        Args:
            entry_id: Unique identifier of the entry to rollback to.

        Returns:
            The snapshot dictionary from that entry.

        Raises:
            KeyError: If the entry_id does not exist.
        """
        entry = self._entries[entry_id]
        return entry.snapshot.copy()

    def list_entries(self) -> list[RollbackEntry]:
        """Get all recorded rollback entries.

        Returns:
            List of RollbackEntry records in insertion order.
        """
        return list(self._entries.values())


@dataclass(frozen=True)
class RollbackSnapshot:
    """Persisted snapshot record for `WORK_STREAM.md` content."""

    snapshot_id: str
    timestamp: str
    work_stream_content: str
    cycle_id: str

    _created_at: datetime

    def __init__(
        self,
        snapshot_id: str,
        timestamp: str,
        work_stream_content: str,
        cycle_id: str,
    ) -> None:
        created_at = datetime.fromisoformat(timestamp)
        object.__setattr__(self, "snapshot_id", snapshot_id)
        object.__setattr__(self, "timestamp", timestamp)
        object.__setattr__(self, "work_stream_content", work_stream_content)
        object.__setattr__(self, "cycle_id", cycle_id)
        object.__setattr__(self, "_created_at", created_at)

    @classmethod
    def from_file(cls, snapshot_path: Path) -> "RollbackSnapshot":
        """Load and validate a persisted snapshot."""
        data = json.loads(snapshot_path.read_text(encoding="utf-8"))
        return cls(
            snapshot_id=data["snapshot_id"],
            timestamp=data["timestamp"],
            work_stream_content=data["work_stream_content"],
            cycle_id=data.get("cycle_id", ""),
        )

    def to_file_dict(self) -> dict[str, str]:
        """Return a JSON-serializable dictionary."""
        return {
            "snapshot_id": self.snapshot_id,
            "timestamp": self.timestamp,
            "work_stream_content": self.work_stream_content,
            "cycle_id": self.cycle_id,
        }


class ReflectionRollbackManager:
    """Disk-backed work stream snapshot manager."""

    _DEFAULT_SNAPSHOTS_DIR_NAME: ClassVar[str] = "docs/reference/rollback_snapshots"
    _SNAPSHOT_ID_LENGTH: ClassVar[int] = 8

    def __init__(self, snapshots_dir: Path | str = _DEFAULT_SNAPSHOTS_DIR_NAME) -> None:
        self._snapshots_dir = Path(snapshots_dir)
        self._snapshots_dir.mkdir(parents=True, exist_ok=True)

    @staticmethod
    def _generate_snapshot_id() -> str:
        """Generate stable-length snapshot IDs."""
        return uuid4().hex[: ReflectionRollbackManager._SNAPSHOT_ID_LENGTH]

    def take_snapshot(self, work_stream_path: Path, cycle_id: str | None = None) -> RollbackSnapshot:
        """Read a work stream file, persist a snapshot, and return the in-memory snapshot."""
        content = work_stream_path.read_text(encoding="utf-8")
        snapshot = RollbackSnapshot(
            snapshot_id=self._generate_snapshot_id(),
            timestamp=datetime.now(tz=UTC).isoformat(),
            work_stream_content=content,
            cycle_id=cycle_id or "",
        )
        snapshot_file = self._snapshots_dir / f"{snapshot.snapshot_id}.json"
        ordered = OrderedDict(
            snapshot_id=snapshot.snapshot_id,
            timestamp=snapshot.timestamp,
            work_stream_content=snapshot.work_stream_content,
            cycle_id=snapshot.cycle_id,
        )
        snapshot_file.write_text(json.dumps(ordered).decode(), encoding="utf-8")
        return snapshot

    def list_snapshots(self) -> list[RollbackSnapshot]:
        """List known snapshots sorted newest first."""
        snapshots: list[RollbackSnapshot] = []
        for snapshot_path in self._snapshots_dir.glob("*.json"):
            snapshot = RollbackSnapshot.from_file(snapshot_path)
            snapshots.append(snapshot)
        snapshots.sort(key=lambda snapshot: snapshot._created_at, reverse=True)
        return snapshots

    def rollback_to(self, snapshot_id: str, work_stream_path: Path) -> None:
        """Restore the content of a previously captured snapshot."""
        snapshot = self._load_snapshot_by_id(snapshot_id)
        work_stream_path.write_text(snapshot.work_stream_content, encoding="utf-8")

    def cleanup_old_snapshots(self, *, keep_last_n: int = 5) -> None:
        """Delete persisted snapshots until only `keep_last_n` newest remain."""
        snapshots = self.list_snapshots()
        if len(snapshots) <= keep_last_n:
            return

        for snapshot in snapshots[keep_last_n:]:
            (self._snapshots_dir / f"{snapshot.snapshot_id}.json").unlink(missing_ok=True)

    def _load_snapshot_by_id(self, snapshot_id: str) -> RollbackSnapshot:
        """Load a snapshot by identifier and fail loudly when missing."""
        snapshot_path = self._snapshots_dir / f"{snapshot_id}.json"
        if not snapshot_path.exists():
            raise FileNotFoundError(f"snapshot not found: {snapshot_id}")
        return RollbackSnapshot.from_file(snapshot_path)


__all__ = [
    "ReflectionRollbackManager",
    "ReflectionRollbackStore",
    "RollbackEntry",
    "RollbackSnapshot",
]
