"""GW-68: ML meta-model routing -- classify task -> best model.

Implements lightweight task classification for model routing.
Uses keyword-based classification as the default (no external deps).
Can be extended with embedding-based or ML-based classifiers.

Task types and their preferred models:
  coding         -> claude-opus-4-6 or gpt-4o (strong code)
  reasoning      -> claude-opus-4-6 or o3 (strong reasoning)
  summarization  -> claude-haiku-4-5 or gpt-4o-mini (fast + cheap)
  creative       -> claude-opus-4-6 or gpt-4o (creative writing)
  retrieval      -> gpt-4o-mini or claude-haiku-4-5 (RAG queries)
  general        -> gpt-4o or claude-sonnet-4-6 (default)

# @trace FR-AROUTE-068
"""

from __future__ import annotations

import logging
import re
from dataclasses import dataclass, field

_log = logging.getLogger(__name__)

TASK_TYPES: tuple[str, ...] = ("coding", "reasoning", "summarization", "creative", "retrieval", "general")

# Keyword patterns compiled with IGNORECASE for each task type.
_PATTERNS: dict[str, re.Pattern[str]] = {
    "coding": re.compile(
        r"\b(code|function|implement|debug|fix bug|refactor|class|method|algorithm|script|program)\b",
        re.IGNORECASE,
    ),
    "reasoning": re.compile(
        r"\b(reason|analyze|logic|deduce|inference|proof|mathematical|step.by.step|think through)\b",
        re.IGNORECASE,
    ),
    "summarization": re.compile(
        r"\b(summarize|tldr|summary|brief|condense|overview|key points)\b",
        re.IGNORECASE,
    ),
    "creative": re.compile(
        r"\b(write|story|poem|creative|fiction|narrative|essay|blog|song|screenplay)\b",
        re.IGNORECASE,
    ),
    "retrieval": re.compile(
        r"\b(find|search|lookup|retrieve|what is|who is|when did|where is)\b",
        re.IGNORECASE,
    ),
}


@dataclass
class TaskClassification:
    """Result of classifying a prompt into a task type."""

    task_type: str  # one of TASK_TYPES
    confidence: float  # 0.0-1.0
    signals: list[str] = field(default_factory=list)  # keywords or features that triggered classification


@dataclass
class ModelPreference:
    """Describes a model and its task affinity."""

    model: str
    provider: str
    priority: int  # lower = higher priority
    task_types: list[str]  # task types this model excels at


DEFAULT_MODEL_PREFERENCES: list[ModelPreference] = [
    ModelPreference("claude-opus-4-6", "anthropic", 1, ["coding", "reasoning", "creative"]),
    ModelPreference("gpt-4o", "openai", 2, ["coding", "general", "creative"]),
    ModelPreference("claude-sonnet-4-6", "anthropic", 3, ["general", "summarization"]),
    ModelPreference("claude-haiku-4-5", "anthropic", 4, ["summarization", "retrieval"]),
    ModelPreference("gpt-4o-mini", "openai", 5, ["summarization", "retrieval"]),
]


def classify_task(prompt: str) -> TaskClassification:
    """Classify the task type from the prompt using keyword signals."""
    hit_counts: dict[str, int] = {}
    matched_signals: dict[str, list[str]] = {}

    for task_type, pattern in _PATTERNS.items():
        matches = pattern.findall(prompt)
        if matches:
            hit_counts[task_type] = len(matches)
            matched_signals[task_type] = [m if isinstance(m, str) else m[0] for m in matches]

    total_hits = sum(hit_counts.values())

    if total_hits == 0:
        return TaskClassification(task_type="general", confidence=0.5, signals=[])

    # Find task type with most hits; ties broken by TASK_TYPES order (first wins)
    best_type = max(
        hit_counts.keys(),
        key=lambda t: (hit_counts[t], -list(_PATTERNS.keys()).index(t)),
    )
    confidence = hit_counts[best_type] / total_hits
    signals = matched_signals.get(best_type, [])

    _log.debug(
        "classify_task: best_type=%r confidence=%.2f hits=%r",
        best_type,
        confidence,
        hit_counts,
    )

    return TaskClassification(task_type=best_type, confidence=confidence, signals=signals)


def select_model(
    classification: TaskClassification,
    preferences: list[ModelPreference] | None = None,
    available_models: list[str] | None = None,
) -> ModelPreference | None:
    """Select best model for the classified task.

    Filters by available_models if provided.
    Returns None if no preferences match.
    """
    prefs = preferences if preferences is not None else DEFAULT_MODEL_PREFERENCES

    candidates = [
        p
        for p in prefs
        if classification.task_type in p.task_types and (available_models is None or p.model in available_models)
    ]

    if not candidates:
        return None

    # Sort by priority ascending (lower = higher priority)
    candidates.sort(key=lambda p: p.priority)
    chosen = candidates[0]
    _log.debug("select_model: chose %r for task_type=%r", chosen.model, classification.task_type)
    return chosen


def ml_route(
    prompt: str,
    preferences: list[ModelPreference] | None = None,
    available_models: list[str] | None = None,
) -> ModelPreference | None:
    """Convenience: classify + select in one call."""
    classification = classify_task(prompt)
    return select_model(classification, preferences=preferences, available_models=available_models)
