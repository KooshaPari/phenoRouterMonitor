"""LiteLLM Router wrapper with full feature support.

Provides comprehensive LiteLLM integration including:
- Multi-provider routing (cheapest, fastest, latency-based, round_robin)
- Response caching (in-memory or Redis)
- Fallback chains with cooldown times
- Context window validation
- Cost tracking and budget alerts
- Webhook alerting for latency/errors/budget
- Streaming support
- Donut Architecture integration
"""

from __future__ import annotations

import logging
import os
import time
from dataclasses import dataclass
from threading import Lock
from typing import TYPE_CHECKING, Any

from cachetools import TTLCache
from litellm.router import Router

if TYPE_CHECKING:
    from collections.abc import Iterator

from thegent.models.catalog import Route, _get_catalog
from thegent.utils.routing_impl.circuit_breaker import (
    ProviderCircuitBreakerRegistry,
    get_healthy_deployments,
)
from thegent.utils.routing_impl.harness_model_mapping import CANONICAL_TO_OPENROUTER
from thegent.utils.routing_impl.model_metadata import (
    get_all_models_with_metadata,
    get_model_metadata,
    has_model_metadata,
)
from thegent.utils.routing_impl.provider_types import (
    NATIVE_CLI_PROVIDERS,
    ExecutionPath,
    get_execution_path,
)

logger = logging.getLogger(__name__)

# OpenRouter attribution defaults — overridden via environment variables
_OPENROUTER_HTTP_REFERER_DEFAULT = "https://thegent.dev"
_OPENROUTER_SITE_NAME_DEFAULT = "thegent"

# Model context windows for validation (in tokens)
# Source: provider documentation as of 2026-02
MODEL_CONTEXT_WINDOWS: dict[str, int] = {
    # OpenAI
    "gpt-4": 8192,
    "gpt-4-turbo": 128000,
    "gpt-4o": 128000,
    "gpt-4o-mini": 128000,
    "gpt-5-mini": 128000,
    "gpt-5.3-codex-spark": 128000,
    "gpt-5.3-codex-high": 128000,
    # Anthropic
    "claude-opus-4.6": 200000,
    "claude-sonnet-4.5": 200000,
    "claude-haiku-4.5": 200000,
    # Google
    "gemini-3-flash": 1000000,
    "gemini-3.1-pro": 1000000,
    # DeepSeek
    "deepseek-v3.2": 64000,
    # Zhipu
    "glm-5": 128000,
    "GLM-5": 128000,
    "z-ai/glm-5": 128000,
    # MiniMax
    "minimax-m2.5": 128000,
    "MiniMax-M2.5": 128000,
    # Kimi
    "kimi-k2.5": 200000,
    # Kilo
    "kilo-default": 128000,
    # Roo
    "roo-default": 128000,
    # Meta
    "llama-nemotron-ultra": 128000,
    # Qwen
    "qwen3-coder": 32000,
    # Default fallback
    "default": 8192,
}


@dataclass
class RoutingResult:
    """Result from a routing operation."""

    success: bool
    model: str
    provider: str
    response: Any | None = None
    latency_ms: float = 0.0
    tokens_used: int = 0
    cost_usd: float = 0.0
    error: str | None = None
    is_fallback: bool = False
    is_cached: bool = False


@dataclass
class RouterConfig:
    """Configuration for LiteLLM Router."""

    routing_policy: str = "cheapest"
    timeout: int = 300
    num_retries: int = 2
    retry_after: int = 5
    cooldown_time: int = 60
    enable_cache: bool = True
    cache_type: str = "in-memory"
    redis_url: str | None = None
    enable_streaming: bool = True
    enable_cost_tracking: bool = True
    cost_budget: float | None = None
    alert_webhook: str | None = None
    latency_threshold_ms: float = 500.0
    context_window_validation: bool = True
    fallback_enabled: bool = True


def _get_openrouter_attribution_headers() -> dict[str, str]:
    """Build OpenRouter attribution headers from environment or defaults."""
    return {
        "HTTP-Referer": os.environ.get("OPENROUTER_HTTP_REFERER", _OPENROUTER_HTTP_REFERER_DEFAULT),
        "X-Title": os.environ.get("OPENROUTER_SITE_NAME", _OPENROUTER_SITE_NAME_DEFAULT),
    }


