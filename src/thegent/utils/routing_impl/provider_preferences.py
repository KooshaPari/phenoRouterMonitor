"""GW-11: Provider routing preferences.

Implements OpenRouter-compatible ``provider`` object parsing and Vercel-
compatible ``providerOptions`` parsing for the thegent AI gateway.

The main entry points are:

* :func:`extract_provider_preferences` — reads the ``"provider"`` key from a
  request body and returns a :class:`ProviderPreferences` instance.
* :func:`extract_provider_options` — reads the ``"providerOptions"`` key and
  returns a :class:`ProviderOptions` instance.
* :func:`filter_models_by_preferences` — applies ``only``/``ignore``/``order``
  and quantization filters to a list of ``"provider/model"`` strings.
* :func:`to_openrouter_provider_body` — serializes
  :class:`ProviderPreferences` back to the OpenRouter camelCase wire format.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Literal


# ---------------------------------------------------------------------------
# PriceConstraint
# ---------------------------------------------------------------------------


@dataclass
class PriceConstraint:
    """Maximum acceptable price per 1 million tokens (USD).

    Attributes:
        prompt: Maximum price for prompt/input tokens, in USD per 1 M tokens.
            *None* means no upper bound.
        completion: Maximum price for completion/output tokens, in USD per
            1 M tokens.  *None* means no upper bound.
    """

    prompt: float | None = None
    completion: float | None = None


# ---------------------------------------------------------------------------
# ProviderPreferences
# ---------------------------------------------------------------------------


@dataclass
class ProviderPreferences:
    """OpenRouter-compatible provider routing preferences.

    Controls which providers are eligible for a request, in what order they
    are tried, and how the router selects among them.

    Attributes:
        order: Prioritized list of provider names.  Providers earlier in the
            list are preferred when multiple candidates are available.
        only: Whitelist — only providers in this list are considered.  An
            empty list means *no restriction* (all providers allowed).
        ignore: Blacklist — providers in this list are excluded.
        allow_fallbacks: When ``True`` the router may fall back to providers
            not in ``order`` if all preferred providers fail or are
            unavailable.
        data_collection: Whether providers may use request data for model
            training.  ``"deny"`` restricts to providers that opt out of
            data collection.
        quantizations: Restrict to providers that serve the model at one of
            the listed quantization levels (e.g. ``["fp16", "bf16"]``).  An
            empty list means *no restriction*.
        sort: Secondary sort criterion applied after filtering.
            ``"price"`` sorts cheapest-first; ``"throughput"`` sorts by
            tokens-per-second; ``"latency"`` sorts by time-to-first-token.
            *None* means no secondary sort (``order`` list is authoritative).
        max_price: Hard price ceiling.  Providers exceeding either bound are
            excluded from consideration.
    """

    order: list[str] = field(default_factory=list)
    only: list[str] = field(default_factory=list)
    ignore: list[str] = field(default_factory=list)
    allow_fallbacks: bool = True
    data_collection: Literal["allow", "deny"] = "allow"
    quantizations: list[str] = field(default_factory=list)
    sort: Literal["price", "throughput", "latency"] | None = None
    max_price: PriceConstraint = field(default_factory=PriceConstraint)


# ---------------------------------------------------------------------------
# ProviderOptions
# ---------------------------------------------------------------------------


@dataclass
class ProviderOptions:
    """Vercel AI SDK-compatible per-provider option bags.

    Allows callers to supply arbitrary, provider-specific configuration
    without polluting the top-level request body.

    Attributes:
        options: Mapping from provider name to an arbitrary dict of provider-
            specific options (e.g. ``{"openai": {"organization": "org-xyz"}}``).
    """

    options: dict[str, dict[str, Any]] = field(default_factory=dict)


# ---------------------------------------------------------------------------
# Extraction helpers
# ---------------------------------------------------------------------------


def extract_provider_preferences(
    body: dict[str, Any],
) -> ProviderPreferences | None:
    """Parse the ``"provider"`` object from a request body.

    Returns a populated :class:`ProviderPreferences` when the key is present,
    or *None* when the key is absent.

    Args:
        body: Parsed JSON request body.

    Returns:
        :class:`ProviderPreferences` or *None*.
    """
    raw = body.get("provider")
    if raw is None or not isinstance(raw, dict):
        return None

    max_price_raw = raw.get("max_price", {})
    max_price = PriceConstraint(
        prompt=_optional_float(max_price_raw.get("prompt")),
        completion=_optional_float(max_price_raw.get("completion")),
    )

    return ProviderPreferences(
        order=list(raw.get("order", [])),
        only=list(raw.get("only", [])),
        ignore=list(raw.get("ignore", [])),
        allow_fallbacks=bool(raw.get("allow_fallbacks", True)),
        data_collection=raw.get("data_collection", "allow"),
        quantizations=list(raw.get("quantizations", [])),
        sort=raw.get("sort"),
        max_price=max_price,
    )


def extract_provider_options(
    body: dict[str, Any],
) -> ProviderOptions | None:
    """Parse the ``"providerOptions"`` object from a request body.

    Returns a populated :class:`ProviderOptions` when the key is present and
    is a dict, or *None* otherwise.

    Args:
        body: Parsed JSON request body.

    Returns:
        :class:`ProviderOptions` or *None*.
    """
    raw = body.get("providerOptions")
    if raw is None or not isinstance(raw, dict):
        return None

    options: dict[str, dict[str, Any]] = {}
    for provider_name, opts in raw.items():
        if isinstance(opts, dict):
            options[str(provider_name)] = opts
    return ProviderOptions(options=options)


# ---------------------------------------------------------------------------
# Filtering
# ---------------------------------------------------------------------------


def filter_models_by_preferences(
    models: list[str],
    prefs: ProviderPreferences,
) -> list[str]:
    """Filter and sort a ``"provider/model"`` list by :class:`ProviderPreferences`.

    The filtering pipeline is applied in order:

    1. **Whitelist** (``prefs.only``): Keep only entries whose provider
       matches one of the ``only`` values.  Skipped when ``only`` is empty.
    2. **Blacklist** (``prefs.ignore``): Remove entries whose provider appears
       in ``ignore``.
    3. **Quantization filter** (``prefs.quantizations``): Keep entries whose
       model name contains one of the quantization tags as a substring.
       Skipped when ``quantizations`` is empty.
    4. **Ordering** (``prefs.order``): Move entries whose provider appears in
       ``order`` to the front, preserving their relative order within the
       priority group.  Entries not in ``order`` are appended after,
       preserving their original relative order.  When ``allow_fallbacks`` is
       ``False``, entries not in ``order`` are dropped entirely.

    The ``sort`` field and ``max_price`` constraints require external pricing
    data and are intentionally *not* applied here; callers that have pricing
    data should apply those constraints before or after this function.

    Args:
        models: List of ``"provider/model"`` strings (bare model names without
            a ``"/"`` separator are kept as-is and treated as having no
            provider).
        prefs: Routing preferences to apply.

    Returns:
        Filtered and ordered list of model strings.
    """
    result = list(models)

    # 1. Whitelist
    if prefs.only:
        only_set = set(prefs.only)
        result = [m for m in result if _provider_of(m) in only_set]

    # 2. Blacklist
    if prefs.ignore:
        ignore_set = set(prefs.ignore)
        result = [m for m in result if _provider_of(m) not in ignore_set]

    # 3. Quantization filter
    if prefs.quantizations:
        result = [m for m in result if any(q in m for q in prefs.quantizations)]

    # 4. Order / fallback
    if prefs.order:
        order_set = set(prefs.order)
        priority: list[str] = []
        fallback: list[str] = []
        for m in result:
            if _provider_of(m) in order_set:
                priority.append(m)
            else:
                fallback.append(m)

        # Sort priority group by position in prefs.order
        priority.sort(
            key=lambda m: prefs.order.index(_provider_of(m)) if _provider_of(m) in prefs.order else len(prefs.order)
        )

        if prefs.allow_fallbacks:
            result = priority + fallback
        else:
            result = priority

    return result


# ---------------------------------------------------------------------------
# Serialization
# ---------------------------------------------------------------------------


def to_openrouter_provider_body(prefs: ProviderPreferences) -> dict[str, Any]:
    """Serialize :class:`ProviderPreferences` to the OpenRouter wire format.

    OpenRouter uses camelCase keys in its ``"provider"`` object.  This
    function produces the exact dict that should be embedded under the
    ``"provider"`` key when forwarding a request to the OpenRouter API.

    Fields with default/empty values are included to ensure deterministic
    output regardless of whether OpenRouter treats missing keys as defaults.

    Args:
        prefs: Populated :class:`ProviderPreferences` instance.

    Returns:
        Dict ready for JSON serialization in the ``"provider"`` field.
    """
    body: dict[str, Any] = {
        "order": list(prefs.order),
        "only": list(prefs.only),
        "ignore": list(prefs.ignore),
        "allow_fallbacks": prefs.allow_fallbacks,
        "data_collection": prefs.data_collection,
        "quantizations": list(prefs.quantizations),
    }

    if prefs.sort is not None:
        body["sort"] = prefs.sort

    max_price: dict[str, Any] = {}
    if prefs.max_price.prompt is not None:
        max_price["prompt"] = prefs.max_price.prompt
    if prefs.max_price.completion is not None:
        max_price["completion"] = prefs.max_price.completion
    if max_price:
        body["max_price"] = max_price

    return body


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _provider_of(model_string: str) -> str:
    """Return the provider component of a ``"provider/model"`` string.

    When the string contains no ``"/"`` separator the whole string is
    returned as the provider identifier.
    """
    if "/" in model_string:
        return model_string.split("/", 1)[0]
    return model_string


def _optional_float(value: Any) -> float | None:
    """Convert *value* to ``float`` or return *None* if it is *None*."""
    if value is None:
        return None
    return float(value)
