# @trace WL-051
"""WL-051: API key rotation monitoring and webhook notification.

Implements:
  - ApiKeyRecord: key metadata with expiry tracking
  - KeyRotationMonitor: warns when keys expire within 7 days
  - KeyRotationWebhook: posts rotation events to a configurable URL (httpx)
"""

from __future__ import annotations

import orjson as json
import logging
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

import httpx
from pydantic import BaseModel, ConfigDict, Field

_log = logging.getLogger(__name__)

_DEFAULT_WARN_DAYS = 7
_DEFAULT_KEY_REGISTRY_PATH = Path.home() / ".thegent" / "keys" / "registry.jsonl"

# ---------------------------------------------------------------------------
# ApiKeyRecord
# ---------------------------------------------------------------------------


class ApiKeyRecord(BaseModel):
    """Persistent metadata for an API key (WL-051)."""

    model_config = ConfigDict(extra="forbid", frozen=True)

    key_id: str = Field(min_length=1)
    provider: str = Field(min_length=1)
    expires_at: str = Field(min_length=1, description="ISO-8601 UTC datetime")
    last_rotated: str = Field(min_length=1, description="ISO-8601 UTC datetime")
    description: str = Field(default="")

    def expires_at_dt(self) -> datetime:
        return datetime.fromisoformat(self.expires_at)

    def days_until_expiry(self) -> float:
        """Return days remaining until expiry (negative = already expired)."""
        delta = self.expires_at_dt() - datetime.now(UTC)
        return delta.total_seconds() / 86400.0

    def is_expiring_soon(self, warn_days: int = _DEFAULT_WARN_DAYS) -> bool:
        """Return True if the key expires within `warn_days` days."""
        return self.days_until_expiry() <= warn_days

    def is_expired(self) -> bool:
        return self.days_until_expiry() < 0


# ---------------------------------------------------------------------------
# Key registry (simple JSONL store)
# ---------------------------------------------------------------------------


