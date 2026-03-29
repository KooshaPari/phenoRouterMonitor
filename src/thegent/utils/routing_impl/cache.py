"""GW-22/GW-26: Exact-match cache with DualCache L1+L2 for LLM gateway.

GW-22: Exact-match cache using hash(model+messages) -> cached response.
GW-26: DualCache with in-memory L1 + optional disk/Redis L2.

# @trace WP-2001 FR-CACHE-022 FR-CACHE-026
"""

from __future__ import annotations

import hashlib
import json
import logging
import os
import tempfile
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)

# Keys from kwargs that affect the cache key
_CACHE_KEY_KWARGS: frozenset[str] = frozenset({"temperature", "max_tokens", "tools", "response_format"})


# ---------------------------------------------------------------------------
# Cache key computation (GW-22)
# ---------------------------------------------------------------------------


def compute_cache_key(model: str, messages: list[dict], **kwargs: Any) -> str:
    """Compute a deterministic SHA-256 cache key for a request.

    Hash input: JSON of {"model": model, "messages": messages, "extras": sorted_extras}
    where extras contains temperature, max_tokens, tools, and response_format from kwargs.

    Returns:
        First 32 hex characters of the SHA-256 digest.
    """
    extras: dict[str, Any] = {k: kwargs[k] for k in _CACHE_KEY_KWARGS if k in kwargs}
    payload = {
        "model": model,
        "messages": messages,
        "extras": dict(sorted(extras.items())),
    }
    serialized = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    digest = hashlib.sha256(serialized.encode()).hexdigest()
    return digest[:32]


# ---------------------------------------------------------------------------
# CacheEntry
# ---------------------------------------------------------------------------


@dataclass
class CacheEntry:
    """A single cached response entry.

    Attributes:
        key: The cache key string.
        response: The serialized LLM response dict.
        created_at: Monotonic clock timestamp of creation.
        ttl: Time-to-live in seconds.
        namespace: Logical grouping namespace.
    """

    key: str
    response: dict[str, Any]
    created_at: float
    ttl: float = 300.0
    namespace: str = "default"

    @property
    def is_expired(self) -> bool:
        """Return True if the entry has exceeded its TTL."""
        return time.monotonic() - self.created_at > self.ttl


# ---------------------------------------------------------------------------
# InMemoryCache
# ---------------------------------------------------------------------------


class InMemoryCache:
    """Thread-safe in-memory LRU-ish cache with TTL support.

    Eviction policy: when size exceeds max_size, the oldest entry (by
    insertion order) is removed first.

    Internal key format: ``"{namespace}:{key}"``
    """

    def __init__(self, max_size: int = 1000, default_ttl: float = 300.0) -> None:
        self._max_size = max_size
        self._default_ttl = default_ttl
        # Insertion-ordered dict for O(1) oldest-entry eviction
        self._store: dict[str, CacheEntry] = {}
        self._lock = threading.Lock()

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _full_key(key: str, namespace: str) -> str:
        return f"{namespace}:{key}"

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get(self, key: str, namespace: str = "default") -> CacheEntry | None:
        """Return the cached entry or None on miss / expiry.

        Expired entries are removed on access (lazy eviction).
        """
        full = self._full_key(key, namespace)
        with self._lock:
            entry = self._store.get(full)
            if entry is None:
                return None
            if entry.is_expired:
                del self._store[full]
                _log.debug("InMemoryCache: expired entry evicted key=%s ns=%s", key, namespace)
                return None
            return entry

    def set(
        self,
        key: str,
        response: dict[str, Any],
        ttl: float | None = None,
        namespace: str = "default",
    ) -> CacheEntry:
        """Store a response and return the created CacheEntry."""
        effective_ttl = ttl if ttl is not None else self._default_ttl
        full = self._full_key(key, namespace)
        entry = CacheEntry(
            key=key,
            response=response,
            created_at=time.monotonic(),
            ttl=effective_ttl,
            namespace=namespace,
        )
        with self._lock:
            # Evict oldest entry when at capacity (before inserting new one)
            if full not in self._store and len(self._store) >= self._max_size:
                oldest_full_key = next(iter(self._store))
                del self._store[oldest_full_key]
                _log.debug("InMemoryCache: evicted oldest entry to make room")
            self._store[full] = entry
        return entry

    def delete(self, key: str, namespace: str = "default") -> bool:
        """Remove an entry. Returns True if it existed, False otherwise."""
        full = self._full_key(key, namespace)
        with self._lock:
            if full in self._store:
                del self._store[full]
                return True
            return False

    def clear(self, namespace: str | None = None) -> int:
        """Clear all entries in the given namespace, or all entries if None.

        Returns the count of deleted entries.
        """
        with self._lock:
            if namespace is None:
                count = len(self._store)
                self._store.clear()
                return count
            prefix = f"{namespace}:"
            keys_to_delete = [k for k in self._store if k.startswith(prefix)]
            for k in keys_to_delete:
                del self._store[k]
            return len(keys_to_delete)

    def size(self) -> int:
        """Return the total number of entries currently stored."""
        with self._lock:
            return len(self._store)


