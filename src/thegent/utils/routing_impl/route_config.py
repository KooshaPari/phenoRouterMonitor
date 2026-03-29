"""GW-10: Recursive RouteConfig schema.

Implements Portkey-inspired recursive target trees for the thegent AI gateway.
Supports fallback, loadbalance, and conditional routing strategies with
per-target cache, retry, and circuit-breaker configuration.

The schema is OpenRouter-compatible via the ``models`` shorthand and supports
arbitrarily nested strategy nodes through recursive ``RouteTarget`` trees.

GW-63: Dynamic routing node types — Conditional, Percentage, Budget-Limit.

Extends RouteConfig with declarative dynamic routing nodes:
  - PercentageSplit: routes traffic by percentage split across targets
  - BudgetLimitRoute: routes to fallback when budget is exhausted

# @trace FR-AROUTE-063
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from typing import Any, Literal


# ---------------------------------------------------------------------------
# Sub-configs
# ---------------------------------------------------------------------------


@dataclass
class CacheConfig:
    """Cache behaviour for a route target or top-level route.

    Attributes:
        mode: Caching strategy — ``"exact"`` for byte-identical key match,
            ``"semantic"`` for embedding-based similarity, ``"none"`` to
            disable caching entirely.
        max_age: Time-to-live for cached responses in seconds.
        namespace: Optional cache namespace/partition key.  When *None* the
            provider-level default namespace is used.
    """

    mode: Literal["exact", "semantic", "none"] = "none"
    max_age: int = 300
    namespace: str | None = None


@dataclass
class RetryConfig:
    """Retry policy applied before failing over to the next target.

    Attributes:
        attempts: Maximum number of retry attempts (not counting the initial
            request).
        on_status_codes: HTTP status codes that trigger a retry.
        backoff_factor: Multiplicative factor applied to the wait time between
            successive retries (exponential backoff base).
    """

    attempts: int = 2
    on_status_codes: list[int] = field(default_factory=lambda: [429, 500, 502, 503])
    backoff_factor: float = 1.5


@dataclass
class CircuitBreakerConfig:
    """Circuit-breaker thresholds for a route target.

    Attributes:
        failure_threshold: Number of consecutive failures before the circuit
            opens and the target is marked unavailable.
        success_threshold: Number of consecutive successes required to close
            an open circuit (half-open → closed transition).
        timeout_sec: Duration in seconds to wait before attempting to
            transition an open circuit to half-open.
    """

    failure_threshold: int = 5
    success_threshold: int = 2
    timeout_sec: float = 60.0


# ---------------------------------------------------------------------------
# RouteTarget (recursive)
# ---------------------------------------------------------------------------


@dataclass
class RouteTarget:
    """A single node in a recursive routing tree.

    A ``RouteTarget`` is either a *leaf* node that identifies a concrete
    provider/model pair, or an *inner* node that applies a nested strategy
    over its own ``targets`` list.

    Leaf node example::

        RouteTarget(provider="openai", model="gpt-4o")

    Inner (nested strategy) node example::

        RouteTarget(
            strategy="loadbalance",
            targets=[
                RouteTarget(provider="openai", model="gpt-4o", weight=0.7),
                RouteTarget(provider="anthropic", model="claude-sonnet-4-6", weight=0.3),
            ],
        )

    Attributes:
        provider: Leaf provider identifier (e.g. ``"openai"``).  *None* when
            this is an inner strategy node.
        model: Leaf model identifier (e.g. ``"gpt-4o"``).  *None* when this
            is an inner strategy node.
        weight: Relative weight used by ``"loadbalance"`` strategies.
            Weights need not sum to 1 — the router normalises them.
        strategy: When set, this node is an inner (non-leaf) node and
            ``targets`` must be populated.  One of ``"fallback"``,
            ``"loadbalance"``, or ``"conditional"``.
        targets: Child ``RouteTarget`` nodes when ``strategy`` is set.
        cache: Per-target cache configuration.  Overrides any top-level
            ``RouteConfig.cache`` for requests routed here.
        retry: Per-target retry configuration.
        circuit_breaker: Per-target circuit-breaker configuration.
        on_status_codes: HTTP status codes that cause the router to move to
            the next sibling target (fallback trigger).
    """

    provider: str | None = None
    model: str | None = None
    weight: float = 1.0
    strategy: str | None = None
    targets: list[RouteTarget] = field(default_factory=list)
    cache: CacheConfig | None = None
    retry: RetryConfig | None = None
    circuit_breaker: CircuitBreakerConfig | None = None
    on_status_codes: list[int] = field(default_factory=lambda: [429, 500, 502, 503, 529])

    @property
    def is_leaf(self) -> bool:
        """Return ``True`` when this node is a leaf (concrete provider target).

        A node is a leaf when it has no nested ``strategy`` and has a
        non-*None* ``provider``.
        """
        return self.strategy is None and self.provider is not None


# ---------------------------------------------------------------------------
# RouteConfig (top-level)
# ---------------------------------------------------------------------------


@dataclass
class RouteConfig:
    """Top-level routing configuration for an AI gateway request.

    ``RouteConfig`` describes how the gateway should select and fail over
    between providers/models for a single request.  It supports two
    authoring styles:

    1. **Full target tree** (``targets``): Expressive, supports nested
       strategies, per-target overrides, and circuit-breaker configuration.

    2. **Simple model list** (``models``): OpenRouter-compatible shorthand
       that specifies an ordered fallback chain as plain model-name strings.
       The helper :func:`models_to_targets` converts this into a
       ``targets`` list automatically.

    Attributes:
        strategy: Top-level routing strategy applied across ``targets``.
        targets: Ordered list of ``RouteTarget`` nodes.
        models: Ordered fallback chain as ``"provider/model"`` or bare
            model-name strings.  Used *instead of* ``targets`` when a simple
            fallback is sufficient.
        cache: Default cache config applied to all targets that do not
            specify their own.
        retry: Default retry config.
        circuit_breaker: Default circuit-breaker config.
    """

    strategy: Literal["fallback", "loadbalance", "conditional"] = "fallback"
    targets: list[RouteTarget] = field(default_factory=list)
    models: list[str] = field(default_factory=list)
    cache: CacheConfig = field(default_factory=CacheConfig)
    retry: RetryConfig = field(default_factory=RetryConfig)
    circuit_breaker: CircuitBreakerConfig = field(default_factory=CircuitBreakerConfig)


# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------


def models_to_targets(models: list[str]) -> list[RouteTarget]:
    """Convert a simple model list into an ordered fallback ``RouteTarget`` list.

    Each string in *models* is interpreted as either ``"provider/model"``
    (split on the first ``"/"``), or a bare model name where ``provider`` is
    left as *None*.

    Example::

        >>> models_to_targets(["openai/gpt-4o", "anthropic/claude-sonnet-4-6"])
        [RouteTarget(provider='openai', model='gpt-4o', ...),
         RouteTarget(provider='anthropic', model='claude-sonnet-4-6', ...)]

    Args:
        models: Ordered list of model identifiers.

    Returns:
        List of leaf ``RouteTarget`` objects in the same order.
    """
    targets: list[RouteTarget] = []
    for entry in models:
        if "/" in entry:
            provider, model = entry.split("/", 1)
        else:
            provider = None
            model = entry
        targets.append(RouteTarget(provider=provider, model=model))
    return targets


def from_request_body(body: dict[str, Any]) -> RouteConfig | None:
    """Extract a ``RouteConfig`` from an incoming request body dict.

    The function recognises two sources inside *body*:

    * ``body["route_config"]``: A dict that is deserialised into a full
      :class:`RouteConfig`.  Nested dicts are converted to their dataclass
      equivalents recursively.
    * ``body["models"]``: A list of model-name strings used as a simple
      ordered fallback chain (OpenRouter-compatible shorthand).

    If neither key is present the function returns *None*, signalling that
    the gateway should fall back to its default routing logic.

    Args:
        body: Parsed JSON request body.

    Returns:
        A :class:`RouteConfig` when routing configuration is present, or
        *None* otherwise.
    """
    if "route_config" in body:
        return _deserialize_route_config(body["route_config"])

    if "models" in body and isinstance(body["models"], list):
        models: list[str] = [str(m) for m in body["models"]]
        return RouteConfig(
            strategy="fallback",
            targets=models_to_targets(models),
            models=models,
        )

    return None


# ---------------------------------------------------------------------------
# Internal deserialization helpers
# ---------------------------------------------------------------------------


def _deserialize_cache_config(data: dict[str, Any]) -> CacheConfig:
    return CacheConfig(
        mode=data.get("mode", "none"),
        max_age=int(data.get("max_age", 300)),
        namespace=data.get("namespace"),
    )


def _deserialize_retry_config(data: dict[str, Any]) -> RetryConfig:
    return RetryConfig(
        attempts=int(data.get("attempts", 2)),
        on_status_codes=list(data.get("on_status_codes", [429, 500, 502, 503])),
        backoff_factor=float(data.get("backoff_factor", 1.5)),
    )


def _deserialize_circuit_breaker_config(
    data: dict[str, Any],
) -> CircuitBreakerConfig:
    return CircuitBreakerConfig(
        failure_threshold=int(data.get("failure_threshold", 5)),
        success_threshold=int(data.get("success_threshold", 2)),
        timeout_sec=float(data.get("timeout_sec", 60.0)),
    )


def _deserialize_route_target(data: dict[str, Any]) -> RouteTarget:
    """Recursively deserialize a ``RouteTarget`` from a plain dict."""
    cache = _deserialize_cache_config(data["cache"]) if "cache" in data else None
    retry = _deserialize_retry_config(data["retry"]) if "retry" in data else None
    cb = _deserialize_circuit_breaker_config(data["circuit_breaker"]) if "circuit_breaker" in data else None
    nested_targets = [_deserialize_route_target(t) for t in data.get("targets", [])]
    return RouteTarget(
        provider=data.get("provider"),
        model=data.get("model"),
        weight=float(data.get("weight", 1.0)),
        strategy=data.get("strategy"),
        targets=nested_targets,
        cache=cache,
        retry=retry,
        circuit_breaker=cb,
        on_status_codes=list(data.get("on_status_codes", [429, 500, 502, 503, 529])),
    )


def _deserialize_route_config(data: dict[str, Any]) -> RouteConfig:
    """Deserialize a ``RouteConfig`` from a plain dict."""
    targets = [_deserialize_route_target(t) for t in data.get("targets", [])]
    models: list[str] = [str(m) for m in data.get("models", [])]

    cache = _deserialize_cache_config(data["cache"]) if "cache" in data else CacheConfig()
    retry = _deserialize_retry_config(data["retry"]) if "retry" in data else RetryConfig()
    cb = (
        _deserialize_circuit_breaker_config(data["circuit_breaker"])
        if "circuit_breaker" in data
        else CircuitBreakerConfig()
    )
    return RouteConfig(
        strategy=data.get("strategy", "fallback"),
        targets=targets,
        models=models,
        cache=cache,
        retry=retry,
        circuit_breaker=cb,
    )


# ---------------------------------------------------------------------------
# GW-63: Dynamic routing node types
# ---------------------------------------------------------------------------


@dataclass
class PercentageSplit:
    """Route traffic by percentage split across targets.

    ``weights`` must sum to 100.  When they do not, they are normalised
    before selection so that ``select`` always returns a valid target.

    Attributes:
        targets: Ordered list of model/provider names.
        weights: Percentage weights, same length as ``targets``.
    """

    targets: list[str]
    weights: list[int]

    def select(self, rand_value: float | None = None) -> str:
        """Select a target based on weighted random selection.

        Args:
            rand_value: Value in [0.0, 1.0] used as the random draw.
                Defaults to ``random.random()``.

        Returns:
            Selected target name from ``self.targets``.

        Raises:
            ValueError: When ``targets`` and ``weights`` have different lengths
                or ``targets`` is empty.
        """
        if len(self.targets) != len(self.weights):
            raise ValueError(
                f"targets and weights must have the same length "
                f"(got {len(self.targets)} targets, {len(self.weights)} weights)"
            )
        if not self.targets:
            raise ValueError("targets must not be empty")

        total = sum(self.weights)
        if total <= 0:
            raise ValueError("weights must sum to a positive value")

        draw = rand_value if rand_value is not None else random.random()

        cumulative = 0.0
        for target, weight in zip(self.targets, self.weights):
            cumulative += weight / total
            if draw < cumulative:
                return target
        # Floating-point edge case: return last target
        return self.targets[-1]


@dataclass
class BudgetLimitRoute:
    """Route to ``fallback_target`` when ``budget_usd`` is exhausted.

    Designed to integrate with ``BudgetHierarchy`` from routing/budget.py.
    The ``entity_id`` identifies the entity whose spend is checked.

    Attributes:
        primary_target: Target used when current spend is below budget.
        fallback_target: Target used when budget is exhausted.
        budget_usd: Spend limit in USD.
        entity_id: Entity ID to check spend for in BudgetHierarchy.
    """

    primary_target: str
    fallback_target: str
    budget_usd: float
    entity_id: str

    def select(self, current_spend_usd: float) -> str:
        """Return ``primary_target`` if within budget, else ``fallback_target``.

        The budget is considered exhausted when ``current_spend_usd >= budget_usd``
        (i.e. exactly at the limit triggers fallback).

        Args:
            current_spend_usd: Current accumulated spend for ``entity_id``.

        Returns:
            ``primary_target`` or ``fallback_target``.
        """
        if current_spend_usd >= self.budget_usd:
            return self.fallback_target
        return self.primary_target
