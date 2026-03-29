"""Encrypted Artifact Option (WL-254): Store encrypted artifact metadata.

Provides configuration and storage for artifact encryption metadata.
Stores encryption configuration alongside artifacts (actual encryption is
handled by the storage backend).

# @trace WL-254
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Any

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Encryption Configuration
# ---------------------------------------------------------------------------


@dataclass
class ArtifactEncryptionConfig:
    """Encryption configuration for artifacts.

    Attributes:
        algorithm: Encryption algorithm (e.g., "AES-256").
        key_id: Key identifier for decryption (e.g., "default").
    """

    algorithm: str = "AES-256"
    """Encryption algorithm (default: AES-256)."""

    key_id: str = "default"
    """Key identifier for decryption (default: 'default')."""

    def __post_init__(self) -> None:
        """Validate encryption configuration."""
        if not self.algorithm:
            raise ValueError("algorithm cannot be empty")
        if not self.key_id:
            raise ValueError("key_id cannot be empty")


# ---------------------------------------------------------------------------
# Encrypted Artifact Store
# ---------------------------------------------------------------------------


class EncryptedArtifactStore:
    """Manages encrypted artifact storage and retrieval.

    Stores artifact data along with encryption metadata. The actual encryption
    and decryption are handled by the storage backend; this class manages
    metadata and configuration.

    Example:
        >>> store = EncryptedArtifactStore()
        >>> config = ArtifactEncryptionConfig(algorithm="AES-256", key_id="prod-key")
        >>> artifact_id = store.store("art-1", {"data": "sensitive"}, config)
        >>> artifact_id
        "art-1"
        >>> store.retrieve("art-1")
        {"data": "sensitive"}
        >>> store.get_config("art-1").algorithm
        "AES-256"
    """

    def __init__(self) -> None:
        """Initialize the encrypted artifact store."""
        self._artifacts: dict[str, dict[str, Any]] = {}
        self._configs: dict[str, ArtifactEncryptionConfig] = {}
        logger.debug("EncryptedArtifactStore initialized")

    def store(
        self,
        artifact_id: str,
        data: dict[str, Any],
        config: ArtifactEncryptionConfig | None = None,
    ) -> str:
        """Store an artifact with optional encryption configuration.

        Args:
            artifact_id: Unique artifact identifier.
            data: Artifact data as a dictionary.
            config: Encryption configuration. If None, uses default config.

        Returns:
            The artifact_id.

        Raises:
            ValueError: If artifact_id or data is invalid.

        Example:
            >>> store = EncryptedArtifactStore()
            >>> config = ArtifactEncryptionConfig(algorithm="AES-256", key_id="key-1")
            >>> artifact_id = store.store("art-1", {"info": "data"}, config)
            >>> artifact_id
            "art-1"
        """
        if not artifact_id:
            raise ValueError("artifact_id cannot be empty")
        if data is None:
            raise ValueError("data cannot be None")

        # Use default config if none provided
        if config is None:
            config = ArtifactEncryptionConfig()

        self._artifacts[artifact_id] = data
        self._configs[artifact_id] = config

        logger.debug(
            "Stored encrypted artifact: artifact_id=%r, algorithm=%s, key_id=%s",
            artifact_id,
            config.algorithm,
            config.key_id,
        )
        return artifact_id

    def retrieve(self, artifact_id: str) -> dict[str, Any]:
        """Retrieve an artifact.

        Args:
            artifact_id: The artifact to retrieve.

        Returns:
            The artifact data as a dictionary.

        Raises:
            KeyError: If artifact_id is not found.

        Example:
            >>> store = EncryptedArtifactStore()
            >>> store.store("art-1", {"key": "value"})
            >>> store.retrieve("art-1")
            {"key": "value"}
            >>> store.retrieve("nonexistent")
            Traceback (most recent call last):
                ...
            KeyError: 'Artifact art-1 not found'
        """
        if artifact_id not in self._artifacts:
            raise KeyError(f"Artifact {artifact_id!r} not found")

        logger.debug("Retrieved encrypted artifact: artifact_id=%r", artifact_id)
        return self._artifacts[artifact_id]

    def get_config(self, artifact_id: str) -> ArtifactEncryptionConfig:
        """Get encryption configuration for an artifact.

        Args:
            artifact_id: The artifact identifier.

        Returns:
            The ArtifactEncryptionConfig for this artifact.

        Raises:
            KeyError: If artifact_id is not found.

        Example:
            >>> store = EncryptedArtifactStore()
            >>> config = ArtifactEncryptionConfig(algorithm="AES-256", key_id="prod")
            >>> store.store("art-1", {}, config)
            >>> retrieved_config = store.get_config("art-1")
            >>> retrieved_config.key_id
            "prod"
        """
        if artifact_id not in self._configs:
            raise KeyError(f"Artifact {artifact_id!r} not found")

        return self._configs[artifact_id]

    def list_artifacts(self) -> list[str]:
        """List all stored artifact IDs.

        Returns:
            List of artifact identifiers.

        Example:
            >>> store = EncryptedArtifactStore()
            >>> store.store("art-1", {})
            >>> store.store("art-2", {})
            >>> sorted(store.list_artifacts())
            ["art-1", "art-2"]
        """
        result = list(self._artifacts.keys())
        logger.debug("Listed artifacts: count=%d", len(result))
        return result
