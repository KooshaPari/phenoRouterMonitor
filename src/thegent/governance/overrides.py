"""WP-3003: Override path with TTL and revalidation (FR-011)."""

import json
import logging
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, cast

from thegent.config import ThegentSettings
from thegent.integrations.base import SerializableMixin

_log = logging.getLogger(__name__)


@dataclass
class PolicyOverride(SerializableMixin):
    """An active override for a governance policy."""

    policy_id: str
    reason: str
    by: str
    expires_at: float
    created_at: float
    metadata: dict[str, Any]

    def is_active(self) -> bool:
        """Check if the override is still valid."""
        return time.time() < self.expires_at


class OverrideManager:
    """Manages temporary policy overrides."""

    def __init__(self, settings: ThegentSettings | None = None) -> None:
        self.settings = settings or ThegentSettings()
        self.override_dir = self.settings.session_dir / "overrides"
        self.override_dir.mkdir(parents=True, exist_ok=True)

    def apply_override(
        self,
        policy_id: str,
        reason: str,
        by: str,
        duration_minutes: int = 60,
        metadata: dict[str, Any | None] | None = None,
    ) -> PolicyOverride:
        """Create a new temporary override."""
        now = time.time()
        expires_at = now + (duration_minutes * 60)

        override = PolicyOverride(
            policy_id=policy_id,
            reason=reason,
            by=by,
            expires_at=expires_at,
            created_at=now,
            metadata=metadata or {},
        )

        self._save_override(override)
        _log.info("Applied override for policy %s by %s for %d min", policy_id, by, duration_minutes)
        return override

    def get_override(self, policy_id: str) -> PolicyOverride | None:
        """Get an active override for a policy."""
        p = self.override_dir / f"{policy_id}.json"
        if not p.exists():
            return None

        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            override = cast("PolicyOverride", PolicyOverride.from_dict(data))

            if override.is_active():
                return override
            # WP-3003: Emit governance.override.expired when override has expired
            _log.info(
                "governance.override.expired policy_id=%s by=%s expires_at=%s",
                policy_id,
                override.by,
                override.expires_at,
            )
            # Cleanup expired override
            p.unlink()
            return None
        except Exception as e:
            _log.error("Failed to load override %s: %s", policy_id, e)
            return None

    def cleanup_expired(self) -> int:
        """Remove all expired overrides from disk."""
        count = 0
        for f in self.override_dir.glob("*.json"):
            if self._cleanup_if_expired(f):
                count += 1
        return count

    def _cleanup_if_expired(self, f: Path) -> bool:
        """Helper to check and cleanup a single override file."""
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
            override = cast("PolicyOverride", PolicyOverride.from_dict(data))
            if not override.is_active():
                f.unlink()
                return True
        except Exception as e:
            _log.error("Failed to cleanup override %s: %s", f, e)
        return False

    def _save_override(self, override: PolicyOverride) -> None:
        """Save override to disk."""
        p = self.override_dir / f"{override.policy_id}.json"
        p.write_text(json.dumps(override.to_dict(), indent=2), encoding="utf-8")
