"""Run audio helper services extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

from typing import Any


def build_audio_summary_metadata(*, audio_transcript: str | None, audio_sources: list[str]) -> dict[str, Any] | None:
    """Build compact transcript metadata for run output payloads."""
    if not audio_transcript:
        return None
    return {
        "transcript_length_chars": len(audio_transcript),
        "source_count": len(audio_sources),
        "sources": list(audio_sources),
    }


__all__ = ["build_audio_summary_metadata"]
