"""Model catalog and route resolution for distributed routing."""

import hashlib
from collections import OrderedDict
from dataclasses import dataclass, field
from typing import Literal, cast, get_args

from thegent.utils.provider_names import normalize_provider_name
from thegent.infra import get_cache

# Canonical model ID -> list of routes (provider, backend, model_alias, priority)
# Lower priority = prefer first when using prefer_direct
RoutePolicy = Literal["prefer_direct", "prefer_proxy", "failover", "round_robin", "cheapest", "pareto"]
ROUTE_SCHEMA_VERSION = 1


@dataclass
class Route:
    """A route to serve a model via a specific provider."""

    provider: str
    backend_type: Literal["direct", "proxy"]
    model_alias: str
    priority: int = 0  # lower = prefer first for prefer_direct
    cost_weight: float = 1.0  # lower = cheaper


@dataclass(frozen=True)
class ResolvedRoute:
    """Resolved model routing decision with contract metadata."""

    provider: str
    model_alias: str
    backend_type: Literal["direct", "proxy"]
    priority: int
    cost_weight: float = 1.0
    schema_version: int = ROUTE_SCHEMA_VERSION


@dataclass
class CatalogView:
    """View of the model catalog for discovery."""

    by_provider: dict[str, list[str]] = field(default_factory=dict)
    by_model: dict[str, list[str]] = field(default_factory=dict)


def _build_static_catalog() -> dict[str, list[Route]]:
    """Build static model -> routes. Anthropic 4.5/4.6, Gemini flash + 3.1 pro, Codex 5.3, GLM-5, MiniMax-M2.5."""
    # (provider, backend, model_alias, priority, cost_weight). priority -1=native, 0=direct, 10=proxy
    provider_models: list[tuple[str, str, str, int, float]] = [
        # Anthropic: 4.5 (haiku, sonnet), 4.6 (opus)
        ("claude", "direct", "claude-haiku-4.5", -1, 0.2),
        ("claude", "direct", "claude-sonnet-4.6", -1, 0.5),
        ("claude", "direct", "claude-opus-4.6", -1, 1.0),
        ("claude", "direct", "claude-opus-4.6-1m", -1, 1.2),
        ("claude", "direct", "haiku", -1, 0.2),
        ("claude", "direct", "sonnet", -1, 0.5),
        ("claude", "direct", "opus", -1, 1.0),
        # Gemini: flash + 3.1 pro
        ("gemini", "direct", "gemini-2.0-flash", -1, 0.1),
        ("gemini", "direct", "gemini-2.5-flash", -1, 0.1),
        ("gemini", "direct", "gemini-3-flash", -1, 0.1),
        ("gemini", "direct", "gemini-3.1-pro", -1, 0.6),
        ("copilot", "direct", "claude-haiku-4.5", 0, 0.2),
        ("copilot", "direct", "gpt-5.3-codex", 0, 0.5),
        ("codex", "direct", "gpt-5.3-codex", 0, 0.5),
        ("codex", "direct", "gpt-5.3-codex-high", 0, 0.8),
        ("cursor-agent", "direct", "gemini-3-flash", 0, 0.1),
        ("cursor-agent", "direct", "composer-1", 0, 0.25),
        ("cursor-agent", "direct", "composer-1.5", 0, 0.3),
        ("cursor-api", "proxy", "claude-4.5-opus-high-thinking", 5, 1.2),
        ("cursor-api", "proxy", "claude-4.5-opus-high", 5, 1.1),
        ("cursor-api", "proxy", "claude-4.5-sonnet-thinking", 5, 0.7),
        ("cursor-api", "proxy", "claude-4-sonnet", 5, 0.6),
        ("cursor-api", "proxy", "gpt-4o", 5, 0.8),
        ("cursor-api", "proxy", "gpt-5.1-codex", 5, 0.7),
        ("antigravity", "proxy", "gemini-3-flash", 10, 0.2),
        ("antigravity", "proxy", "gemini-3.1-pro", 10, 0.7),
        ("antigravity", "proxy", "claude-sonnet-4.6", 10, 0.6),
        ("antigravity", "proxy", "claude-haiku-4.5", 10, 0.3),
        ("antigravity", "proxy", "claude-opus-4.6", 10, 1.1),
        ("antigravity", "proxy", "claude-opus-4.6-1m", 10, 1.3),
        ("minimax", "proxy", "minimax-m2.5", 0, 0.4),
        ("glm", "proxy", "glm-5", 0, 0.4),
        ("ollama", "direct", "llama3.3", 0, 0.0),
        ("ollama", "direct", "llama3.2", 0, 0.0),
        ("ollama", "direct", "llama3.1", 0, 0.0),
        ("ollama", "direct", "qwen2.5-coder", 0, 0.0),
        ("ollama", "direct", "mistral", 0, 0.0),
        ("ollama", "direct", "codellama", 0, 0.0),
        ("ollama", "direct", "deepseek-coder-v2", 0, 0.0),
        ("ollama", "direct", "phi4", 0, 0.0),
        ("ollama", "direct", "gemma3", 0, 0.0),
        ("roo", "proxy", "roo-default", 0, 0.5),
        ("kilo", "proxy", "kilo-default", 0, 0.5),
    ]
    # Canonical model ID -> routes
    catalog: dict[str, list[Route]] = {}
    for provider, backend, model_alias, priority, cost in provider_models:
        canonical = _canonicalize(model_alias, provider)
        route = Route(
            provider=provider,
            backend_type=cast("Literal['direct', 'proxy']", backend),
            model_alias=model_alias,
            priority=priority,
            cost_weight=cost,
        )
        if canonical not in catalog:
            catalog[canonical] = []
        catalog[canonical].append(route)
    # Dedupe and sort by priority
    for k, routes in catalog.items():
        seen: set[tuple[str, str]] = set()
        unique: list[Route] = []
        for r in sorted(routes, key=lambda x: (x.priority, x.provider)):
            key = (r.provider, r.model_alias)
            if key not in seen:
                seen.add(key)
                unique.append(r)
        catalog[k] = unique
    return catalog