def _route_to_litellm_config(route: Route) -> dict[str, Any]:
    """Convert a catalog Route to LiteLLM model_list config.

    Args:
        route: Catalog route with provider, model_alias, etc.

    Returns:
        LiteLLM model_list entry dict
    """
    model_name = route.model_alias
    provider = route.provider

    # Map thegent provider to LiteLLM provider
    # For models going through CLIProxyAPIPlus, we use 'openai' provider
    # to ensure LiteLLM uses the OpenAI-compatible handler.
    if get_execution_path(provider) == ExecutionPath.CLIPROXY_API or route.backend_type == "proxy":
        litellm_provider = "openai"
    else:
        provider_mapping = {
            "gemini": "gemini",
            "claude": "anthropic",
            "minimax": "minimax",
            "glm": "zhipu",
            "openrouter": "openrouter",
            "ollama": "ollama",
        }
        litellm_provider = provider_mapping.get(provider, provider)

    # Determine litellm model string
    # LiteLLM format: "provider/model-name"
    litellm_model = f"{litellm_provider}/{model_name}"

    # For API key providers, get API key from environment
    api_key_env = _get_api_key_env(provider)
    api_key = os.environ.get(api_key_env, "dummy-key")

    config = {
        "model_name": model_name,
        "litellm_params": {
            "model": litellm_model,
            "api_key": api_key,
        },
    }

    # Route through CLIProxy for universal parity: Codex harness, LiteLLM, and direct
    # - CLIPROXY_API (login-auth): antigravity, cursor, gemini, copilot, kiro
    # - proxy backend (catalog): minimax, glm, kilo, roo — ensures Codex + LiteLLM use same path
    if get_execution_path(provider) == ExecutionPath.CLIPROXY_API or route.backend_type == "proxy":
        config["litellm_params"]["api_base"] = "http://localhost:8317/v1"

    # OpenRouter routes directly to its API endpoint with attribution headers
    if provider == "openrouter":
        config["litellm_params"]["api_base"] = "https://openrouter.ai/api/v1"
        config["litellm_params"]["extra_headers"] = _get_openrouter_attribution_headers()
    elif provider == "ollama":
        config["litellm_params"]["api_base"] = "http://127.0.0.1:11434/v1"

    return config


def _get_api_key_env(provider: str) -> str:
    """Get environment variable name for provider API key."""
    mapping = {
        "minimax": "MINIMAX_API_KEY",
        "nim": "NVIDIA_API_KEY",
        "glm": "ZHIPU_API_KEY",
        "kilo": "KILO_API_KEY",
        "openrouter": "OPENROUTER_API_KEY",
        "ollama": "OLLAMA_API_KEY",
    }
    return mapping.get(provider, f"{provider.upper()}_API_KEY")


def build_openrouter_model_entry(
    model_alias: str,
    openrouter_model_id: str,
    api_key: str | None = None,
) -> dict[str, Any]:
    """Build a LiteLLM model_list entry for an OpenRouter model.

    Args:
        model_alias: The alias to use as model_name (e.g., "or/claude-opus-4.6")
        openrouter_model_id: The OpenRouter model ID in provider/model format
            (e.g., "anthropic/claude-opus-4-6")
        api_key: Optional API key; if None, reads from OPENROUTER_API_KEY env var

    Returns:
        LiteLLM model_list entry dict for the OpenRouter model
    """
    resolved_api_key = api_key if api_key is not None else os.environ.get("OPENROUTER_API_KEY", "dummy-key")
    attribution_headers = _get_openrouter_attribution_headers()

    return {
        "model_name": model_alias,
        "litellm_params": {
            "model": f"openrouter/{openrouter_model_id}",
            "api_key": resolved_api_key,
            "api_base": "https://openrouter.ai/api/v1",
            "extra_headers": attribution_headers,
        },
    }


