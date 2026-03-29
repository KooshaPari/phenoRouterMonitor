"""RunOptions — extended run-time options for agent execution.

# @trace WL-112
"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel

ReasoningEffort = Literal["minimal", "low", "medium", "high", "xhigh"]

# Anthropic thinking.budget_tokens mapping per reasoning_effort level.
# @trace WL-112
ANTHROPIC_BUDGET_TOKENS: dict[str, int] = {
    "minimal": 1000,
    "low": 2000,
    "medium": 5000,
    "high": 8000,
    "xhigh": 16000,
}

# Codex --config key for reasoning effort.
# @trace WL-112
CODEX_REASONING_CONFIG_KEY = "model_reasoning_effort"

# Agents identified as Codex-backed providers.
# @trace WL-112
CODEX_AGENTS: frozenset[str] = frozenset(
    {
        "codex",
        "copilot",
        "antigravity",
        "gemini",
        "claude",
        "kiro",
        "nim",
        "zen",
        "cliproxy",
        "minimax",
        "glm",
        "kilo",
        "summarizer",
    }
)

# Agents that support --image / image content block inputs (WL-114).
# Codex-backed agents accept --image <path> CLI args.
# Claude Code accepts images via --input-format stream-json content blocks.
# @trace WL-114
IMAGE_CAPABLE_AGENTS: frozenset[str] = frozenset(
    {
        "codex",
        "copilot",
        "antigravity",
        "gemini",
        "claude",
        "kiro",
        "nim",
        "zen",
        "cliproxy",
        "minimax",
        "glm",
        "kilo",
        "summarizer",
    }
)

# Agents that go via the Anthropic direct API (not CLIProxy).
ANTHROPIC_DIRECT_AGENTS: frozenset[str] = frozenset()

# OpenAI o-series models that support reasoning.effort.
OPENAI_O_SERIES_AGENTS: frozenset[str] = frozenset()


class RunOptions(BaseModel):
    """Extended run options for agent execution.

    # @trace WL-112
    """

    reasoning_effort: ReasoningEffort | None = None
    output_schema_path: str | None = None  # WL-113


def translate_reasoning_to_codex_config(effort: str) -> dict[str, str]:
    """Translate a reasoning_effort value to a Codex --config dict.

    Returns a dict suitable for merging into CodexProxyRunner.config_overrides
    or passing to _build_config_flags().

    # @trace WL-112
    """
    return {CODEX_REASONING_CONFIG_KEY: effort}


def translate_reasoning_to_anthropic_budget(effort: str) -> int:
    """Translate a reasoning_effort value to Anthropic thinking.budget_tokens.

    Raises KeyError if the effort value is not a recognised level.

    # @trace WL-112
    """
    return ANTHROPIC_BUDGET_TOKENS[effort]


def translate_reasoning_to_openai_effort(effort: str) -> str:
    """Translate a reasoning_effort value to OpenAI o-series reasoning.effort string.

    Maps xhigh -> high (OpenAI only supports low/medium/high).

    # @trace WL-112
    """
    if effort == "xhigh":
        return "high"
    return effort
