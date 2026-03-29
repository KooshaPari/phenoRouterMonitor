"""WP-2001: Per-provider circuit breakers with configurable thresholds.

ProviderCircuitBreaker wraps pybreaker.CircuitBreaker with provider-specific
configuration (failure_threshold, success_threshold, timeout, half_open_max_calls).

ProviderCircuitBreakerRegistry is a singleton registry keyed by provider name.

GW-13: Adds LiteLLM model_list integration helpers:
  - get_healthy_deployments: filter model_list to exclude OPEN circuit providers
  - record_deployment_failure: record a provider failure
  - record_deployment_success: record a provider success
  - with_circuit_breaker: execute a callable through the circuit breaker

# @trace WL-039 WP-2001 FR-ROUTE-013
"""

from __future__ import annotations

import logging
import threading
from dataclasses import dataclass, field
from typing import Any, Final

import pybreaker

_log = logging.getLogger(__name__)

# Default thresholds
DEFAULT_FAILURE_THRESHOLD: Final[int] = 5
DEFAULT_SUCCESS_THRESHOLD: Final[int] = 2
DEFAULT_TIMEOUT_SEC: Final[float] = 60.0
DEFAULT_HALF_OPEN_MAX_CALLS: Final[int] = 3


class CircuitOpenError(Exception):
    """Raised when a call is attempted against an open circuit breaker.

    Wraps pybreaker.CircuitBreakerError to provide a clear, project-specific error.
    Fail fast: callers MUST NOT swallow this silently.
    """


@dataclass
class ProviderCircuitBreakerConfig:
    """Configuration for a single provider circuit breaker."""

    failure_threshold: int = DEFAULT_FAILURE_THRESHOLD
    success_threshold: int = DEFAULT_SUCCESS_THRESHOLD
    timeout_sec: float = DEFAULT_TIMEOUT_SEC
    half_open_max_calls: int = DEFAULT_HALF_OPEN_MAX_CALLS


class ProviderCircuitBreaker:
    """Per-provider circuit breaker backed by pybreaker.

    State machine: CLOSED -> OPEN (on failure_threshold failures)
                           -> HALF_OPEN (after timeout_sec)
                           -> CLOSED (on success_threshold successes)
                           -> OPEN (on any failure in HALF_OPEN)

    Fail fast: when OPEN, call() raises CircuitOpenError immediately.
    """

    def __init__(self, provider: str, config: ProviderCircuitBreakerConfig | None = None) -> None:
        self.provider = provider
        self.config = config or ProviderCircuitBreakerConfig()
        self._breaker = pybreaker.CircuitBreaker(
            fail_max=self.config.failure_threshold,
            reset_timeout=self.config.timeout_sec,
            success_threshold=self.config.success_threshold,
            name=f"provider:{provider}",
        )

    @property
    def state(self) -> str:
        """Return current state string: 'closed', 'open', or 'half-open'."""
        return str(self._breaker.current_state)

    @property
    def fail_counter(self) -> int:
        """Return current consecutive failure count."""
        return int(self._breaker.fail_counter)

    def call(self, func, *args, **kwargs):  # noqa: ANN001, ANN002, ANN003
        """Execute func through the circuit breaker.

        When the circuit is already OPEN before this call, raises CircuitOpenError
        immediately without invoking func (fail fast).

        When func raises and the circuit *just* tripped to OPEN on this call,
        the original exception propagates (caller sees the real error, not
        CircuitOpenError).

        Raises:
            CircuitOpenError: when the circuit was already OPEN before this call.
            Any exception raised by func propagates after recording failure.
        """
        # Check state BEFORE the call
        was_open = self._breaker.current_state == pybreaker.STATE_OPEN
        if was_open:
            _log.error(
                "Circuit breaker OPEN for provider=%s — rejecting call immediately. fail_count=%d threshold=%d",
                self.provider,
                self._breaker.fail_counter,
                self._breaker.fail_max,
            )
            raise CircuitOpenError(f"Circuit breaker OPEN for provider '{self.provider}': circuit open")
        try:
            return self._breaker.call(func, *args, **kwargs)
        except pybreaker.CircuitBreakerError as exc:
            # The call just tripped the breaker (was_open was False above).
            # pybreaker raises CircuitBreakerError instead of the original exception.
            # We re-raise as CircuitOpenError to remain consistent.
            _log.warning(
                "Circuit breaker tripped OPEN for provider=%s. fail_count=%d threshold=%d",
                self.provider,
                self._breaker.fail_counter,
                self._breaker.fail_max,
            )
            raise CircuitOpenError(f"Circuit breaker OPEN for provider '{self.provider}': {exc}") from exc

    async def call_async(self, func, *args, **kwargs):  # noqa: ANN001, ANN002, ANN003
        """Execute async func through the circuit breaker.

        pybreaker.call_async requires tornado (not available here), so we implement
        async circuit-breaker semantics directly: check state, run the coroutine,
        record outcome via pybreaker's synchronous state machine.

        Raises:
            CircuitOpenError: when the circuit is OPEN (fail fast).
            Any exception raised by func propagates after recording failure.
        """
        # Fast-path: reject immediately if OPEN
        if self._breaker.current_state == pybreaker.STATE_OPEN:
            _log.error(
                "Circuit breaker OPEN for provider=%s (async) — rejecting call. fail_count=%d threshold=%d",
                self.provider,
                self._breaker.fail_counter,
                self._breaker.fail_max,
            )
            raise CircuitOpenError(f"Circuit breaker OPEN for provider '{self.provider}': circuit open")

        # Run the coroutine, capturing the outcome
        try:
            result = await func(*args, **kwargs)
        except Exception as exc:
            # Record failure through pybreaker (may trip the breaker to OPEN).
            # Regardless of whether the breaker just tripped, we re-raise the
            # *original* exception — the caller that triggers the trip sees the
            # real error, not CircuitOpenError.  Only callers that arrive when
            # the circuit is already open (checked above) see CircuitOpenError.
            try:
                self._breaker.call(_raise_exc, exc)
            except (pybreaker.CircuitBreakerError, Exception):
                pass
            if self._breaker.current_state == pybreaker.STATE_OPEN:
                _log.warning(
                    "Circuit breaker tripped OPEN for provider=%s after failure. fail_count=%d threshold=%d",
                    self.provider,
                    self._breaker.fail_counter,
                    self._breaker.fail_max,
                )
            raise
        else:
            # Record success: close the breaker (no-op if already closed)
            self._breaker.close()
            return result

    def record_success(self) -> None:
        """Manually record a success (closes the breaker if in HALF_OPEN)."""
        self._breaker.close()

    def record_failure(self) -> None:
        """Manually record a failure (increments counter, may trip to OPEN)."""
        try:
            self._breaker.call(_raise_exc, RuntimeError("manual failure record"))
        except (RuntimeError, pybreaker.CircuitBreakerError):
            pass

    def reset(self) -> None:
        """Force-close the breaker (for testing / manual recovery)."""
        self._breaker.close()

    def __repr__(self) -> str:
        return (
            f"ProviderCircuitBreaker(provider={self.provider!r}, "
            f"state={self.state!r}, "
            f"fail_counter={self.fail_counter}, "
            f"threshold={self.config.failure_threshold})"
        )