def build_litellm_model_list() -> list[dict[str, Any]]:
    """Build LiteLLM model_list from catalog routes.

    Excludes NATIVE_CLI_PROVIDERS (codex, claude).
    Routes API_KEY_PROVIDERS directly.
    Routes LOGIN_AUTH_PROVIDERS via CLIProxyAPIPlus.

    Also appends OpenRouter model entries from CANONICAL_TO_OPENROUTER under
    "or/<alias>" aliases to avoid conflicts with catalog model names.

    Returns:
        List of LiteLLM model_list entries
    """
    model_list: list[dict[str, Any]] = []
    seen_models: set[str] = set()

    catalog = _get_catalog()
    for routes in catalog.values():
        for route in routes:
            # Skip native CLI providers
            if route.provider in NATIVE_CLI_PROVIDERS:
                continue

            # Avoid duplicates
            key = f"{route.provider}/{route.model_alias}"
            if key in seen_models:
                continue
            seen_models.add(key)

            config = _route_to_litellm_config(route)
            model_list.append(config)

    # Add OpenRouter model entries from CANONICAL_TO_OPENROUTER under "or/" prefix
    seen_or_aliases: set[str] = set()
    for canonical_alias, openrouter_model_id in CANONICAL_TO_OPENROUTER.items():
        or_alias = f"or/{canonical_alias}"
        if or_alias in seen_or_aliases:
            continue
        seen_or_aliases.add(or_alias)
        model_list.append(build_openrouter_model_entry(or_alias, openrouter_model_id))

    return model_list


def build_fallback_chains() -> dict[str, list[str]]:
    """Build fallback chains for models.

    Returns:
        Dict mapping primary model to list of fallback models
    """
    # Fallback chains mapping primary model -> fallback models
    chains_map = {
        # High-end models fallback to capable cheaper alternatives
        "claude-opus-4.6": ["claude-sonnet-4.5", "deepseek-v3.2", "glm-5"],
        "claude-sonnet-4.5": ["deepseek-v3.2", "glm-5", "qwen3-coder"],
        "gpt-4o": ["gpt-4o-mini", "deepseek-v3.2", "glm-5"],
        # Mid-tier models
        "gemini-3-flash": ["deepseek-v3.2", "qwen3-coder"],
        "deepseek-v3.2": ["glm-5", "qwen3-coder", "llama-nemotron-ultra"],
        "glm-5": ["deepseek-v3.2", "qwen3-coder"],
        "kimi-k2.5": ["deepseek-v3.2", "glm-5"],
        # Budget models
        "qwen3-coder": ["llama-nemotron-ultra", "deepseek-v3.2"],
        "llama-nemotron-ultra": ["qwen3-coder", "deepseek-v3.2"],
    }
    return chains_map


def get_context_window(model: str) -> int:
    """Get context window size for a model.

    Args:
        model: Model name (may be alias)

    Returns:
        Context window in tokens
    """
    # Try to get from metadata registry first
    try:
        metadata = get_model_metadata(model)
        if metadata and "context_window" in metadata:
            return metadata["context_window"]
    except Exception:
        pass

    # Fallback to static dictionary with normalization
    normalized = model.lower().replace("-", "").replace(".", "")

    for key, value in MODEL_CONTEXT_WINDOWS.items():
        if key.lower().replace("-", "").replace(".", "") == normalized:
            return value

    # Default fallback
    return MODEL_CONTEXT_WINDOWS["default"]


def validate_context_window(model: str, prompt_tokens: int) -> bool:
    """Validate that prompt fits within model's context window.

    Args:
        model: Model name
        prompt_tokens: Estimated prompt token count

    Returns:
        True if prompt fits, False otherwise
    """
    max_tokens = get_context_window(model)
    # Leave 25% buffer for response
    effective_limit = int(max_tokens * 0.75)
    return prompt_tokens <= effective_limit