# Minimal aliases: anthropic 4.5 (haiku, sonnet), 4.6 (opus). Prefer dynamic over hardcoded.
_ALIASES: dict[str, str] = {
    "haiku": "claude-haiku-4.5",
    "sonnet": "claude-sonnet-4.6",
    "opus": "claude-opus-4.6",
    "opus1m": "claude-opus-4.6-1m",
}


def _is_model_blacklisted(model_id: str, provider: str) -> bool:
    """
    True if model is explicitly older than allowed. Unparseable models return False (allowed).
    Anthropic: 4.5 (haiku, sonnet), 4.6 (opus) only. Gemini: flash + 3.1 pro. Codex: 5.3 only.
    """
    m = (model_id or "").strip().lower()
    if not m:
        return True
    # Anthropic 3.x
    if "claude-3" in m:
        return True
    # Anthropic 4.0/4.1 style (claude-4-haiku etc) - we want 4.5, 4.6
    if "claude-4-" in m and "4.5" not in m and "4.6" not in m:
        return True
    # Gemini 1.x
    if "gemini-1" in m:
        return True
    if "gemini-2.0-flash-exp" in m:
        return True
    # Gemini: allow 3.1 pro variants only
    if "gemini" in m and "-pro" in m and "gemini-3.1-pro" not in m:
        return True
    # GPT-4
    if "gpt-4" in m:
        return True
    # Codex/copilot: gpt-5 without 5.3 is older
    return bool(provider in ("codex", "copilot") and "gpt-5" in m and "5.3" not in m)


def filter_models_for_provider(provider: str, models: list[str]) -> list[str]:
    """Filter scraped models: remove blacklisted, keep unparseable (allow by default)."""
    return [m for m in models if m and not _is_model_blacklisted(m, provider)]


# OPT-019: Cache for normalized model IDs (frequently called)
_NORMALIZE_CACHE = get_cache(l1_size=200, l2_size=1000, l3_path=None, default_ttl=3600)
_USE_NORMALIZE_CACHE = _NORMALIZE_CACHE is not None


