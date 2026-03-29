from __future__ import annotations

"""GW-51: PII masking round-trip guardrail.

Redacts PII entities on input (replacing with tokens like [EMAIL_1]),
stores the original→token mapping, and re-inserts originals in LLM output.

Supported entity types (regex-based, no external dep):
  EMAIL, PHONE, SSN, CREDIT_CARD, IP_ADDRESS

# @trace FR-GUARD-051
"""

import logging
import re
from dataclasses import dataclass

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Compiled PII patterns (module-level)
# ---------------------------------------------------------------------------

_PATTERNS: dict[str, re.Pattern[str]] = {
    "EMAIL": re.compile(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b"),
    "PHONE": re.compile(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b"),
    "SSN": re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
    "CREDIT_CARD": re.compile(r"\b(?:\d{4}[-\s]?){3}\d{4}\b"),
    "IP_ADDRESS": re.compile(r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
}

_ALL_ENTITY_TYPES: list[str] = list(_PATTERNS.keys())


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class PiiEntity:
    entity_type: str  # "EMAIL", "PHONE", "SSN", "CREDIT_CARD", "IP_ADDRESS"
    original: str
    token: str  # replacement token e.g. "[EMAIL_1]"
    start: int
    end: int


@dataclass
class PiiMaskResult:
    masked_text: str
    entities: list[PiiEntity]
    token_map: dict[str, str]  # token → original


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def mask_pii(text: str, entity_types: list[str] | None = None) -> PiiMaskResult:
    """Detect and mask PII entities. entity_types=None masks all known types."""
    active_types = entity_types if entity_types is not None else _ALL_ENTITY_TYPES

    # Collect all matches across all active entity types with their positions
    all_matches: list[tuple[int, int, str, str]] = []  # (start, end, entity_type, original)
    for etype in active_types:
        pattern = _PATTERNS.get(etype)
        if pattern is None:
            _log.warning("Unknown PII entity type: %s — skipping", etype)
            continue
        for m in pattern.finditer(text):
            all_matches.append((m.start(), m.end(), etype, m.group()))

    # Sort by position and remove overlapping matches (keep longest / first)
    all_matches.sort(key=lambda t: (t[0], -(t[1] - t[0])))
    non_overlapping: list[tuple[int, int, str, str]] = []
    last_end = -1
    for start, end, etype, original in all_matches:
        if start >= last_end:
            non_overlapping.append((start, end, etype, original))
            last_end = end

    # Assign numbered tokens per entity type
    type_counters: dict[str, int] = {}
    entities: list[PiiEntity] = []
    for start, end, etype, original in non_overlapping:
        type_counters[etype] = type_counters.get(etype, 0) + 1
        token = f"[{etype}_{type_counters[etype]}]"
        entities.append(PiiEntity(entity_type=etype, original=original, token=token, start=start, end=end))

    # Build masked text by replacing matches back-to-front (preserves offsets)
    masked = text
    for entity in reversed(entities):
        masked = masked[: entity.start] + entity.token + masked[entity.end :]

    token_map: dict[str, str] = {e.token: e.original for e in entities}

    _log.debug("mask_pii: masked %d entities", len(entities))
    return PiiMaskResult(masked_text=masked, entities=entities, token_map=token_map)


def unmask_pii(text: str, token_map: dict[str, str]) -> str:
    """Re-insert original values using the token_map from mask_pii()."""
    result = text
    for token, original in token_map.items():
        result = result.replace(token, original)
    return result


def mask_messages(
    messages: list[dict],
    entity_types: list[str] | None = None,
) -> tuple[list[dict], dict[str, str]]:
    """Mask PII in a messages list (OpenAI format). Returns (masked_messages, token_map)."""
    combined_token_map: dict[str, str] = {}
    masked_messages: list[dict] = []

    for msg in messages:
        new_msg = dict(msg)
        content = msg.get("content", "")
        if isinstance(content, str):
            result = mask_pii(content, entity_types=entity_types)
            new_msg["content"] = result.masked_text
            combined_token_map.update(result.token_map)
        elif isinstance(content, list):
            new_parts = []
            for part in content:
                if isinstance(part, dict) and part.get("type") == "text":
                    result = mask_pii(part.get("text", ""), entity_types=entity_types)
                    new_part = dict(part)
                    new_part["text"] = result.masked_text
                    combined_token_map.update(result.token_map)
                    new_parts.append(new_part)
                else:
                    new_parts.append(part)
            new_msg["content"] = new_parts
        masked_messages.append(new_msg)

    return masked_messages, combined_token_map


def unmask_content(content: str, token_map: dict[str, str]) -> str:
    """Unmask LLM response content using stored token_map."""
    return unmask_pii(content, token_map)
