from __future__ import annotations

"""GW-54: Content moderation guardrail.

Checks text against configurable blocklists and built-in severity patterns.
Supports input (prompt) and output (response) checking.

# @trace FR-GUARD-054
"""

import logging
import re
from dataclasses import dataclass

_log = logging.getLogger(__name__)

_SEVERITY_RANK: dict[str, int] = {"none": 0, "low": 1, "medium": 2, "high": 3}


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class ModerationCategory:
    name: str  # e.g. "violence", "hate_speech", "explicit", "self_harm"
    severity: str  # "low", "medium", "high"
    pattern: str  # regex pattern


DEFAULT_CATEGORIES: list[ModerationCategory] = [
    ModerationCategory(
        name="violence",
        severity="high",
        pattern=r"\b(kill|murder|stab|shoot|bomb|assassinate|massacre)\b",
    ),
    ModerationCategory(
        name="hate_speech",
        severity="high",
        pattern=r"\bhate\s+(race|religion|gender)\b",
    ),
    ModerationCategory(
        name="explicit",
        severity="medium",
        pattern=r"\b(explicit|nsfw|pornograph\w*)\b",
    ),
    ModerationCategory(
        name="self_harm",
        severity="high",
        pattern=r"\b(suicide|self.harm|cut myself)\b",
    ),
    ModerationCategory(
        name="spam",
        severity="low",
        pattern=r"(?:https?://\S+\s*){3,}",
    ),
]


@dataclass
class ModerationResult:
    flagged: bool
    categories: list[str]  # category names that matched
    severity: str  # highest matched severity or "none"
    score: float  # 0.0–1.0 (fraction of categories triggered)


@dataclass
class ModerationConfig:
    enabled: bool = True
    categories: list[ModerationCategory] | None = None  # None = use defaults
    block_on_severity: str = "high"  # block if severity >= this level
    custom_blocklist: list[str] | None = None  # additional blocked words/phrases


# ---------------------------------------------------------------------------
# Module-level compiled pattern cache
# ---------------------------------------------------------------------------

_compiled_defaults: list[tuple[ModerationCategory, re.Pattern[str]]] | None = None


def _get_compiled_defaults() -> list[tuple[ModerationCategory, re.Pattern[str]]]:
    """Return compiled (category, regex) pairs for DEFAULT_CATEGORIES (cached)."""
    global _compiled_defaults  # noqa: PLW0603
    if _compiled_defaults is None:
        _compiled_defaults = [(cat, re.compile(cat.pattern, re.IGNORECASE)) for cat in DEFAULT_CATEGORIES]
    return _compiled_defaults


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def check_moderation(text: str, config: ModerationConfig | None = None) -> ModerationResult:
    """Check text against moderation rules. Uses DEFAULT_CATEGORIES if config is None."""
    cfg = config or ModerationConfig()

    if not cfg.enabled:
        return ModerationResult(flagged=False, categories=[], severity="none", score=0.0)

    # Determine which categories to use
    if cfg.categories is not None:
        compiled: list[tuple[ModerationCategory, re.Pattern[str]]] = [
            (cat, re.compile(cat.pattern, re.IGNORECASE)) for cat in cfg.categories
        ]
    else:
        compiled = _get_compiled_defaults()

    matched_names: list[str] = []
    highest_severity = "none"

    for cat, regex in compiled:
        if regex.search(text):
            matched_names.append(cat.name)
            if _SEVERITY_RANK.get(cat.severity, 0) > _SEVERITY_RANK.get(highest_severity, 0):
                highest_severity = cat.severity

    # Check custom blocklist — any match adds a "custom_blocklist" category at high severity
    if cfg.custom_blocklist:
        for phrase in cfg.custom_blocklist:
            escaped = re.escape(phrase)
            if re.search(escaped, text, re.IGNORECASE):
                if "custom_blocklist" not in matched_names:
                    matched_names.append("custom_blocklist")
                if _SEVERITY_RANK.get("high", 0) > _SEVERITY_RANK.get(highest_severity, 0):
                    highest_severity = "high"

    total = len(compiled) + (len(cfg.custom_blocklist) if cfg.custom_blocklist else 0)
    if total == 0:
        score = 0.0
    else:
        score = min(1.0, len(matched_names) / total)

    flagged = bool(matched_names)

    _log.debug(
        "check_moderation: flagged=%s categories=%s severity=%s score=%.3f",
        flagged,
        matched_names,
        highest_severity,
        score,
    )

    return ModerationResult(
        flagged=flagged,
        categories=matched_names,
        severity=highest_severity,
        score=score,
    )


def should_block(result: ModerationResult, config: ModerationConfig | None = None) -> bool:
    """Return True if the moderation result warrants blocking the request."""
    cfg = config or ModerationConfig()
    if not result.flagged:
        return False
    threshold_rank = _SEVERITY_RANK.get(cfg.block_on_severity, 3)
    result_rank = _SEVERITY_RANK.get(result.severity, 0)
    return result_rank >= threshold_rank
