"""Connector field mapping cache with TTL support.

# @trace WL-191
"""

from __future__ import annotations

import json as _json
import orjson as json
import re
import time
from dataclasses import asdict, dataclass
from pathlib import Path

WL_ID_PATTERN = re.compile(r"^WL-\d+$")


@dataclass
class MappingEntry:
    """Cached mapping entry for a connector field.

    Attributes:
        connector: Connector name (e.g., 'github', 'linear').
        field_id: Remote field identifier.
        field_name: Local field name.
        cached_at: Timestamp when the entry was cached (seconds since epoch).
        ttl_seconds: Time-to-live in seconds.
    """

    connector: str
    field_id: str
    field_name: str
    cached_at: float
    ttl_seconds: int


class ConnectorMappingCache:
    """Cache for connector field name → field_id mappings with TTL support."""

    DEFAULT_CACHE_FILE = Path("docs/reference/connector_mapping_cache.json")

    def __init__(self, cache_file: Path | None = None) -> None:
        """Initialize the mapping cache.

        Args:
            cache_file: Path to cache JSON file. Defaults to
                       docs/reference/connector_mapping_cache.json.
        """
        self._cache_file = cache_file or self.DEFAULT_CACHE_FILE
        self._entries: dict[str, MappingEntry] = {}
        self._load()

    def _load(self) -> None:
        """Load cache from file if it exists."""
        if not self._cache_file.exists():
            return

        try:
            data = json.loads(self._cache_file.read_text(encoding="utf-8"))
            for key, entry_dict in data.items():
                entry = MappingEntry(
                    connector=entry_dict["connector"],
                    field_id=entry_dict["field_id"],
                    field_name=entry_dict["field_name"],
                    cached_at=entry_dict["cached_at"],
                    ttl_seconds=entry_dict["ttl_seconds"],
                )
                self._entries[key] = entry
        except (json.JSONDecodeError, KeyError):
            # If cache file is corrupt, start fresh
            self._entries = {}

    def _save(self) -> None:
        """Persist cache to file."""
        self._cache_file.parent.mkdir(parents=True, exist_ok=True)
        data = {k: asdict(v) for k, v in self._entries.items()}
        self._cache_file.write_text(_json.dumps(data, indent=2), encoding="utf-8")

    def _make_key(self, connector: str, field_name: str) -> str:
        """Create a cache key from connector and field name."""
        return f"{self._normalize_connector(connector)}:{self._normalize_field_name(field_name)}"

    @staticmethod
    def _normalize_connector(connector: str) -> str:
        normalized = connector.strip().lower()
        if not normalized:
            raise ValueError("connector cannot be empty")
        return normalized

    @staticmethod
    def _normalize_field_name(field_name: str) -> str:
        normalized = field_name.strip()
        if not normalized:
            raise ValueError("field_name cannot be empty")
        return normalized

    def bootstrap_required(self, connector: str, required_fields: list[str]) -> bool:
        """Return whether bootstrap mappings are missing for required fields.

        A bootstrap is required when at least one required field has no
        non-stale mapping in the cache for the given connector.
        """
        normalized_required = [field.strip() for field in required_fields if field.strip()]
        if not normalized_required:
            return False
        return any(self.get(connector, field_name) is None for field_name in normalized_required)

    def bootstrap(self, connector: str, mappings: dict[str, str], ttl_seconds: int = 3600) -> None:
        """Persist an initial mapping set for a connector.

        Raises:
            ValueError: If any field name or field id is empty.
        """
        for field_name, field_id in mappings.items():
            if not field_name.strip():
                raise ValueError("mapping field_name cannot be empty")
            if not field_id.strip():
                raise ValueError("mapping field_id cannot be empty")
            self.put(connector, field_name.strip(), field_id.strip(), ttl_seconds=ttl_seconds)

    def is_stale(self, entry: MappingEntry) -> bool:
        """Check if an entry has expired based on TTL.

        Args:
            entry: MappingEntry to check.

        Returns:
            True if the entry has expired, False otherwise.
        """
        current_time = time.time()
        elapsed = current_time - entry.cached_at
        return elapsed > entry.ttl_seconds

    def get(self, connector: str, field_name: str) -> str | None:
        """Get cached field_id for a connector+field_name pair.

        Returns None if not cached, expired, or not found.

        Args:
            connector: Connector name.
            field_name: Field name to look up.

        Returns:
            Cached field_id or None.
        """
        key = self._make_key(connector, field_name)
        entry = self._entries.get(key)

        if entry is None:
            return None

        if self.is_stale(entry):
            del self._entries[key]
            self._save()
            return None

        return entry.field_id

    def get_with_status(self, connector: str, field_name: str) -> dict[str, object]:
        """Get mapping state with explicit freshness marker.

        Returns:
            Dict containing:
                - field_id: str | None
                - status: "missing" | "stale" | "fresh"
        """
        key = self._make_key(connector, field_name)
        entry = self._entries.get(key)
        if entry is None:
            return {"field_id": None, "status": "missing"}
        if self.is_stale(entry):
            del self._entries[key]
            self._save()
            return {"field_id": None, "status": "stale"}
        return {"field_id": entry.field_id, "status": "fresh"}

    def put(self, connector: str, field_name: str, field_id: str, ttl_seconds: int = 3600) -> None:
        """Cache a connector field mapping.

        Args:
            connector: Connector name.
            field_name: Local field name.
            field_id: Remote field identifier.
            ttl_seconds: Time-to-live in seconds (default: 3600 = 1 hour).
        """
        normalized_connector = self._normalize_connector(connector)
        normalized_field_name = self._normalize_field_name(field_name)
        normalized_field_id = field_id.strip()
        if not normalized_field_id:
            raise ValueError("field_id cannot be empty")
        key = self._make_key(normalized_connector, normalized_field_name)
        entry = MappingEntry(
            connector=normalized_connector,
            field_id=normalized_field_id,
            field_name=normalized_field_name,
            cached_at=time.time(),
            ttl_seconds=ttl_seconds,
        )
        self._entries[key] = entry
        self._save()

    def invalidate(self, connector: str, field_name: str) -> None:
        """Remove a cached entry.

        Args:
            connector: Connector name.
            field_name: Field name to invalidate.
        """
        key = self._make_key(connector, field_name)
        if key in self._entries:
            del self._entries[key]
            self._save()

    def clear_stale(self) -> int:
        """Remove all expired entries.

        Returns:
            Number of entries removed.
        """
        stale_keys = [key for key, entry in self._entries.items() if self.is_stale(entry)]
        for key in stale_keys:
            del self._entries[key]

        if stale_keys:
            self._save()

        return len(stale_keys)

    def list_entries(self, connector: str, *, include_stale: bool = False) -> list[MappingEntry]:
        """List entries for one connector.

        Args:
            connector: Connector name.
            include_stale: Whether stale records should be included.
        """
        normalized = self._normalize_connector(connector)
        results: list[MappingEntry] = []
        stale_keys: list[str] = []
        for key, entry in self._entries.items():
            if entry.connector != normalized:
                continue
            if not include_stale and self.is_stale(entry):
                stale_keys.append(key)
                continue
            results.append(entry)

        if stale_keys:
            for key in stale_keys:
                del self._entries[key]
            self._save()

        return sorted(results, key=lambda entry: entry.field_name)

    def list_cached_wl_ids(self, connector: str) -> list[str]:
        """Return WL IDs cached as field names for a connector."""
        wl_ids: list[str] = []
        for entry in self.list_entries(connector):
            if WL_ID_PATTERN.fullmatch(entry.field_name):
                wl_ids.append(entry.field_name)
        return sorted(wl_ids)