class KeyRegistry:
    """Persists ApiKeyRecord entries in a JSONL file (WL-051)."""

    def __init__(self, registry_path: Path | str = _DEFAULT_KEY_REGISTRY_PATH) -> None:
        self.registry_path = Path(registry_path).expanduser()

    def add(self, record: ApiKeyRecord) -> None:
        """Append a key record. Raises ValueError if key_id already exists."""
        existing = {r.key_id for r in self.list_all()}
        if record.key_id in existing:
            raise ValueError(f"key_id already registered: {record.key_id!r}")
        self.registry_path.parent.mkdir(parents=True, exist_ok=True)
        with self.registry_path.open("a", encoding="utf-8") as f:
            f.write(record.model_dump_json() + "\n")

    def list_all(self) -> list[ApiKeyRecord]:
        if not self.registry_path.exists():
            return []
        records: list[ApiKeyRecord] = []
        with self.registry_path.open("r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                records.append(ApiKeyRecord.model_validate_json(line))
        return records

    def get(self, key_id: str) -> ApiKeyRecord:
        for r in self.list_all():
            if r.key_id == key_id:
                return r
        raise KeyError(f"ApiKeyRecord not found: {key_id!r}")

    def update(self, updated: ApiKeyRecord) -> None:
        """Replace the record with the same key_id. Raises KeyError if not found."""
        all_records = self.list_all()
        found = False
        tmp = self.registry_path.with_suffix(".jsonl.tmp")
        with tmp.open("w", encoding="utf-8") as f:
            for rec in all_records:
                if rec.key_id == updated.key_id:
                    f.write(updated.model_dump_json() + "\n")
                    found = True
                else:
                    f.write(rec.model_dump_json() + "\n")
        if not found:
            tmp.unlink()
            raise KeyError(f"ApiKeyRecord not found for update: {updated.key_id!r}")
        tmp.replace(self.registry_path)


# ---------------------------------------------------------------------------
# KeyRotationMonitor
# ---------------------------------------------------------------------------


class KeyRotationWarning:
    """Structured warning emitted by KeyRotationMonitor."""

    def __init__(self, record: ApiKeyRecord, warn_days: int) -> None:
        self.record = record
        self.warn_days = warn_days
        self.days_remaining = record.days_until_expiry()

    def to_dict(self) -> dict[str, Any]:
        return {
            "key_id": self.record.key_id,
            "provider": self.record.provider,
            "expires_at": self.record.expires_at,
            "days_remaining": round(self.days_remaining, 2),
            "warn_threshold_days": self.warn_days,
            "message": (
                f"API key '{self.record.key_id}' (provider={self.record.provider}) "
                f"expires in {self.days_remaining:.1f} days."
            ),
        }

    def __repr__(self) -> str:
        return f"KeyRotationWarning(key_id={self.record.key_id!r}, days={self.days_remaining:.1f})"


class KeyRotationMonitor:
    """Monitors API keys and emits warnings for keys expiring within the threshold (WL-051).

    Usage::

        monitor = KeyRotationMonitor(registry)
        warnings = monitor.check_all()
        for w in warnings:
            print(w.to_dict())
    """

    def __init__(
        self,
        registry: KeyRegistry,
        warn_days: int = _DEFAULT_WARN_DAYS,
    ) -> None:
        self.registry = registry
        self.warn_days = warn_days

    def check_all(self) -> list[KeyRotationWarning]:
        """Return warnings for all keys expiring within warn_days (or already expired)."""
        warnings: list[KeyRotationWarning] = []
        for record in self.registry.list_all():
            if record.is_expiring_soon(self.warn_days):
                w = KeyRotationWarning(record, self.warn_days)
                _log.warning(
                    "Key expiry warning: key_id=%s provider=%s days_remaining=%.1f",
                    record.key_id,
                    record.provider,
                    w.days_remaining,
                )
                warnings.append(w)
        return warnings

    def check_provider(self, provider: str) -> list[KeyRotationWarning]:
        """Return warnings for keys of a specific provider."""
        all_warnings = self.check_all()
        return [w for w in all_warnings if w.record.provider == provider]


# ---------------------------------------------------------------------------
# KeyRotationWebhook
# ---------------------------------------------------------------------------


class KeyRotationWebhook:
    """Posts key rotation events to a configurable URL via httpx (WL-051).

    The webhook payload format::

        {
            "event": "key_rotation",
            "key_id": "...",
            "provider": "...",
            "rotated_at": "...",
            "prev_expires_at": "...",
            "new_expires_at": "...",
        }
    """

    def __init__(
        self,
        webhook_url: str,
        registry: KeyRegistry,
        timeout_seconds: float = 10.0,
    ) -> None:
        if not webhook_url:
            raise ValueError("webhook_url must not be empty")
        self.webhook_url = webhook_url
        self.registry = registry
        self.timeout_seconds = timeout_seconds

    def rotate(
        self,
        *,
        key_id: str,
        new_expires_at: str,
        updated_last_rotated: str | None = None,
    ) -> dict[str, Any]:
        """Record a key rotation and notify the webhook.

        Updates the key registry with new_expires_at, then POSTs a rotation
        event to webhook_url.  Raises httpx.HTTPError on non-2xx response.

        Returns the webhook response payload (or empty dict if dry-run with
        no URL set).
        """
        old_record = self.registry.get(key_id)
        rotated_at = updated_last_rotated or datetime.now(UTC).isoformat()

        updated = ApiKeyRecord(
            key_id=old_record.key_id,
            provider=old_record.provider,
            expires_at=new_expires_at,
            last_rotated=rotated_at,
            description=old_record.description,
        )
        self.registry.update(updated)

        payload: dict[str, Any] = {
            "event": "key_rotation",
            "key_id": key_id,
            "provider": old_record.provider,
            "rotated_at": rotated_at,
            "prev_expires_at": old_record.expires_at,
            "new_expires_at": new_expires_at,
        }

        _log.info("Posting key rotation event for key_id=%s to %s", key_id, self.webhook_url)
        response = httpx.post(
            self.webhook_url,
            content=json.dumps(payload).decode().encode(),
            headers={"Content-Type": "application/json"},
            timeout=self.timeout_seconds,
        )
        response.raise_for_status()

        result: dict[str, Any] = {"sent": True, "status_code": response.status_code, "payload": payload}
        _log.info("Webhook response: status=%d for key_id=%s", response.status_code, key_id)
        return result

    def build_rotation_payload(self, key_id: str, new_expires_at: str) -> dict[str, Any]:
        """Build the webhook payload without executing the rotation (for inspection/testing)."""
        record = self.registry.get(key_id)
        return {
            "event": "key_rotation",
            "key_id": key_id,
            "provider": record.provider,
            "rotated_at": datetime.now(UTC).isoformat(),
            "prev_expires_at": record.expires_at,
            "new_expires_at": new_expires_at,
        }


# ---------------------------------------------------------------------------
# Convenience: build a key that expires in N days from now
# ---------------------------------------------------------------------------


def make_expiry_utc(days_from_now: int) -> str:
    """Return an ISO-8601 UTC datetime string for N days from now."""
    return (datetime.now(UTC) + timedelta(days=days_from_now)).isoformat()
