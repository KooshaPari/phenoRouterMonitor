"""Prompt constraint helpers extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations


def inject_time_constraint(
    *,
    prompt: str,
    timeout: int,
    seconds_per_tool_call: float,
    summary_mode: bool = True,
) -> str:
    """Append timeout-aware tool budget guidance to an agent prompt."""
    n_calls = max(1, int(timeout / seconds_per_tool_call))
    suffix = (
        f"\n\n[TIME CONSTRAINT: You have approximately {n_calls} tool calls (~{timeout}s). "
        "When done or when approaching this limit, wrap up and report. "
        "Do not start new multi-step work.]"
    )
    if summary_mode:
        suffix += (
            "\n\n[OUTPUT FORMAT: End your response with a brief worker status report: "
            "**Summary** (1–2 sentences), **Items Done** (bullet list), **Issues** (if any), "
            "**Next Steps** (bullet list). Use markdown. This is the primary output shown.]"
        )
    return prompt + suffix


__all__ = ["inject_time_constraint"]
