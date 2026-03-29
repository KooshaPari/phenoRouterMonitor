"""GW-62: Pre-call context window validation.

Checks if the request's token count fits within the model's context window.
Triggers context_window_fallbacks if the check fails.

# @trace FR-AROUTE-062
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Known context window limits (in tokens)
# ---------------------------------------------------------------------------

CONTEXT_WINDOW_LIMITS: dict[str, int] = {
    "gpt-4o": 128_000,
    "gpt-4o-mini": 128_000,
    "claude-opus-4-5": 200_000,
    "claude-sonnet-4-5": 200_000,
    "claude-haiku-4-5": 200_000,
    "gemini-1.5-pro": 1_000_000,
    "gemini-1.5-flash": 1_000_000,
}


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


@dataclass
class ContextWindowCheckResult:
    """Result of a context window validation check."""

    fits: bool
    estimated_tokens: int
    model_limit: int | None  # None if model unknown
    overflow: int  # 0 if fits, else estimated - limit


# ---------------------------------------------------------------------------
# Functions
# ---------------------------------------------------------------------------


def estimate_token_count(messages: list[dict]) -> int:
    """Estimate the token count for a list of messages.

    Rough heuristic: 1 token ≈ 4 characters of text.

    Args:
        messages: List of message dicts (e.g., OpenAI chat format).

    Returns:
        Integer token estimate.
    """
    return sum(len(str(msg)) // 4 for msg in messages)


def check_context_window(
    model: str,
    messages: list[dict],
) -> ContextWindowCheckResult:
    """Check whether the request fits within the model's context window.

    Unknown models are allowed through (fits=True) to avoid blocking
    requests to models not yet in the registry.

    Args:
        model: Model identifier string (e.g. "gpt-4o").
        messages: List of message dicts whose tokens to estimate.

    Returns:
        ContextWindowCheckResult with fit status, estimated tokens, and overflow.
    """
    estimated = estimate_token_count(messages)
    limit = CONTEXT_WINDOW_LIMITS.get(model)

    if limit is None:
        _log.debug(
            "Context window check: unknown model=%r, allowing through (estimated=%d)",
            model,
            estimated,
        )
        return ContextWindowCheckResult(
            fits=True,
            estimated_tokens=estimated,
            model_limit=None,
            overflow=0,
        )

    if estimated > limit:
        overflow = estimated - limit
        _log.warning(
            "Context window overflow: model=%r estimated=%d limit=%d overflow=%d",
            model,
            estimated,
            limit,
            overflow,
        )
        return ContextWindowCheckResult(
            fits=False,
            estimated_tokens=estimated,
            model_limit=limit,
            overflow=overflow,
        )

    _log.debug(
        "Context window OK: model=%r estimated=%d limit=%d",
        model,
        estimated,
        limit,
    )
    return ContextWindowCheckResult(
        fits=True,
        estimated_tokens=estimated,
        model_limit=limit,
        overflow=0,
    )


def select_fallback_model(
    model: str,
    fallbacks: list[str],
    messages: list[dict],
) -> str:
    """Select the first fallback model whose context window fits the messages.

    Iterates through fallbacks in order and returns the first that fits.
    If none fit, returns the last fallback (better to attempt than to block).
    If fallbacks is empty, returns the original model unchanged.

    Args:
        model: The original model that failed the context window check.
        fallbacks: Ordered list of fallback model identifiers to try.
        messages: The message list to check against each fallback's limit.

    Returns:
        A model identifier string — the first fitting fallback, or last if none fit.
    """
    if not fallbacks:
        return model

    estimated = estimate_token_count(messages)

    for candidate in fallbacks:
        limit = CONTEXT_WINDOW_LIMITS.get(candidate)
        if limit is None or estimated <= limit:
            _log.debug(
                "Selected fallback model=%r (estimated=%d fits limit=%s)",
                candidate,
                estimated,
                limit,
            )
            return candidate

    last = fallbacks[-1]
    _log.warning(
        "No fallback fits estimated=%d; using last fallback model=%r",
        estimated,
        last,
    )
    return last
