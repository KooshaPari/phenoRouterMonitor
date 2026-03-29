"""Dead-Letter Queue for remote write failures — persistent recovery.

Queues failed remote writes to JSONL for deterministic replay after fixes.

# @trace WL-213
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import orjson


@dataclass
class DeadLetterEntry:
    """A single failed remote write queued for replay.

    Attributes:
        entry_id: Unique identifier for this entry.
        wl_id: The workstream item that failed.
        connector: The connector name (e.g., 'github', 'linear').
        operation: The operation type (e.g., 'write_item', 'sync_field').
        payload: The original request payload (as dict).
        error: The error message from the failed write.
        created_at: Timestamp when entry was created.
        retry_count: Number of replay attempts (default 0).
    """

    entry_id: str
    wl_id: str
    connector: str
    operation: str
    payload: dict
    error: str
    created_at: datetime
    retry_count: int = 0


class DeadLetterQueue:
    """Persistent dead-letter queue for failed remote writes.

    Stores entries as JSONL for durability and deterministic replay.
    """

    def __init__(self, store_path: Path, max_retries: int = 3) -> None:
        """Initialize dead-letter queue.

        Args:
            store_path: Path to JSONL file for storing entries.
            max_retries: Maximum retry attempts before entry is considered resolved.
        """
        self.store_path = Path(store_path)
        self.max_retries = max_retries
        self.store_path.parent.mkdir(parents=True, exist_ok=True)

    def enqueue(self, entry: DeadLetterEntry) -> None:
        """Append entry to dead-letter queue.

        Args:
            entry: The DeadLetterEntry to persist.
        """
        line = orjson.dumps(
            {
                "entry_id": entry.entry_id,
                "wl_id": entry.wl_id,
                "connector": entry.connector,
                "operation": entry.operation,
                "payload": entry.payload,
                "error": entry.error,
                "created_at": entry.created_at.isoformat(),
                "retry_count": entry.retry_count,
            }
        ).decode("utf-8")
        with open(self.store_path, "a") as f:
            f.write(line + "\n")

    def read_all(self) -> list[DeadLetterEntry]:
        """Read all entries from queue.

        Returns:
            List of all DeadLetterEntry objects (resolved and pending).
        """
        if not self.store_path.exists():
            return []

        entries = []
        with open(self.store_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                data = orjson.loads(line)
                entry = DeadLetterEntry(
                    entry_id=data["entry_id"],
                    wl_id=data["wl_id"],
                    connector=data["connector"],
                    operation=data["operation"],
                    payload=data["payload"],
                    error=data["error"],
                    created_at=datetime.fromisoformat(data["created_at"]),
                    retry_count=data["retry_count"],
                )
                entries.append(entry)
        return entries

    def pending(self) -> list[DeadLetterEntry]:
        """Get pending entries (retry_count < max_retries).

        Returns:
            List of DeadLetterEntry objects eligible for replay.
        """
        return [e for e in self.read_all() if e.retry_count < self.max_retries]

    def mark_retried(self, entry_id: str) -> None:
        """Increment retry_count for an entry and rewrite file.

        Args:
            entry_id: The entry ID to increment.

        Raises:
            ValueError: If entry_id not found.
        """
        all_entries = self.read_all()
        found = False

        for entry in all_entries:
            if entry.entry_id == entry_id:
                entry.retry_count += 1
                found = True
                break

        if not found:
            raise ValueError(f"Entry {entry_id} not found in queue")

        # Rewrite entire file
        with open(self.store_path, "w") as f:
            for entry in all_entries:
                line = orjson.dumps(
                    {
                        "entry_id": entry.entry_id,
                        "wl_id": entry.wl_id,
                        "connector": entry.connector,
                        "operation": entry.operation,
                        "payload": entry.payload,
                        "error": entry.error,
                        "created_at": entry.created_at.isoformat(),
                        "retry_count": entry.retry_count,
                    }
                ).decode("utf-8")
                f.write(line + "\n")

    def purge_resolved(self) -> int:
        """Remove entries where retry_count >= max_retries.

        Returns:
            Count of entries removed.
        """
        all_entries = self.read_all()
        pending_entries = [e for e in all_entries if e.retry_count < self.max_retries]
        removed_count = len(all_entries) - len(pending_entries)

        # Rewrite file with only pending entries
        with open(self.store_path, "w") as f:
            for entry in pending_entries:
                line = orjson.dumps(
                    {
                        "entry_id": entry.entry_id,
                        "wl_id": entry.wl_id,
                        "connector": entry.connector,
                        "operation": entry.operation,
                        "payload": entry.payload,
                        "error": entry.error,
                        "created_at": entry.created_at.isoformat(),
                        "retry_count": entry.retry_count,
                    }
                ).decode("utf-8")
                f.write(line + "\n")

        return removed_count
