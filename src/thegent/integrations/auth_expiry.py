"""Auth token expiry detection for connectors.

# @trace WL-241
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, UTC
from enum import Enum
from typing import Any


class ExpiryStatus(str, Enum):
    """Status of token expiry."""

    VALID = "valid"
    EXPIRING_SOON = "expiring_soon"
    EXPIRED = "expired"


@dataclass
class AuthExpiryInfo:
    """Information about token expiry."""

    status: ExpiryStatus
    expires_at: datetime | None = None
    hours_remaining: float | None = None
    is_critical: bool = False


class AuthExpiryDetector:
    """Detects and monitors auth token expiry."""

    # Threshold: consider token expiring if < 24 hours remain
    EXPIRING_SOON_THRESHOLD_HOURS = 24

    def __init__(self, expiring_soon_threshold_hours: float = 24) -> None:
        """Initialize the auth expiry detector.

        Args:
            expiring_soon_threshold_hours: Hours before expiry to consider "expiring soon".
        """
        self.expiring_soon_threshold_hours = expiring_soon_threshold_hours

    def detect_expiry(self, token_info: dict[str, Any]) -> AuthExpiryInfo:
        """Detect the expiry status of a token.

        Args:
            token_info: Token information dictionary. May contain:
                - expires_at (datetime): When the token expires
                - expiry_timestamp (int): Unix timestamp of expiry
                - ttl (int): Time to live in seconds
                - expires_in (int): Seconds until expiry

        Returns:
            AuthExpiryInfo with status and metadata.

        Raises:
            ValueError: If token_info has no expiry information.
        """
        expires_at = self._extract_expiry_time(token_info)

        if expires_at is None:
            raise ValueError(
                "token_info must contain expiry information (expires_at, expiry_timestamp, ttl, or expires_in)"
            )

        now = datetime.now(UTC)
        hours_remaining = (expires_at - now).total_seconds() / 3600

        if expires_at <= now:
            status = ExpiryStatus.EXPIRED
            is_critical = True
        elif hours_remaining < self.expiring_soon_threshold_hours:
            status = ExpiryStatus.EXPIRING_SOON
            is_critical = hours_remaining < 1  # Critical if < 1 hour
        else:
            status = ExpiryStatus.VALID
            is_critical = False

        return AuthExpiryInfo(
            status=status,
            expires_at=expires_at,
            hours_remaining=max(0, hours_remaining),
            is_critical=is_critical,
        )

    def _extract_expiry_time(self, token_info: dict[str, Any]) -> datetime | None:
        """Extract expiry time from various token_info formats.

        Args:
            token_info: Token information dictionary.

        Returns:
            Expiry datetime in UTC, or None if no expiry info found.
        """
        # Direct datetime
        if "expires_at" in token_info:
            expires_at = token_info["expires_at"]
            if isinstance(expires_at, datetime):
                # Ensure timezone aware
                if expires_at.tzinfo is None:
                    return expires_at.replace(tzinfo=UTC)
                return expires_at
            if isinstance(expires_at, str):
                # Try parsing ISO format
                try:
                    dt = datetime.fromisoformat(expires_at)
                    if dt.tzinfo is None:
                        return dt.replace(tzinfo=UTC)
                    return dt
                except (ValueError, TypeError):
                    return None

        # Unix timestamp
        if "expiry_timestamp" in token_info:
            try:
                timestamp = int(token_info["expiry_timestamp"])
                return datetime.fromtimestamp(timestamp, tz=UTC)
            except (ValueError, TypeError, OSError):
                return None

        # TTL (time to live in seconds)
        if "ttl" in token_info:
            try:
                ttl_seconds = int(token_info["ttl"])
                now = datetime.now(UTC)
                return now + timedelta(seconds=ttl_seconds)
            except (ValueError, TypeError):
                return None

        # expires_in (seconds until expiry)
        if "expires_in" in token_info:
            try:
                expires_in = int(token_info["expires_in"])
                now = datetime.now(UTC)
                return now + timedelta(seconds=expires_in)
            except (ValueError, TypeError):
                return None

        return None

    def is_expired(self, token_info: dict[str, Any]) -> bool:
        """Quick check if token is expired.

        Args:
            token_info: Token information dictionary.

        Returns:
            True if token is expired, False otherwise.
        """
        try:
            info = self.detect_expiry(token_info)
            return info.status == ExpiryStatus.EXPIRED
        except ValueError:
            # No expiry info, assume not expired
            return False

    def is_expiring_soon(self, token_info: dict[str, Any]) -> bool:
        """Quick check if token is expiring soon.

        Args:
            token_info: Token information dictionary.

        Returns:
            True if token is expiring within threshold, False otherwise.
        """
        try:
            info = self.detect_expiry(token_info)
            return info.status in (ExpiryStatus.EXPIRING_SOON, ExpiryStatus.EXPIRED)
        except ValueError:
            # No expiry info, assume not expiring soon
            return False