def normalize_model_id(model_id: str) -> str:
    """Normalize provider-agnostic model aliases to canonical IDs."""
    # OPT-019: Check cache first
    if _USE_NORMALIZE_CACHE:
        cached = _NORMALIZE_CACHE.get(model_id)
        if cached is not None:
            return cached

    candidate = (model_id or "").strip()
    result = _ALIASES.get(candidate, candidate)

    # OPT-019: Cache the result
    if _USE_NORMALIZE_CACHE:
        _NORMALIZE_CACHE.set(model_id, result, ttl=3600)

    return result


def normalize_route_policy(policy: str | None) -> RoutePolicy:
    """Validate and normalize routing policy. Raises ValueError on invalid policy."""
    normalized = (policy or "prefer_direct").strip().lower()
    if normalized in ("prefer_direct", "prefer_proxy", "failover", "round_robin", "cheapest", "pareto"):
        return cast("RoutePolicy", normalized)
    raise ValueError(
        f"Invalid routing policy '{policy}'. Valid values: prefer_direct, prefer_proxy, failover, round_robin, cheapest, pareto."
    )


def route_contract() -> dict[str, object]:
    """Return catalog contract metadata for auditing and compatibility checks."""
    return {
        "schema_version": ROUTE_SCHEMA_VERSION,
        "backend_types": ["direct", "proxy"],
        "policy_names": list(get_args(RoutePolicy)),
    }


def _canonicalize(model_id: str, provider: str) -> str:
    """Map alias or provider-specific ID to canonical model ID."""
    return normalize_model_id(model_id)


_PROXY_PROVIDERS: frozenset[str] = frozenset({"antigravity", "minimax", "glm"})


def _scraped_to_routes(by_provider: dict[str, list[str]]) -> dict[str, list[Route]]:
    """Convert scraped by_provider to model_id -> list[Route] for routing merge (Phase 7)."""
    by_model: dict[str, list[Route]] = {}
    for provider, models in by_provider.items():
        backend = "proxy" if provider in _PROXY_PROVIDERS else "direct"
        priority = 10 if backend == "proxy" else 0
        cost_weight = 0.8 if backend == "proxy" else 0.3
        for model_id in models:
            if not model_id or not isinstance(model_id, str):
                continue
            canonical = _canonicalize(model_id, provider)
            if canonical not in by_model:
                by_model[canonical] = []
            route = Route(
                provider=provider,
                backend_type=cast("Literal['direct', 'proxy']", backend),
                model_alias=model_id,
                priority=priority,
                cost_weight=cost_weight,
            )
            if not any(r.provider == provider and r.model_alias == model_id for r in by_model[canonical]):
                by_model[canonical].append(route)
    return by_model


def _merge_routes(base_routes: list[Route], extra_routes: list[Route]) -> list[Route]:
    """Merge route lists preserving first-seen priority and stable ordering."""
    seen: set[tuple[str, str]] = {(r.provider, r.model_alias) for r in base_routes}
    merged = list(base_routes)
    for route in extra_routes:
        key = (route.provider, route.model_alias)
        if key in seen:
            continue
        seen.add(key)
        merged.append(route)
    return merged


_STATIC_CATALOG: dict[str, list[Route]] | None = None

# OPT-019: Multi-tier cache for static catalog (rarely changes, benefits from persistence)
_CATALOG_CACHE = get_cache(l1_size=1, l2_size=1, l3_path=None, default_ttl=3600)
_USE_CATALOG_CACHE = _CATALOG_CACHE is not None


