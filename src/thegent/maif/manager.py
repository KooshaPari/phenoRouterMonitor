"""MAIF manager for artifact lifecycle management.

WBS: MAIF implementation
FR Traceability: FR-AUDIT-001
"""

from __future__ import annotations

from pathlib import Path

from thegent.maif.artifacts import (
    MAIFArtifact,
    generate_key_pair,
    sign_artifact,
    verify_artifact,
)
from thegent.maif.store import MAIFArtifactStore


class MAIFManager:
    """Manager for MAIF artifact lifecycle.

    Handles key generation, artifact creation, signing, and verification.
    """

    def __init__(self, db_path: Path) -> None:
        self.db_path = db_path
        self.store = MAIFArtifactStore(db_path)

        # Generate or load keys
        self.private_key, self.public_key = generate_key_pair()

    def create_artifact(
        self,
        action_type: str,
        payload: dict,
        agent_id: str,
        session_id: str,
    ) -> MAIFArtifact:
        """Create and sign an artifact.

        The artifact is automatically signed with the manager's private key.
        """
        artifact = MAIFArtifact(
            action_type=action_type,
            payload=payload,
            agent_id=agent_id,
            session_id=session_id,
        )

        # Sign the artifact
        sign_artifact(artifact, self.private_key)

        # Store it
        self.store.store(artifact)

        return artifact

    def verify(self, artifact_id: str) -> bool:
        """Verify an artifact's signature.

        Returns True if valid, False otherwise.
        """
        artifact = self.store.get(artifact_id)
        if artifact is None:
            return False

        return verify_artifact(artifact, self.private_key)

    def get_artifact(self, artifact_id: str) -> MAIFArtifact | None:
        """Get an artifact by ID."""
        return self.store.get(artifact_id)

    def get_session_history(self, session_id: str) -> list[MAIFArtifact]:
        """Get all artifacts for a session."""
        return self.store.list_by_session(session_id)

    def get_agent_history(self, agent_id: str) -> list[MAIFArtifact]:
        """Get all artifacts for an agent."""
        return self.store.list_by_agent(agent_id)
