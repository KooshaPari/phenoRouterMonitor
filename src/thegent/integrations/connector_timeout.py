"""Per-connector timeout controls.

Manages per-connector timeout configurations for reliable request handling.

# @trace WL-193
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

logger = logging.getLogger(__name__)

_INVALID_DEFAULT_TIMEOUT = "default_timeout must be > 0"
_INVALID_TIMEOUT_SECONDS = "timeout_seconds must be > 0"


@dataclass
class ConnectorTimeoutConfig:
    """Configuration for connector timeout."""

    connector_id: str
    timeout_seconds: float = 30.0


class ConnectorTimeoutRegistry:
    """Registry for per-connector timeout configurations."""

    def __init__(self, default_timeout: float = 30.0) -> None:
        """Initialize the timeout registry.

        Args:
            default_timeout: Default timeout in seconds for all connectors.

        Raises:
            ValueError: If default_timeout <= 0.

        """
        if default_timeout <= 0:
            msg = _INVALID_DEFAULT_TIMEOUT
            raise ValueError(msg)

        self._default_timeout = default_timeout
        self._timeouts: dict[str, float] = {}

        logger.debug("Initialized ConnectorTimeoutRegistry with default_timeout=%s", default_timeout)

    def set_timeout(self, connector_id: str, timeout_seconds: float) -> None:
        """Set the timeout for a specific connector.

        Args:
            connector_id: The connector identifier.
            timeout_seconds: Timeout in seconds.

        Raises:
            ValueError: If timeout_seconds <= 0.

        """
        if timeout_seconds <= 0:
            msg = _INVALID_TIMEOUT_SECONDS
            raise ValueError(msg)

        self._timeouts[connector_id] = timeout_seconds
        logger.debug("Set timeout for %s to %s", connector_id, timeout_seconds)

    def get_timeout(self, connector_id: str) -> float:
        """Get the timeout for a connector.

        Args:
            connector_id: The connector identifier.

        Returns:
            Timeout in seconds. Returns default if not explicitly configured.
        """
        return self._timeouts.get(connector_id, self._default_timeout)

    def remove(self, connector_id: str) -> None:
        """Remove custom timeout configuration for a connector.

        After removal, the connector will use the default timeout.

        Args:
            connector_id: The connector identifier.

        """
        self._timeouts.pop(connector_id, None)
        logger.debug("Removed custom timeout for %s, will use default", connector_id)

    def all_configs(self) -> list[ConnectorTimeoutConfig]:
        """Get all configured timeouts.

        Returns:
            List of ConnectorTimeoutConfig for all connectors with custom timeouts.
        """
        return [
            ConnectorTimeoutConfig(connector_id=cid, timeout_seconds=timeout)
            for cid, timeout in sorted(self._timeouts.items())
        ]
