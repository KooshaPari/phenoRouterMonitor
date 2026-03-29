"""Concurrency control helpers for run execution.

WP-5001: Resource-based dynamic limits and concurrency management.
Extracted from run_execution_core_helpers.py for maintainability.
"""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from thegent.config import ThegentSettings

logger = logging.getLogger(__name__)


@dataclass
class ConcurrencyState:
    """State for concurrency control."""

    active_runs: int = 0
    max_concurrent: int = 4
    resource_limit: float = 0.8
    current_load: float = 0.0
    bottlenecks: list[str] | None = None

    def __post_init__(self) -> None:
        if self.bottlenecks is None:
            self.bottlenecks = []


@dataclass
class ConcurrencyResult:
    """Result of concurrency check."""

    allowed: bool
    reason: str = ""
    wait_time: float = 0.0
    current_position: int = 0
    estimated_wait: float = 0.0


def get_resource_based_limit(settings: "ThegentSettings | None" = None) -> int:
    """Get maximum concurrent runs based on system resources.

    Args:
        settings: Optional settings for limits

    Returns:
        Maximum concurrent runs allowed
    """
    import os

    # Get CPU count
    cpu_count = os.cpu_count() or 4

    # Base limit on CPU count (leave 1 CPU for system)
    cpu_limit = max(1, cpu_count - 1)

    # Check memory (rough heuristic)
    try:
        import psutil

        mem = psutil.virtual_memory()
        mem_gb = mem.total / (1024**3)
        # Assume 2GB per run minimum
        mem_limit = max(1, int(mem_gb / 2))
    except ImportError:
        mem_limit = cpu_limit

    # Take minimum of limits
    max_concurrent = min(cpu_limit, mem_limit)

    # Apply settings override if set
    settings_limit = getattr(settings, "max_concurrent_runs", None) if settings else None
    if settings_limit:
        if settings_limit:
            max_concurrent = min(max_concurrent, settings_limit)

    return max(1, max_concurrent)


def check_concurrency(
    state: ConcurrencyState,
    run_id: str | None = None,
) -> ConcurrencyResult:
    """Check if a new run is allowed under concurrency limits.

    Args:
        state: Current concurrency state
        run_id: Optional run ID for queue position

    Returns:
        ConcurrencyResult with allow/deny and wait info
    """
    if state.active_runs < state.max_concurrent:
        return ConcurrencyResult(
            allowed=True,
            reason="Under limit",
            current_position=state.active_runs + 1,
        )

    if state.current_load >= state.resource_limit:
        return ConcurrencyResult(
            allowed=False,
            reason=f"Resource limit reached: {state.current_load:.1%} >= {state.resource_limit:.1%}",
            wait_time=30.0,
            current_position=state.active_runs,
            estimated_wait=30.0 * (state.active_runs - state.max_concurrent + 1),
        )

    # Check bottlenecks
    if state.bottlenecks:
        return ConcurrencyResult(
            allowed=False,
            reason=f"Bottlenecks detected: {', '.join(state.bottlenecks)}",
            wait_time=15.0,
            current_position=state.active_runs,
        )

    # Allow if under resource limit
    return ConcurrencyResult(
        allowed=True,
        reason="Under resource limit",
        current_position=state.active_runs + 1,
    )


def classify_burst_load(
    recent_requests: list[float],
    threshold: float = 5.0,
) -> tuple[bool, str]:
    """Classify if current load is a burst.

    WP-5002: Burst load classification.

    Args:
        recent_requests: List of recent request timestamps
        threshold: Requests per second threshold for burst

    Returns:
        Tuple of (is_burst, classification)
    """
    if not recent_requests:
        return False, "normal"

    now = time.monotonic()
    recent_count = sum(1 for t in recent_requests if now - t < 10)

    # Calculate rate
    if len(recent_requests) >= 2:
        time_span = recent_requests[-1] - recent_requests[0]
        if time_span > 0:
            rate = len(recent_requests) / time_span
        else:
            rate = 0
    else:
        rate = recent_count / 10 if recent_count else 0

    # Classify
    if rate > threshold * 3:
        return True, "extreme"
    elif rate > threshold * 2:
        return True, "high"
    elif rate > threshold:
        return True, "moderate"

    return False, "normal"


def get_adaptive_timeout(
    base_timeout: int,
    current_load: float,
    is_burst: bool = False,
) -> int:
    """Get adaptive timeout based on current load.

    Args:
        base_timeout: Base timeout in seconds
        current_load: Current system load (0-1)
        is_burst: If currently in burst mode

    Returns:
        Adjusted timeout in seconds
    """
    timeout = base_timeout

    # Increase timeout under high load
    if current_load > 0.8:
        timeout = int(timeout * 1.5)
    elif current_load > 0.6:
        timeout = int(timeout * 1.2)

    # Further increase during bursts
    if is_burst:
        timeout = int(timeout * 1.3)

    return timeout


__all__ = [
    "ConcurrencyState",
    "ConcurrencyResult",
    "get_resource_based_limit",
    "check_concurrency",
    "classify_burst_load",
    "get_adaptive_timeout",
]
