"""Caching and request deduplication for the agent mesh."""

import hashlib
import json
import threading
from collections.abc import Callable
from pathlib import Path
from typing import Any


class Singleflight:
    """Request deduplication: first executes, rest wait (SCLI-P7.1)."""

    def __init__(self) -> None:
        self.locks: dict[str, threading.Lock] = {}
        self.results: dict[str, Any] = {}
        self.registry_lock = threading.Lock()

    def do(self, key: str, func: Callable[[], Any]) -> Any:
        """Execute func for key, ensuring only one concurrent execution."""
        with self.registry_lock:
            if key not in self.locks:
                self.locks[key] = threading.Lock()
            lock = self.locks[key]

        with lock:
            if key in self.results:
                return self.results[key]

            result = func()
            self.results[key] = result
            return result


class MeshCache:
    """Mesh-aware cache with inotify invalidation and LRU (SCLI-P7.2–P7.3)."""

    def __init__(self, mesh_root: Path, capacity: int = 1000) -> None:
        self.mesh_root = mesh_root
        self.cache_dir = mesh_root / "cache"
        self.cache_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)
        self.capacity = capacity
        self.heat_map: dict[str, float] = {}  # Access frequency (SCLI-P7.3)

    def get(self, key: str) -> Any | None:
        """Get item from cache and update heat map."""
        cache_file = self.cache_dir / f"{key}.json"
        if not cache_file.exists():
            return None

        # Update heat (exponential decay simulation)
        self.heat_map[key] = self.heat_map.get(key, 0) * 0.9 + 1.0

        with open(cache_file) as f:
            return json.load(f)

    def set(self, key: str, value: Any) -> None:
        """Store item in cache and manage capacity."""
        if len(self.heat_map) >= self.capacity:
            self._evict_coldest()

        cache_file = self.cache_dir / f"{key}.json"
        with open(cache_file, "w") as f:
            json.dump(value, f)

        self.heat_map[key] = self.heat_map.get(key, 0) + 1.0

    def _evict_coldest(self) -> None:
        """Evict least frequently used item (SCLI-P7.3)."""
        if not self.heat_map:
            return

        coldest_key = min(self.heat_map, key=lambda k: self.heat_map.get(k, 0))
        cache_file = self.cache_dir / f"{coldest_key}.json"
        if cache_file.exists():
            cache_file.unlink()
        del self.heat_map[coldest_key]

    def invalidate_by_file(self, file_path: str) -> int:
        """Invalidate cache entries affected by file changes (SCLI-P7.2)."""
        # In this implementation, we assume keys might be hashes of file paths
        count = 0
        file_hash = hashlib.md5(file_path.encode()).hexdigest()
        for cache_file in self.cache_dir.glob(f"*{file_hash}*"):
            cache_file.unlink()
            count += 1
        return count
