"""Pareto-first router: hard constraints → Pareto frontier → lexicographic selection.

Implements the ChatGPT Pareto research design:
- Offer = provider + model + cost_weight + quality proxy
- Hard constraints filter first (capability, cost cap, quality floor)
- Pareto frontier = non-dominated offers on (speed, cost, quality)
- Lexicographic tie-break: quality → cost → speed (role-specific soft_order)
- Shadow pricing when budget pressure (Phase 1)
- Degraded mode at 85% budget burn (cheap offers only)
- Route trace output (Phase 0)

Also exposes the simple ParetoRouter public API:
- RouteCandidate: lightweight dataclass {model, provider, cost_per_1k, quality_score}
- ParetoRouter.select(): returns the Pareto-optimal candidate with highest quality/cost ratio
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from thegent.models.catalog import Route

_log = logging.getLogger(__name__)

# Degraded mode: when budget burn >= this ratio, disable premium offers
DEGRADED_BURN_THRESHOLD = 0.85

# Shadow pricing epsilon to avoid div-by-zero
_SHADOW_EPSILON = 0.01

# Fallback by failure type (Phase 4). On 429 → cheapest/fast; on timeout → fastest.
FAILURE_TYPE_FALLBACK_ORDER: dict[str, tuple[str, ...]] = {
    "rate_limit": ("cost", "speed", "quality"),  # Switch to cheaper/faster
    "timeout": ("speed", "cost", "quality"),  # Switch to fastest
    "schema_tool": ("quality", "cost", "speed"),  # Switch to highest adherence
    "default": ("quality", "cost", "speed"),
}

# Quality proxy: model alias -> rough quality tier (0-1). Benchmarks would improve this.
QUALITY_PROXY: dict[str, float] = {
    "claude-opus-4.6": 0.95,
    "claude-opus-4.6-1m": 0.96,
    "claude-sonnet-4.6": 0.88,
    "claude-haiku-4.5": 0.75,
    "gpt-5.3-codex-high": 0.92,
    "gpt-5.3-codex": 0.82,
    "claude-4.5-opus-high-thinking": 0.94,
    "claude-4.5-opus-high": 0.92,
    "claude-4.5-sonnet-thinking": 0.85,
    "claude-4-sonnet": 0.80,
    "gpt-4o": 0.85,
    "gpt-5.1-codex": 0.80,
    "gemini-3-flash": 0.78,
    "gemini-3.1-pro": 0.90,
    "gemini-2.5-flash": 0.76,
    "gemini-2.0-flash": 0.72,
    "glm-5": 0.78,
    "minimax-m2.5": 0.75,
    "deepseek-v3.2": 0.80,
    "composer-1.5": 0.82,
    "composer-1": 0.78,
    "roo-default": 0.70,
    "kilo-default": 0.70,
}


@dataclass
class RouteCandidate:
    """A routable model candidate with cost and quality metrics.

    Attributes:
        model: Model identifier (e.g. "claude-sonnet-4.6").
        provider: Provider name (e.g. "claude", "gemini").
        cost_per_1k: Cost in USD per 1 000 tokens.  Use 0.0 for free-tier routes.
        quality_score: Quality proxy in [0, 1].  Higher is better.
    """

    model: str
    provider: str
    cost_per_1k: float
    quality_score: float


class ParetoRouter:
    """Select the Pareto-optimal route that maximises quality per dollar (WP-1004).

    A candidate is *dominated* when another candidate has both strictly lower cost
    AND strictly higher quality (or equal on both with one strictly better).  The
    Pareto frontier is the set of non-dominated candidates.  Among frontier members,
    the one with the highest ``quality_score / cost_per_1k`` ratio is returned.

    Fallback (zero-cost): when every candidate has ``cost_per_1k == 0``, the
    ratio is undefined; select the candidate with the highest ``quality_score``.
    """

    def select(self, candidates: list[RouteCandidate]) -> RouteCandidate:
        """Return the best Pareto-optimal candidate using default balanced strategy."""
        if not candidates:
            raise ValueError("candidates must be non-empty")

        frontier = self._pareto_frontier(candidates)
        return self._best_from_frontier(frontier)

    def select_by_strategy(self, strategy: str, candidates: list[RouteCandidate]) -> RouteCandidate:
        """Applies a strategy slice to the Pareto Front (cost, speed, quality, balanced)."""
        if not candidates:
            raise ValueError("candidates must be non-empty")

        frontier = self._pareto_frontier(candidates)
        if not frontier:
            return candidates[0]

        if strategy == "cost":
            return min(frontier, key=lambda p: p.cost_per_1k)
        if strategy == "quality":
            return max(frontier, key=lambda p: p.quality_score)
        if strategy == "speed":
            # Speed is proxied by cost in this simplified model (cheaper models are usually faster)
            # In a real model, we'd have a separate latency metric.
            return min(frontier, key=lambda p: p.cost_per_1k)

        # Default to balanced
        return self._best_from_frontier(frontier)

    def get_optimal_providers(self, candidates: list[RouteCandidate]) -> list[RouteCandidate]:
        """Returns the non-dominated set (Pareto Front)."""
        return self._pareto_frontier(candidates)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _is_dominated(a: RouteCandidate, b: RouteCandidate) -> bool:
        """True if *b* dominates *a*: b is at least as good on both axes and strictly better on one."""
        cost_ok = b.cost_per_1k <= a.cost_per_1k
        quality_ok = b.quality_score >= a.quality_score
        strictly_better = b.cost_per_1k < a.cost_per_1k or b.quality_score > a.quality_score
        return cost_ok and quality_ok and strictly_better

    def _pareto_frontier(self, candidates: list[RouteCandidate]) -> list[RouteCandidate]:
        """Return non-dominated candidates."""
        frontier: list[RouteCandidate] = []
        for candidate in candidates:
            if not any(self._is_dominated(candidate, other) for other in candidates if other is not candidate):
                frontier.append(candidate)
        return frontier

    @staticmethod
    def _best_from_frontier(frontier: list[RouteCandidate]) -> RouteCandidate:
        """Select candidate with highest quality/cost ratio; fall back to highest quality when cost is zero."""
        all_zero_cost = all(c.cost_per_1k == 0.0 for c in frontier)
        if all_zero_cost:
            return max(frontier, key=lambda c: c.quality_score)

        # Among candidates with positive cost, prefer quality/cost ratio.
        # Zero-cost candidates are implicitly infinite ratio — treat them as best.
        def _ratio(c: RouteCandidate) -> float:
            if c.cost_per_1k == 0.0:
                return float("inf")
            return c.quality_score / c.cost_per_1k

        return max(frontier, key=_ratio)


@dataclass
class RoleConfig:
    """Role definition from roles.schema.yaml (Helios spec)."""

    name: str
    min_quality: float
    soft_order: tuple[str, ...]  # e.g. (quality, cost, speed)
    output_tokens_multiplier: float
    benchmark_weights: dict[str, float]  # role-specific weights for quality index
    needs_tools: bool = False
    min_context_tokens: int = 0


@dataclass
class Offer:
    """Routable offer: provider + model + indices."""

    provider: str
    model_alias: str
    cost_weight: float
    quality: float
    speed_score: float = 1.0  # Lower = faster; includes conciseness (output_tokens_multiplier)
    route: "Route | None" = None
    effective_cost: float | None = None  # After shadow pricing


@dataclass
class RouteTrace:
    """Route trace output per Helios spec (why offer won)."""

    selected_offer_id: str
    provider: str
    model_alias: str
    pareto_set: list[str]
    fallback_chain: list[tuple[str, str]]
    scores: dict[str, float]
    shadow_multiplier: float = 1.0
    degraded_mode: bool = False  # True when budget burn >= 85%
    role: str | None = None  # Role used for routing (if any)


def _get_quality(model_alias: str) -> float:
    """Get quality proxy for model. 0.5 default for unknown."""
    key = (model_alias or "").strip().lower()
    for k, v in QUALITY_PROXY.items():
        if k.lower() in key or key in k.lower():
            return v
    return 0.5


def _roles_config_path() -> Path:
    """Path to roles.schema.yaml. Tries project config, then package-relative."""
    try:
        from thegent.resources import get_resource_path

        p = get_resource_path("config/routing/roles.schema.yaml")
        if p.exists():
            return p
    except Exception:
        pass
    # Fallback: project root relative to this file
    return Path(__file__).resolve().parents[3] / "config" / "routing" / "roles.schema.yaml"


_ROLES_CACHE: dict[str, RoleConfig] | None = None


def _load_roles() -> dict[str, RoleConfig]:
    """Load roles from roles.schema.yaml. Cached."""
    global _ROLES_CACHE
    if _ROLES_CACHE is not None:
        return _ROLES_CACHE
    path = _roles_config_path()
    if not path.exists():
        _ROLES_CACHE = {}
        return _ROLES_CACHE
    try:
        import yaml

        data = yaml.safe_load(path.read_text()) or {}
        roles_data = data.get("roles", {}) or {}
        result: dict[str, RoleConfig] = {}
        for name, cfg in roles_data.items():
            if not isinstance(cfg, dict):
                continue
            hard = cfg.get("hard") or {}
            soft = cfg.get("soft_order") or ["quality", "cost", "speed"]
            soft_order = tuple(str(s) for s in soft)
            mult = float(cfg.get("output_tokens_multiplier", 1.0))
            min_q = float(hard.get("min_quality", 0.6))
            needs_tools = bool(hard.get("needs_tools", False))
            min_ctx = int(hard.get("min_context_tokens", 0))
            bw_raw = cfg.get("benchmark_weights") or {}
            benchmark_weights = {str(k): float(v) for k, v in bw_raw.items()} if isinstance(bw_raw, dict) else {}
            result[name] = RoleConfig(
                name=name,
                min_quality=min_q,
                soft_order=soft_order,
                output_tokens_multiplier=mult,
                benchmark_weights=benchmark_weights,
                needs_tools=needs_tools,
                min_context_tokens=min_ctx,
            )
        _ROLES_CACHE = result
        return result
    except Exception as e:
        _log.debug("Failed to load roles from %s: %s", path, e)
        _ROLES_CACHE = {}
        return _ROLES_CACHE


def _get_role(role: str | None) -> RoleConfig | None:
    """Resolve role config. Returns None if role unknown or roles not loaded."""
    if not role:
        return None
    roles = _load_roles()
    r = roles.get(role or "")
    if r is not None:
        return r
    return roles.get("default")


def _is_degraded_mode() -> bool:
    """True when budget burn >= 85%. Prefer cheap offers, disable premium."""
    try:
        from thegent.utils.routing_impl.cost_tracker import get_cost_tracker

        tracker = get_cost_tracker()
        ratio = tracker.get_budget_burn_ratio()
        return ratio is not None and ratio >= DEGRADED_BURN_THRESHOLD
    except Exception:
        return False


def _get_shadow_multiplier() -> float:
    """Budget shadow: 1 / max(remaining_ratio, ε). Higher when budget depleting."""
    try:
        from thegent.config import ThegentSettings
        from thegent.utils.routing_impl.cost_tracker import get_cost_tracker

        tracker = get_cost_tracker()
        settings = ThegentSettings()
        # Use daily budget (litellm) or derive from MTD
        budget = getattr(settings, "litellm_cost_budget", None)
        if budget is None or budget <= 0:
            budget = float(getattr(settings, "cost_budget_mtd", 100.0)) / 30.0  # Daily proxy
        if budget <= 0:
            return 1.0
        remaining = tracker.get_budget_remaining()
        if remaining is None:
            return 1.0
        ratio = remaining / max(budget, 0.01)
        return min(5.0, 1.0 / max(ratio, _SHADOW_EPSILON))  # Cap at 5x
    except Exception:
        return 1.0


def _offers_from_catalog(
    complexity_tier: str = "moderate",
    needs_tools: bool = False,
    min_quality: float = 0.0,
    max_cost_weight: float = 2.0,
    use_shadow_pricing: bool = True,
    role: str | None = None,
    output_tokens_multiplier: float = 1.0,
) -> list[Offer]:
    """Build offers from catalog, filtered by hard constraints. Applies shadow pricing when enabled.
    Degraded mode (85% budget burn) caps max_cost_weight to exclude premium offers."""
    from thegent.models.catalog import ModelCatalog

    catalog = ModelCatalog.to_contract_view(use_scraped=True)
    routes_map = catalog.get("routes", {}) or {}
    offers: list[Offer] = []
    shadow = _get_shadow_multiplier() if use_shadow_pricing else 1.0

    # Degraded mode: cap cost to prefer cheap offers
    effective_max_cost = max_cost_weight
    if _is_degraded_mode():
        effective_max_cost = min(max_cost_weight, 1.0)
        _log.debug("Degraded mode: capping max_cost_weight to %.1f", effective_max_cost)

    for model_id, route_list in routes_map.items():  # type: ignore[union-attr]
        if not isinstance(route_list, list):
            continue
        for r in route_list:
            if not isinstance(r, dict):
                continue
            provider = r.get("provider") or ""
            model_alias = r.get("model_alias") or model_id
            cost_weight = float(r.get("cost_weight", 1.0))
            effective_cost = cost_weight * shadow
            # Quality: use role benchmark weights when available
            rcfg = _get_role(role)
            if rcfg and rcfg.benchmark_weights:
                try:
                    from thegent.models.quality_values import get_model_quality_for_role

                    quality = get_model_quality_for_role(model_alias, rcfg.benchmark_weights)
                except Exception:
                    quality = _get_quality(model_alias)
            else:
                quality = _get_quality(model_alias)

            # Hard constraints (use effective_cost for budget pressure)
            if effective_cost > effective_max_cost:
                continue
            if quality < min_quality:
                continue

            # Speed index with conciseness: base (cost_weight proxy) * output_tokens_multiplier.
            # Higher multiplier = more verbose role = worse speed. Lower = better.
            speed_score = cost_weight * output_tokens_multiplier

            offers.append(
                Offer(
                    provider=provider,
                    model_alias=model_alias,
                    cost_weight=cost_weight,
                    quality=quality,
                    speed_score=speed_score,
                    effective_cost=effective_cost,
                )
            )

    return offers


def _is_dominated(a: Offer, b: Offer) -> bool:
    """True if b dominates a: b is better or equal on all axes, strictly better on one."""
    cost_a = a.effective_cost or a.cost_weight
    cost_b = b.effective_cost or b.cost_weight
    speed_ok = b.speed_score <= a.speed_score
    cost_ok = cost_b <= cost_a
    quality_ok = b.quality >= a.quality
    strictly_better = b.speed_score < a.speed_score or cost_b < cost_a or b.quality > a.quality
    return speed_ok and cost_ok and quality_ok and strictly_better


def _pareto_frontier(offers: list[Offer]) -> list[Offer]:
    """Return non-dominated offers (Pareto frontier)."""
    if not offers:
        return []
    frontier: list[Offer] = []
    for o in offers:
        dominated = False
        for other in offers:
            if other is o:
                continue
            if _is_dominated(o, other):
                dominated = True
                break
        if not dominated:
            frontier.append(o)
    return frontier


def _lexicographic_select(
    offers: list[Offer],
    order: tuple[str, ...] = ("quality", "cost", "speed"),
    epsilon: dict[str, float] | None = None,
) -> Offer | None:
    """Select single offer by lexicographic tie-break. Default: quality → cost → speed."""
    if not offers:
        return None
    eps = epsilon or {"quality": 0.02, "cost": 0.10, "speed": 0.15}

    current = list(offers)
    for axis in order:
        if not current:
            return None
        if axis == "quality":
            best = max(o.quality for o in current)
            threshold = best - eps.get("quality", 0.02)
            current = [o for o in current if o.quality >= threshold]
        elif axis == "cost":
            best = min((o.effective_cost or o.cost_weight) for o in current)
            threshold = best * (1 + eps.get("cost", 0.10))
            current = [o for o in current if (o.effective_cost or o.cost_weight) <= threshold]
        elif axis == "speed":
            best = min(o.speed_score for o in current)
            threshold = best * (1 + eps.get("speed", 0.15))
            current = [o for o in current if o.speed_score <= threshold]

    return current[0] if current else None


def _resolve_role_params(
    role: str | None,
    complexity_tier: str,
    min_quality: float,
    opt_order: tuple[str, ...],
) -> tuple[float, tuple[str, ...], float]:
    """Resolve min_quality, opt_order, output_tokens_multiplier from role + tier."""
    tier_min = {"simple": 0.5, "moderate": 0.6, "complex": 0.75}.get(complexity_tier, 0.6)
    effective_min = max(min_quality, tier_min)
    order = opt_order
    mult = 1.0
    r = _get_role(role)
    if r:
        effective_min = max(effective_min, r.min_quality)
        order = r.soft_order
        mult = r.output_tokens_multiplier
    return effective_min, order, mult


def select_offer(
    complexity_tier: str = "moderate",
    min_quality: float = 0.0,
    max_cost_weight: float = 2.0,
    opt_order: tuple[str, ...] = ("quality", "cost", "speed"),
    role: str | None = None,
) -> tuple[str, str] | None:
    """
    Select (provider, model_alias) via Pareto + lexicographic.

    Args:
        complexity_tier: simple | moderate | complex (adjusts min_quality)
        min_quality: Minimum quality floor (0-1)
        max_cost_weight: Maximum cost weight
        opt_order: Lexicographic order (overridden by role if set)
        role: Role name (fast_chat, doc_writer, code_complex, high_accuracy) for role-specific params

    Returns:
        (provider, model_alias) or None
    """
    trace = select_offer_with_trace(
        complexity_tier=complexity_tier,
        min_quality=min_quality,
        max_cost_weight=max_cost_weight,
        opt_order=opt_order,
        role=role,
    )
    if trace:
        return (trace.provider, trace.model_alias)
    return None


def select_offer_with_trace(
    complexity_tier: str = "moderate",
    min_quality: float = 0.0,
    max_cost_weight: float = 2.0,
    opt_order: tuple[str, ...] = ("quality", "cost", "speed"),
    role: str | None = None,
) -> RouteTrace | None:
    """
    Select offer with full route trace (why offer won). Per Helios spec.
    Uses role from roles.schema.yaml when provided (min_quality, soft_order, output_tokens_multiplier).
    """
    effective_min, order, output_mult = _resolve_role_params(role, complexity_tier, min_quality, opt_order)
    shadow = _get_shadow_multiplier()

    offers = _offers_from_catalog(
        complexity_tier=complexity_tier,
        min_quality=effective_min,
        max_cost_weight=max_cost_weight,
        role=role,
        output_tokens_multiplier=output_mult,
    )
    if not offers:
        _log.warning("No offers passed hard constraints")
        return None

    frontier = _pareto_frontier(offers)
    if not frontier:
        frontier = offers

    selected = _lexicographic_select(frontier, order=order)
    if not selected:
        return None

    # Build fallback chain (top k from frontier excluding selected)
    def key(o: Offer) -> tuple[float, float, float]:
        return (-o.quality, o.cost_weight, o.speed_score)

    sorted_f = sorted(frontier, key=key)
    fallbacks = [
        (o.provider, o.model_alias)
        for o in sorted_f
        if (o.provider, o.model_alias) != (selected.provider, selected.model_alias)
    ][:5]
    pareto_ids = [f"{o.provider}:{o.model_alias}" for o in frontier]
    offer_id = f"{selected.provider}:{selected.model_alias}"

    return RouteTrace(
        selected_offer_id=offer_id,
        provider=selected.provider,
        model_alias=selected.model_alias,
        pareto_set=pareto_ids,
        fallback_chain=fallbacks,
        scores={
            "quality": selected.quality,
            "cost_weight": selected.cost_weight,
            "speed_score": selected.speed_score,
            "effective_cost": selected.effective_cost or selected.cost_weight,
        },
        shadow_multiplier=shadow,
        degraded_mode=_is_degraded_mode(),
        role=role,
    )


def select_offer_with_fallbacks(
    complexity_tier: str = "moderate",
    min_quality: float = 0.0,
    max_cost_weight: float = 2.0,
    k: int = 3,
    role: str | None = None,
) -> list[tuple[str, str]]:
    """
    Select primary + fallback chain (top k from Pareto frontier by lexicographic).
    Uses role from roles.schema.yaml when provided.

    Returns:
        [(provider, model_alias), ...] primary first
    """
    effective_min, _order, output_mult = _resolve_role_params(
        role, complexity_tier, min_quality, ("quality", "cost", "speed")
    )

    offers = _offers_from_catalog(
        complexity_tier=complexity_tier,
        min_quality=effective_min,
        max_cost_weight=max_cost_weight,
        role=role,
        output_tokens_multiplier=output_mult,
    )
    if not offers:
        return []

    frontier = _pareto_frontier(offers)
    if not frontier:
        frontier = offers

    # Sort by role's soft_order (default: quality desc, cost asc, speed asc)
    def key(o: Offer) -> tuple[float, float, float]:
        c = o.effective_cost or o.cost_weight
        # Map order to sort key: higher quality first, lower cost first, lower speed first
        q, co, s = -o.quality, c, o.speed_score
        return (q, co, s)

    sorted_frontier = sorted(frontier, key=key)
    return [(o.provider, o.model_alias) for o in sorted_frontier[:k]]
