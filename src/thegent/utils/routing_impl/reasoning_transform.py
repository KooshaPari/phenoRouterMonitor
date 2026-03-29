"""GW-40: Unified reasoning interface.

Normalizes {effort: "high"/"medium"/"low"} to provider-specific reasoning params:
- Anthropic: extended_thinking with budget_tokens
- OpenAI: reasoning_effort (high/medium/low)
- Google Gemini: thinking_config with thinking_budget

# @trace FR-REQEXT-040
"""

from __future__ import annotations

from enum import Enum


class ReasoningEffort(str, Enum):
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


THINKING_BUDGET: dict[ReasoningEffort, int] = {
    ReasoningEffort.HIGH: 10000,
    ReasoningEffort.MEDIUM: 5000,
    ReasoningEffort.LOW: 1000,
}


def extract_reasoning_effort(body: dict) -> ReasoningEffort | None:
    """Extract reasoning effort from request body.

    Reads body["reasoning"]["effort"], body["reasoning_effort"], or body["variant"].
    Returns None if not present or invalid.
    """
    # Try nested form first: {"reasoning": {"effort": "high"}}
    reasoning = body.get("reasoning")
    if isinstance(reasoning, dict):
        raw = reasoning.get("effort")
        if raw is not None:
            try:
                return ReasoningEffort(raw)
            except ValueError:
                return None

    # Try flat form: {"reasoning_effort": "medium"}
    flat = body.get("reasoning_effort")
    if flat is not None:
        try:
            return ReasoningEffort(flat)
        except ValueError:
            return None

    # Try variant form (used by OpenWork/Cursor for codex models): {"variant": "high"}
    variant = body.get("variant")
    if variant is not None:
        try:
            return ReasoningEffort(variant)
        except ValueError:
            return None

    return None


def apply_anthropic_reasoning(body: dict, effort: ReasoningEffort) -> dict:
    """Add extended_thinking to Anthropic request body.

    Sets body["thinking"] = {"type": "enabled", "budget_tokens": THINKING_BUDGET[effort]}.
    Removes "reasoning" key if present.
    Returns modified copy of body (does not mutate in place).
    """
    result = dict(body)
    result["thinking"] = {
        "type": "enabled",
        "budget_tokens": THINKING_BUDGET[effort],
    }
    result.pop("reasoning", None)
    return result


def apply_openai_reasoning(body: dict, effort: ReasoningEffort) -> dict:
    """Add reasoning_effort to OpenAI request body.

    Sets body["reasoning_effort"] = effort.value.
    Removes "reasoning" and "variant" keys if present.
    Returns modified copy of body.
    """
    result = dict(body)
    result["reasoning_effort"] = effort.value
    result.pop("reasoning", None)
    result.pop("variant", None)
    return result


def apply_gemini_reasoning(body: dict, effort: ReasoningEffort) -> dict:
    """Add thinking_config to Google Gemini request body.

    Sets body["thinking_config"] = {"thinking_budget": THINKING_BUDGET[effort]}.
    Removes "reasoning" key if present.
    Returns modified copy of body.
    """
    result = dict(body)
    result["thinking_config"] = {"thinking_budget": THINKING_BUDGET[effort]}
    result.pop("reasoning", None)
    return result


def apply_reasoning_for_provider(body: dict, provider: str) -> dict:
    """Apply provider-specific reasoning transform for body["reasoning"]["effort"].

    If no reasoning effort in body, returns body unchanged.
    Dispatches to apply_anthropic_reasoning / apply_openai_reasoning / apply_gemini_reasoning
    based on provider prefix matching ("anthropic", "openai", "google", "gemini").
    For "codex" provider, treats as OpenAI-compatible (uses reasoning_effort).
    Unknown providers: strips "reasoning" key and returns.
    """
    effort = extract_reasoning_effort(body)
    if effort is None:
        return body

    provider_lower = provider.lower()

    if provider_lower.startswith("anthropic"):
        return apply_anthropic_reasoning(body, effort)
    if provider_lower.startswith("openai") or provider_lower.startswith("codex"):
        # Codex uses OpenAI-compatible API, so apply OpenAI reasoning transform
        return apply_openai_reasoning(body, effort)
    if provider_lower.startswith("google") or provider_lower.startswith("gemini"):
        return apply_gemini_reasoning(body, effort)

    # Unknown provider: strip reasoning key and return copy
    result = dict(body)
    result.pop("reasoning", None)
    return result
