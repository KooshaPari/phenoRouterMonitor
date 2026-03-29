"""Encryption adapter for workstream autosync.

Handles artifact encryption/decryption using XOR cipher.
"""

import base64
import hashlib


def xor_encrypt(data: bytes, key: str) -> str:
    """Encrypt data using XOR cipher with key.

    Args:
        data: Raw bytes to encrypt
        key: Encryption key string

    Returns:
        Base64-encoded encrypted string
    """
    key_bytes = key.encode()
    key_hash = hashlib.sha256(key_bytes).digest()

    encrypted = bytearray()
    for i, byte in enumerate(data):
        encrypted.append(byte ^ key_hash[i % len(key_hash)])

    return base64.b64encode(bytes(encrypted)).decode()


def xor_decrypt(payload: str, key: str) -> str:
    """Decrypt XOR-encrypted payload.

    Args:
        payload: Base64-encoded encrypted string
        key: Encryption key string

    Returns:
        Decrypted string
    """
    key_bytes = key.encode()
    key_hash = hashlib.sha256(key_bytes).digest()

    encrypted = base64.b64decode(payload.encode())

    decrypted = bytearray()
    for i, byte in enumerate(encrypted):
        decrypted.append(byte ^ key_hash[i % len(key_hash)])

    return bytes(decrypted).decode()


def compute_artifact_key(actor_id: str, artifact_id: str) -> str:
    """Compute encryption key for artifact.

    Args:
        actor_id: Actor identifier
        artifact_id: Artifact identifier

    Returns:
        Encryption key string
    """
    combined = f"{actor_id}:{artifact_id}"
    return hashlib.sha256(combined.encode()).hexdigest()[:32]


__all__ = ["compute_artifact_key", "xor_decrypt", "xor_encrypt"]
