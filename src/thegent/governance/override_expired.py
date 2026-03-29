"""Governance override expiration handling."""

import logging
from datetime import UTC, datetime
from typing import Any

logger = logging.getLogger(__name__)


class OverrideExpirationHandler:
    """Handle governance override expiration."""

    def __init__(self) -> None:
        """Initialize expiration handler."""
        self.overrides: dict[str, dict[str, Any]] = {}

    def register_override(self, override_id: str, expires_at: datetime, policy: str) -> None:
        """Register a governance override.

        Args:
            override_id: Override identifier
            expires_at: Expiration timestamp
            policy: Policy being overridden
        """
        self.overrides[override_id] = {
            "id": override_id,
            "expires_at": expires_at,
            "policy": policy,
            "created_at": datetime.now(UTC),
        }

    def check_expired(self) -> list[dict[str, Any]]:
        """Check for expired overrides.

        Returns:
            List of expired override dictionaries
        """
        now = datetime.now(UTC)
        expired = []

        for override_id, override in list(self.overrides.items()):
            if override["expires_at"] < now:
                expired.append(override)
                self.overrides.pop(override_id)
                logger.warning(f"Override expired: {override_id}")

        return expired

    def emit_expired_event(self, override: dict[str, Any]) -> None:
        """Emit expired override event.

        Args:
            override: Expired override dictionary
        """
        event = {
            "type": "governance.override.expired",
            "override_id": override["id"],
            "policy": override["policy"],
            "expired_at": datetime.now(UTC).isoformat(),
        }
        logger.info(f"Emitted expired event: {event}")
