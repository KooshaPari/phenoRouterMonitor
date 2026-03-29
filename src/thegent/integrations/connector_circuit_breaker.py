"""Connector circuit breakers.

Implements circuit breaker pattern for connector health management, allowing
graceful degradation when a connector experiences repeated failures.

# @trace WL-194
"""

from __future__ import annotations

import logging
from datetime import datetime, UTC
from enum import Enum

logger = logging.getLogger(__name__)


class CircuitState(Enum):
    """Circuit breaker state enum."""

    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class ConnectorCircuitBreaker:
    """Circuit breaker for connector failures.

    Tracks failure count and transitions between states (CLOSED -> OPEN -> HALF_OPEN -> CLOSED).
    Prevents cascading failures by blocking requests when the circuit is open.
    """

    def __init__(self, failure_threshold: int = 5, recovery_timeout_seconds: float = 60.0) -> None:
        """Initialize the circuit breaker.

        Args:
            failure_threshold: Number of failures required to open the circuit.
            recovery_timeout_seconds: Time (in seconds) before attempting recovery.

        Raises:
            ValueError: If failure_threshold <= 0 or recovery_timeout_seconds < 0.
        """
        if failure_threshold <= 0:
            raise ValueError("failure_threshold must be > 0")
        if recovery_timeout_seconds < 0:
            raise ValueError("recovery_timeout_seconds must be >= 0")

        self._failure_threshold = failure_threshold
        self._recovery_timeout_seconds = recovery_timeout_seconds
        self._failure_count = 0
        self._state = CircuitState.CLOSED
        self._opened_at: datetime | None = None

        logger.debug(f"Initialized circuit breaker: threshold={failure_threshold}, timeout={recovery_timeout_seconds}s")

    def record_failure(self) -> None:
        """Record a failure and update circuit state.

        If failure count reaches threshold, opens the circuit.
        If circuit is HALF_OPEN, any failure returns to OPEN.
        """
        self._failure_count += 1

        if self._failure_count >= self._failure_threshold and self._state == CircuitState.CLOSED:
            self._state = CircuitState.OPEN
            self._opened_at = datetime.now(UTC)
            logger.warning(
                f"Circuit breaker opened after {self._failure_count} failures (threshold={self._failure_threshold})"
            )
        elif self._state == CircuitState.HALF_OPEN:
            self._state = CircuitState.OPEN
            self._opened_at = datetime.now(UTC)
            logger.warning("Circuit breaker returned to OPEN after failure in HALF_OPEN state")

    def record_success(self) -> None:
        """Record a successful request and reset failures.

        If circuit is HALF_OPEN, transitions to CLOSED.
        If circuit is CLOSED, resets failure counter.
        """
        if self._state == CircuitState.HALF_OPEN:
            self._state = CircuitState.CLOSED
            self._failure_count = 0
            self._opened_at = None
            logger.info("Circuit breaker closed after successful recovery")
        elif self._state == CircuitState.CLOSED:
            self._failure_count = 0
            logger.debug("Failure count reset after successful request")

    def is_open(self) -> bool:
        """Check if the circuit is currently open (blocking requests).

        Returns:
            True if the circuit is OPEN or HALF_OPEN, False if CLOSED.
        """
        # If OPEN and recovery timeout has elapsed, transition to HALF_OPEN
        if self._state == CircuitState.OPEN and self._opened_at is not None:
            elapsed = (datetime.now(UTC) - self._opened_at).total_seconds()
            if elapsed >= self._recovery_timeout_seconds:
                self._state = CircuitState.HALF_OPEN
                logger.info("Circuit breaker transitioned to HALF_OPEN for recovery attempt")
                return False

        return self._state in (CircuitState.OPEN, CircuitState.HALF_OPEN)

    @property
    def state(self) -> CircuitState:
        """Get the current circuit state.

        Returns:
            Current CircuitState (CLOSED, OPEN, or HALF_OPEN).
        """
        # Trigger half-open check if needed
        if self._state == CircuitState.OPEN and self._opened_at is not None:
            elapsed = (datetime.now(UTC) - self._opened_at).total_seconds()
            if elapsed >= self._recovery_timeout_seconds:
                self._state = CircuitState.HALF_OPEN

        return self._state

    @property
    def failure_count(self) -> int:
        """Get the current failure count.

        Returns:
            Number of consecutive failures recorded.
        """
        return self._failure_count
