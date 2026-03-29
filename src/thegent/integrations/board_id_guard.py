"""Board-ID Collision Guard for workstream sync integrity.

# @trace WL-183
"""

from __future__ import annotations

import re


class BoardIdCollisionError(Exception):
    """Exception raised when board ID collisions are detected."""


class BoardIdRegistry:
    """Registry for tracking board IDs across connectors."""

    def __init__(self) -> None:
        """Initialize the board ID registry."""
        self._registry: dict[str, str] = {}

    def register(self, board_id: str, connector: str) -> None:
        """Register a board ID for a connector.

        Args:
            board_id: The board ID to register.
            connector: The connector name.

        Raises:
            BoardIdCollisionError: If board_id is already registered with a different connector.
        """
        if board_id in self._registry:
            existing_connector = self._registry[board_id]
            if existing_connector != connector:
                raise BoardIdCollisionError(
                    f"Board ID '{board_id}' is already registered with "
                    f"connector '{existing_connector}', cannot register with '{connector}'"
                )
        else:
            self._registry[board_id] = connector

    def check_collision(self, board_id: str) -> bool:
        """Check if a board ID exists in the registry.

        Args:
            board_id: The board ID to check.

        Returns:
            True if board_id is already registered, False otherwise.
        """
        return board_id in self._registry

    def get_all(self) -> dict[str, str]:
        """Get all registered board IDs and their connectors.

        Returns:
            Dictionary mapping board_id -> connector_name.
        """
        return dict(self._registry)

    def clear(self) -> None:
        """Clear all registered board IDs."""
        self._registry.clear()


_LEGACY_BOARD_ID_PATTERN = re.compile(r"^(?:wl[-_ ]?)?(\d+)$|^board[-_ ]?(\d+)$", re.IGNORECASE)


def migrate_legacy_board_id(legacy_id: str) -> str:
    """Convert legacy board IDs to canonical WL namespace IDs."""
    token = legacy_id.strip()
    match = _LEGACY_BOARD_ID_PATTERN.fullmatch(token)
    if not match:
        raise ValueError(f"Unsupported legacy board ID: {legacy_id}")
    numeric_part = match.group(1) or match.group(2)
    if numeric_part is None:
        raise ValueError(f"Unable to parse legacy board ID: {legacy_id}")
    return f"WL-{int(numeric_part)}"


def validate_no_collisions(registry: BoardIdRegistry) -> None:
    """Validate that no duplicate board IDs exist across connectors.

    Args:
        registry: The BoardIdRegistry to validate.

    Raises:
        BoardIdCollisionError: If duplicate board IDs are detected.
    """
    all_ids = registry.get_all()

    # Count board IDs per connector to detect duplicates
    board_id_counts: dict[str, int] = {}
    for board_id in all_ids:
        board_id_counts[board_id] = board_id_counts.get(board_id, 0) + 1

    # Check for duplicates (count > 1 would indicate issue)
    duplicates = {bid: count for bid, count in board_id_counts.items() if count > 1}

    if duplicates:
        raise BoardIdCollisionError(f"Found duplicate board IDs: {duplicates}")
