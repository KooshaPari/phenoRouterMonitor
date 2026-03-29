"""Artifact format versioning for schema management.

# @trace WL-277
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, UTC


@dataclass
class ArtifactVersion:
    """Represents a versioned artifact format."""

    format_version: str
    schema_hash: str
    created_at: datetime


class ArtifactFormatRegistry:
    """Registry for managing artifact format versions."""

    def __init__(self) -> None:
        """Initialize the artifact format registry."""
        self._versions: dict[str, ArtifactVersion] = {}

    def register(self, format_version: str, schema_hash: str) -> ArtifactVersion:
        """Register a new artifact format version.

        Args:
            format_version: The version identifier.
            schema_hash: The hash of the schema definition.

        Returns:
            The created ArtifactVersion.
        """
        version = ArtifactVersion(
            format_version=format_version,
            schema_hash=schema_hash,
            created_at=datetime.now(UTC),
        )
        self._versions[format_version] = version
        return version

    def get(self, format_version: str) -> ArtifactVersion:
        """Get a specific artifact format version.

        Args:
            format_version: The version to retrieve.

        Returns:
            The ArtifactVersion object.

        Raises:
            KeyError: If the version is not found.
        """
        if format_version not in self._versions:
            raise KeyError(f"Version {format_version} not found")
        return self._versions[format_version]

    def latest(self) -> ArtifactVersion | None:
        """Get the latest registered artifact format version.

        Returns:
            The most recently created ArtifactVersion, or None if no versions exist.
        """
        if not self._versions:
            return None
        return max(self._versions.values(), key=lambda v: v.created_at)

    def all_versions(self) -> list[ArtifactVersion]:
        """Get all registered artifact format versions.

        Returns:
            A list of all ArtifactVersion objects.
        """
        return list(self._versions.values())
