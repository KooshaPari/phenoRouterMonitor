"""API Rate-Limit Backoff Controls (WL-169): Exponential backoff for rate-limited APIs.

Provides retry and backoff configuration for APIs that return 429 (Too Many Requests)
or 503 (Service Unavailable) responses. Uses tenacity for retry logic with
exponential backoff and jitter to avoid thundering herd.

The RateLimitBackoffManager can be used standalone for computing backoff times,
or integrated with tenacity decorators for automatic retry.
"""

import logging
import random
from dataclasses import dataclass
from typing import ClassVar

from tenacity import (
    retry_if_result,
    stop_after_attempt,
    wait_exponential,
)

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------


@dataclass
class RateLimitConfig:
    """Configuration for rate-limit backoff behavior."""

    max_retries: int = 5
    """Maximum number of retry attempts (total attempts = max_retries + 1)."""

    initial_wait: float = 1.0
    """Initial wait time in seconds after first rate-limit response."""

    max_wait: float = 60.0
    """Maximum wait time in seconds (caps exponential backoff)."""

    multiplier: float = 2.0
    """Exponential backoff multiplier (e.g., 2.0 for doubling)."""

    def validate(self) -> None:
        """Validate configuration parameters.

        Raises:
            ValueError: If configuration is invalid.
        """
        if self.max_retries < 0:
            raise ValueError(f"max_retries must be >= 0, got {self.max_retries}")
        if self.initial_wait <= 0:
            raise ValueError(f"initial_wait must be > 0, got {self.initial_wait}")
        if self.max_wait < self.initial_wait:
            raise ValueError(f"max_wait ({self.max_wait}) must be >= initial_wait ({self.initial_wait})")
        if self.multiplier < 1.0:
            raise ValueError(f"multiplier must be >= 1.0, got {self.multiplier}")


# ---------------------------------------------------------------------------
# Manager
# ---------------------------------------------------------------------------


class RateLimitBackoffManager:
    """Manager for rate-limit backoff and retry configuration."""

    # HTTP status codes indicating rate limiting
    RATE_LIMIT_CODES: ClassVar[set[int]] = {429, 503}

    def __init__(self, config: RateLimitConfig | None = None):
        """Initialize the rate-limit manager.

        Args:
            config: Rate-limit configuration. Defaults to RateLimitConfig().

        Raises:
            ValueError: If configuration is invalid.
        """
        self.config = config or RateLimitConfig()
        self.config.validate()

    def is_rate_limited(self, response_code: int) -> bool:
        """Check if a response code indicates rate limiting.

        Args:
            response_code: HTTP status code (e.g., 429, 503).

        Returns:
            True if the code indicates rate limiting, False otherwise.
        """
        return response_code in self.RATE_LIMIT_CODES

    def compute_wait(self, attempt: int) -> float:
        """Compute wait time for a given attempt number.

        Uses exponential backoff with jitter:
            wait = min(initial_wait * (multiplier ** attempt) + random_jitter, max_wait)

        Args:
            attempt: The attempt number (0-indexed, so first retry is attempt=1).

        Returns:
            Wait time in seconds.
        """
        if attempt <= 0:
            return 0.0

        # Exponential backoff: initial_wait * (multiplier ** (attempt - 1))
        base_wait = self.config.initial_wait * (self.config.multiplier ** (attempt - 1))

        # Add jitter (±10% of base_wait)
        jitter = random.uniform(-0.1, 0.1) * base_wait
        wait_time = base_wait + jitter

        # Cap at max_wait
        return min(wait_time, self.config.max_wait)

    def get_retry_config(self) -> dict:
        """Return a tenacity-compatible retry configuration.

        Returns:
            A dictionary of kwargs for @retry() decorator.
            Example: tenacity.retry(**manager.get_retry_config())

        Example:
            >>> manager = RateLimitBackoffManager()
            >>> @tenacity.retry(**manager.get_retry_config())
            ... def call_api():
            ...     ...
        """
        return {
            "wait": wait_exponential(
                multiplier=self.config.initial_wait,
                min=self.config.initial_wait,
                max=self.config.max_wait,
            ),
            "stop": stop_after_attempt(self.config.max_retries + 1),
            "reraise": True,
        }

    def make_retry_decorator(self):
        """Create a tenacity retry decorator for this config.

        Returns a decorator that retries on rate-limit codes.

        Example:
            >>> manager = RateLimitBackoffManager()
            >>> @manager.make_retry_decorator()
            ... def call_api():
            ...     # raises exception with response_code attribute
            ...     ...
        """
        from tenacity import retry as tenacity_retry

        config = self.get_retry_config()
        # Add a predicate to detect rate-limit exceptions
        return tenacity_retry(
            wait=config["wait"],
            stop=config["stop"],
            retry=retry_if_result(lambda result: isinstance(result, int) and self.is_rate_limited(result)),
            reraise=True,
        )