def get_router_config() -> RouterConfig:
    """Get router configuration from settings.

    Returns:
        RouterConfig with values from ThegentSettings
    """
    try:
        from thegent.config import ThegentSettings

        settings = ThegentSettings()
        return RouterConfig(
            routing_policy=settings.litellm_routing_policy,
            timeout=settings.litellm_timeout,
            num_retries=settings.litellm_num_retries,
            retry_after=settings.litellm_retry_after,
            cooldown_time=settings.litellm_cooldown_time,
            enable_cache=settings.litellm_enable_cache,
            cache_type=settings.litellm_cache_type,
            redis_url=settings.litellm_redis_url,
            enable_streaming=settings.litellm_enable_streaming,
            enable_cost_tracking=settings.litellm_enable_cost_tracking,
            cost_budget=settings.litellm_cost_budget,
            alert_webhook=settings.litellm_alert_webhook,
            latency_threshold_ms=settings.litellm_latency_threshold_ms,
            context_window_validation=settings.litellm_context_window_validation,
            fallback_enabled=settings.litellm_fallback_enabled,
        )
    except Exception:
        # Fallback to defaults
        return RouterConfig()


# TTLCache for get_litellm_router: keyed by policy, reuse router for 5 minutes.
# Keep multiple policy variants cached concurrently to avoid rebuild churn when
# callers switch between routing strategies.
_router_cache: TTLCache = TTLCache(maxsize=8, ttl=300)
_router_lock = Lock()

# TTLCache for build_dynamic_fallback_router model list caching
_model_list_cache: TTLCache = TTLCache(maxsize=1, ttl=300)

# Track previous circuit breaker states for cache invalidation
_previous_cb_states: dict[str, str] = {}


def invalidate_router_cache() -> None:
    """Invalidate the cached LiteLLM Router instance.

    Call when external state (e.g. circuit breaker) changes and the
    cached router should be rebuilt on the next ``get_litellm_router()`` call.
    """
    with _router_lock:
        _router_cache.clear()
        _model_list_cache.clear()
    logger.debug("Router cache manually invalidated")


def _invalidate_router_cache_on_circuit_breaker_change() -> None:
    """Check circuit breaker states and invalidate cache if they changed.

    When any provider's circuit breaker state changes (e.g., opens or closes),
    the cached router becomes stale because it was built with a model list
    filtered based on the previous circuit breaker states.
    """
    global _previous_cb_states

    try:
        registry = ProviderCircuitBreakerRegistry.get_instance()
        current_states = registry.all_states()

        # Check if any state changed
        if current_states != _previous_cb_states:
            # State changed - invalidate both caches
            _router_cache.clear()
            _model_list_cache.clear()
            _previous_cb_states = current_states.copy()
            logger.debug("Circuit breaker state changed, invalidated router and model list caches")
    except Exception:
        # On any error, don't block - just skip invalidation
        pass


def _build_litellm_router(policy: str) -> Router:
    """Build a fresh LiteLLM Router instance.

    Args:
        policy: Routing policy (cost-based-routing, fastest, round_robin, latency-based-routing)

    Returns:
        Newly constructed LiteLLM Router
    """
    config = get_router_config()
    model_list = get_healthy_deployments(build_litellm_model_list())

    # Map thegent policy names to LiteLLM policy names
    policy_mapping = {
        "cheapest": "cost-based-routing",
        "pareto": "cost-based-routing",
        "fastest": "latency-based-routing",
        "round_robin": "simple-shuffle",
    }
    effective_policy = policy_mapping.get(policy, policy)

    # Build router kwargs
    router_kwargs: dict[str, Any] = {
        "model_list": model_list,
        "routing_strategy": effective_policy,
        "num_retries": config.num_retries,
        "timeout": config.timeout,
        "retry_after": config.retry_after,
        "cooldown_time": config.cooldown_time,
    }

    # Add caching configuration
    # LiteLLM uses cache_responses and redis_url parameters
    if config.enable_cache:
        router_kwargs["cache_responses"] = True
        if config.cache_type == "redis" and config.redis_url:
            router_kwargs["redis_url"] = config.redis_url

    # Add fallback configuration
    if config.fallback_enabled:
        chains = build_fallback_chains()
        # Convert to LiteLLM format: list of dicts
        router_kwargs["fallbacks"] = [{model: fallbacks} for model, fallbacks in chains.items()]

    return Router(**router_kwargs)