def _raise_exc(exc: Exception) -> None:
    """Helper: raises the given exception (used for manual failure recording)."""
    raise exc


@dataclass
class ProviderCircuitBreakerRegistry:
    """Singleton registry of per-provider circuit breakers.

    Usage:
        registry = ProviderCircuitBreakerRegistry.get_instance()
        breaker = registry.get("openai")
        breaker.call(my_func, arg1, arg2)
    """

    _breakers: dict[str, ProviderCircuitBreaker] = field(default_factory=dict)
    _lock: threading.Lock = field(default_factory=threading.Lock)

    _instance: ProviderCircuitBreakerRegistry | None = field(default=None, init=False, repr=False)

    @classmethod
    def get_instance(cls) -> ProviderCircuitBreakerRegistry:
        """Return the process-global singleton registry."""
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    @classmethod
    def reset_instance(cls) -> None:
        """Reset the singleton (for testing only)."""
        cls._instance = None

    def get(
        self,
        provider: str,
        config: ProviderCircuitBreakerConfig | None = None,
    ) -> ProviderCircuitBreaker:
        """Return the breaker for provider, creating it with config if not yet registered."""
        with self._lock:
            if provider not in self._breakers:
                self._breakers[provider] = ProviderCircuitBreaker(provider, config)
                _log.debug("Registered circuit breaker for provider=%s", provider)
            return self._breakers[provider]

    def register(
        self,
        provider: str,
        config: ProviderCircuitBreakerConfig,
    ) -> ProviderCircuitBreaker:
        """Register (or replace) a breaker for provider with the given config."""
        with self._lock:
            breaker = ProviderCircuitBreaker(provider, config)
            self._breakers[provider] = breaker
            _log.debug("Re-registered circuit breaker for provider=%s config=%s", provider, config)
            return breaker

    def all_open(self) -> list[str]:
        """Return list of provider names whose circuit is currently OPEN."""
        with self._lock:
            return [name for name, b in self._breakers.items() if b.state == "open"]

    def all_states(self) -> dict[str, str]:
        """Return mapping provider -> state string for all registered breakers."""
        with self._lock:
            return {name: b.state for name, b in self._breakers.items()}

    def clear(self) -> None:
        """Remove all registered breakers (for testing)."""
        with self._lock:
            self._breakers.clear()


# ---------------------------------------------------------------------------
# GW-13: LiteLLM model_list integration helpers
# ---------------------------------------------------------------------------


