"""GW-14: OpenRouter-style model suffix routing.

Parses suffixes appended to model names:
  :nitro      — fastest inference (maps to performance tier)
  :floor      — cheapest inference (maps to economy tier)
  :free       — free tier models
  :thinking   — enable extended thinking/reasoning
  :online     — enable web search plugin
  :extended   — extended context window

Supports multiple suffixes: "model:thinking:online"

# @trace FR-ROUTE-014
"""

from __future__ import annotations

import copy
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class ModelSuffix(str, Enum):
    NITRO = "nitro"  # fastest/highest priority inference
    FLOOR = "floor"  # cheapest/lowest priority inference
    FREE = "free"  # free tier models only
    THINKING = "thinking"  # extended thinking mode
    ONLINE = "online"  # web search enabled
    EXTENDED = "extended"  # extended context window


SUFFIX_ROUTING_HINTS: dict[ModelSuffix, dict[str, Any]] = {
    ModelSuffix.NITRO: {"priority": "high", "provider_sort": "throughput"},
    ModelSuffix.FLOOR: {"priority": "low", "provider_sort": "price"},
    ModelSuffix.FREE: {"provider_only_free": True, "provider_sort": "price"},
    ModelSuffix.THINKING: {"reasoning_effort": "high"},
    ModelSuffix.ONLINE: {"plugins": [{"id": "web"}]},
    ModelSuffix.EXTENDED: {"context_length_extended": True},
}

_KNOWN_SUFFIXES: frozenset[str] = frozenset(s.value for s in ModelSuffix)


# ---------------------------------------------------------------------------
# ParsedModel dataclass (multi-suffix support)
# ---------------------------------------------------------------------------


@dataclass
class ParsedModel:
    """Result of parsing a model string with optional suffix(es).

    Attributes:
        base_model: Model name without suffix(es).
        suffixes: Ordered list of parsed ModelSuffix values.
        raw: Original input string, preserved unchanged.
    """

    base_model: str
    suffixes: list[ModelSuffix] = field(default_factory=list)
    raw: str = ""

    @property
    def has_suffix(self) -> bool:
        """Return True when at least one suffix is present."""
        return len(self.suffixes) > 0

    @property
    def is_thinking(self) -> bool:
        """Return True when THINKING suffix is present."""
        return ModelSuffix.THINKING in self.suffixes

    @property
    def is_free_tier(self) -> bool:
        """Return True when FREE suffix is present."""
        return ModelSuffix.FREE in self.suffixes

    @property
    def is_performance_tier(self) -> bool:
        """Return True when NITRO suffix is present."""
        return ModelSuffix.NITRO in self.suffixes

    @property
    def is_economy_tier(self) -> bool:
        """Return True when FLOOR suffix is present."""
        return ModelSuffix.FLOOR in self.suffixes

    @property
    def needs_web_search(self) -> bool:
        """Return True when ONLINE suffix is present."""
        return ModelSuffix.ONLINE in self.suffixes


def parse_model_suffix(model: str) -> tuple[str, ModelSuffix | None]:
    """Parse a model string, extracting any trailing :suffix.

    Examples:
        "gpt-4o:nitro" -> ("gpt-4o", ModelSuffix.NITRO)
        "claude-opus-4.5:thinking" -> ("claude-opus-4.5", ModelSuffix.THINKING)
        "gpt-4o" -> ("gpt-4o", None)
        "openai/gpt-4o:free" -> ("openai/gpt-4o", ModelSuffix.FREE)
        "unknown:xyz" -> ("unknown:xyz", None)  # unknown suffix -> unchanged

    Returns (base_model, suffix_or_none).
    """
    colon_idx = model.rfind(":")
    if colon_idx == -1:
        return model, None

    candidate = model[colon_idx + 1 :]
    if candidate not in _KNOWN_SUFFIXES:
        return model, None

    base = model[:colon_idx]
    return base, ModelSuffix(candidate)


