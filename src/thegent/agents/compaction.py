"""WL-103 context compaction layer with pydantic models.

# @trace FR-CTX-103
"""

from __future__ import annotations

from enum import StrEnum
from typing import Any

from pydantic import BaseModel, Field


class CompactionTrigger(StrEnum):
    """When to trigger context compaction."""

    NEVER = "never"
    TOKEN_THRESHOLD = "token_threshold"
    TURN_COUNT = "turn_count"


class CompactionConfig(BaseModel, frozen=True):
    """Configuration for context compaction behavior."""

    trigger: CompactionTrigger = CompactionTrigger.NEVER
    token_threshold: int = 100_000
    turn_threshold: int = 50
    summary_prompt: str = "Summarize the conversation so far, preserving key decisions and context."
    max_summary_tokens: int = 2_000


class ContextWindow(BaseModel):
    """Mutable state tracking the current conversation context."""

    messages: list[dict[str, Any]] = Field(default_factory=list)
    token_count: int = 0
    turn_count: int = 0
    compacted_at_turn: int | None = None


class CompactionResult(BaseModel, frozen=True):
    """Result of a compaction operation."""

    summary: str
    tokens_before: int
    tokens_after: int
    turns_compacted: int


class ContextCompactor:
    """Manages context window compaction based on configurable triggers."""

    def __init__(self, config: CompactionConfig) -> None:
        self._config = config

    def should_compact(self, window: ContextWindow) -> bool:
        """Return True if compaction is needed based on configured trigger."""
        match self._config.trigger:
            case CompactionTrigger.NEVER:
                return False
            case CompactionTrigger.TOKEN_THRESHOLD:
                return window.token_count > self._config.token_threshold
            case CompactionTrigger.TURN_COUNT:
                return window.turn_count > self._config.turn_threshold

    def compact(self, window: ContextWindow, summary: str) -> CompactionResult:
        """Apply compaction: replace messages with summary, update window state."""
        tokens_before = window.token_count
        turns_compacted = window.turn_count

        window.messages.clear()
        window.messages.append({"role": "system", "content": summary})

        estimated_summary_tokens = self.estimate_tokens(window.messages)
        window.token_count = estimated_summary_tokens
        window.compacted_at_turn = window.turn_count

        return CompactionResult(
            summary=summary,
            tokens_before=tokens_before,
            tokens_after=estimated_summary_tokens,
            turns_compacted=turns_compacted,
        )

    def estimate_tokens(self, messages: list[dict[str, Any]]) -> int:
        """Rough token estimate: sum of len(str(m)) // 4 for each message."""
        return sum(len(str(m)) // 4 for m in messages)

    def build_compaction_prompt(self, window: ContextWindow) -> str:
        """Build the prompt to send to an LLM for summarization."""
        return (
            f"{self._config.summary_prompt}\n\n"
            f"Context: {window.turn_count} turns, "
            f"{window.token_count} estimated tokens."
        )
