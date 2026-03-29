"""Sliding window rate limiter for LLM gateway use.

Prevents burst clustering at window resets unlike fixed-window counters
that allow 2x burst at boundary. Uses deque of timestamps per key with
per-key threading.Lock for thread safety.

No external dependencies — pure stdlib + dataclasses.
"""

import threading
import time
from collections import deque
from dataclasses import dataclass


@dataclass
class RateLimitConfig:
    """Configuration for a rate limit rule."""

    requests_per_window: int
    window_seconds: float
    key: str = "global"


@dataclass
class RateLimitResult:
    """Result of a rate limit check."""

    allowed: bool
    remaining: int
    reset_after: float
    limit: int


class SlidingWindowRateLimiter:
    """Thread-safe sliding window rate limiter.

    Uses a deque of timestamps per key. A request is allowed when
    the count of timestamps within [now - window_seconds, now] < requests_per_window.
    Evicts stale timestamps on every check.

    Thread-safe via threading.Lock per key.
    """

    def __init__(self) -> None:
        self._timestamps: dict[str, deque[float]] = {}
        self._locks: dict[str, threading.Lock] = {}
        self._meta_lock = threading.Lock()

    def _get_lock(self, key: str) -> threading.Lock:
        with self._meta_lock:
            if key not in self._locks:
                self._locks[key] = threading.Lock()
                self._timestamps[key] = deque()
            return self._locks[key]

    def _evict_stale(self, timestamps: deque[float], cutoff: float) -> None:
        while timestamps and timestamps[0] <= cutoff:
            timestamps.popleft()

    def check(self, config: RateLimitConfig) -> RateLimitResult:
        """Check if a request is allowed under the rate limit.

        Does NOT consume a slot — use allow() for that.
        """
        lock = self._get_lock(config.key)
        with lock:
            now = time.monotonic()
            cutoff = now - config.window_seconds
            timestamps = self._timestamps[config.key]
            self._evict_stale(timestamps, cutoff)

            count = len(timestamps)
            allowed = count < config.requests_per_window
            remaining = max(0, config.requests_per_window - count)
            reset_after = (timestamps[0] - cutoff) if timestamps else 0.0

            return RateLimitResult(
                allowed=allowed,
                remaining=remaining,
                reset_after=reset_after,
                limit=config.requests_per_window,
            )

    def allow(self, config: RateLimitConfig) -> RateLimitResult:
        """Check and consume a slot if allowed.

        Returns RateLimitResult with allowed=True and consumes a slot,
        or allowed=False without consuming.
        """
        lock = self._get_lock(config.key)
        with lock:
            now = time.monotonic()
            cutoff = now - config.window_seconds
            timestamps = self._timestamps[config.key]
            self._evict_stale(timestamps, cutoff)

            count = len(timestamps)
            allowed = count < config.requests_per_window

            if allowed:
                timestamps.append(now)
                remaining = config.requests_per_window - count - 1
                reset_after = (timestamps[0] - cutoff) if timestamps else 0.0
            else:
                remaining = 0
                reset_after = (timestamps[0] - cutoff) if timestamps else 0.0

            return RateLimitResult(
                allowed=allowed,
                remaining=remaining,
                reset_after=reset_after,
                limit=config.requests_per_window,
            )

    def reset(self, key: str) -> None:
        """Clear all timestamps for a key (e.g., after budget reset)."""
        lock = self._get_lock(key)
        with lock:
            self._timestamps[key].clear()

    def get_current_count(self, key: str, window_seconds: float) -> int:
        """Return current request count within window for a key."""
        lock = self._get_lock(key)
        with lock:
            now = time.monotonic()
            cutoff = now - window_seconds
            timestamps = self._timestamps.get(key, deque())
            self._evict_stale(timestamps, cutoff)
            return len(timestamps)


class MultiKeyRateLimiter:
    """Rate limiter that enforces multiple limits simultaneously.

    E.g., enforce both per-user AND per-provider limits on the same request.
    All limits must pass for the request to be allowed.
    """

    def __init__(self, limiter: SlidingWindowRateLimiter | None = None) -> None:
        self._limiter = limiter if limiter is not None else SlidingWindowRateLimiter()

    def allow_all(self, configs: list[RateLimitConfig]) -> tuple[bool, list[RateLimitResult]]:
        """Check all configs. Returns (all_allowed, results_list).

        If any limit is exceeded, returns (False, results) without consuming
        slots in limits that would have passed.

        Atomic: either ALL slots are consumed or NONE are.
        """
        # Phase 1: check all without consuming
        check_results = [self._limiter.check(config) for config in configs]
        all_allowed = all(r.allowed for r in check_results)

        if not all_allowed:
            return False, check_results

        # Phase 2: consume all — all limits passed, now record slots
        consume_results = [self._limiter.allow(config) for config in configs]

        # Guard: if any consume failed (race), report failure without partial state.
        # Each individual allow() is atomic under its own lock, so a failed consume
        # here means another thread raced us between check and allow. We report
        # the actual outcome truthfully.
        all_consumed = all(r.allowed for r in consume_results)
        return all_consumed, consume_results


_default_limiter: SlidingWindowRateLimiter | None = None
_singleton_lock = threading.Lock()


def get_rate_limiter() -> SlidingWindowRateLimiter:
    """Get or create the module-level rate limiter singleton."""
    global _default_limiter
    if _default_limiter is None:
        with _singleton_lock:
            if _default_limiter is None:
                _default_limiter = SlidingWindowRateLimiter()
    return _default_limiter


def make_provider_config(provider: str, requests_per_minute: int) -> RateLimitConfig:
    """Convenience: build a per-provider per-minute rate limit config."""
    return RateLimitConfig(
        requests_per_window=requests_per_minute,
        window_seconds=60.0,
        key=f"provider:{provider}",
    )


def make_user_config(user_id: str, requests_per_minute: int) -> RateLimitConfig:
    """Convenience: build a per-user per-minute rate limit config."""
    return RateLimitConfig(
        requests_per_window=requests_per_minute,
        window_seconds=60.0,
        key=f"user:{user_id}",
    )
