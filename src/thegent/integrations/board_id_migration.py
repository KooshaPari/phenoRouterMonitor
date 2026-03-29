from __future__ import annotations

from dataclasses import dataclass


@dataclass
class MigrationEntry:
    """Represents a single board ID migration record."""

    old_id: str
    new_id: str
    migrated: bool = False


class LegacyBoardIdMigrationTool:
    """Tool for managing legacy board ID migrations.

    # @trace WL-247
    """

    def __init__(self) -> None:
        """Initialize the migration tool with an empty registry."""
        self._entries: dict[str, MigrationEntry] = {}

    def register(self, old_id: str, new_id: str) -> MigrationEntry:
        """Register a new migration entry.

        Args:
            old_id: The legacy board ID
            new_id: The new board ID

        Returns:
            The created MigrationEntry
        """
        entry = MigrationEntry(old_id=old_id, new_id=new_id, migrated=False)
        self._entries[old_id] = entry
        return entry

    def migrate(self, old_id: str) -> MigrationEntry:
        """Mark a migration as complete.

        Args:
            old_id: The legacy board ID to mark as migrated

        Returns:
            The updated MigrationEntry

        Raises:
            KeyError: If the old_id is not registered
        """
        entry = self._entries[old_id]
        entry.migrated = True
        return entry

    def lookup_new(self, old_id: str) -> str:
        """Look up the new board ID for a legacy ID.

        Args:
            old_id: The legacy board ID

        Returns:
            The new board ID

        Raises:
            KeyError: If the old_id is not registered
        """
        return self._entries[old_id].new_id

    def pending(self) -> list[MigrationEntry]:
        """Get all pending (not yet migrated) entries.

        Returns:
            List of MigrationEntry objects with migrated=False
        """
        return [e for e in self._entries.values() if not e.migrated]

    def completed(self) -> list[MigrationEntry]:
        """Get all completed migrations.

        Returns:
            List of MigrationEntry objects with migrated=True
        """
        return [e for e in self._entries.values() if e.migrated]
