"""Diff payload models and ANSI renderer for HITL approval workflows (WL-100).

Traces to: FR-HITL-100
"""

from __future__ import annotations

import difflib
from typing import Any

from pydantic import BaseModel, Field


class DiffPayload(BaseModel, frozen=True):
    """Immutable container for a unified diff between two strings."""

    before: str
    after: str
    path: str = ""
    unified_diff: str = ""

    @classmethod
    def from_strings(
        cls,
        before: str,
        after: str,
        path: str = "",
    ) -> DiffPayload:
        """Create a DiffPayload with pre-computed unified diff."""
        before_lines = before.splitlines(keepends=True)
        after_lines = after.splitlines(keepends=True)
        fromfile = f"a/{path}" if path else "a/file"
        tofile = f"b/{path}" if path else "b/file"
        diff_lines = list(
            difflib.unified_diff(
                before_lines,
                after_lines,
                fromfile=fromfile,
                tofile=tofile,
            )
        )
        unified = "".join(diff_lines)
        return cls(before=before, after=after, path=path, unified_diff=unified)


class DiffRenderer:
    """Renders a DiffPayload as ANSI-colored, plain, or summary text."""

    @staticmethod
    def render_ansi(diff_payload: DiffPayload) -> str:
        """Return ANSI-colored unified diff string."""
        if not diff_payload.unified_diff:
            return "\033[0m"
        lines = diff_payload.unified_diff.splitlines()
        colored: list[str] = []
        for line in lines:
            if line.startswith("@@"):
                colored.append(f"\033[36m{line}")
            elif line.startswith("+"):
                colored.append(f"\033[32m{line}")
            elif line.startswith("-"):
                colored.append(f"\033[31m{line}")
            else:
                colored.append(line)
        return "\n".join(colored) + "\033[0m"

    @staticmethod
    def render_plain(diff_payload: DiffPayload) -> str:
        """Return plain unified diff string (no ANSI codes)."""
        return diff_payload.unified_diff

    @staticmethod
    def render_summary(diff_payload: DiffPayload) -> str:
        """Return a one-line summary like '+5 -3 lines in path/to/file.py'."""
        additions = 0
        deletions = 0
        for line in diff_payload.unified_diff.splitlines():
            if line.startswith("+") and not line.startswith("+++"):
                additions += 1
            elif line.startswith("-") and not line.startswith("---"):
                deletions += 1
        path_part = f" in {diff_payload.path}" if diff_payload.path else ""
        return f"+{additions} -{deletions} lines{path_part}"


class HITLDiffPayload(BaseModel, frozen=True):
    """Immutable payload bundling an approval ID with a diff for HITL review."""

    approval_id: str
    diff: DiffPayload
    context: dict[str, Any] = Field(default_factory=dict)
    requested_at_utc: str = ""
