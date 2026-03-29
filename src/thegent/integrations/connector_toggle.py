"""Runtime connector enable/disable toggle controls.

Manages a registry of connectors with enable/disable state for runtime control
of connector activation.

FR traceability: WL-306 (Runtime Connector Toggle Controls)
"""

from __future__ import annotations

import logging

logger = logging.getLogger(__name__)


class ConnectorToggleRegistry:
    """Registry for managing connector enabled/disabled states at runtime."""

    def __init__(self) -> None:
        """Initialize the registry with empty state."""
        self._registry: dict[str, bool] = {}

    def register(self, connector: str, enabled: bool = True) -> None:
        """Register a connector with initial enabled state.

        Args:
            connector: Name of the connector.
            enabled: Whether the connector starts enabled (default: True).

        Raises:
            ValueError: If connector is already registered.
        """
        if connector in self._registry:
            raise ValueError(f"Connector {connector} is already registered")

        self._registry[connector] = enabled
        logger.debug(f"Registered connector {connector} (enabled={enabled})")

    def enable(self, connector: str) -> None:
        """Enable a connector.

        Args:
            connector: Name of the connector.

        Raises:
            ValueError: If connector is not registered.
        """
        if connector not in self._registry:
            raise ValueError(f"Connector {connector} is not registered")

        self._registry[connector] = True
        logger.debug(f"Enabled connector {connector}")

    def disable(self, connector: str) -> None:
        """Disable a connector.

        Args:
            connector: Name of the connector.

        Raises:
            ValueError: If connector is not registered.
        """
        if connector not in self._registry:
            raise ValueError(f"Connector {connector} is not registered")

        self._registry[connector] = False
        logger.debug(f"Disabled connector {connector}")

    def is_enabled(self, connector: str) -> bool:
        """Check if a connector is enabled.

        Args:
            connector: Name of the connector.

        Returns:
            True if connector is registered and enabled, False otherwise.
        """
        return self._registry.get(connector, False)

    def toggle(self, connector: str) -> bool:
        """Toggle the state of a connector.

        Args:
            connector: Name of the connector.

        Returns:
            The new state (True if now enabled, False if now disabled).

        Raises:
            ValueError: If connector is not registered.
        """
        if connector not in self._registry:
            raise ValueError(f"Connector {connector} is not registered")

        new_state = not self._registry[connector]
        self._registry[connector] = new_state
        logger.debug(f"Toggled connector {connector} to {new_state}")
        return new_state

    def list_all(self) -> dict[str, bool]:
        """Get a copy of the entire registry.

        Returns:
            Dictionary mapping connector names to enabled state.
        """
        return dict(self._registry)
