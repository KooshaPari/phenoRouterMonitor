"""GW-53: Webhook guardrail — POST to external URL for verdict.

Sends request data to a webhook URL, expects {verdict: "allow"|"block", transformedData?: {...}}.

# @trace FR-GUARD-053
"""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass

import httpx

_log = logging.getLogger(__name__)

_VALID_VERDICTS: frozenset[str] = frozenset({"allow", "block"})


@dataclass
class WebhookGuardrailConfig:
    url: str
    timeout_sec: float = 2.0
    secret: str = ""  # sent as X-Webhook-Secret header if set
    on_failure: str = "allow"  # "allow" | "block" — what to do if webhook unreachable


@dataclass
class WebhookVerdict:
    verdict: str  # "allow" | "block"
    transformed_data: dict | None = None
    error: str = ""  # set if webhook call failed


def call_webhook_guardrail(
    config: WebhookGuardrailConfig,
    payload: dict,
) -> WebhookVerdict:
    """POST payload to the webhook URL and return its verdict.

    On timeout or any connection error the ``config.on_failure`` verdict is
    returned so that routing can proceed (or block) as configured.

    Args:
        config: Webhook configuration including URL, timeout, secret, and
            on-failure policy.
        payload: JSON-serialisable dict to POST as the request body.

    Returns:
        WebhookVerdict with ``verdict`` ("allow" or "block"),
        optional ``transformed_data`` from the response, and an ``error``
        string when the call failed.
    """
    headers: dict[str, str] = {"Content-Type": "application/json"}
    if config.secret:
        headers["X-Webhook-Secret"] = config.secret

    try:
        response = httpx.post(
            config.url,
            content=json.dumps(payload).decode(),
            headers=headers,
            timeout=config.timeout_sec,
        )
        response.raise_for_status()
        body = response.json()
    except Exception as exc:  # noqa: BLE001
        _log.warning(
            "Webhook guardrail call failed url=%s error=%s on_failure=%s",
            config.url,
            exc,
            config.on_failure,
        )
        return WebhookVerdict(verdict=config.on_failure, error=str(exc))

    raw_verdict: str = body.get("verdict", "allow")
    if raw_verdict not in _VALID_VERDICTS:
        _log.warning(
            "Webhook returned unrecognised verdict=%r; treating as 'allow'",
            raw_verdict,
        )
        raw_verdict = "allow"

    transformed = body.get("transformedData")
    if transformed is not None and not isinstance(transformed, dict):
        transformed = None

    return WebhookVerdict(verdict=raw_verdict, transformed_data=transformed)