def get_litellm_router(policy: str = "cost-based-routing") -> Router:
    """Get configured LiteLLM Router instance.

    Returns a cached instance; construction is throttled to once per 5 minutes
    per policy value via TTLCache so the expensive build path is not hit on
    every call.

    Args:
        policy: Routing policy (cost-based-routing, fastest, round_robin, latency-based-routing)

    Returns:
        Configured LiteLLM Router (possibly shared from cache)
    """
    # Check and invalidate cache if circuit breaker state changed
    _invalidate_router_cache_on_circuit_breaker_change()

    with _router_lock:
        if policy not in _router_cache:
            _router_cache[policy] = _build_litellm_router(policy)
        return _router_cache[policy]


def build_dynamic_fallback_router(models: list[str], base_policy: str = "cost-based-routing") -> Router:
    """Build a LiteLLM Router with a dynamic model fallback chain.

    Results are cached for 5 minutes (same TTL as main router) to avoid
    rebuilding the model list on every call. Cache is invalidated when
    circuit breaker state changes.

    Args:
        models: Ordered list of model names to try (first = primary, rest = fallbacks)
        base_policy: LiteLLM routing strategy

    Returns:
        Router configured with the given fallback chain
    """
    if not models:
        msg = "build_dynamic_fallback_router: models list must not be empty"
        raise ValueError(msg)

    # Check and invalidate cache if circuit breaker state changed
    _invalidate_router_cache_on_circuit_breaker_change()

    # Create cache key from models tuple and base_policy
    cache_key = (tuple(models), base_policy)

    # Try to get from cache first
    with _router_lock:
        if cache_key in _model_list_cache:
            filtered_model_list = _model_list_cache[cache_key]
        else:
            config = get_router_config()
            full_model_list = build_litellm_model_list()

            # Build a set of known model names for fast lookup
            known_model_names: set[str] = {entry["model_name"] for entry in full_model_list}

            # Filter full model list to only the requested models, preserving order.
            # For any model not in the catalog, create a minimal passthrough config so
            # LiteLLM can attempt to call it and fail loudly rather than silently skip.
            filtered_model_list = []
            seen: set[str] = set()
            for model_name in models:
                if model_name in seen:
                    continue
                seen.add(model_name)
                if model_name in known_model_names:
                    for entry in full_model_list:
                        if entry["model_name"] == model_name:
                            filtered_model_list.append(entry)
                            break
                else:
                    filtered_model_list.append(
                        {
                            "model_name": model_name,
                            "litellm_params": {
                                "model": f"openai/{model_name}",
                                "api_key": "dummy-key",
                                "api_base": "http://localhost:8317/v1",
                            },
                        }
                    )

            # Exclude deployments whose provider circuit breaker is OPEN
            filtered_model_list = get_healthy_deployments(filtered_model_list)

            # Cache the filtered model list
            _model_list_cache[cache_key] = filtered_model_list

    config = get_router_config()

    primary = models[0]
    fallback_models = models[1:]

    router_kwargs: dict[str, Any] = {
        "model_list": filtered_model_list,
        "routing_strategy": base_policy,
        "num_retries": config.num_retries,
        "timeout": config.timeout,
        "retry_after": config.retry_after,
        "cooldown_time": config.cooldown_time,
    }

    if fallback_models:
        router_kwargs["fallbacks"] = [{primary: fallback_models}]

    return Router(**router_kwargs)


def get_pareto_preferred_model(complexity_tier: str = "moderate") -> str | None:
    """Pre-select model via Pareto for LiteLLM when policy=pareto. Returns provider/model or None."""
    try:
        from thegent.utils.routing_impl.pareto_router import select_offer

        route = select_offer(complexity_tier=complexity_tier)
        if route:
            return f"{route[0]}/{route[1]}"
    except Exception:
        pass
    return None


