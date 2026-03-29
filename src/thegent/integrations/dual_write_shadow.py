"""Dual-write shadow mode for observing external system mutations.

Implements an observe-only shadow mode before enabling full external mutation.
Enables comparison between primary and shadow write results to identify divergences.

# @trace WL-243
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Callable

logger = logging.getLogger(__name__)


@dataclass
class ShadowWriteResult:
    """Result of a dual-write operation comparing primary and shadow writes."""

    record_id: str
    primary_ok: bool
    shadow_ok: bool


class DualWriteShadowMode:
    """Manages dual-write shadow mode for observing system mutations."""

    def __init__(self, enabled: bool = True) -> None:
        """Initialize the dual-write shadow mode.

        Args:
            enabled: Whether shadow mode is enabled.
        """
        self._enabled = enabled
        self._results: list[ShadowWriteResult] = []
        logger.debug(f"Initialized dual-write shadow mode (enabled={enabled})")

    def write(
        self,
        record_id: str,
        primary_fn: Callable[[], bool],
        shadow_fn: Callable[[], bool],
    ) -> ShadowWriteResult:
        """Execute a dual-write operation with shadow mode.

        Always executes primary_fn. If enabled, also executes shadow_fn
        and records the results for later analysis.

        Args:
            record_id: Identifier for the record being written.
            primary_fn: Function that performs the primary write. Should return True on success.
            shadow_fn: Function that performs the shadow write. Should return True on success.

        Returns:
            ShadowWriteResult containing the outcome of both writes.
        """
        primary_ok = False
        shadow_ok = False

        try:
            primary_ok = primary_fn()
        except Exception as e:
            logger.error(f"Primary write failed for record {record_id}: {e}")

        if self._enabled:
            try:
                shadow_ok = shadow_fn()
            except Exception as e:
                logger.warning(f"Shadow write failed for record {record_id}: {e}")

        result = ShadowWriteResult(record_id=record_id, primary_ok=primary_ok, shadow_ok=shadow_ok)
        self._results.append(result)

        if primary_ok != shadow_ok:
            logger.warning(
                f"Divergence detected for record {record_id}: primary_ok={primary_ok}, shadow_ok={shadow_ok}"
            )

        return result

    def divergences(self, results: list[ShadowWriteResult]) -> list[ShadowWriteResult]:
        """Filter results where primary and shadow outcomes diverged.

        Args:
            results: List of ShadowWriteResult objects to filter.

        Returns:
            List of results where primary_ok != shadow_ok.
        """
        diverged = [r for r in results if r.primary_ok != r.shadow_ok]
        logger.debug(f"Found {len(diverged)} divergences out of {len(results)} writes")
        return diverged

    def is_enabled(self) -> bool:
        """Check if shadow mode is enabled.

        Returns:
            True if shadow mode is enabled, False otherwise.
        """
        return self._enabled
