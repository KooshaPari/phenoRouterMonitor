"""WP-3002: Signed action artifacts and provenance signatures (FR-010).

BKM-03: When THGENT_USE_NATIVE_CRYPTO=1, uses thegent_crypto Rust extension
for hash/sign/verify. Falls back to Python hashlib/hmac otherwise.
"""

import hashlib
import hmac
import importlib.util
import logging
import os
from datetime import datetime
from typing import Any

import orjson

from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)

_thegent_crypto: Any = None
SIGNING_KEY_ENV_VAR = "THGENT_GOVERNANCE_SIGNING_KEY"


def _get_native_crypto() -> Any:
    """Lazy import of thegent_crypto native extension. Returns None if unavailable."""
    global _thegent_crypto
    if _thegent_crypto is not None:
        return _thegent_crypto
    if not ThegentSettings().use_native_crypto:
        return None
    spec = importlib.util.find_spec("thegent_crypto.thegent_crypto")
    if spec is not None and spec.loader is not None:
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        _thegent_crypto = mod
        return mod
    return None


def _resolve_signing_key(signing_key: str | None) -> str:
    if signing_key:
        return signing_key

    env_key = os.getenv(SIGNING_KEY_ENV_VAR)
    if env_key:
        return env_key

    msg = f"Missing required signing key. Set {SIGNING_KEY_ENV_VAR} or pass signing_key explicitly."
    raise ValueError(msg)


def _canonical_json(data: dict[str, Any]) -> bytes:
    """Canonical JSON (sorted keys) for hashing/signing. Uses orjson for ~10x speedup."""
    return orjson.dumps(data, option=orjson.OPT_SORT_KEYS)


def generate_artifact_hash(data: dict[str, Any]) -> str:
    """Generate SHA-256 hash of a dictionary artifact."""
    canonical = _canonical_json(data)
    native = _get_native_crypto()
    if native is not None:
        return native.artifact_hash_bytes(canonical)
    return hashlib.sha256(canonical).hexdigest()


def sign_artifact(data: dict[str, Any], signing_key: str | None = None) -> str:
    """Produce a provenance signature for an artifact using HMAC-SHA256."""
    canonical = _canonical_json(data)
    key = _resolve_signing_key(signing_key)
    native = _get_native_crypto()
    if native is not None:
        return native.sign_artifact_bytes(canonical, key)
    return hmac.new(
        key.encode("utf-8"),
        canonical,
        hashlib.sha256,
    ).hexdigest()


def verify_signature(data: dict[str, Any], signature: str, signing_key: str | None = None) -> bool:
    """Verify the provenance signature of an artifact."""
    key = _resolve_signing_key(signing_key)
    canonical = _canonical_json(data)
    native = _get_native_crypto()
    if native is not None:
        return native.verify_signature_bytes(canonical, signature, key)
    expected = sign_artifact(data, key)
    return hmac.compare_digest(expected, signature)


class ArtifactSigner:
    """Manager for signing and verifying governance artifacts."""

    def __init__(self, settings: ThegentSettings | None = None) -> None:
        self.settings = settings or ThegentSettings()
        self.signing_key = _resolve_signing_key(None)

    def create_signed_artifact(self, artifact_type: str, payload: dict[str, Any]) -> dict[str, Any]:
        """Create a signed artifact with metadata."""
        envelope = {
            "type": artifact_type,
            "payload": payload,
            "metadata": {
                "hash": generate_artifact_hash(payload),
                "timestamp": datetime.now().isoformat(),
            },
        }
        envelope["signature"] = sign_artifact(envelope, self.signing_key)
        return envelope

    def verify_envelope(self, envelope: dict[str, Any]) -> bool:
        """Verify the signature of an artifact envelope."""
        if "signature" not in envelope:
            return False

        signature = envelope.pop("signature")
        is_valid = verify_signature(envelope, signature, self.signing_key)
        # Restore signature for further use
        envelope["signature"] = signature
        return is_valid