class EnhancedRouter:
    """Enhanced router with full feature support.

    Wraps LiteLLM Router with:
    - Cost tracking integration
    - Alert management
    - Donut Architecture integration
    - Context window validation
    - Streaming support
    """

    def __init__(self, policy: str | None = None) -> None:
        """Initialize enhanced router.

        Args:
            policy: Optional routing policy override
        """
        self._config = get_router_config()
        self._policy = policy or self._config.routing_policy
        self._router = get_litellm_router(self._policy)
        self._fallback_chains = build_fallback_chains()

        # Lazy-load integrations
        self._cost_tracker = None
        self._alert_manager = None
        self._donut_adapter = None

        # Validate model metadata availability
        self._validate_model_metadata()

    @property
    def cost_tracker(self):
        """Get cost tracker (lazy initialization)."""
        if self._cost_tracker is None and self._config.enable_cost_tracking:
            try:
                from thegent.utils.routing_impl.cost_tracker import get_cost_tracker

                self._cost_tracker = get_cost_tracker()
            except Exception as e:
                logger.debug("Could not initialize cost tracker: %s", e)
        return self._cost_tracker

    @property
    def alert_manager(self):
        """Get alert manager (lazy initialization)."""
        if self._alert_manager is None and self._config.alert_webhook:
            try:
                from thegent.utils.routing_impl.alerting import get_alert_manager

                self._alert_manager = get_alert_manager()
            except Exception as e:
                logger.debug("Could not initialize alert manager: %s", e)
        return self._alert_manager

    @property
    def donut_adapter(self):
        """Get Donut adapter (lazy initialization)."""
        if self._donut_adapter is None:
            try:
                from thegent.utils.routing_impl.donut_adapter import get_donut_adapter

                self._donut_adapter = get_donut_adapter()
            except Exception as e:
                logger.debug("Could not initialize Donut adapter: %s", e)
        return self._donut_adapter

    def route(
        self,
        prompt: str,
        model: str | None = None,
        stream: bool = False,
        **kwargs: Any,
    ) -> RoutingResult:
        """Route a request through LiteLLM.

        Args:
            prompt: The prompt to send
            model: Optional model override (otherwise router selects)
            stream: Whether to stream the response
            **kwargs: Additional LiteLLM parameters

        Returns:
            RoutingResult with response and metadata
        """
        start_time = time.time()
        queue_metadata = kwargs.pop("queue_metadata", None)
        if model is None and isinstance(queue_metadata, dict):
            preferred_model = queue_metadata.get("model") or queue_metadata.get("preferred_model")
            if isinstance(preferred_model, str) and preferred_model.strip():
                selected_model = preferred_model.strip()
            else:
                selected_model = None
        else:
            selected_model = model
        is_fallback = False

        # Validate model metadata availability (silent check, no warning spam)
        if selected_model and not has_model_metadata(selected_model):
            # Try to get from litellm model format (provider/model)
            if "/" in selected_model:
                provider_model = selected_model.split("/", 1)[1]
                if not has_model_metadata(provider_model):
                    logger.debug("Model metadata not found for %s (using fallback)", selected_model)
            else:
                logger.debug("Model metadata not found for %s (using fallback)", selected_model)

        # Check budget before routing
        if self._config.cost_budget and self.cost_tracker:
            if self.cost_tracker.is_over_budget():
                if self.alert_manager:
                    self.alert_manager.alert_budget_exceeded(
                        self.cost_tracker.get_daily_spend(),
                        self._config.cost_budget,
                    )
                return RoutingResult(
                    success=False,
                    model=model or "unknown",
                    provider="unknown",
                    error="Daily budget exceeded",
                )

        # Context window validation
        if self._config.context_window_validation and selected_model:
            # Estimate token count (rough: ~4 chars per token)
            estimated_tokens = len(prompt) // 4
            if not validate_context_window(selected_model, estimated_tokens):
                logger.warning(
                    "Prompt may exceed context window for %s: ~%d tokens",
                    selected_model,
                    estimated_tokens,
                )

        # Make the request
        try:
            effective_model: str = selected_model or "unknown"
            if stream and self._config.enable_streaming:
                response = self._router.completion(
                    model=effective_model,
                    messages=[{"role": "user", "content": prompt}],
                    stream=True,
                    **kwargs,
                )
            else:
                response = self._router.completion(
                    model=effective_model,
                    messages=[{"role": "user", "content": prompt}],
                    **kwargs,
                )

            latency_ms = (time.time() - start_time) * 1000

            # Extract metadata from response — narrow to str
            _raw_model = getattr(response, "model", None) if response else None
            model_used: str = str(_raw_model) if _raw_model else effective_model
            provider = self._extract_provider(model_used)

            # Track tokens and cost
            tokens_used = 0
            cost_usd = 0.0
            response_usage = getattr(response, "usage", None) if response else None
            if response_usage is not None:
                tokens_used = getattr(response_usage, "total_tokens", 0)
                # LiteLLM calculates cost internally; we estimate here
                cost_usd = self._estimate_cost(model_used, tokens_used)

            # Track cost
            if self.cost_tracker and tokens_used > 0:
                self.cost_tracker.track(
                    provider=provider,
                    model=model_used,
                    usage={
                        "prompt_tokens": getattr(response_usage, "prompt_tokens", 0) if response_usage else 0,
                        "completion_tokens": getattr(response_usage, "completion_tokens", 0) if response_usage else 0,
                    },
                    cost=cost_usd,
                    latency_ms=latency_ms,
                    is_fallback=is_fallback,
                )

            # Record in Donut adapter
            if self.donut_adapter:
                self.donut_adapter.record_request(
                    model=model_used,
                    provider=provider,
                    tokens=tokens_used,
                    cost_usd=cost_usd,
                    is_fallback=is_fallback,
                )

            # Check latency threshold
            if latency_ms > self._config.latency_threshold_ms:
                if self.alert_manager:
                    self.alert_manager.alert_high_latency(
                        model=model_used,
                        latency_ms=latency_ms,
                        threshold_ms=self._config.latency_threshold_ms,
                        provider=provider,
                    )

            return RoutingResult(
                success=True,
                model=model_used,
                provider=provider,
                response=response,
                latency_ms=latency_ms,
                tokens_used=tokens_used,
                cost_usd=cost_usd,
                is_fallback=is_fallback,
            )

        except Exception as e:
            latency_ms = (time.time() - start_time) * 1000
            error_str = str(e)
            logger.error("Routing error: %s", error_str)

            # Try fallback if enabled
            if self._config.fallback_enabled and selected_model:
                fallbacks = self._fallback_chains.get(selected_model, [])
                for fallback_model in fallbacks:
                    try:
                        logger.info("Trying fallback model: %s", fallback_model)
                        is_fallback = True
                        # Recurse with fallback model
                        result = self.route(prompt, model=fallback_model, stream=stream, **kwargs)
                        result.is_fallback = True
                        return result
                    except Exception as fb_e:  # noqa: PERF203 - intentional per-item error handling
                        logger.warning("Fallback %s failed: %s", fallback_model, fb_e)
                        continue

            # Alert on error
            if self.alert_manager:
                self.alert_manager.alert_provider_error(
                    provider=self._extract_provider(selected_model) if selected_model else "unknown",
                    error=error_str,
                    model=selected_model or "unknown",
                    is_rate_limit="rate" in error_str.lower(),
                )

            return RoutingResult(
                success=False,
                model=selected_model or "unknown",
                provider=self._extract_provider(selected_model) if selected_model else "unknown",
                error=error_str,
                latency_ms=latency_ms,
                is_fallback=is_fallback,
            )

    def route_stream(
        self,
        prompt: str,
        model: str | None = None,
        **kwargs: Any,
    ) -> Iterator[Any]:
        """Route with streaming response.

        Args:
            prompt: The prompt to send
            model: Optional model override
            **kwargs: Additional LiteLLM parameters

        Yields:
            Stream chunks from the model
        """
        result = self.route(prompt, model=model, stream=True, **kwargs)
        if result.success and result.response is not None and hasattr(result.response, "__iter__"):
            yield from result.response
        else:
            raise RuntimeError(result.error or "Streaming failed")

    def _extract_provider(self, model: str) -> str:
        """Extract provider from model string."""
        if "/" in model:
            return model.split("/", maxsplit=1)[0]
        # Try to match from model_list
        for entry in self._router.model_list:
            if entry.get("model_name") == model:
                litellm_model = entry.get("litellm_params", {}).get("model", "")
                if "/" in litellm_model:
                    return litellm_model.split("/")[0]
        return "unknown"

    def _estimate_cost(self, model: str, tokens: int) -> float:
        """Estimate cost for a model call.

        Rough estimates based on typical pricing.
        For accurate costs, enable LiteLLM's built-in cost tracking.
        """
        # Try to get from metadata registry first
        try:
            metadata = get_model_metadata(model)
            if metadata and "cost_per_mtok" in metadata:
                # Convert from per MTok to per 1K tokens
                cost_per_1k = metadata["cost_per_mtok"] / 1000.0
                return (tokens / 1000) * cost_per_1k
        except Exception:
            pass

        # Fallback to static dictionary
        cost_per_1k = {
            "claude-opus-4.6": 0.075,
            "claude-sonnet-4.5": 0.015,
            "claude-haiku-4.5": 0.001,
            "gpt-4o": 0.025,
            "gpt-4o-mini": 0.0003,
            "gemini-3-flash": 0.000075,
            "deepseek-v3.2": 0.0005,
            "glm-5": 0.0007,
            "GLM-5": 0.0007,
            "z-ai/glm-5": 0.0007,
            "minimax-m2.5": 0.0004,
            "minimaxm2.5": 0.0004,
            "MiniMax-M2.5": 0.0004,
            "kimi-k2.5": 0.0005,
            "kilo-default": 0.0005,
            "roo-default": 0.0005,
            "qwen3-coder": 0.0003,
            "llama-nemotron-ultra": 0.0002,
        }

        # Normalize model name
        normalized = model.lower().replace("-", "").replace(".", "").replace("/", "")

        for key, cost in cost_per_1k.items():
            if key.lower().replace("-", "").replace(".", "") in normalized:
                return (tokens / 1000) * cost

        # Default: assume budget model pricing
        return (tokens / 1000) * 0.0005

    def _validate_model_metadata(self) -> None:
        """Validate that all models in router have metadata available.

        Silently checks models and ensures metadata is available.
        This prevents warnings from Codex CLI about missing model metadata.
        """
        try:
            from thegent.utils.routing_impl.model_metadata import has_model_metadata

            # Check all models in router and ensure they have metadata
            for entry in self._router.model_list:
                model_name = entry.get("model_name", "")
                if not model_name:
                    continue

                # Check direct model name
                if not has_model_metadata(model_name):
                    # Check if it's a provider/model format (e.g., "glm/glm-5")
                    litellm_model = entry.get("litellm_params", {}).get("model", "")
                    if "/" in litellm_model:
                        provider_model = litellm_model.split("/", 1)[1]
                        if not has_model_metadata(provider_model):
                            # Try normalized version
                            normalized = (
                                provider_model.lower()
                                .replace("-", "")
                                .replace(".", "")
                                .replace("/", "")
                                .replace("_", "")
                            )
                            found = False
                            for key in get_all_models_with_metadata():
                                key_normalized = (
                                    key.lower().replace("-", "").replace(".", "").replace("/", "").replace("_", "")
                                )
                                if key_normalized == normalized:
                                    found = True
                                    break
                            if not found:
                                logger.debug("Model metadata not found for %s (alias: %s)", provider_model, model_name)
                    else:
                        logger.debug("Model metadata not found for %s", model_name)
        except Exception as e:
            logger.debug("Could not validate model metadata: %s", e)


def get_circuit_breaker_status() -> dict[str, str]:
    """Return current circuit breaker state for all tracked providers.

    Queries the global :class:`ProviderCircuitBreakerRegistry` singleton and
    returns a snapshot of every registered provider's current state.

    Returns:
        Dict mapping provider name to state string: ``"closed"``, ``"open"``,
        or ``"half-open"``.
    """
    registry = ProviderCircuitBreakerRegistry.get_instance()
    return registry.all_states()


# Global enhanced router instance
_enhanced_router: EnhancedRouter | None = None


def get_enhanced_router(policy: str | None = None) -> EnhancedRouter:
    """Get global enhanced router instance.

    Args:
        policy: Optional routing policy override

    Returns:
        EnhancedRouter instance
    """
    global _enhanced_router
    if _enhanced_router is None:
        _enhanced_router = EnhancedRouter(policy)
    return _enhanced_router


def reset_enhanced_router() -> None:
    """Reset the global enhanced router (useful for testing)."""
    global _enhanced_router
    _enhanced_router = None
