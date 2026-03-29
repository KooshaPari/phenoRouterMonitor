"""GW-59: Traffic mirroring — shadow A/B deployment (send to secondary silently).

Sends a copy of the request to a secondary endpoint asynchronously.
The primary response is returned; secondary response is discarded.

# @trace FR-AROUTE-059
"""

from __future__ import annotations

import logging
import random
from dataclasses import dataclass

_log = logging.getLogger(__name__)


@dataclass
class MirrorConfig:
    """Configuration for traffic mirroring to a secondary endpoint."""

    enabled: bool = False
    target_url: str = ""  # secondary endpoint URL
    sample_rate: float = 1.0  # 0.0–1.0, fraction of requests to mirror
    timeout_sec: float = 5.0


@dataclass
class MirrorResult:
    """Result of a mirroring attempt."""

    mirrored: bool
    error: str = ""


def should_mirror(config: MirrorConfig) -> bool:
    """Determine whether the current request should be mirrored.

    Args:
        config: Mirror configuration.

    Returns:
        True if mirroring should occur for this request.
    """
    return config.enabled and config.target_url != "" and random.random() < config.sample_rate


async def mirror_request(
    config: MirrorConfig,
    url: str,
    body: dict,
    headers: dict,
) -> MirrorResult:
    """Fire-and-forget async POST to the mirror target.

    Sends the request body to config.target_url + url path. Any exception is
    caught and returned as a MirrorResult with mirrored=False. The primary
    response is never affected.

    Args:
        config: Mirror configuration including target URL and timeout.
        url: The request path/URL to append to the target base URL.
        body: The request body to POST as JSON.
        headers: HTTP headers to forward with the mirror request.

    Returns:
        MirrorResult indicating success or failure.
    """
    import httpx  # optional dep: imported inside function

    try:
        mirror_url = config.target_url.rstrip("/") + "/" + url.lstrip("/")
        async with httpx.AsyncClient(timeout=config.timeout_sec) as client:
            await client.post(mirror_url, json=body, headers=headers)
        _log.debug("Mirrored request to %s", mirror_url)
        return MirrorResult(mirrored=True)
    except Exception as exc:  # noqa: BLE001
        _log.debug("Mirror request failed: %s", exc)
        return MirrorResult(mirrored=False, error=str(exc))
