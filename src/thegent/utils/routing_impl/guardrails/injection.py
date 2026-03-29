"""GW-50: Prompt injection detection for LLM gateway.

Detects common prompt injection patterns in user messages before forwarding
to the LLM. Returns a verdict with confidence score.

# @trace FR-GUARD-050
"""

from __future__ import annotations

import re
from dataclasses import dataclass

_SEVERITY_RANK: dict[str, int] = {"none": 0, "low": 1, "medium": 2, "high": 3}


@dataclass
class InjectionPattern:
    name: str
    pattern: str  # regex pattern
    severity: str  # "low" | "medium" | "high"
    description: str


INJECTION_PATTERNS: list[InjectionPattern] = [
    InjectionPattern(
        name="ignore_instructions",
        pattern=r"ignore\b.{0,30}?\binstructions\b",
        severity="high",
        description="Attempts to override previous instructions.",
    ),
    InjectionPattern(
        name="system_override",
        pattern=r"you are now|act as|pretend you are|forget.*instructions",
        severity="high",
        description="Attempts to reassign model identity or erase instructions.",
    ),
    InjectionPattern(
        name="jailbreak_dan",
        pattern=r"\bDAN\b|do anything now",
        severity="high",
        description="DAN-style jailbreak pattern.",
    ),
    InjectionPattern(
        name="reveal_prompt",
        pattern=r"reveal.*prompt|show.*system.?(prompt|message)|what.*your (instructions|prompt)",
        severity="medium",
        description="Attempts to extract the system prompt.",
    ),
    InjectionPattern(
        name="role_confusion",
        pattern=r"you (must|should|will) (obey|follow|comply)",
        severity="medium",
        description="Attempts to override model's role via commands.",
    ),
    InjectionPattern(
        name="escape_sequence",
        pattern=r"```\s*system|<\|im_start\|>|<\|im_end\|>",
        severity="medium",
        description="Special token / escape sequence injection.",
    ),
]


@dataclass
class InjectionCheckResult:
    detected: bool
    patterns_matched: list[str]  # names of matched patterns
    severity: str  # highest severity matched, or "none"
    confidence: float  # 0.0 – 1.0


# ---------------------------------------------------------------------------
# Module-level compiled pattern cache
# ---------------------------------------------------------------------------

_compiled_cache: list[tuple[InjectionPattern, re.Pattern]] | None = None


def get_compiled_patterns() -> list[tuple[InjectionPattern, re.Pattern]]:
    """Return compiled (pattern, regex) pairs (cached at module level)."""
    global _compiled_cache  # noqa: PLW0603
    if _compiled_cache is None:
        _compiled_cache = [(p, re.compile(p.pattern, re.IGNORECASE)) for p in INJECTION_PATTERNS]
    return _compiled_cache


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def check_injection(
    text: str,
    patterns: list[InjectionPattern] | None = None,
) -> InjectionCheckResult:
    """Check text for prompt injection patterns.

    Args:
        text: The text to scan.
        patterns: Override the default INJECTION_PATTERNS list.  When None,
            the module-level compiled cache is used for efficiency.

    Returns:
        InjectionCheckResult with all matched pattern names, highest severity,
        and a confidence score in [0, 1].
    """
    if patterns is not None:
        compiled = [(p, re.compile(p.pattern, re.IGNORECASE)) for p in patterns]
    else:
        compiled = get_compiled_patterns()
        patterns = INJECTION_PATTERNS  # needed for denominator

    matched: list[str] = []
    highest_severity = "none"

    for pat, regex in compiled:
        if regex.search(text):
            matched.append(pat.name)
            if _SEVERITY_RANK.get(pat.severity, 0) > _SEVERITY_RANK.get(highest_severity, 0):
                highest_severity = pat.severity

    total = len(compiled) if compiled else 1
    confidence = min(1.0, len(matched) / total)

    return InjectionCheckResult(
        detected=bool(matched),
        patterns_matched=matched,
        severity=highest_severity,
        confidence=confidence,
    )


def check_messages_for_injection(messages: list[dict]) -> InjectionCheckResult:
    """Check a list of OpenAI-style message dicts for prompt injection.

    Only user-role messages are scanned.  The content strings are concatenated
    with a newline separator and passed to check_injection.

    Args:
        messages: List of dicts with at least a ``role`` and ``content`` key.

    Returns:
        InjectionCheckResult aggregated across all user messages.
    """
    user_texts: list[str] = []
    for msg in messages:
        if msg.get("role") == "user":
            content = msg.get("content", "")
            if isinstance(content, str):
                user_texts.append(content)
            elif isinstance(content, list):
                # Handle multi-part content (text blocks)
                for part in content:
                    if isinstance(part, dict) and part.get("type") == "text":
                        user_texts.append(part.get("text", ""))

    combined = "\n".join(user_texts)
    return check_injection(combined)
