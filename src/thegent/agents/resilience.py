"""Retry and fallback logic for agent runs.

Distinguishes:
- rate_limit / transient: retry same provider (429, 502/503/504, etc.)
- usage_limit: subscription/quota exhausted; fallback to different provider.

GOVERNANCE NOTE (Phase 1B Zero-Bloat Refactor):
For generic retry logic (not agent-specific), prefer thegent.resilience module
which uses tenacity decorators. This module contains agent-specific failure
classification and provider fallback strategies (not for generic retry).
See: src/thegent/resilience.py for unified @transient_retry, @cas_retry, etc.
"""

import contextlib
import re
from collections.abc import Callable
from dataclasses import dataclass
from enum import StrEnum
from types import MappingProxyType
from typing import Any, ClassVar, TypeVar

from pybreaker import STATE_OPEN, CircuitBreaker
from tenacity import (
    retry,
    retry_if_exception_type,
    stop_after_attempt,
    wait_random_exponential,
)

from thegent.agents.base import RunResult

T = TypeVar("T")


class FailureKind(StrEnum):
    """Classification of agent run failure."""

    RATE_LIMIT = "rate_limit"  # 429, too many requests; retry same provider
    TRANSIENT = "transient"  # 502/503/504, reconnecting; retry same provider
    USAGE_LIMIT = "usage_limit"  # Quota/subscription exhausted; fallback to different provider
    LOGIC_ERROR = "logic_error"  # Model output invalid or policy violation
    TIMEOUT = "timeout"
    PERMANENT = "permanent"  # Config error, unknown model
    UNKNOWN = "unknown"  # Not retryable, not fallback-worthy


class ToolClass(StrEnum):
    """WP-2002: Classification of tool calls for specialized retries."""

    MODEL = "model"
    STORAGE = "storage"
    NETWORK = "network"
    COMPUTE = "compute"


@dataclass
class RetryBudget:
    """WP-2002: SLO-aware retry budget."""

    max_retries: int = 3
    total_timeout_ms: int = 60000
    min_success_rate: float = 0.9


class ToolCircuitBreaker:
    """WP-2003: Circuit breaker for individual tools and models. Uses pybreaker."""

    def __init__(self, name: str, threshold: int = 5, window_s: int = 300) -> None:
        self.name = name
        self._breaker = CircuitBreaker(
            fail_max=threshold,
            reset_timeout=window_s,
            name=name,
        )

    def record_failure(self) -> None:
        """Record a failure event."""
        with contextlib.suppress(Exception):
            self._breaker.call(lambda: (_ for _ in ()).throw(RuntimeError("recorded")))

    def is_open(self) -> bool:
        """True if the circuit is open (too many recent failures)."""
        return self._breaker.current_state == STATE_OPEN


class RecoveryEngine:
    """WP-2004: Automated recovery playbooks for known failure patterns."""

    PLAYBOOKS: ClassVar[Any] = MappingProxyType(
        {
            "rate_limit": "Reduce concurrency or switch to high-quota provider",
            "context_overflow": "Summarize history or use model with larger context",
            "structural_drift": "Refresh parser schemas or switch to legacy adapter",
        }
    )

    def suggest_playbook(self, failure_type: str) -> str:
        """Return a recovery playbook for the given failure type."""
        return self.PLAYBOOKS.get(failure_type, "Standard retry with backoff")


class FailureTaxonomy(StrEnum):
    """WP-2005: Granular taxonomy of failures for root cause analysis."""

    API_AUTH = "api.auth"
    API_QUOTA = "api.quota"
    MODEL_HALLUCINATION = "model.hallucination"
    PARSER_INVALID_XML = "parser.invalid_xml"
    CONTRACT_VIOLATION = "contract.violation"


def classify_to_taxonomy(error_msg: str) -> FailureTaxonomy:
    """Classify a raw error message into the failure taxonomy."""
    msg = error_msg.lower()
    if "auth" in msg or "key" in msg:
        return FailureTaxonomy.API_AUTH
    if "quota" in msg or "limit" in msg:
        return FailureTaxonomy.API_QUOTA
    if "xml" in msg:
        return FailureTaxonomy.PARSER_INVALID_XML
    return FailureTaxonomy.CONTRACT_VIOLATION


# Retry same provider: rate limit or transient gateway/network
_RETRYABLE_PATTERNS = (
    r"429",
    r"rate[\s_-]?limit",
    r"502\s+bad\s+gateway",
    r"503\s+service\s+unavailable",
    r"504\s+gateway\s+timeout",
    r"\b502\b",
    r"\b503\b",
    r"\b504\b",
    r"too\s+many\s+requests",
    r"reconnecting",
    r"retry\s+after",
)

# Usage/subscription limits: fallback to different provider (do not retry same)
_USAGE_LIMIT_PATTERNS = (
    r"quota\s+exceeded",
    r"quota\s+limit",
    r"usage\s+limit",
    r"subscription\s+(exceeded|limit)",
    r"billing\s+(exceeded|limit)",
    r"insufficient\s+(quota|credits)",
    r"out\s+of\s+(quota|credits)",
    r"monthly\s+limit",
    r"daily\s+limit",
)


class TransientAgentError(Exception):
    """Raised when agent failed due to retryable condition (rate limit, 502, etc.)."""

    def __init__(self, result: RunResult) -> None:
        self.result = result
        msg = (result.stderr or "")[:300]
        super().__init__(msg)


class UsageLimitError(Exception):
    """Raised when provider hit usage/quota limit; caller should fallback to different provider."""

    def __init__(self, result: RunResult, agent: str = "") -> None:
        self.result = result
        self.agent = agent
        msg = (result.stderr or "")[:300]
        super().__init__(msg)


def classify_failure(result: RunResult) -> FailureKind:
    """Classify failure as rate_limit (retry), usage_limit (fallback), or unknown."""
    if result.exit_code == 0:
        return FailureKind.UNKNOWN
    text = (result.stderr or "").lower()
    # Check usage limit first (subscription/quota exhausted)
    if any(re.search(p, text, re.IGNORECASE) for p in _USAGE_LIMIT_PATTERNS):
        return FailureKind.USAGE_LIMIT
    # Then rate limit or transient
    if any(re.search(p, text, re.IGNORECASE) for p in _RETRYABLE_PATTERNS):
        return (
            FailureKind.RATE_LIMIT if "429" in text or "rate" in text or "too many" in text else FailureKind.TRANSIENT
        )
    return FailureKind.UNKNOWN


def is_retryable(result: RunResult) -> bool:
    """Return True if failure is rate_limit or transient (retry same provider)."""
    k = classify_failure(result)
    return k in (FailureKind.RATE_LIMIT, FailureKind.TRANSIENT)


def is_usage_limit(result: RunResult) -> bool:
    """Return True if failure indicates usage/quota limit (fallback to different provider)."""
    return classify_failure(result) == FailureKind.USAGE_LIMIT


def with_retry(
    max_attempts: int = 4,
    min_wait: float = 2.0,
    max_wait: float = 60.0,
) -> Callable[[Callable[..., T]], Callable[..., T]]:
    """Decorator that retries on TransientAgentError with exponential backoff."""

    def decorator(fn: Callable[..., T]) -> Callable[..., T]:
        return retry(
            stop=stop_after_attempt(max_attempts),
            wait=wait_random_exponential(multiplier=1, min=min_wait, max=max_wait),
            retry=retry_if_exception_type(TransientAgentError),
            reraise=True,
        )(fn)

    return decorator
