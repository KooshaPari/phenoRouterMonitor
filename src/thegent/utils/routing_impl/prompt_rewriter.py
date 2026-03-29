"""GW-69: Auto prompt rewriting per model/provider.

Implements provider-specific and model-specific prompt normalization.
Rules are matched by provider and model prefix, then applied in priority order.

# @trace FR-PROMPT-069
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Any

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Public data structures
# ---------------------------------------------------------------------------


@dataclass
class RewriteRule:
    """A prompt rewriting rule matched by provider and/or model."""

    name: str  # e.g. "anthropic_xml_tools"
    providers: list[str]  # e.g. ["anthropic"] — match if provider in list (empty = all)
    models: list[str]  # e.g. ["claude-opus-4-6"] — match if model startswith any (empty = all)
    transform: str  # name of transform to apply (see _TRANSFORMS dict)
    priority: int = 0  # higher = applied first


@dataclass
class RewriteConfig:
    """Configuration for the prompt rewriter."""

    enabled: bool = True
    rules: list[RewriteRule] | None = None  # None = use DEFAULT_RULES
    max_system_length: int = 4096  # truncate system prompt to this many chars


@dataclass
class RewriteResult:
    """Result of a prompt rewrite operation."""

    messages: list[dict]  # rewritten messages (same format as input)
    applied_rules: list[str]  # names of rules applied
    original_token_estimate: int  # rough character count of original
    rewritten_token_estimate: int  # rough character count of result


# ---------------------------------------------------------------------------
# Transform functions
# ---------------------------------------------------------------------------


def _add_cot_suffix(messages: list[dict]) -> list[dict]:
    """Append 'Think step by step.' to the last user message content string.

    Only applied when the last message has role 'user' and content is a string.
    Returns a new list; the last message is replaced with a new dict if modified.
    """
    if not messages:
        return messages
    last = messages[-1]
    if last.get("role") != "user":
        return messages
    content = last.get("content")
    if not isinstance(content, str):
        return messages
    new_last = dict(last)
    new_last["content"] = content + " Think step by step."
    return list(messages[:-1]) + [new_last]


def _normalize_system_prompt(messages: list[dict]) -> list[dict]:
    """No-op: if first message has role 'system', return as-is.

    Otherwise returns messages unchanged — provider already handles
    system-less prompts fine.
    """
    return messages


def _truncate_long_system(messages: list[dict], max_length: int) -> list[dict]:
    """Truncate system prompt content if it exceeds max_length characters.

    Appends '... [truncated]' to the end of any truncated system prompt.
    Returns a new list if truncation occurred; otherwise returns the original.
    """
    if not messages:
        return messages
    first = messages[0]
    if first.get("role") != "system":
        return messages
    content = first.get("content")
    if not isinstance(content, str):
        return messages
    if len(content) <= max_length:
        return messages
    new_first = dict(first)
    new_first["content"] = content[:max_length] + "... [truncated]"
    return [new_first] + list(messages[1:])


def _remove_empty_turns(messages: list[dict]) -> list[dict]:
    """Filter out any messages where content is the empty string ''."""
    return [m for m in messages if m.get("content") != ""]


# ---------------------------------------------------------------------------
# Module-level constants
# ---------------------------------------------------------------------------

DEFAULT_RULES: list[RewriteRule] = [
    RewriteRule(
        name="add_cot_reasoning",
        providers=[],  # all providers
        models=["o1", "o3", "claude-opus", "claude-sonnet"],  # prefix match
        transform="add_cot_suffix",
        priority=10,
    ),
    RewriteRule(
        name="remove_empty_turns",
        providers=[],
        models=[],
        transform="remove_empty_turns",
        priority=5,
    ),
]

_TRANSFORMS: dict[str, Any] = {
    "add_cot_suffix": _add_cot_suffix,
    "normalize_system": _normalize_system_prompt,
    "remove_empty_turns": _remove_empty_turns,
}


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def _rule_matches(rule: RewriteRule, provider: str, model: str) -> bool:
    """Return True if rule applies to this provider+model combination.

    Provider match: empty list matches all; otherwise provider must be in rule.providers.
    Model match: empty list matches all; otherwise model must startswith any of rule.models.
    """
    if rule.providers and provider not in rule.providers:
        return False
    if rule.models and not any(model.startswith(prefix) for prefix in rule.models):
        return False
    return True


def rewrite_prompt(
    messages: list[dict],
    *,
    provider: str = "",
    model: str = "",
    config: RewriteConfig | None = None,
) -> RewriteResult:
    """Rewrite messages for provider/model conventions.

    Returns a RewriteResult with the (possibly modified) messages.
    When config.enabled=False or no rules match, returns original messages unchanged.
    """
    cfg = config or RewriteConfig()

    if not cfg.enabled:
        orig_len = sum(len(str(m.get("content", ""))) for m in messages)
        return RewriteResult(
            messages=list(messages),
            applied_rules=[],
            original_token_estimate=orig_len,
            rewritten_token_estimate=orig_len,
        )

    rules = cfg.rules if cfg.rules is not None else DEFAULT_RULES
    # Sort by priority descending (higher priority first)
    active_rules = sorted(
        [r for r in rules if _rule_matches(r, provider, model)],
        key=lambda r: r.priority,
        reverse=True,
    )

    original_len = sum(len(str(m.get("content", ""))) for m in messages)
    result_messages = [dict(m) for m in messages]  # shallow copy
    applied: list[str] = []

    for rule in active_rules:
        transform_fn = _TRANSFORMS.get(rule.transform)
        if transform_fn is None:
            _log.warning("Unknown transform: %s (rule=%s)", rule.transform, rule.name)
            continue
        result_messages = transform_fn(result_messages)
        applied.append(rule.name)

    # Apply truncation as a final pass (not a named rule)
    result_messages = _truncate_long_system(result_messages, cfg.max_system_length)

    rewritten_len = sum(len(str(m.get("content", ""))) for m in result_messages)
    return RewriteResult(
        messages=result_messages,
        applied_rules=applied,
        original_token_estimate=original_len,
        rewritten_token_estimate=rewritten_len,
    )
