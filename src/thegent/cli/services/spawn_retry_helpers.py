"""Spawn retry helper services extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

import errno

EAGAIN_ERRNOS: frozenset[int] = frozenset({errno.EAGAIN, errno.EWOULDBLOCK})


def retry_if_eagain(exc: BaseException) -> bool:
    """Return True when *exc* is an OSError due to EAGAIN/EWOULDBLOCK."""
    return isinstance(exc, OSError) and exc.errno in EAGAIN_ERRNOS


__all__ = ["EAGAIN_ERRNOS", "retry_if_eagain"]
