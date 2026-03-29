"""GW-33: Per-request LLM cost calculation from model pricing tables.

Computes USD cost from prompt_tokens, completion_tokens, and model pricing.
Integrates with model_metadata for current pricing data.

# @trace FR-COST-033
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from typing import Any

_log = logging.getLogger(__name__)


@dataclass
class ModelPricing:
    """Prompt/completion pricing for a single model."""

    model: str
    prompt_usd_per_1m: float  # USD per 1M prompt tokens
    completion_usd_per_1m: float  # USD per 1M completion tokens
    currency: str = field(default="USD")


# Well-known model pricing used when model_metadata lookup lacks separate prompt/completion data.
FALLBACK_PRICING: dict[str, ModelPricing] = {
    "gpt-4o": ModelPricing("gpt-4o", prompt_usd_per_1m=2.50, completion_usd_per_1m=10.00),
    "gpt-4o-mini": ModelPricing("gpt-4o-mini", prompt_usd_per_1m=0.15, completion_usd_per_1m=0.60),
    "claude-opus-4-5": ModelPricing("claude-opus-4-5", prompt_usd_per_1m=15.00, completion_usd_per_1m=75.00),
    "claude-sonnet-4-5": ModelPricing("claude-sonnet-4-5", prompt_usd_per_1m=3.00, completion_usd_per_1m=15.00),
    "claude-haiku-4-5": ModelPricing("claude-haiku-4-5", prompt_usd_per_1m=0.80, completion_usd_per_1m=4.00),
    "gemini-1.5-flash": ModelPricing("gemini-1.5-flash", prompt_usd_per_1m=0.075, completion_usd_per_1m=0.30),
    "gemini-1.5-pro": ModelPricing("gemini-1.5-pro", prompt_usd_per_1m=1.25, completion_usd_per_1m=5.00),
}


def get_model_pricing(model: str) -> ModelPricing | None:
    """Look up pricing for a model.

    First tries model_metadata (thegent.routing.model_metadata.get_model_metadata).
    The metadata dict must have a 'pricing' key with 'prompt_usd_per_1m' and
    'completion_usd_per_1m' sub-keys for this path to succeed.

    Falls back to FALLBACK_PRICING. Returns None if completely unknown.
    """
    # Try model_metadata first — only succeeds if the metadata entry has a 'pricing' dict
    try:
        from thegent.utils.routing_impl.model_metadata import get_model_metadata, has_model_metadata

        if has_model_metadata(model):
            meta = get_model_metadata(model)
            if meta and isinstance(meta, dict):
                pricing_dict = meta.get("pricing")
                if isinstance(pricing_dict, dict):
                    prompt = pricing_dict.get("prompt_usd_per_1m")
                    completion = pricing_dict.get("completion_usd_per_1m")
                    if prompt is not None and completion is not None:
                        return ModelPricing(
                            model=model,
                            prompt_usd_per_1m=float(prompt),
                            completion_usd_per_1m=float(completion),
                        )
    except Exception:
        pass

    # Fall back to FALLBACK_PRICING — exact match first, then strip provider prefix
    if model in FALLBACK_PRICING:
        return FALLBACK_PRICING[model]

    # Try stripping provider prefix (e.g. "openai/gpt-4o" → "gpt-4o")
    if "/" in model:
        base = model.split("/", 1)[1]
        if base in FALLBACK_PRICING:
            return FALLBACK_PRICING[base]

    return None


def calculate_cost(
    model: str,
    prompt_tokens: int,
    completion_tokens: int,
) -> float:
    """Compute USD cost for a single LLM call.

    Returns 0.0 if pricing unknown (never raises).

    Formula:
        cost = (prompt_tokens / 1_000_000) * prompt_usd_per_1m
              + (completion_tokens / 1_000_000) * completion_usd_per_1m
    """
    pricing = get_model_pricing(model)
    if pricing is None:
        _log.debug("No pricing found for model=%s; cost set to 0.0", model)
        return 0.0
    return (prompt_tokens / 1_000_000) * pricing.prompt_usd_per_1m + (
        completion_tokens / 1_000_000
    ) * pricing.completion_usd_per_1m


def calculate_cost_from_response(response: Any) -> float:
    """Extract token counts from a response object and compute cost.

    Handles both dict responses (JSON) and object responses (LiteLLM ModelResponse).
    Returns 0.0 if token counts or pricing unavailable.
    """
    if isinstance(response, dict):
        usage = response.get("usage", {})
        if not usage:
            return 0.0
        model = response.get("model", "unknown")
        prompt_tokens = int(usage.get("prompt_tokens", usage.get("input_tokens", 0)))
        completion_tokens = int(usage.get("completion_tokens", usage.get("output_tokens", 0)))
    else:
        # Object format (ModelResponse or similar)
        usage = getattr(response, "usage", None)
        if usage is None:
            return 0.0
        model = getattr(response, "model", "unknown") or "unknown"
        prompt_tokens = int(getattr(usage, "prompt_tokens", 0) or 0)
        completion_tokens = int(getattr(usage, "completion_tokens", 0) or 0)

    return calculate_cost(model, prompt_tokens, completion_tokens)


def format_cost_header_value(cost_usd: float) -> str:
    """Format cost as a header value: 6 decimal places USD.

    e.g., 0.000125 → '0.000125'
    """
    return f"{cost_usd:.6f}"
