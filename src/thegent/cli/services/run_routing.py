"""Routing helpers for run execution.

Extracted from run_execution_core_helpers.py for maintainability.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from thegent.config import ThegentSettings

logger = logging.getLogger(__name__)


def build_route_candidates(
    model: str | None,
    provider: str | None,
    agent: str | None,
    settings: "ThegentSettings",
) -> list[dict[str, Any]]:
    """Build list of route candidates for Pareto routing.

    Args:
        model: Requested model
        provider: Requested provider
        agent: Requested agent
        settings: Thegent settings

    Returns:
        List of route candidate dicts
    """
    from thegent.models.catalog import _get_catalog

    candidates = []
    catalog = _get_catalog()

    if model:
        # Find providers that support this model
        for entry in catalog.get(model, []):
            if provider and entry.provider != provider:
                continue
            candidates.append(
                {
                    "provider": entry.provider,
                    "model": entry.model_alias or model,
                    "quality": 0.8,
                    "cost": entry.cost_weight,
                    "latency": 500,
                }
            )
    elif agent:
        # Map agent to preferred provider/model
        agent_map = {
            "claude": {"provider": "anthropic", "model": settings.default_claude_model},
            "codex": {"provider": "openai", "model": settings.default_codex_model},
            "gemini": {"provider": "google", "model": settings.default_gemini_model},
            "cursor": {"provider": "cursor", "model": settings.default_cursor_model},
        }
        if agent in agent_map:
            candidates.append(agent_map[agent])

    return candidates


def select_pareto_route(
    candidates: list[dict[str, Any]],
    quality_weight: float = 0.4,
    cost_weight: float = 0.3,
    latency_weight: float = 0.3,
) -> dict[str, Any] | None:
    """Select best route using Pareto optimization.

    Args:
        candidates: List of route candidates
        quality_weight: Weight for quality score
        cost_weight: Weight for cost (inverse)
        latency_weight: Weight for latency (inverse)

    Returns:
        Best candidate or None if no candidates
    """
    if not candidates:
        return None

    if len(candidates) == 1:
        return candidates[0]

    # Normalize and score
    max_cost = max(c.get("cost", 0) for c in candidates) or 1
    max_latency = max(c.get("latency", 0) for c in candidates) or 1

    best_score = -1
    best_candidate = candidates[0]

    for c in candidates:
        quality = c.get("quality", 0.5)
        cost_norm = 1 - (c.get("cost", 0) / max_cost)
        latency_norm = 1 - (c.get("latency", 0) / max_latency)

        score = quality * quality_weight + cost_norm * cost_weight + latency_norm * latency_weight

        if score > best_score:
            best_score = score
            best_candidate = c

    return best_candidate


def classify_auto_route(
    prompt: str,
    model: str | None,
    agent: str | None,
) -> tuple[str, str]:
    """Classify and select route for auto-routing.

    Args:
        prompt: User prompt
        model: Requested model
        agent: Requested agent

    Returns:
        Tuple of (provider, model)
    """
    # Simple keyword-based classification
    prompt_lower = prompt.lower()

    # Check for code-heavy tasks
    code_keywords = ["implement", "code", "function", "class", "refactor", "debug"]
    if any(kw in prompt_lower for kw in code_keywords):
        return "anthropic", "claude-sonnet-4"

    # Check for reasoning tasks
    reasoning_keywords = ["analyze", "explain", "why", "reasoning", "compare"]
    if any(kw in prompt_lower for kw in reasoning_keywords):
        return "anthropic", "claude-opus-4"

    # Check for quick tasks
    quick_keywords = ["fix", "quick", "simple", "short"]
    if any(kw in prompt_lower for kw in quick_keywords):
        return "anthropic", "claude-haiku-4"

    # Default to balanced model
    return "anthropic", "claude-sonnet-4"


__all__ = [
    "build_route_candidates",
    "select_pareto_route",
    "classify_auto_route",
]
