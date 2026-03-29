"""Connector capability discovery and probing system.

Manages connector capabilities and feature flags for runtime behavior gates.

FR traceability: WL-228 (Connector Capability Discovery)
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

logger = logging.getLogger(__name__)


@dataclass
class ConnectorCapability:
    """Represents a connector and its capabilities.

    # @trace WL-228
    """

    connector_id: str
    """Unique connector identifier."""

    capabilities: list[str]
    """List of capability identifiers (e.g., 'oauth2', 'webhook', 'streaming')."""


class ConnectorCapabilityRegistry:
    """Registry for managing connector capabilities and feature flags.

    # @trace WL-228
    """

    def __init__(self) -> None:
        """Initialize the registry with empty state."""
        self._registry: dict[str, list[str]] = {}

    def register(self, connector_id: str, capabilities: list[str]) -> ConnectorCapability:
        """Register a connector with its capabilities.

        Args:
            connector_id: Unique connector identifier.
            capabilities: List of capability strings.

        Returns:
            ConnectorCapability with the registered details.

        Raises:
            ValueError: If connector is already registered.
        """
        if connector_id in self._registry:
            raise ValueError(f"Connector {connector_id} is already registered")

        self._registry[connector_id] = capabilities
        logger.debug(f"Registered connector {connector_id} with capabilities {capabilities}")
        return ConnectorCapability(connector_id=connector_id, capabilities=capabilities)

    def has_capability(self, connector_id: str, capability: str) -> bool:
        """Check if a connector has a specific capability.

        Args:
            connector_id: Unique connector identifier.
            capability: Capability to check for.

        Returns:
            True if connector has the capability, False otherwise.
        """
        return capability in self._registry.get(connector_id, [])

    def connectors_with(self, capability: str) -> list[str]:
        """Get all connectors that have a specific capability.

        Args:
            capability: Capability to search for.

        Returns:
            List of connector IDs that have the capability.
        """
        return [connector_id for connector_id, capabilities in self._registry.items() if capability in capabilities]

    def get(self, connector_id: str) -> ConnectorCapability:
        """Get capabilities for a connector.

        Args:
            connector_id: Unique connector identifier.

        Returns:
            ConnectorCapability with the connector's details.

        Raises:
            KeyError: If connector is not registered.
        """
        if connector_id not in self._registry:
            raise KeyError(f"Connector {connector_id} not found in registry")

        return ConnectorCapability(connector_id=connector_id, capabilities=self._registry[connector_id])