def _get_catalog() -> dict[str, list[Route]]:
    """Get static catalog with multi-tier caching."""
    global _STATIC_CATALOG

    # OPT-019: Check cache first
    if _USE_CATALOG_CACHE:
        cached = _CATALOG_CACHE.get("static_catalog")
        if cached is not None:
            return cached

    if _STATIC_CATALOG is None:
        _STATIC_CATALOG = _build_static_catalog()

        # WP-X8: Load custom model catalog from user configuration
        try:
            from thegent.config import get_settings

            settings = get_settings()
            custom_path = settings.custom_models_path
            if custom_path.exists():
                import yaml

                custom_data = yaml.safe_load(custom_path.read_text(encoding="utf-8"))
                if custom_data and isinstance(custom_data, dict):
                    for model_id, routes_data in custom_data.items():
                        custom_routes = []
                        for r in routes_data:
                            if not isinstance(r, dict):
                                continue
                            try:
                                custom_routes.append(
                                    Route(
                                        provider=r["provider"],
                                        backend_type=r.get("backend_type", "direct"),
                                        model_alias=r["model_alias"],
                                        priority=r.get("priority", 0),
                                        cost_weight=r.get("cost_weight", 1.0),
                                    )
                                )
                            except KeyError:
                                continue

                        if custom_routes:
                            if model_id in _STATIC_CATALOG:
                                # Prepend custom routes to override/augment static ones
                                _STATIC_CATALOG[model_id] = _merge_routes(custom_routes, _STATIC_CATALOG[model_id])
                            else:
                                _STATIC_CATALOG[model_id] = custom_routes
        except Exception as e:
            import logging

            logging.getLogger(__name__).warning("Failed to load custom models: %s", e)

    # OPT-019: Cache the result
    if _USE_CATALOG_CACHE:
        _CATALOG_CACHE.set("static_catalog", _STATIC_CATALOG, ttl=3600)

    return _STATIC_CATALOG


def _by_provider_to_by_model(by_provider: dict[str, list[str]]) -> dict[str, list[str]]:
    """Invert by_provider to by_model."""
    by_model: dict[str, list[str]] = {}
    for provider, models in by_provider.items():
        for m in models:
            canonical = _canonicalize(m, provider)
            if canonical not in by_model:
                by_model[canonical] = []
            if provider not in by_model[canonical]:
                by_model[canonical].append(provider)
    return by_model


class ModelCatalog:
    """Model catalog for route resolution. Merges static + scraped when use_scraped=True (Phase 7)."""

    @staticmethod
    def routes_for(model_id: str, use_scraped: bool = True) -> list[Route]:
        """Return all routes that can serve the given model. Merges scraped data when use_scraped."""
        canonical = normalize_model_id(model_id)
        catalog = _get_catalog()
        static_routes = catalog.get(canonical) or catalog.get(model_id) or []

        if not use_scraped:
            return static_routes

        try:
            from thegent.models.scrapers import get_scraped_catalog

            scraped = get_scraped_catalog()
            if not scraped:
                return static_routes
            scraped_routes_map = _scraped_to_routes(scraped)
            scraped_routes = scraped_routes_map.get(canonical) or scraped_routes_map.get(model_id) or []
        except Exception:
            return static_routes

        # Merge: static first, then scraped routes not already present (by provider+model_alias)
        seen: set[tuple[str, str]] = {(r.provider, r.model_alias) for r in static_routes}
        merged = list(static_routes)
        for r in scraped_routes:
            if (r.provider, r.model_alias) not in seen:
                seen.add((r.provider, r.model_alias))
                merged.append(r)
        return merged

    @staticmethod
    def to_catalog_view(use_scraped: bool = True) -> CatalogView:
        """Build CatalogView for discovery (by_provider, by_model). Uses scraped data when available."""
        catalog = _get_catalog()
        by_provider: dict[str, list[str]] = {}
        by_model: dict[str, list[str]] = {}
        for model_id, routes in catalog.items():
            by_model[model_id] = list(dict.fromkeys(r.provider for r in routes))
            for r in routes:
                if r.provider not in by_provider:
                    by_provider[r.provider] = []
                if model_id not in by_provider[r.provider]:
                    by_provider[r.provider].append(model_id)
        if use_scraped:
            try:
                from thegent.models.scrapers import get_scraped_catalog

                scraped = get_scraped_catalog()
                if scraped:
                    by_provider = scraped
                    by_model = _by_provider_to_by_model(scraped)
            except Exception:
                pass
        return CatalogView(by_provider=by_provider, by_model=by_model)

    @staticmethod
    def to_contract_view(
        use_scraped: bool = True,
        provider_filter: str | None = None,
        use_cache: bool = True,
    ) -> dict[str, object]:
        """Return catalog with schema metadata and route details for structured consumers."""
        catalog = dict(_get_catalog())
        if use_scraped:
            try:
                from thegent.models.scrapers import get_scraped_catalog

                scraped = get_scraped_catalog(use_cache=use_cache)
                if scraped:
                    scraped_routes = _scraped_to_routes(scraped)
                    for model_id, routes in scraped_routes.items():
                        catalog[model_id] = _merge_routes(catalog.get(model_id, []), routes)
            except Exception:
                pass

        detail: dict[str, list[dict[str, object]]] = {}
        for model_id in sorted(catalog):
            routes = sorted(catalog[model_id], key=lambda r: (r.provider, r.backend_type, r.model_alias))
            route_rows = [
                {
                    "provider": r.provider,
                    "backend_type": r.backend_type,
                    "model_alias": r.model_alias,
                    "priority": r.priority,
                    "cost_weight": r.cost_weight,
                    "schema_version": ROUTE_SCHEMA_VERSION,
                }
                for r in routes
            ]
            if provider_filter:
                route_rows = [row for row in route_rows if row["provider"] == provider_filter]
            if not route_rows:
                continue
            detail[model_id] = cast("list[dict[str, object]]", route_rows)
        return {
            "schema_version": ROUTE_SCHEMA_VERSION,
            "count": len(detail),
            "routes": detail,
            "contract": route_contract(),
            **({"provider_filter": provider_filter} if provider_filter else {}),
        }


