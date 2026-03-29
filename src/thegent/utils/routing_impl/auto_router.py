"""Auto router: uses headless Gemini Flash to classify task complexity, then Pareto routing.

Flow:
1. Classify prompt via Gemini Flash (simple | moderate | complex)
2. Select (provider, model) from Pareto frontier based on complexity
3. Return resolved agent + model for run_impl
"""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass

from thegent.utils.routing_impl.pareto_router import (
    RouteTrace,
    select_offer_with_trace,
)

_log = logging.getLogger(__name__)

# System prompt for robust task/complexity classification
AUTO_ROUTER_SYSTEM_PROMPT = """You are a routing classifier. Your ONLY job is to classify the user's task/prompt into exactly one complexity tier.

Output MUST be valid JSON with this exact structure (no other text):
{"complexity": "simple" | "moderate" | "complex", "reason": "brief one-line reason"}

Rules:
- simple: trivial edits, one-liner fixes, format changes, simple refactors, quick answers
- moderate: multi-file changes, feature additions, debugging, documentation, tests
- complex: architecture changes, security-critical, multi-step reasoning, large refactors, formal verification

Be conservative: when uncertain, use "moderate". Use "complex" only for clearly demanding tasks."""

AUTO_ROUTER_USER_TEMPLATE = """Classify this task:

---
{prompt_preview}
---

Respond with ONLY the JSON object, no markdown or explanation."""

# Max chars of prompt to send to classifier (keep cheap)
CLASSIFIER_PROMPT_PREVIEW_LEN = 1200


def _call_classifier(prompt_preview: str, model: str = "gemini-3-flash") -> dict | None:
    """Call Gemini Flash headless to classify. Returns {"complexity": "...", "reason": "..."} or None."""
    try:
        from litellm import completion
    except ImportError:
        _log.warning("litellm not installed; auto router classifier unavailable")
        return None

    # Prefer zen/antigravity (CLIProxy) for gemini-3-flash; fallback to google
    model_candidates = [
        "zen/gemini-3-flash",
        "antigravity/gemini-3-flash",
        "google/gemini-2.0-flash",
        "gemini/gemini-2.0-flash",
    ]

    user_content = AUTO_ROUTER_USER_TEMPLATE.format(prompt_preview=prompt_preview)
    messages = [
        {"role": "system", "content": AUTO_ROUTER_SYSTEM_PROMPT},
        {"role": "user", "content": user_content},
    ]

    last_err: Exception | None = None
    for model_str in model_candidates:
        try:  # noqa: PERF203 -- fallback chain pattern, trying multiple classifier models
            response = completion(
                model=model_str,
                messages=messages,
                max_tokens=128,
                timeout=15,
            )
            content = (response.choices[0].message.content or "").strip()  # type: ignore[union-attr]
            # Strip markdown code blocks if present
            if content.startswith("```"):
                content = content.split("```")[1]
                content = content.removeprefix("json")
                content = content.strip()
            data = json.loads(content)
            if isinstance(data, dict) and "complexity" in data:
                return data
        except Exception as e:  # noqa: PERF203 - intentional per-item error handling
            last_err = e
            _log.debug("Classifier model %s failed: %s", model_str, e)
            continue

    _log.warning("Auto router classifier failed: %s", last_err)
    return None


@dataclass
class AutoRouteResult:
    """Result from auto router."""

    agent: str
    model: str
    complexity: str
    reason: str | None = None
    fallback_chain: list[tuple[str, str]] | None = None
    route_trace: RouteTrace | None = None


# Map complexity tier to role for role-based routing (Helios spec)
COMPLEXITY_TO_ROLE: dict[str, str | None] = {
    "simple": "fast_chat",
    "moderate": "default",
    "complex": "code_complex",
}


def auto_route(
    prompt: str,
    classifier_model: str = "gemini-3-flash",
    use_classifier: bool = True,
    min_quality: float = 0.0,
    max_cost_weight: float = 2.0,
    role: str | None = None,
) -> AutoRouteResult | None:
    """
    Auto-route: classify prompt, then Pareto select (agent, model).

    Args:
        prompt: User prompt (preview used for classification)
        classifier_model: Model for classification (headless)
        use_classifier: If False, assume "moderate" complexity
        min_quality: Minimum quality floor
        max_cost_weight: Max cost weight
        role: Override role (fast_chat, doc_writer, code_complex, high_accuracy). If None, inferred from complexity.

    Returns:
        AutoRouteResult or None if routing fails
    """
    complexity = "moderate"
    reason: str | None = None

    if use_classifier:
        preview = prompt[:CLASSIFIER_PROMPT_PREVIEW_LEN]
        if len(prompt) > CLASSIFIER_PROMPT_PREVIEW_LEN:
            preview += "\n..."
        result = _call_classifier(preview, model=classifier_model)
        if result:
            complexity = str(result.get("complexity", "moderate")).lower()
            if complexity not in ("simple", "moderate", "complex"):
                complexity = "moderate"
            reason = str(result.get("reason", "")) or None

    # Infer role from complexity when not explicitly set
    effective_role = role or COMPLEXITY_TO_ROLE.get(complexity)

    trace = select_offer_with_trace(
        complexity_tier=complexity,
        min_quality=min_quality,
        max_cost_weight=max_cost_weight,
        role=effective_role,
    )
    if not trace:
        return None

    fallbacks = trace.fallback_chain[:2] if trace.fallback_chain else None

    return AutoRouteResult(
        agent=trace.provider,
        model=trace.model_alias,
        complexity=complexity,
        reason=reason,
        fallback_chain=fallbacks,
        route_trace=trace,
    )
