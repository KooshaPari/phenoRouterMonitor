"""Phase 9: Request Coalescing v2 implementation.
Includes Singleflight, inotify cache invalidation, heat-based LRU, and multi-tier cache.
"""

import collections
import logging
import threading
import time
from collections.abc import Callable
from pathlib import Path
from typing import Any

# Library-first (LIBRARY_FIRST_POLICY.md): Using cachetools for in-memory caching
from cachetools import LRUCache, TTLCache

try:
    from PersistDict import PersistDict

    PERSISTDICT_AVAILABLE = True
except ImportError:
    PERSISTDICT_AVAILABLE = False

try:
    import watchdog.events
    import watchdog.observers

    HAS_WATCHDOG = True
except ImportError:
    HAS_WATCHDOG = False

logger = logging.getLogger(__name__)


class CacheV2:
    """Async-friendly TTL cache used by newer infra modules."""

    def __init__(self, root: Path, namespace: str = "default") -> None:
        self.root = root
        self.namespace = namespace
        self.root.mkdir(parents=True, exist_ok=True)
        self._store: dict[str, tuple[float | None, Any]] = {}
        self._lock = threading.Lock()

    async def get(self, key: str) -> Any | None:
        with self._lock:
            item = self._store.get(key)
            if item is None:
                return None
            expires_at, value = item
            if expires_at is not None and expires_at <= time.time():
                del self._store[key]
                return None
            return value

    async def set(self, key: str, value: Any, ttl: int | None = None) -> None:
        expires_at = None if ttl is None else time.time() + ttl
        with self._lock:
            self._store[key] = (expires_at, value)

    async def clear_expired(self) -> None:
        now = time.time()
        with self._lock:
            expired_keys = [k for k, (exp, _) in self._store.items() if exp is not None and exp <= now]
            for key in expired_keys:
                del self._store[key]

    async def clear(self) -> None:
        with self._lock:
            self._store.clear()


class Singleflight:
    """Implementation of Singleflight pattern to prevent duplicate requests."""

    def __init__(self) -> None:
        self.calls: dict[str, threading.Event] = {}
        self.results: dict[str, Any] = {}
        self.lock = threading.Lock()

    def do(self, key: str, func: Callable[[], Any]) -> Any:
        """Execute func for key, coalescing concurrent calls."""
        with self.lock:
            if key in self.calls:
                event = self.calls[key]
                self.lock.release()
                event.wait()
                self.lock.acquire()
                return self.results.get(key)

            event = threading.Event()
            self.calls[key] = event

        try:
            result = func()
            with self.lock:
                self.results[key] = result
            return result
        finally:
            with self.lock:
                event.set()
                del self.calls[key]