# ---------------------------------------------------------------------------
# DiskCache
# ---------------------------------------------------------------------------


class DiskCache:
    """Persistent disk-backed cache using JSON files.

    File layout: ``{cache_dir}/{namespace}/{key[:2]}/{key}.json``

    Writes are atomic (write to temp file, then os.replace).
    """

    def __init__(
        self,
        cache_dir: str | None = None,
        default_ttl: float = 3600.0,
    ) -> None:
        if cache_dir is None:
            cache_dir = str(Path.home() / ".thegent" / "cache" / "responses")
        self._cache_dir = Path(cache_dir)
        self._cache_dir.mkdir(parents=True, exist_ok=True)
        self._default_ttl = default_ttl

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    def _entry_path(self, key: str, namespace: str) -> Path:
        ns_dir = self._cache_dir / namespace / key[:2]
        return ns_dir / f"{key}.json"

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get(self, key: str, namespace: str = "default") -> CacheEntry | None:
        """Return cached entry from disk, or None on miss / expiry.

        Expired files are deleted on access.
        """
        path = self._entry_path(key, namespace)
        if not path.exists():
            return None
        try:
            raw = path.read_text(encoding="utf-8")
            data = json.loads(raw)
        except (OSError, json.JSONDecodeError) as exc:
            _log.warning("DiskCache: failed to read %s: %s", path, exc)
            return None

        entry = CacheEntry(
            key=data["key"],
            response=data["response"],
            created_at=data["created_at"],
            ttl=data["ttl"],
            namespace=data.get("namespace", namespace),
        )
        if entry.is_expired:
            try:
                path.unlink(missing_ok=True)
            except OSError:
                pass
            _log.debug("DiskCache: expired entry deleted key=%s ns=%s", key, namespace)
            return None
        return entry

    def set(
        self,
        key: str,
        response: dict[str, Any],
        ttl: float | None = None,
        namespace: str = "default",
    ) -> CacheEntry:
        """Store a response to disk atomically and return the CacheEntry."""
        effective_ttl = ttl if ttl is not None else self._default_ttl
        entry = CacheEntry(
            key=key,
            response=response,
            created_at=time.monotonic(),
            ttl=effective_ttl,
            namespace=namespace,
        )
        path = self._entry_path(key, namespace)
        path.parent.mkdir(parents=True, exist_ok=True)

        data = {
            "key": entry.key,
            "response": entry.response,
            "created_at": entry.created_at,
            "ttl": entry.ttl,
            "namespace": entry.namespace,
        }
        serialized = json.dumps(data, separators=(",", ":"))

        # Atomic write: temp file in same directory, then rename
        fd, tmp_path = tempfile.mkstemp(dir=path.parent, suffix=".tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                f.write(serialized)
            os.replace(tmp_path, path)
        except OSError:
            # Clean up temp file on failure, then re-raise
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
            raise

        _log.debug("DiskCache: wrote key=%s ns=%s path=%s", key, namespace, path)
        return entry

    def delete(self, key: str, namespace: str = "default") -> bool:
        """Remove the cache file. Returns True if it existed."""
        path = self._entry_path(key, namespace)
        if path.exists():
            try:
                path.unlink()
                return True
            except OSError as exc:
                _log.warning("DiskCache: failed to delete %s: %s", path, exc)
        return False

    def clear(self, namespace: str | None = None) -> int:
        """Delete cache files for a namespace, or all namespaces if None.

        Returns the count of deleted files.
        """
        count = 0
        if namespace is None:
            search_root = self._cache_dir
        else:
            search_root = self._cache_dir / namespace
            if not search_root.exists():
                return 0

        for json_file in search_root.rglob("*.json"):
            try:
                json_file.unlink()
                count += 1
            except OSError as exc:
                _log.warning("DiskCache: failed to delete %s: %s", json_file, exc)
        return count


# ---------------------------------------------------------------------------
# DualCache (GW-26)
# ---------------------------------------------------------------------------


class DualCache:
    """Two-level cache: in-memory L1 with optional persistent disk L2.

    Read strategy: L1 -> L2 (backfill L1 on L2 hit) -> miss.
    Write strategy: write to both L1 and L2 (if L2 present).
    """

    def __init__(
        self,
        l1: InMemoryCache | None = None,
        l2: DiskCache | None = None,
    ) -> None:
        self._l1: InMemoryCache = l1 if l1 is not None else InMemoryCache()
        self._l2: DiskCache | None = l2

    def get(self, key: str, namespace: str = "default") -> CacheEntry | None:
        """Return the first valid cache hit (L1 then L2), or None on double miss.

        On an L2 hit, the entry is backfilled into L1 for future fast access.
        """
        entry = self._l1.get(key, namespace=namespace)
        if entry is not None:
            return entry

        if self._l2 is not None:
            entry = self._l2.get(key, namespace=namespace)
            if entry is not None:
                # Backfill L1
                self._l1.set(key, entry.response, ttl=entry.ttl, namespace=namespace)
                _log.debug("DualCache: L2 hit, backfilled L1 key=%s ns=%s", key, namespace)
                return entry

        return None

    def set(
        self,
        key: str,
        response: dict[str, Any],
        ttl: float | None = None,
        namespace: str = "default",
    ) -> CacheEntry:
        """Write to both L1 and L2 (if present). Returns the L1 entry."""
        entry = self._l1.set(key, response, ttl=ttl, namespace=namespace)
        if self._l2 is not None:
            self._l2.set(key, response, ttl=ttl, namespace=namespace)
        return entry

    def delete(self, key: str, namespace: str = "default") -> bool:
        """Delete from both L1 and L2. Returns True if either had the entry."""
        deleted_l1 = self._l1.delete(key, namespace=namespace)
        deleted_l2 = self._l2.delete(key, namespace=namespace) if self._l2 is not None else False
        return deleted_l1 or deleted_l2

    def clear(self, namespace: str | None = None) -> int:
        """Clear both levels. Returns the total count of entries deleted."""
        total = self._l1.clear(namespace=namespace)
        if self._l2 is not None:
            total += self._l2.clear(namespace=namespace)
        return total


# ---------------------------------------------------------------------------
# Singleton helpers
# ---------------------------------------------------------------------------

_cache_instance: DualCache | None = None
_cache_lock = threading.Lock()


def get_cache(l2_enabled: bool = False) -> DualCache:
    """Return the process-global DualCache singleton.

    Args:
        l2_enabled: When True and the instance has not yet been created,
            attach a DiskCache as the L2 layer.  Ignored if the singleton
            already exists.
    """
    global _cache_instance
    with _cache_lock:
        if _cache_instance is None:
            l2 = DiskCache() if l2_enabled else None
            _cache_instance = DualCache(l1=InMemoryCache(), l2=l2)
            _log.debug("DualCache singleton created l2_enabled=%s", l2_enabled)
    return _cache_instance


def reset_cache() -> None:
    """Reset the process-global DualCache singleton (for testing)."""
    global _cache_instance
    with _cache_lock:
        _cache_instance = None
        _log.debug("DualCache singleton reset")


# ---------------------------------------------------------------------------
# Convenience functions
# ---------------------------------------------------------------------------


def cache_get(
    model: str,
    messages: list[dict],
    namespace: str = "default",
    **kwargs: Any,
) -> dict[str, Any] | None:
    """Look up a cached response for this request. Returns None on miss.

    Args:
        model: Model identifier used for key computation.
        messages: Chat messages list used for key computation.
        namespace: Cache namespace to query.
        **kwargs: Additional parameters (temperature, max_tokens, etc.) that
            affect the cache key.

    Returns:
        The cached response dict, or None if not found.
    """
    key = compute_cache_key(model, messages, **kwargs)
    entry = get_cache().get(key, namespace=namespace)
    return entry.response if entry is not None else None


def cache_set(
    model: str,
    messages: list[dict],
    response: dict[str, Any],
    ttl: float = 300.0,
    namespace: str = "default",
    **kwargs: Any,
) -> None:
    """Store a response in the cache.

    Args:
        model: Model identifier used for key computation.
        messages: Chat messages list used for key computation.
        response: The response dict to cache.
        ttl: Time-to-live in seconds.
        namespace: Cache namespace for storage.
        **kwargs: Additional parameters (temperature, max_tokens, etc.) that
            affect the cache key.
    """
    key = compute_cache_key(model, messages, **kwargs)
    get_cache().set(key, response, ttl=ttl, namespace=namespace)
