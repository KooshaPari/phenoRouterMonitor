"""WL-103 context compaction primitive.

# @trace WL-103
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from math import ceil
from typing import Any

_log = logging.getLogger(__name__)

# Tiktoken encoding to use when a model is not recognized or not specified.
_FALLBACK_TIKTOKEN_ENCODING = "cl100k_base"

# Map of model-name prefixes to tiktoken encoding names.
_MODEL_ENCODING_MAP: dict[str, str] = {
    "gpt-4": "cl100k_base",
    "gpt-3.5": "cl100k_base",
    "claude": "cl100k_base",
    "gemini": "cl100k_base",
    "text-embedding-ada": "cl100k_base",
}


def _encoding_for_model(model: str) -> Any:
    """Return a tiktoken encoding for *model*, falling back to cl100k_base."""
    import tiktoken  # noqa: PLC0415 -- deferred import to avoid startup cost

    for prefix, encoding_name in _MODEL_ENCODING_MAP.items():
        if model.startswith(prefix):
            return tiktoken.get_encoding(encoding_name)
    try:
        return tiktoken.encoding_for_model(model)
    except KeyError:
        return tiktoken.get_encoding(_FALLBACK_TIKTOKEN_ENCODING)


@dataclass(frozen=True)
class ContextCompactionResult:
    turns: list[dict[str, str]]
    usage_ratio: float
    compacted: bool


class ContextCompactor:
    """Deterministic context usage estimator + compaction helper.

    Supports two token-counting modes:
    - tiktoken mode: when *model* is provided, uses tiktoken for accurate token counts.
    - char-based mode: when *model* is None, estimates tokens as ceil(chars / chars_per_token).

    Compaction triggers when usage_ratio > threshold_ratio and len(turns) >= 4.
    Old turns are summarized; the two most-recent turns are kept verbatim.

    # @trace WL-103
    """

    def __init__(
        self,
        *,
        threshold_ratio: float = 0.8,
        chars_per_token: float = 4.0,
        model: str | None = None,
    ) -> None:
        if threshold_ratio <= 0:
            raise ValueError("threshold_ratio must be > 0")
        if chars_per_token <= 0:
            raise ValueError("chars_per_token must be > 0")
        self._threshold_ratio = threshold_ratio
        self._chars_per_token = chars_per_token
        self._model = model
        self._tiktoken_encoding: Any = None
        if model is not None:
            self._tiktoken_encoding = _encoding_for_model(model)

    def should_compact(self, tokens_used: int, context_max: int) -> bool:
        """Return True if tokens_used / context_max > threshold_ratio.

        # @trace WL-103
        """
        if context_max <= 0:
            raise ValueError("context_max must be > 0")
        return (tokens_used / float(context_max)) > self._threshold_ratio

    def count_tokens(self, text: str) -> int:
        """Count tokens in *text* using tiktoken when a model is configured.

        Falls back to char-based estimation when no model was provided.

        # @trace WL-103
        """
        if not text:
            return 0
        if self._tiktoken_encoding is not None:
            return len(self._tiktoken_encoding.encode(text))
        return ceil(len(text) / self._chars_per_token)

    def estimate_tokens(self, text: str) -> int:
        """Alias for count_tokens for backward compatibility.

        # @trace WL-103
        """
        return self.count_tokens(text)

    def estimate_turn_tokens(self, turn: dict[str, Any]) -> int:
        """Count tokens in a single conversation turn (role + content).

        # @trace WL-103
        """
        role = str(turn.get("role", ""))
        content = str(turn.get("content", ""))
        return self.count_tokens(role) + self.count_tokens(content)

    def count_turns_tokens(self, turns: list[dict[str, Any]]) -> int:
        """Return total token count across all turns.

        # @trace WL-103
        """
        return sum(self.estimate_turn_tokens(t) for t in turns)

    def usage_ratio(self, turns: list[dict[str, Any]], context_window_max: int) -> float:
        """Return tokens_used / context_window_max across all turns.

        # @trace WL-103
        """
        if context_window_max <= 0:
            raise ValueError("context_window_max must be > 0")
        used = self.count_turns_tokens(turns)
        return used / float(context_window_max)

    def compact(self, turns: list[dict[str, str]], context_window_max: int) -> ContextCompactionResult:
        """Compact turns if usage_ratio > threshold_ratio and len(turns) >= 4.

        Old turns (all except the final two) are replaced with a deterministic
        summary message. The two most-recent turns are kept verbatim.

        # @trace WL-103
        """
        ratio = self.usage_ratio(turns, context_window_max)
        if ratio <= self._threshold_ratio or len(turns) < 4:
            return ContextCompactionResult(turns=turns, usage_ratio=ratio, compacted=False)

        turns_to_summarize = turns[:-2]
        summary_lines = ["Summary of prior context:"]
        for turn in turns_to_summarize:
            role = str(turn.get("role", "unknown")).strip() or "unknown"
            content = str(turn.get("content", "")).strip()
            summary_lines.append(f"- {role}: {len(content)} chars")

        summary_turn = {"role": "system", "content": "\n".join(summary_lines)}
        compacted_turns = [summary_turn, *turns[-2:]]
        compacted_ratio = self.usage_ratio(compacted_turns, context_window_max)
        return ContextCompactionResult(turns=compacted_turns, usage_ratio=compacted_ratio, compacted=True)