class CrossProcessSingleflight:
    """Implementation of Singleflight pattern across processes using file locks."""

    def __init__(self, coordination_dir: Path) -> None:
        self.coordination_dir = coordination_dir
        self.coordination_dir.mkdir(parents=True, exist_ok=True)

    def do(self, key: str, func: Callable[[], Any], ttl: int = 300) -> Any:
        """Execute func for key, coalescing concurrent calls across processes."""
        import hashlib
        import json
        import os

        hashed_key = hashlib.sha256(key.encode()).hexdigest()
        lock_file = self.coordination_dir / f"{hashed_key}.lock"
        result_file = self.coordination_dir / f"{hashed_key}.result"

        # 1. Check if a recent result already exists
        if result_file.exists():
            try:
                data = json.loads(result_file.read_text())
                if time.time() - data.get("timestamp", 0) < ttl:
                    return data.get("result")
            except Exception:
                pass

        # 2. Try to acquire lock
        try:
            fd = os.open(lock_file, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
            with os.fdopen(fd, "w") as f:
                f.write(f"{os.getpid()}|{time.time()}")
        except FileExistsError:
            # Lock exists, check if stale
            try:
                content = lock_file.read_text().split("|")
                lock_time = float(content[1])
                if time.time() - lock_time > 120:  # 2 minute lock TTL
                    logger.warning(f"Stale lock found for {key}, breaking.")
                    lock_file.unlink()
                    return self.do(key, func, ttl)
            except (IndexError, ValueError, OSError):
                pass

            # Wait for result
            start_wait = time.time()
            while time.time() - start_wait < 120:
                if result_file.exists():
                    try:
                        data = json.loads(result_file.read_text())
                        return data.get("result")
                    except Exception:
                        pass
                if not lock_file.exists():
                    # Lock released but no result? Retry.
                    return self.do(key, func, ttl)
                time.sleep(1)

            logger.error(f"Timeout waiting for singleflight result: {key}")
            return None

        # 3. We have the lock, execute func
        try:
            result = func()
            result_file.write_text(json.dumps({"result": result, "timestamp": time.time()}))
            return result
        finally:
            if lock_file.exists():
                lock_file.unlink()


class HeatBasedLRU:
    """LRU cache with heat-based eviction (frequency + decay)."""

    def __init__(self, capacity: int = 100, decay_factor: float = 0.9) -> None:
        self.capacity = capacity
        self.decay_factor = decay_factor
        self.cache: dict[str, Any] = {}
        self.heat: dict[str, float] = collections.defaultdict(float)
        self.last_access: dict[str, float] = {}
        self.lock = threading.Lock()

    def get(self, key: str) -> Any | None:
        with self.lock:
            if key in self.cache:
                self._update_heat(key)
                return self.cache[key]
            return None

    def put(self, key: str, value: Any):
        with self.lock:
            if len(self.cache) >= self.capacity and key not in self.cache:
                self._evict()
            self.cache[key] = value
            self._update_heat(key)

    def _update_heat(self, key: str):
        now = time.time()
        prev_time = self.last_access.get(key, now)
        elapsed = now - prev_time
        # Apply decay to existing heat
        self.heat[key] = self.heat[key] * (self.decay_factor**elapsed) + 1.0
        self.last_access[key] = now

    def _evict(self):
        # Evict item with lowest heat
        if not self.heat:
            return

        # Recalculate heat for all items before eviction to apply decay
        now = time.time()
        for k in list(self.heat.keys()):
            elapsed = now - self.last_access[k]
            self.heat[k] *= self.decay_factor**elapsed
            self.last_access[k] = now

        victim = min(self.heat, key=lambda k: self.heat.get(k, 0.0))
        del self.cache[victim]
        del self.heat[victim]
        del self.last_access[victim]
        logger.info(f"Evicted {victim} from cache based on heat")


class CacheInvalidator:
    """inotify-based cache invalidation."""

    def __init__(self, cache: Any) -> None:
        self.cache = cache
        if HAS_WATCHDOG:
            self.observer = watchdog.observers.Observer()
        else:
            self.observer = None

    def watch(self, directory: Path):
        if not self.observer:
            logger.warning("watchdog not installed, inotify invalidation disabled")
            return

        class Handler(watchdog.events.FileSystemEventHandler):
            def __init__(self, cache) -> None:
                self.cache = cache

            def on_modified(self, event):
                if not event.is_directory:
                    logger.info(f"Invalidating cache for {event.src_path}")
                    # In a real implementation, we'd map path to cache keys
                    # For now, clear all or match specifically if keys are paths
                    if hasattr(self.cache, "clear"):
                        self.cache.clear()

        self.observer.schedule(Handler(self.cache), str(directory), recursive=True)
        self.observer.start()

    def stop(self):
        if self.observer:
            self.observer.stop()
            self.observer.join()


class MultiTierCache:
    """Multi-tier caching system with automatic tier management.

    Tiers:
    1. L1: cachetools TTLCache (fastest, automatic TTL, smallest)
    2. L2: cachetools LRUCache (medium-term, configurable size)
    3. L3: PersistDict (persistent, survives restarts, safe serialization)
    """

    def __init__(
        self,
        l1_size: int = 100,
        l2_size: int = 1000,
        l3_path: str | None = None,
        default_ttl: float | None = None,
    ) -> None:
        self.l1: TTLCache = TTLCache(maxsize=l1_size, ttl=default_ttl or 60)
        self.l1_size = l1_size
        self.l2: LRUCache = LRUCache(maxsize=l2_size)
        self.l2_size = l2_size
        # Use PersistDict for L3 (safe serialization via LMDB)
        self._l3_type: str = "none"
        if l3_path and PERSISTDICT_AVAILABLE:
            self.l3: "PersistDict | None" = PersistDict(
                database_path=l3_path,
                expiration_days=int(default_ttl / 86400) if default_ttl else 7,
                background_thread=False,
            )
            self._l3_type = "persistdict"
        else:
            self.l3 = None
        self.default_ttl = default_ttl

    def get(self, key: str) -> Any | None:
        if key in self.l1:
            return self.l1[key]
        if key in self.l2:
            value = self.l2[key]
            self.l1[key] = value
            return value
        if self.l3:
            value = self._get_l3(key)
            if value is not None:
                self.l2[key] = value
                self.l1[key] = value
                return value
        return None

    def _get_l3(self, key: str) -> Any | None:
        """Get from L3 cache (PersistDict)."""
        try:
            return self.l3[key]
        except KeyError:
            return None
        except Exception:
            return None

    def _set_l3(self, key: str, value: Any, ttl: float | None) -> None:
        """Set L3 cache (PersistDict)."""
        try:
            self.l3[key] = value
        except Exception:
            pass

    def _delete_l3(self, key: str) -> None:
        """Delete from L3 cache (PersistDict)."""
        try:
            self.l3.pop(key, None)
        except Exception:
            pass

    def _clear_l3(self) -> None:
        """Clear L3 cache (PersistDict)."""
        try:
            self.l3.clear()
        except Exception:
            pass

    def _len_l3(self) -> int:
        """Get L3 cache size (PersistDict)."""
        try:
            l3 = self.l3
            return len(l3) if l3 is not None else 0
        except Exception:
            return 0

    def set(self, key: str, value: Any, ttl: float | None = None) -> None:
        ttl = ttl or self.default_ttl
        self.l1[key] = value
        self.l2[key] = value
        self._set_l3(key, value, ttl)

    def delete(self, key: str) -> None:
        self.l1.pop(key, None)
        self.l2.pop(key, None)
        self._delete_l3(key)

    def clear(self) -> None:
        self.l1.clear()
        self.l2.clear()
        self._clear_l3()

    def stats(self) -> dict[str, Any]:
        stats: dict[str, Any] = {
            "l1_size": len(self.l1),
            "l1_max": self.l1_size,
            "l2_size": len(self.l2),
            "l2_max": self.l2_size,
        }
        if self.l3:
            try:
                stats["l3_size"] = self._len_l3()
                stats["l3_volume"] = "N/A"  # PersistDict doesn't have volume()
            except Exception:
                stats["l3_size"] = 0
                stats["l3_volume"] = 0
        else:
            stats["l3_size"] = 0
            stats["l3_volume"] = 0
        return stats

    def get_with_fetch(self, key: str, fetch_func: Any, ttl: float | None = None) -> Any:
        """Get value from cache, or fetch and store if missing (Singleflight coalescing)."""
        value = self.get(key)
        if value is not None:
            return value

        if not hasattr(self, "_singleflight"):
            self._singleflight = Singleflight()

        def _fetch_and_store():
            res = fetch_func()
            if res is not None:
                self.set(key, res, ttl=ttl)
            return res

        return self._singleflight.do(key, _fetch_and_store)

    def enable_invalidation(self, directory: str | Path) -> None:
        """Enable real-time cache invalidation based on file changes."""
        self.invalidator = CacheInvalidator(self)
        self.invalidator.watch(Path(directory))


_global_cache: MultiTierCache | None = None


def get_cache(
    l1_size: int = 100,
    l2_size: int = 1000,
    l3_path: str | None = None,
    default_ttl: float | None = None,
) -> MultiTierCache:
    """Get global multi-tier cache instance."""
    global _global_cache
    if _global_cache is None:
        _global_cache = MultiTierCache(l1_size=l1_size, l2_size=l2_size, l3_path=l3_path, default_ttl=default_ttl)
    return _global_cache
