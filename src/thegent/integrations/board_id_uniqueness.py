"""Strict board ID uniqueness enforcement.

# @trace WL-309
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


class DuplicateBoardIdError(Exception):
    """Raised when a duplicate board ID is registered."""


@dataclass
class BoardIdUniquenessPolicy:
    """Policy for enforcing board ID uniqueness."""

    enforce_global_uniqueness: bool = True


class UniquenesEnforcer:
    """Enforce strict board ID uniqueness across all sync artifacts."""

    def __init__(self, policy: BoardIdUniquenessPolicy | None = None) -> None:
        """Initialize the uniqueness enforcer.

        Args:
            policy: BoardIdUniquenessPolicy instance (default: BoardIdUniquenessPolicy()).
        """
        self.policy = policy or BoardIdUniquenessPolicy()
        self._registry: set[str] = set()

    def register_id(self, board_id: str, _context: dict[str, Any] | None = None) -> None:
        """Register a board ID and enforce uniqueness.

        Args:
            board_id: The board ID to register.
            _context: Optional context information for the registration.

        Raises:
            DuplicateBoardIdError: If the board ID is already registered
                and enforce_global_uniqueness is True.
        """
        if self.policy.enforce_global_uniqueness:
            if board_id in self._registry:
                raise DuplicateBoardIdError(
                    f"Board ID '{board_id}' is already registered. Duplicate board IDs are not allowed."
                )

        self._registry.add(board_id)

    def is_registered(self, board_id: str) -> bool:
        """Check if a board ID is already registered.

        Args:
            board_id: The board ID to check.

        Returns:
            True if the board ID is registered, False otherwise.
        """
        return board_id in self._registry

    def reset(self) -> None:
        """Clear all registered board IDs.

        This method resets the registry to an empty state.
        """
        self._registry.clear()


def validate_unique_board_ids(board_ids: list[str]) -> None:
    """Validate a list of canonical board IDs is globally unique."""
    enforcer = UniquenesEnforcer()
    for board_id in board_ids:
        enforcer.register_id(board_id)
