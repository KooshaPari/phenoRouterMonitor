"""Idempotency Index Cache (WL-166): Track applied operations to prevent duplicates.

Provides a persistent cache of applied operations, keyed by operation_id.
Used to ensure that workstream updates, sync operations, and other potentially
expensive actions are idempotent — if the same operation is reapplied with the
same content_hash, it is skipped.

The cache persists to docs/reference/idempotency_cache.json for durability
across process restarts.
"""

import logging
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

import orjson

from thegent.integrations.base import SerializableMixin

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data Structures
# ---------------------------------------------------------------------------


@dataclass
class IdempotencyRecord(SerializableMixin):
    """Record of an applied operation."""

    operation_id: str
    """Unique operation identifier (e.g., 'sync-wl-160-001')."""

    wl_id: str
    """Work item ID that triggered this operation (e.g., 'WL-160')."""

    connector: str
    """Source connector (e.g., 'github', 'linear', 'workstream')."""

    timestamp: str
    """ISO 8601 timestamp when operation was recorded."""

    content_hash: str
    """SHA256 hash of the operation's content (for change detection)."""

    @classmethod
    def from_dict(cls, data: dict) -> "IdempotencyRecord":
        """Deserialize from a dictionary."""
        return cls(
            operation_id=data["operation_id"],
            wl_id=data["wl_id"],
            connector=data["connector"],
            timestamp=data["timestamp"],
            content_hash=data["content_hash"],
        )


# ---------------------------------------------------------------------------
# Cache Implementation
# ---------------------------------------------------------------------------


class IdempotencyCache:
    """Persistent cache of applied operations.

    Prevents duplicate application of the same operation by tracking
    operation_id + content_hash pairs. Records are stored in a JSON file
    at docs/reference/idempotency_cache.json.

    Thread-safe for read operations, but writes should be serialized
    externally (e.g., via SingleWriterLock).
    """

    DEFAULT_CACHE_PATH = Path("docs/reference/idempotency_cache.json")

    def __init__(self, cache_path: Path | None = None):
        """Initialize the cache.

        Args:
            cache_path: Path to the JSON cache file. Defaults to
                docs/reference/idempotency_cache.json.
        """
        self.cache_path = cache_path or self.DEFAULT_CACHE_PATH
        self._records: dict[str, IdempotencyRecord] = {}
        self._content_index: dict[tuple[str, str, str], IdempotencyRecord] = {}
        self._load()

    def _load(self) -> None:
        """Load records from the cache file."""
        if not self.cache_path.exists():
            logger.debug("Idempotency cache not found at %s", self.cache_path)
            return

        try:
            data = orjson.loads(self.cache_path.read_text(encoding="utf-8"))
            for record_dict in data.get("records", []):
                record = IdempotencyRecord.from_dict(record_dict)
                self._records[record.operation_id] = record
                self._index_record(record)
            logger.debug("Loaded %d records from idempotency cache", len(self._records))
        except Exception as e:
            logger.error("Failed to load idempotency cache: %s", e)

    def _save(self) -> None:
        """Save records to the cache file."""
        self.cache_path.parent.mkdir(parents=True, exist_ok=True)

        try:
            data = {
                "version": "1.0",
                "records": [record.to_dict() for record in self._records.values()],
            }
            self.cache_path.write_text(
                orjson.dumps(data, option=orjson.OPT_INDENT_2 | orjson.OPT_SORT_KEYS).decode("utf-8"),
                encoding="utf-8",
            )
        except Exception as e:
            logger.error("Failed to save idempotency cache: %s", e)

    def check(self, operation_id: str) -> bool:
        """Check if an operation has already been applied.

        Args:
            operation_id: The operation identifier to check.

        Returns:
            True if the operation has been recorded, False otherwise.
        """
        return operation_id in self._records

    def check_content(self, connector: str, wl_id: str, content_hash: str) -> bool:
        """Check whether equivalent content was already recorded.

        Args:
            connector: Connector name (for example: github, linear).
            wl_id: Work item identifier.
            content_hash: Deterministic content hash for the mutation payload.

        Returns:
            True when an equivalent record exists in the idempotency index.
        """
        key = (connector.strip().lower(), wl_id.strip().upper(), content_hash.strip().lower())
        return key in self._content_index

    def record(
        self,
        operation_id: str,
        wl_id: str,
        connector: str,
        content_hash: str,
    ) -> None:
        """Record an applied operation.

        Args:
            operation_id: Unique operation identifier.
            wl_id: Work item ID that triggered this operation.
            connector: Source connector name.
            content_hash: SHA256 hash of the operation's content.
        """
        record = IdempotencyRecord(
            operation_id=operation_id,
            wl_id=wl_id,
            connector=connector,
            timestamp=datetime.now(timezone.utc).isoformat(),
            content_hash=content_hash,
        )
        self._records[operation_id] = record
        self._index_record(record)
        self._save()
        logger.debug("Recorded operation %s for %s via %s", operation_id, wl_id, connector)

    def invalidate(self, operation_id: str) -> None:
        """Remove a record from the cache.

        Used when an operation has been rolled back or needs to be re-applied.

        Args:
            operation_id: The operation to remove.
        """
        if operation_id in self._records:
            record = self._records[operation_id]
            self._deindex_record(record)
            del self._records[operation_id]
            self._save()
            logger.debug("Invalidated operation %s", operation_id)

    def clear_older_than(self, dt: datetime) -> int:
        """Remove records with timestamps older than the given datetime.

        Args:
            dt: Cutoff datetime (ISO 8601 or datetime object).

        Returns:
            The number of records removed.
        """
        if isinstance(dt, str):
            cutoff = datetime.fromisoformat(dt)
        else:
            cutoff = dt

        removed = 0
        keys_to_remove = []

        for operation_id, record in self._records.items():
            record_time = datetime.fromisoformat(record.timestamp)
            if record_time < cutoff:
                keys_to_remove.append(operation_id)
                removed += 1

        for key in keys_to_remove:
            record = self._records[key]
            self._deindex_record(record)
            del self._records[key]

        if removed > 0:
            self._save()
            logger.debug("Cleared %d records older than %s", removed, cutoff)

        return removed

    def get_all_records(self) -> list[IdempotencyRecord]:
        """Return all cached records (for inspection/testing).

        Returns:
            List of all IdempotencyRecord objects.
        """
        return list(self._records.values())

    @staticmethod
    def _index_key(record: IdempotencyRecord) -> tuple[str, str, str]:
        return (
            record.connector.strip().lower(),
            record.wl_id.strip().upper(),
            record.content_hash.strip().lower(),
        )

    def _index_record(self, record: IdempotencyRecord) -> None:
        self._content_index[self._index_key(record)] = record

    def _deindex_record(self, record: IdempotencyRecord) -> None:
        self._content_index.pop(self._index_key(record), None)