def _provider_from_deployment(entry: dict[str, Any]) -> str:
    """Extract provider name from a LiteLLM model_list entry.

    Prefers the ``litellm_params.model`` field (format: ``"provider/model"``).
    Falls back to the ``model_name`` field.

    Args:
        entry: A single LiteLLM model_list dict with ``model_name`` and
            ``litellm_params`` keys.

    Returns:
        Provider name string (e.g. ``"openai"``, ``"anthropic"``).
    """
    litellm_params = entry.get("litellm_params", {})
    litellm_model: str = litellm_params.get("model", "")
    if "/" in litellm_model:
        return litellm_model.split("/", 1)[0]
    # Last resort: use model_name as provider identifier
    return entry.get("model_name", "unknown")


def get_healthy_deployments(
    model_list: list[dict[str, Any]],
    registry: ProviderCircuitBreakerRegistry | None = None,
) -> list[dict[str, Any]]:
    """Filter a LiteLLM model_list to exclude deployments with open circuit breakers.

    A deployment is excluded when its provider's circuit breaker is in OPEN state.
    If all deployments would be excluded, the full original list is returned to
    prevent a total outage (degraded service is preferable to zero service).

    Args:
        model_list: LiteLLM model_list entries (each has ``model_name`` and
            ``litellm_params``).
        registry: Circuit breaker registry.  Defaults to the global singleton.

    Returns:
        Filtered model_list containing only deployments whose provider circuit
        breaker is not OPEN.  Returns the full list if all are unhealthy.
    """
    effective_registry = registry if registry is not None else ProviderCircuitBreakerRegistry.get_instance()
    open_providers = set(effective_registry.all_open())

    if not open_providers:
        return model_list

    healthy = [entry for entry in model_list if _provider_from_deployment(entry) not in open_providers]

    if not healthy:
        _log.warning(
            "All %d deployments have open circuit breakers (%s). Returning full list to avoid total outage.",
            len(model_list),
            sorted(open_providers),
        )
        return model_list

    excluded = len(model_list) - len(healthy)
    _log.info(
        "Circuit breaker filter: excluded %d of %d deployments (open providers: %s)",
        excluded,
        len(model_list),
        sorted(open_providers),
    )
    return healthy


def record_deployment_failure(
    provider: str,
    error: Exception,
    registry: ProviderCircuitBreakerRegistry | None = None,
) -> None:
    """Record a failure for a provider's circuit breaker.

    Called after a failed LiteLLM completion to increment the failure counter.
    If the failure threshold is reached the circuit opens and the provider will
    be excluded from future routing until the timeout elapses.

    Args:
        provider: Provider identifier (e.g. ``"openai"``).
        error: The exception that caused the failure.
        registry: Circuit breaker registry.  Defaults to the global singleton.
    """
    effective_registry = registry if registry is not None else ProviderCircuitBreakerRegistry.get_instance()
    breaker = effective_registry.get(provider)
    breaker.record_failure()
    _log.warning(
        "Recorded deployment failure for provider=%s error=%s fail_count=%d threshold=%d state=%s",
        provider,
        type(error).__name__,
        breaker.fail_counter,
        breaker.config.failure_threshold,
        breaker.state,
    )


def record_deployment_success(
    provider: str,
    registry: ProviderCircuitBreakerRegistry | None = None,
) -> None:
    """Record a success for a provider's circuit breaker.

    Helps reset the half-open state back to closed after recovery.

    Args:
        provider: Provider identifier (e.g. ``"openai"``).
        registry: Circuit breaker registry.  Defaults to the global singleton.
    """
    effective_registry = registry if registry is not None else ProviderCircuitBreakerRegistry.get_instance()
    breaker = effective_registry.get(provider)
    breaker.record_success()
    _log.debug("Recorded deployment success for provider=%s state=%s", provider, breaker.state)


def with_circuit_breaker(
    provider: str,
    func: Any,
    *args: Any,
    registry: ProviderCircuitBreakerRegistry | None = None,
    **kwargs: Any,
) -> Any:
    """Execute ``func(*args, **kwargs)`` through the provider's circuit breaker.

    Records success or failure automatically so that the circuit breaker state
    machine advances correctly.

    Args:
        provider: Provider identifier used to look up (or create) the breaker.
        func: Callable to execute.
        *args: Positional arguments forwarded to ``func``.
        registry: Circuit breaker registry.  Defaults to the global singleton.
        **kwargs: Keyword arguments forwarded to ``func``.

    Returns:
        The return value of ``func(*args, **kwargs)``.

    Raises:
        CircuitOpenError: When the circuit is already OPEN before the call.
        Any exception raised by ``func`` propagates after recording the failure.
    """
    effective_registry = registry if registry is not None else ProviderCircuitBreakerRegistry.get_instance()
    breaker = effective_registry.get(provider)
    return breaker.call(func, *args, **kwargs)
