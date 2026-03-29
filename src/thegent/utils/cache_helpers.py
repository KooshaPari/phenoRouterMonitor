"""Caching utilities for thegent.

Common caching decorators and helpers.
"""

from __future__ import annotations

import time
from functools import wraps
from typing import Any, Callable, TypeVar

T = TypeVar("T")


def cached(max_age: float = 60.0) -> Callable[[Callable[..., T]], Callable[..., T]]:
    """Decorator that caches function results for a time period.

    Args:
        max_age: Cache age in seconds
    """
    cache: dict[str, tuple[float, T]] = {}

    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        @wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> T:
            key = f"{func.__name__}:{args}:{kwargs}"
            now = time.time()
            if key in cache:
                cached_time, result = cache[key]
                if now - cached_time < max_age:
                    return result
            result = func(*args, **kwargs)
            cache[key] = (now, result)
            return result

        return wrapper

    return decorator


class Cache:
    """Simple in-memory cache."""

    def __init__(self, max_size: int = 100):
        self._cache: dict[str, Any] = {}
        self._max_size = max_size

    def get(self, key: str) -> Any | None:
        return self._cache.get(key)

    def set(self, key: str, value: Any) -> None:
        if len(self._cache) >= self._max_size:
            self._cache.pop(next(iter(self._cache)))
        self._cache[key] = value

    def clear(self) -> None:
        self._cache.clear()

    def __len__(self) -> int:
        return len(self._cache)
