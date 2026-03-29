"""GW-60: EWMA latency tracking for fastest-provider routing.

Exponential Weighted Moving Average latency per provider/model.

# @trace FR-AROUTE-060
"""

from __future__ import annotations

import logging
import threading
import time
from dataclasses import dataclass, field

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Configuration and data types
# ---------------------------------------------------------------------------


@dataclass
class EWMAConfig:
    """Configuration for the EWMA latency tracker."""

    alpha: float = 0.3  # smoothing factor (higher = more responsive)
    initial_latency_ms: float = 1000.0  # assumed latency for new providers


@dataclass
class LatencyRecord:
    """EWMA latency record for a single provider+model pair."""

    provider: str
    model: str
    ewma_ms: float
    sample_count: int = 0
    last_updated: float = field(default_factory=time.monotonic)


# ---------------------------------------------------------------------------
# Tracker
# ---------------------------------------------------------------------------


class EWMALatencyTracker:
    """Thread-safe EWMA latency tracker keyed by (provider, model).

    Records latency samples and maintains an exponential weighted moving
    average per provider+model combination. Used for fastest-provider routing.
    """

    def __init__(self, config: EWMAConfig | None = None) -> None:
        self._config = config or EWMAConfig()
        self._records: dict[str, LatencyRecord] = {}
        self._lock = threading.Lock()

    def _key(self, provider: str, model: str) -> str:
        """Construct the dict key for a (provider, model) pair."""
        return f"{provider}:{model}"

    def record(self, provider: str, model: str, latency_ms: float) -> None:
        """Record a latency sample and update the EWMA.

        EWMA update: new = alpha * sample + (1 - alpha) * old_ewma

        Args:
            provider: Provider identifier (e.g. "openai").
            model: Model identifier (e.g. "gpt-4o").
            latency_ms: Observed latency in milliseconds.
        """
        key = self._key(provider, model)
        alpha = self._config.alpha

        with self._lock:
            if key in self._records:
                rec = self._records[key]
                rec.ewma_ms = alpha * latency_ms + (1.0 - alpha) * rec.ewma_ms
                rec.sample_count += 1
                rec.last_updated = time.monotonic()
            else:
                self._records[key] = LatencyRecord(
                    provider=provider,
                    model=model,
                    ewma_ms=latency_ms,
                    sample_count=1,
                    last_updated=time.monotonic(),
                )

        _log.debug(
            "Recorded latency provider=%s model=%s sample_ms=%.1f ewma_ms=%.1f",
            provider,
            model,
            latency_ms,
            self._records[key].ewma_ms,
        )

    def get_latency(self, provider: str, model: str) -> float:
        """Return the current EWMA latency for a provider+model pair.

        Returns initial_latency_ms if no data has been recorded.

        Args:
            provider: Provider identifier.
            model: Model identifier.

        Returns:
            EWMA latency in milliseconds.
        """
        key = self._key(provider, model)
        with self._lock:
            if key in self._records:
                return self._records[key].ewma_ms
        return self._config.initial_latency_ms

    def rank_by_latency(
        self,
        candidates: list[tuple[str, str]],
    ) -> list[tuple[str, str]]:
        """Rank candidates ascending by EWMA latency (fastest first).

        Args:
            candidates: List of (provider, model) tuples to rank.

        Returns:
            New list sorted with lowest EWMA latency first.
        """
        return sorted(candidates, key=lambda pair: self.get_latency(pair[0], pair[1]))


# ---------------------------------------------------------------------------
# Singleton
# ---------------------------------------------------------------------------

_singleton: EWMALatencyTracker | None = None
_singleton_lock = threading.Lock()


def get_latency_tracker() -> EWMALatencyTracker:
    """Return the process-global EWMALatencyTracker singleton."""
    global _singleton  # noqa: PLW0603
    with _singleton_lock:
        if _singleton is None:
            _singleton = EWMALatencyTracker()
    return _singleton


def reset_latency_tracker() -> None:
    """Reset the singleton (for testing only)."""
    global _singleton  # noqa: PLW0603
    with _singleton_lock:
        _singleton = None
