"""Preset prompt catalog for Lifecycle loops."""

from dataclasses import dataclass
from typing import Any


@dataclass
class PresetEntry:
    """Entry in the preset catalog."""

    id: str
    prompt: str
    trigger_keywords: list[str]


PRESET_CATALOG: dict[str, PresetEntry] = {
    "write_tests": PresetEntry(
        id="write_tests",
        prompt="Write unit tests for the changed modules. Cover edge cases.",
        trigger_keywords=["test", "coverage", "pytest"],
    ),
    "write_docs": PresetEntry(
        id="write_docs",
        prompt="Update documentation for the modified components.",
        trigger_keywords=["doc", "readme", "guide"],
    ),
    "add_polish": PresetEntry(
        id="add_polish",
        prompt="Add polish: error messages, type hints, docstrings.",
        trigger_keywords=["polish", "refine", "cleanup"],
    ),
    "continue": PresetEntry(
        id="continue",
        prompt="Continue with the next item in the WBS.",
        trigger_keywords=["continue", "next", "proceed"],
    ),
}


def get_preset(preset_id: str) -> PresetEntry | None:
    """Return preset entry by id."""
    return PRESET_CATALOG.get(preset_id)


def match_preset(text: str) -> PresetEntry | None:
    """Match text against preset trigger keywords."""
    text_lower = text.lower()
    for preset in PRESET_CATALOG.values():
        if any(kw in text_lower for kw in preset.trigger_keywords):
            return preset
    return None


def list_presets() -> list[dict[str, Any]]:
    """Return all presets for CLI/MCP discovery."""
    return [
        {
            "id": p.id,
            "prompt": p.prompt,
            "trigger_keywords": p.trigger_keywords,
        }
        for p in PRESET_CATALOG.values()
    ]
