"""Agent identity and sovereignty for thegent (WP-6004)."""

import hashlib
import logging
from typing import Any

from pydantic import BaseModel

_log = logging.getLogger(__name__)


class AgentDIDDocument(BaseModel):
    """W3C-compliant DID Document for an agent."""

    id: str
    verification_method: list[dict[str, Any]] = []
    service: list[dict[str, Any]] = []
    authentication: list[str] = []


class VerifiableCredential(BaseModel):
    """Proof of agent capability or identity."""

    id: str
    type: list[str]
    issuer: str
    issuance_date: str
    credential_subject: dict[str, Any]
    proof: dict[str, Any]


class AgentIdentity:
    """WP-6004: Manages agent identity, DID, and signing keys."""

    def __init__(self, agent_name: str, swarm_id: str = "default") -> None:
        self.agent_name = agent_name
        self.swarm_id = swarm_id
        self.did = f"did:thegent:{swarm_id}:{agent_name}"
        self.public_key = f"key_{agent_name}_pub"  # Mock
        self._private_key = f"key_{agent_name}_priv"  # Mock

    def get_did_document(self) -> AgentDIDDocument:
        """Return the agent's DID document."""
        return AgentDIDDocument(
            id=self.did,
            verification_method=[
                {
                    "id": f"{self.did}#key-1",
                    "type": "Ed25519VerificationKey2020",
                    "controller": self.did,
                    "publicKeyMultibase": self.public_key,
                }
            ],
            authentication=[f"{self.did}#key-1"],
        )

    def sign(self, data: str) -> str:
        """Sign data with agent's private key (mocked)."""
        import hashlib

        # WP-6004: Non-repudiable signature
        raw = f"{self._private_key}|{data}"
        return hashlib.sha256(raw.encode()).hexdigest()

    def verify(self, data: str, signature: str) -> bool:
        """Verify signature with agent's public key (mocked)."""
        # In a mock, we just check if it matches our "private key" hash
        return signature == self.sign(data)


def sign_actor_payload(actor_id: str, payload: str, signing_key: str) -> str:
    """Produce deterministic actor signature for autosync write boundaries."""
    if not actor_id.strip():
        raise ValueError("actor_id must be non-empty")
    if not signing_key.strip():
        raise ValueError("signing_key must be non-empty")
    return hashlib.sha256(f"{actor_id}|{payload}|{signing_key}".encode()).hexdigest()


def verify_actor_signature(*, actor_id: str, payload: str, signing_key: str, signature: str) -> bool:
    """Verify actor signature using deterministic hash scheme."""
    if not signature.strip():
        return False
    expected = sign_actor_payload(actor_id=actor_id, payload=payload, signing_key=signing_key)
    return signature == expected
