"""Run event output/detail helpers extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

from math import isfinite
from typing import Any


def resolve_audio_transcript_for_output(
    *,
    injected_audio_transcript: str | None,
    result_audio_transcript: str | None,
) -> str | None:
    """Resolve audio transcript with explicit result metadata taking precedence."""
    if result_audio_transcript:
        return result_audio_transcript
    return injected_audio_transcript


def build_run_event_details(
    *,
    grounding_sources: list[str],
    audio_transcript: str | None,
    audio_sources: list[str],
    context_usage_ratio: float | None,
) -> dict[str, Any] | None:
    """Build optional serialized metadata for run end events."""
    details: dict[str, Any] = {}
    if grounding_sources:
        details["grounding_sources"] = grounding_sources
    if audio_transcript:
        details["audio_transcript"] = audio_transcript
        details["audio_sources"] = list(audio_sources)
    normalized_ratio = _normalize_context_usage_ratio(context_usage_ratio)
    if normalized_ratio is not None:
        details["context_usage_ratio"] = normalized_ratio
    return details or None


def _normalize_context_usage_ratio(value: float | None) -> float | None:
    if value is None:
        return None
    if isinstance(value, bool):
        return None
    try:
        ratio = float(value)
    except (TypeError, ValueError, OverflowError):
        return None
    if not isfinite(ratio):
        return None
    if ratio < 0.0 or ratio > 1.0:
        return None
    return round(ratio, 4)


__all__ = ["build_run_event_details", "resolve_audio_transcript_for_output"]
