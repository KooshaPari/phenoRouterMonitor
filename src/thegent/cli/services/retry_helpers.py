"""Retry helper services extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

import random


def backoff_delay(attempt: int, max_delay: float = 60.0) -> float:
    """Return a capped exponential-jitter retry delay in seconds."""
    # Retry delay jitter does not require cryptographic randomness.
    return random.uniform(0, min(2**attempt, max_delay))  # noqa: S311


__all__ = ["backoff_delay"]