def parse_model_suffixes(model: str) -> ParsedModel:
    """Parse a model string extracting all :suffix tokens.

    Supports multiple suffixes: "model:thinking:online" yields both THINKING
    and ONLINE. Unknown suffix tokens are silently ignored (not treated as
    part of the base model name). The first segment before any colon sequence
    is always treated as the base model.

    Examples:
        "gpt-4o:nitro" -> ParsedModel(base_model="gpt-4o", suffixes=[NITRO], raw="gpt-4o:nitro")
        "anthropic/claude-sonnet-4-5:thinking:online" ->
            ParsedModel(base_model="anthropic/claude-sonnet-4-5",
                        suffixes=[THINKING, ONLINE], raw="...")
        "gpt-4o" -> ParsedModel(base_model="gpt-4o", suffixes=[], raw="gpt-4o")
        "model:unknown" -> ParsedModel(base_model="model", suffixes=[], raw="model:unknown")

    Args:
        model: Raw model string, optionally with colon-separated suffixes.

    Returns:
        ParsedModel with base_model, parsed suffixes, and raw preserved.
    """
    parts = model.split(":")
    base_model = parts[0]
    suffixes: list[ModelSuffix] = []
    for part in parts[1:]:
        if part in _KNOWN_SUFFIXES:
            suffixes.append(ModelSuffix(part))
        # Unknown tokens are silently ignored per spec
    return ParsedModel(base_model=base_model, suffixes=suffixes, raw=model)


def apply_suffix_to_request(body: dict[str, Any], parsed: ParsedModel) -> dict[str, Any]:
    """Return a modified copy of body reflecting the parsed suffixes.

    Applies the following transformations based on suffixes present:
      - THINKING: adds ``{"reasoning": {"effort": "high"}}`` if not already set.
      - ONLINE: adds/merges ``{"plugins": [{"id": "web", "max_results": 5}]}``.
      - NITRO: adds ``{"tg_tier": "performance"}`` to body metadata.
      - FLOOR: adds ``{"tg_tier": "economy"}`` to body metadata.
      - FREE: adds ``{"tg_tier": "free"}`` to body metadata.

    Does NOT mutate the original body dict.

    Args:
        body: Original request body dict.
        parsed: ParsedModel containing the suffixes to apply.

    Returns:
        A new dict with suffix-driven fields merged in.
    """
    result = copy.deepcopy(body)

    if ModelSuffix.THINKING in parsed.suffixes:
        if "reasoning" not in result:
            result["reasoning"] = {"effort": "high"}

    if ModelSuffix.ONLINE in parsed.suffixes:
        web_plugin = {"id": "web", "max_results": 5}
        existing_plugins: list[dict[str, Any]] = result.get("plugins", [])
        # Merge: add if web plugin not already listed
        has_web = any(p.get("id") == "web" for p in existing_plugins)
        if not has_web:
            result["plugins"] = [*existing_plugins, web_plugin]

    if ModelSuffix.NITRO in parsed.suffixes:
        result["tg_tier"] = "performance"
    elif ModelSuffix.FLOOR in parsed.suffixes:
        result["tg_tier"] = "economy"
    elif ModelSuffix.FREE in parsed.suffixes:
        result["tg_tier"] = "free"

    return result


def resolve_suffix_model(
    parsed: ParsedModel,
    model_map: dict[str, str] | None = None,
) -> str:
    """Resolve a ParsedModel to a concrete model name.

    For NITRO, FLOOR, and FREE suffixes: if ``model_map`` contains a key
    ``"{base_model}:{suffix}"`` the mapped value is returned. Otherwise the
    base_model is returned and tier selection is deferred to other routing
    layers.

    Args:
        parsed: ParsedModel from parse_model_suffixes().
        model_map: Optional mapping of "base:suffix" -> concrete model name.

    Returns:
        Resolved model name string.
    """
    tier_suffixes = {ModelSuffix.NITRO, ModelSuffix.FLOOR, ModelSuffix.FREE}
    if model_map:
        for suffix in parsed.suffixes:
            if suffix in tier_suffixes:
                key = f"{parsed.base_model}:{suffix.value}"
                if key in model_map:
                    return model_map[key]
    return parsed.base_model


def get_routing_hints(suffix: ModelSuffix | None) -> dict[str, Any]:
    """Return routing hint dict for the given suffix, or {} for None."""
    if suffix is None:
        return {}
    return SUFFIX_ROUTING_HINTS.get(suffix, {})


def resolve_model_and_hints(model: str) -> tuple[str, dict[str, Any]]:
    """Parse model suffix and return (base_model, routing_hints).

    Convenience wrapper combining parse_model_suffix + get_routing_hints.
    """
    base, suffix = parse_model_suffix(model)
    return base, get_routing_hints(suffix)