def resolve_route_contract(
    model_id: str,
    provider_hint: str | None = None,
    policy: RoutePolicy = "prefer_direct",
) -> ResolvedRoute | None:
    """Resolve model and return contract-rich route metadata."""
    route = resolve_route(model_id, provider_hint=provider_hint, policy=policy)
    if route is None:
        return None
    provider, model_alias = route
    for r in ModelCatalog.routes_for(model_id):
        if r.provider == provider and r.model_alias == model_alias:
            return ResolvedRoute(
                provider=r.provider,
                model_alias=r.model_alias,
                backend_type=r.backend_type,
                priority=r.priority,
                cost_weight=r.cost_weight,
            )
    return None


_RR_COUNTER: dict[str, int] = {}


# OPT-020: Route resolution memo with model ID hash prefix (LRU, 1000 entries)
# Cache key: hash(model_id + provider_hint + policy + quality_floor + lane) -> (provider, model_alias)
# Enhanced with multi-tier caching for better performance
_ROUTE_CACHE = get_cache(l1_size=100, l2_size=1000, l3_path=None, default_ttl=300)
_USE_MULTI_TIER_CACHE = _ROUTE_CACHE is not None
_ROUTE_RESOLVE_CACHE: OrderedDict[str, tuple[str, str] | None] = OrderedDict()
_ROUTE_CACHE_MAX_SIZE = 1000


def _make_route_cache_key(
    model_id: str,
    provider_hint: str | None,
    policy: RoutePolicy,
    quality_floor: float,
    lane: str | None,
) -> str:
    """Create cache key for route resolution."""
    key_parts = [model_id or "", provider_hint or "", policy, str(quality_floor), lane or ""]
    key_str = "|".join(key_parts)
    # Use hash prefix for shorter keys (first 16 chars of SHA256)
    return hashlib.sha256(key_str.encode()).hexdigest()[:16]


