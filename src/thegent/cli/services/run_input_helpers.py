"""Run input/output helper services extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

import contextlib
import orjson as json
import math
from pathlib import Path
from typing import TYPE_CHECKING, Any
from urllib.parse import urlparse

if TYPE_CHECKING:
    from thegent.agents.base import RunResult

_CONTEXT_RATIO_CONSISTENCY_TOLERANCE = 1e-3


def normalize_image_paths(image_paths: list[str] | None, *, supported_image_suffixes: set[str]) -> list[str]:
    """Validate and normalize WL-114 image inputs."""
    if not image_paths:
        return []

    normalized: list[str] = []
    seen: set[str] = set()
    for raw in image_paths:
        if not isinstance(raw, str):
            raise ValueError("Image input values must be strings.")
        candidate = (raw or "").strip()
        if not candidate:
            raise ValueError("Empty image input is not allowed.")

        parsed = urlparse(candidate)
        if parsed.scheme:
            if parsed.scheme.lower() != "https":
                raise ValueError(f"Image URL must use HTTPS: {candidate}")
            if not parsed.netloc:
                raise ValueError(f"Image URL must include a hostname: {candidate}")
            suffix = Path(parsed.path).suffix.lower()
            if suffix not in supported_image_suffixes:
                allowed = ", ".join(sorted(supported_image_suffixes))
                raise ValueError(f"Image URL must end with a supported extension ({allowed}): {candidate}")
            if candidate in seen:
                continue
            seen.add(candidate)
            normalized.append(candidate)
            continue

        path = Path(candidate).expanduser()
        suffix = path.suffix.lower()
        if suffix not in supported_image_suffixes:
            allowed = ", ".join(sorted(supported_image_suffixes))
            raise ValueError(f"Image file must use a supported extension ({allowed}): {candidate}")
        if not path.exists():
            raise ValueError(f"Image file does not exist: {candidate}")
        if not path.is_file():
            raise ValueError(f"Image path must point to a file: {candidate}")
        resolved = str(path.resolve())
        if resolved in seen:
            continue
        seen.add(resolved)
        normalized.append(resolved)

    return normalized


def append_context_usage(payload: dict[str, Any], result: RunResult) -> None:
    """WL-108/WL-103: add context usage details and top-level usage ratio."""
    context_usage = build_context_usage_payload(
        used=result.context_tokens_used,
        max_tokens=result.context_window_max,
        ratio=result.context_usage_ratio,
    )
    if context_usage is None:
        normalized_ratio = _normalize_context_usage_ratio(result.context_usage_ratio)
        if normalized_ratio is not None:
            payload["context_usage_ratio"] = normalized_ratio
        return
    payload["context_usage"] = context_usage
    payload["context_usage_ratio"] = float(context_usage["ratio"])


def _normalize_context_usage_ratio(value: Any) -> float | None:
    if value is None or isinstance(value, bool):
        return None
    with contextlib.suppress(TypeError, ValueError, OverflowError):
        ratio = float(value)
        if math.isfinite(ratio) and 0.0 <= ratio <= 1.0:
            return round(ratio, 4)
    return None


def build_context_usage_payload(
    *,
    used: int | None,
    max_tokens: int | None,
    ratio: float | None,
) -> dict[str, Any] | None:
    """WL-108: shared context usage payload used by JSON and status output paths."""
    if (
        used is None
        or max_tokens is None
        or isinstance(used, bool)
        or isinstance(max_tokens, bool)
        or not isinstance(used, int)
        or not isinstance(max_tokens, int)
    ):
        return None
    if used < 0 or max_tokens <= 0 or used > max_tokens:
        return None

    from thegent.tui.widgets.statusbar import compute_context_usage_display

    computed_ratio = used / max_tokens if max_tokens > 0 else 0.0
    ratio_for_output = computed_ratio
    if ratio is not None and not isinstance(ratio, bool):
        with contextlib.suppress(TypeError, ValueError):
            candidate_ratio = float(ratio)
            if math.isfinite(candidate_ratio) and 0.0 <= candidate_ratio <= 1.0:
                if abs(candidate_ratio - computed_ratio) <= _CONTEXT_RATIO_CONSISTENCY_TOLERANCE:
                    ratio_for_output = candidate_ratio
    display, css_class = compute_context_usage_display(used, max_tokens)
    return {
        "used": used,
        "max": max_tokens,
        "ratio": round(float(ratio_for_output), 4),
        "display": display,
        "level": css_class.removeprefix("ctx-") if css_class else None,
    }


def resolve_grounding_sources_for_output(
    *,
    stdout: str,
    result_grounding_sources: list[str] | None,
) -> list[str]:
    """Resolve grounding source URLs from structured data first, text fallback second."""
    from thegent.utils.routing_impl.grounding import extract_grounding_sources, extract_grounding_sources_from_payload

    if result_grounding_sources:
        structured_sources = extract_grounding_sources_from_payload({"sources": result_grounding_sources})
        if structured_sources:
            return structured_sources

    if stdout.strip().startswith(("{", "[")):
        with contextlib.suppress(json.JSONDecodeError):
            structured_sources = extract_grounding_sources_from_payload(json.loads(stdout))
            if structured_sources:
                return structured_sources

    return extract_grounding_sources(stdout)


def model_supports_vision(model: str, *, model_indexes_path: Path) -> bool:
    """Return True when model metadata explicitly marks vision capability."""
    target = model.strip().lower()
    if not target:
        raise ValueError("Model name is required for vision capability checks.")

    catalog = json.loads(model_indexes_path.read_text(encoding="utf-8"))
    providers = catalog.get("providers")
    if not isinstance(providers, dict):
        raise ValueError("Invalid model index catalog: missing providers map.")

    for provider_payload in providers.values():
        if not isinstance(provider_payload, dict):
            continue
        models = provider_payload.get("models")
        if not isinstance(models, dict):
            continue
        for model_name, model_payload in models.items():
            if not isinstance(model_name, str) or model_name.lower() != target:
                continue
            if not isinstance(model_payload, dict):
                break
            modalities = model_payload.get("modalities")
            if isinstance(modalities, dict) and "vision" in modalities:
                return bool(modalities["vision"])
            if "vision" in model_payload:
                return bool(model_payload["vision"])
            return False

    raise ValueError(f"Vision capability metadata not found for model '{model}'.")


def validate_image_capability(
    *,
    agent: str,
    model: str | None,
    model_supports_vision_impl: Any,
) -> None:
    """Validate that agent/model support image inputs.

    # @trace WL-114
    """
    from thegent.agents.run_options import IMAGE_CAPABLE_AGENTS

    if agent not in IMAGE_CAPABLE_AGENTS:
        raise ValueError(f"--image is not supported for agent '{agent}'.")
    if model is None:
        return
    if not model_supports_vision_impl(model):
        raise ValueError(f"Model '{model}' does not advertise vision capability required by --image.")


__all__ = [
    "append_context_usage",
    "build_context_usage_payload",
    "model_supports_vision",
    "normalize_image_paths",
    "resolve_grounding_sources_for_output",
    "validate_image_capability",
]
