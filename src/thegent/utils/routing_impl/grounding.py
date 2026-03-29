"""Helpers for WL-119 grounding source extraction."""

from __future__ import annotations

import re
from typing import Any
from urllib.parse import urlsplit, urlunsplit

_URL_RE = re.compile(r"https?://[^\s)>\]\"']+")


def normalize_grounding_source_url(url: str) -> str:
    """Normalize URL formatting so equivalent grounding sources dedupe reliably."""
    raw = (url or "").strip()
    if not raw:
        return ""
    # Some providers return wrapped URL literals like "<https://...>".
    while len(raw) > 1:
        if raw.startswith("<") and raw.endswith(">"):
            raw = raw[1:-1].strip()
            continue
        if raw.startswith("(") and raw.endswith(")"):
            raw = raw[1:-1].strip()
            continue
        if raw.startswith("[") and raw.endswith("]"):
            raw = raw[1:-1].strip()
            continue
        break
    trimmed = raw.rstrip(".,;:!?")
    parts = urlsplit(trimmed)
    if not parts.scheme or not parts.netloc:
        return trimmed
    normalized_scheme = parts.scheme.lower()
    normalized_netloc = parts.netloc.lower()
    if normalized_scheme == "http" and normalized_netloc.endswith(":80"):
        normalized_netloc = normalized_netloc[:-3]
    if normalized_scheme == "https" and normalized_netloc.endswith(":443"):
        normalized_netloc = normalized_netloc[:-4]
    normalized_path = parts.path
    if normalized_path == "/" and not parts.query and not parts.fragment:
        normalized_path = ""
    return urlunsplit((normalized_scheme, normalized_netloc, normalized_path, parts.query, parts.fragment))


def extract_grounding_sources(text: str) -> list[str]:
    """Extract unique URL-like sources from model output."""
    if not text:
        return []
    seen: set[str] = set()
    ordered: list[str] = []
    for match in _URL_RE.findall(text):
        normalized = normalize_grounding_source_url(match)
        if normalized and normalized not in seen:
            seen.add(normalized)
            ordered.append(normalized)
    return ordered


def extract_grounding_sources_from_payload(payload: Any) -> list[str]:
    """Extract grounding URLs from provider metadata payloads."""
    if payload is None:
        return []

    seen: set[str] = set()
    ordered: list[str] = []

    def _push(value: str) -> None:
        for url in extract_grounding_sources(value):
            if url not in seen:
                seen.add(url)
                ordered.append(url)

    def _walk(node: Any) -> None:
        if isinstance(node, dict):
            for key, value in node.items():
                lowered = str(key).lower()
                if (lowered in {"uri", "url", "link"} or lowered.endswith(("uri", "url", "link"))) and isinstance(
                    value, str
                ):
                    _push(value)
                elif lowered in {"groundingmetadata", "groundingchunks", "websearchqueries", "sources", "citations"}:
                    _walk(value)
                else:
                    _walk(value)
        elif isinstance(node, list):
            for item in node:
                _walk(item)
        elif isinstance(node, str):
            _push(node)

    _walk(payload)
    return ordered
