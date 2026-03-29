"""WP-2003: Provider loop time bounds (max 5 min by default).

Provides PROVIDER_LOOP_TIMEOUT_SEC and the async helper run_with_provider_loop_timeout()
that wraps any coroutine with asyncio.wait_for and raises ProviderLoopTimeout on expiry.

Fail fast: asyncio.TimeoutError is NEVER swallowed silently — it is always re-raised
as ProviderLoopTimeout with full log context.

# @trace WL-039 WP-2003
"""

from __future__ import annotations

import asyncio
import logging
import os
from typing import TYPE_CHECKING, TypeVar

if TYPE_CHECKING:
    from collections.abc import Coroutine

_log = logging.getLogger(__name__)

PROVIDER_LOOP_TIMEOUT_SEC: int = int(os.getenv("THGENT_PROVIDER_LOOP_TIMEOUT", "300"))

T = TypeVar("T")


class ProviderLoopTimeout(Exception):
    """Raised when the provider selection + retry loop exceeds PROVIDER_LOOP_TIMEOUT_SEC.

    Callers MUST NOT swallow this silently.  Log the event, surface the error,
    and abort the current run.
    """

    def __init__(self, timeout_sec: int, context: str = "") -> None:
        self.timeout_sec = timeout_sec
        self.context = context
        msg = f"Provider loop timed out after {timeout_sec}s" + (f" [{context}]" if context else "")
        super().__init__(msg)


async def run_with_provider_loop_timeout(
    coro: Coroutine[object, object, T],
    timeout_sec: int | None = None,
    context: str = "",
) -> T:
    """Run coro, raising ProviderLoopTimeout if it exceeds timeout_sec.

    Args:
        coro: Coroutine to execute.
        timeout_sec: Override for timeout; defaults to PROVIDER_LOOP_TIMEOUT_SEC.
        context: Human-readable label for logging (e.g., provider name).

    Returns:
        The return value of coro.

    Raises:
        ProviderLoopTimeout: If coro does not complete within timeout_sec.
        Any exception raised by coro propagates directly (fail fast).
    """
    limit = timeout_sec if timeout_sec is not None else PROVIDER_LOOP_TIMEOUT_SEC
    try:
        return await asyncio.wait_for(coro, timeout=limit)
    except TimeoutError as exc:
        _log.error(
            "PROVIDER_LOOP_TIMEOUT: provider loop exceeded %ds. context=%r — aborting.",
            limit,
            context,
        )
        raise ProviderLoopTimeout(limit, context) from exc
