"""Provenance metadata for sync operations.

Adds per-item provenance stamps to tracked sync records with sync ID,
timestamp, source, operator, and cycle number.

FR traceability: WL-201 (Sync Provenance Stamps)
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from datetime import datetime, UTC
from typing import Any
from uuid import uuid4

from thegent.integrations.base import SerializableMixin

logger = logging.getLogger(__name__)

# Provenance metadata key in records
_PROVENANCE_KEY = "__provenance__"
_OWNER_KEYS = ("owner", "github_owner", "linear_assignee", "assignee")
_SYNC_METADATA_KEY = "__sync_metadata__"


@dataclass
class SyncProvenanceStamp(SerializableMixin):
    """Provenance metadata for a sync operation.

    Attributes:
        sync_id: Unique identifier for the sync operation.
        timestamp: ISO 8601 timestamp of when the sync occurred.
        source: Source system (e.g., "github", "linear", "board").
        operator: The operator or service that performed the sync.
        cycle_number: Monotonic cycle counter for repeated syncs.
    """

    sync_id: str
    timestamp: str
    source: str
    operator: str
    cycle_number: int
    correlation_id: str | None = None
    prev_hash: str = ""
    signature: str = ""

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> SyncProvenanceStamp:
        """Create a stamp from a dictionary.

        Args:
            data: Dictionary with required keys.

        Returns:
            A new SyncProvenanceStamp instance.

        Raises:
            KeyError: If required fields are missing.
        """
        return cls(
            sync_id=data["sync_id"],
            timestamp=data["timestamp"],
            source=data["source"],
            operator=data["operator"],
            cycle_number=data["cycle_number"],
            prev_hash=str(data.get("prev_hash", "")),
            signature=str(data.get("signature", "")),
        )

    def canonical_payload(self) -> str:
        """Render a deterministic payload used for hash/signature generation."""
        return f"{self.sync_id}|{self.timestamp}|{self.source}|{self.operator}|{self.cycle_number}|{self.prev_hash}"


def sign_provenance_stamp(stamp: SyncProvenanceStamp, secret: str) -> str:
    """Create a deterministic signature for a provenance stamp."""
    if not secret:
        raise ValueError("secret must be non-empty")
    import hashlib

    return hashlib.sha256(f"{stamp.canonical_payload()}|{secret}".encode()).hexdigest()


def verify_provenance_signature(stamp: SyncProvenanceStamp, secret: str) -> bool:
    """Verify that the signature matches the stamp payload."""
    if not stamp.signature:
        return False
    return sign_provenance_stamp(stamp, secret) == stamp.signature


def chain_provenance_stamps(stamps: list[SyncProvenanceStamp], secret: str) -> list[SyncProvenanceStamp]:
    """Attach deterministic hash chain fields and signatures to stamps."""
    import hashlib

    prev_hash = ""
    chained: list[SyncProvenanceStamp] = []
    for stamp in stamps:
        with_prev = SyncProvenanceStamp(
            sync_id=stamp.sync_id,
            timestamp=stamp.timestamp,
            source=stamp.source,
            operator=stamp.operator,
            cycle_number=stamp.cycle_number,
            prev_hash=prev_hash,
            signature="",
        )
        digest = hashlib.sha256(with_prev.canonical_payload().encode("utf-8")).hexdigest()
        signed = SyncProvenanceStamp(
            sync_id=with_prev.sync_id,
            timestamp=with_prev.timestamp,
            source=with_prev.source,
            operator=with_prev.operator,
            cycle_number=with_prev.cycle_number,
            prev_hash=with_prev.prev_hash,
            signature=sign_provenance_stamp(with_prev, secret),
        )
        chained.append(signed)
        prev_hash = digest
    return chained


def verify_provenance_chain(stamps: list[SyncProvenanceStamp], secret: str) -> tuple[bool, str]:
    """Verify prev-hash continuity and signatures for a chain of stamps."""
    import hashlib

    expected_prev = ""
    for index, stamp in enumerate(stamps):
        if stamp.prev_hash != expected_prev:
            return False, f"chain break at index {index}: expected prev_hash={expected_prev}, got {stamp.prev_hash}"
        if not verify_provenance_signature(stamp, secret):
            return False, f"signature verification failed at index {index}"
        expected_prev = hashlib.sha256(stamp.canonical_payload().encode("utf-8")).hexdigest()
    return True, ""


def stamp_sync_record(
    record: dict[str, Any],
    stamp: SyncProvenanceStamp,
) -> dict[str, Any]:
    """Attach a provenance stamp to a record.

    Creates a shallow copy of the record with the stamp attached.

    Args:
        record: The record to stamp (dict).
        stamp: The provenance stamp to attach.

    Returns:
        A new dict with the stamp attached under the provenance key.

    Raises:
        ValueError: If record is not a dictionary.
    """
    if not isinstance(record, dict):
        raise ValueError("record must be a dictionary")

    result = record.copy()
    result[_PROVENANCE_KEY] = stamp.to_dict()
    return result


def extract_provenance(record: dict[str, Any]) -> SyncProvenanceStamp | None:
    """Extract the provenance stamp from a record.

    Args:
        record: The record to extract from.

    Returns:
        The SyncProvenanceStamp if present, None otherwise.

    Raises:
        ValueError: If the provenance data is malformed.
    """
    if not isinstance(record, dict):
        raise ValueError("record must be a dictionary")

    if _PROVENANCE_KEY not in record:
        return None

    prov_data = record[_PROVENANCE_KEY]
    if not isinstance(prov_data, dict):
        raise ValueError("Provenance data must be a dictionary")

    try:
        return SyncProvenanceStamp.from_dict(prov_data)
    except KeyError as e:
        raise ValueError(f"Malformed provenance data: missing key {e}") from e


def has_provenance(record: dict[str, Any]) -> bool:
    """Check if a record has provenance metadata.

    Args:
        record: The record to check.

    Returns:
        True if the record has provenance metadata.
    """
    return isinstance(record, dict) and _PROVENANCE_KEY in record


def remove_provenance(record: dict[str, Any]) -> dict[str, Any]:
    """Remove provenance metadata from a record.

    Creates a shallow copy without the provenance key.

    Args:
        record: The record to clean.

    Returns:
        A new dict without provenance metadata.
    """
    if not isinstance(record, dict):
        raise ValueError("record must be a dictionary")

    result = record.copy()
    result.pop(_PROVENANCE_KEY, None)
    return result


def get_current_timestamp() -> str:
    """Get the current timestamp in ISO 8601 format.

    Returns:
        ISO 8601 formatted timestamp.
    """
    return datetime.now(tz=UTC).isoformat().replace("+00:00", "Z")


def new_run_correlation_id() -> str:
    """Return a run-scoped correlation identifier."""
    return f"run-{uuid4()}"


def canonical_owner(record: dict[str, Any]) -> str | None:
    """Resolve canonical owner across local/GitHub/Linear field variants."""
    for key in _OWNER_KEYS:
        value = record.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return None


def propagate_owner_metadata(record: dict[str, Any], owner: str) -> dict[str, Any]:
    """Propagate canonical owner into connector-specific owner fields."""
    normalized = owner.strip()
    if not normalized:
        raise ValueError("owner must not be empty")
    result = record.copy()
    result["owner"] = normalized
    result["github_owner"] = normalized
    result["linear_assignee"] = normalized
    return result


def enrich_sync_metadata(
    record: dict[str, Any],
    *,
    source_url: str,
    source_tag: str,
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Attach standardized metadata enrichment fields to a sync record."""
    if not isinstance(record, dict):
        raise ValueError("record must be a dictionary")
    if not source_url.strip():
        raise ValueError("source_url must be non-empty")
    if not source_tag.strip():
        raise ValueError("source_tag must be non-empty")

    enriched = record.copy()
    metadata = {
        "source_url": source_url.strip(),
        "source_tag": source_tag.strip().lower(),
    }
    if extra:
        metadata.update(extra)
    enriched[_SYNC_METADATA_KEY] = metadata
    return enriched