def resolve_route(
    model_id: str,
    provider_hint: str | None = None,
    policy: RoutePolicy = "prefer_direct",
    quality_floor: float = 0.0,
    lane: str | None = None,
) -> tuple[str, str] | None:
    """
    Resolve model to (provider, model_alias). Returns None if no route.

    OPT-020: Uses LRU cache (1000 entries) for sub-1ms repeated route lookups.

    - provider_hint: Use this provider if it serves the model.
    - policy: prefer_direct (default) | prefer_proxy | failover | round_robin | cheapest
    - quality_floor: Minimum quality threshold (0-1) for cost_quality routing.
    - lane: Execution lane (reserved for future routing strategies).
    """
    normalized_provider_hint = normalize_provider_name(provider_hint) if provider_hint else None

    # OPT-020: Check cache first (multi-tier if available)
    cache_key = _make_route_cache_key(model_id, normalized_provider_hint, policy, quality_floor, lane)
    if _USE_MULTI_TIER_CACHE:
        cached_result = _ROUTE_CACHE.get(cache_key)
        if cached_result is not None:
            return cached_result
    elif cache_key in _ROUTE_RESOLVE_CACHE:
        # Move to end (most recently used)
        result = _ROUTE_RESOLVE_CACHE.pop(cache_key)
        _ROUTE_RESOLVE_CACHE[cache_key] = result
        return result

    routes = ModelCatalog.routes_for(model_id)
    if not routes:
        return None

    if normalized_provider_hint:
        for r in routes:
            if r.provider == normalized_provider_hint:
                return (r.provider, r.model_alias)
        return None

    # Sort based on policy
    if policy == "pareto":
        try:
            from thegent.utils.routing_impl.pareto_router import (
                Offer,
                _get_quality,
                _get_shadow_multiplier,
                _lexicographic_select,
                _pareto_frontier,
            )

            shadow = _get_shadow_multiplier()
            offers = []
            for r in routes:
                q = _get_quality(r.model_alias)
                if q < quality_floor:
                    continue
                speed_score = r.cost_weight
                effective_cost = r.cost_weight * shadow
                offers.append(
                    Offer(
                        provider=r.provider,
                        model_alias=r.model_alias,
                        cost_weight=r.cost_weight,
                        quality=q,
                        speed_score=speed_score,
                        route=r,
                        effective_cost=effective_cost,
                    )
                )
            if offers:
                frontier = _pareto_frontier(offers)
                sel = _lexicographic_select(frontier or offers, order=("quality", "cost", "speed"))
                if sel:
                    return (sel.provider, sel.model_alias)
        except Exception:
            pass
        routes = sorted(routes, key=lambda r: (r.cost_weight, r.priority, r.provider))
    elif policy == "prefer_proxy":
        # prefer_proxy = proxy first (priority 10), then direct (0)
        routes = sorted(routes, key=lambda r: (-r.priority, r.provider))
    elif policy == "round_robin":
        global _RR_COUNTER
        idx = _RR_COUNTER.get(model_id, 0)
        # Use stable sort for the base order before indexing
        routes = sorted(routes, key=lambda r: (r.priority, r.provider))
        _RR_COUNTER[model_id] = (idx + 1) % len(routes)
        r = routes[idx % len(routes)]
        return (r.provider, r.model_alias)
    elif policy == "cheapest":
        # Sort by cost_weight, then priority, then provider
        routes = sorted(routes, key=lambda r: (r.cost_weight, r.priority, r.provider))
    else:
        # prefer_direct (default) = direct first (priority 0), then proxy (10)
        routes = sorted(routes, key=lambda r: (r.priority, r.provider))

    # ROB-017: Explicit fallback chain: prefer_direct → prefer_proxy → error
    # If policy is prefer_direct and no direct route found, fallback to prefer_proxy
    if policy == "prefer_direct" and routes:
        # Try direct routes first (priority 0)
        direct_routes = [r for r in routes if r.priority == 0]
        if direct_routes:
            r = direct_routes[0]
            return (r.provider, r.model_alias)
        # Fallback to proxy routes (priority 10)
        proxy_routes = [r for r in routes if r.priority > 0]
        if proxy_routes:
            r = proxy_routes[0]
            return (r.provider, r.model_alias)
        # No routes available - return None (error)
        return None

    # For other policies, use first route from sorted list
    if not routes:
        result = None
    else:
        r = routes[0]
        result = (r.provider, r.model_alias)

    # OPT-020: Cache result (multi-tier if available, else LRU eviction)
    if _USE_MULTI_TIER_CACHE:
        _ROUTE_CACHE.set(cache_key, result, ttl=300)
    else:
        if len(_ROUTE_RESOLVE_CACHE) >= _ROUTE_CACHE_MAX_SIZE:
            # Remove oldest entry (first in OrderedDict)
            _ROUTE_RESOLVE_CACHE.popitem(last=False)
        _ROUTE_RESOLVE_CACHE[cache_key] = result

    return result
