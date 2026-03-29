"""WL ignore list helper."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class WLIgnoreList:
    """Deterministic set-like ignore list."""

    _ids: set[str] = field(default_factory=set)

    def add(self, wl_id: str) -> None:
        """Add an ID to the ignore list."""
        normalized = wl_id.strip()
        if normalized:
            self._ids.add(normalized)

    def remove(self, wl_id: str) -> None:
        """Remove an ID from the ignore list."""
        self._ids.discard(wl_id.strip())

    def is_ignored(self, wl_id: str) -> bool:
        """Return whether an ID is ignored."""
        return wl_id.strip() in self._ids

    def all_ignored(self) -> list[str]:
        """Return all ignored IDs in sorted order."""
        return sorted(self._ids)

    def filter(self, wl_ids: list[str]) -> list[str]:
        """Return input IDs excluding ignored values while preserving order."""
        return [wl_id for wl_id in wl_ids if wl_id.strip() not in self._ids]


__all__ = ["WLIgnoreList"]
