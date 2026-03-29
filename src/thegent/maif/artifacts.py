"""MAIF (Machine Action Interpretation Framework) artifacts.

WBS: MAIF implementation
FR Traceability: FR-AUDIT-001
"""

from __future__ import annotations

import hashlib
import hmac
import json
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any


@dataclass
class MAIFArtifact:
    """A signed artifact representing a machine action.

    Artifacts are cryptographically signed to enable tamper detection.
    """

    action_type: str
    payload: dict[str, Any]
    agent_id: str
    session_id: str
    artifact_id: str | None = None
    signature: str | None = None
    timestamp: str | None = None

    def __post_init__(self) -> None:
        """Generate artifact ID and timestamp if not provided."""
        if self.artifact_id is None:
            self.artifact_id = self._generate_id()
        if self.timestamp is None:
            self.timestamp = datetime.utcnow().isoformat()

    def _generate_id(self) -> str:
        """Generate a unique artifact ID."""
        content = f"{self.action_type}:{self.agent_id}:{self.session_id}:{self.timestamp}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]

    def to_signable_content(self) -> str:
        """Get the content to be signed."""
        return json.dumps(
            {
                "action_type": self.action_type,
                "payload": self.payload,
                "agent_id": self.agent_id,
                "session_id": self.session_id,
                "artifact_id": self.artifact_id,
                "timestamp": self.timestamp,
            },
            sort_keys=True,
        )


def generate_key_pair() -> tuple[str, str]:
    """Generate a key pair for artifact signing.

    Returns (private_key, public_key) as hex strings.
    """
    import secrets

    private_key = secrets.token_hex(32)
    public_key = hashlib.sha256(bytes.fromhex(private_key)).hexdigest()
    return private_key, public_key


def sign_artifact(artifact: MAIFArtifact, private_key: str) -> str:
    """Sign an artifact with the private key.

    Returns the signature.
    """
    content = artifact.to_signable_content()
    signature = hmac.new(
        bytes.fromhex(private_key),
        content.encode(),
        hashlib.sha256,
    ).hexdigest()
    artifact.signature = signature
    return signature


def verify_artifact(artifact: MAIFArtifact, signing_key: str) -> bool:
    """Verify an artifact's signature.

    Args:
        artifact: The artifact to verify.
        signing_key: The key used to sign (private key hex string).

    Returns True if valid, False otherwise.
    """
    if artifact.signature is None:
        return False

    # Recreate the signature with the same key
    content = artifact.to_signable_content()
    expected_signature = hmac.new(
        bytes.fromhex(signing_key),
        content.encode(),
        hashlib.sha256,
    ).hexdigest()

    return hmac.compare_digest(artifact.signature, expected_signature)
