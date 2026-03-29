"""Connector sandbox project mode tracking and promotion.

Manages connectors in sandbox mode for testing before production promotion.

FR traceability: WL-274 (Connector Sandbox Project Mode)
# @trace WL-274
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

logger = logging.getLogger(__name__)


@dataclass
class SandboxConnector:
    """A connector registered with sandbox mode status."""

    connector_id: str
    project_id: str
    sandbox: bool = True


class ConnectorSandboxRegistry:
    """Manages sandbox connectors and promotion to production."""

    def __init__(self) -> None:
        """Initialize the connector sandbox registry."""
        self._connectors: dict[str, SandboxConnector] = {}
        logger.debug("Initialized connector sandbox registry")

    def register(self, connector_id: str, project_id: str, sandbox: bool = True) -> SandboxConnector:
        """Register a connector with sandbox mode status.

        Args:
            connector_id: Unique connector identifier.
            project_id: Associated project identifier.
            sandbox: Whether connector is in sandbox mode (default: True).

        Returns:
            The registered SandboxConnector.
        """
        connector = SandboxConnector(
            connector_id=connector_id,
            project_id=project_id,
            sandbox=sandbox,
        )
        self._connectors[connector_id] = connector
        mode = "sandbox" if sandbox else "production"
        logger.debug(f"Registered connector {connector_id} in {mode} mode")
        return connector

    def is_sandbox(self, connector_id: str) -> bool:
        """Check if a connector is in sandbox mode.

        Args:
            connector_id: Connector identifier.

        Returns:
            True if registered and in sandbox mode, False otherwise.
        """
        connector = self._connectors.get(connector_id)
        if connector is None:
            return False

        return connector.sandbox

    def all_sandbox(self) -> list[SandboxConnector]:
        """Get all connectors currently in sandbox mode.

        Returns:
            List of SandboxConnector objects in sandbox mode.
        """
        return [c for c in self._connectors.values() if c.sandbox]

    def promote(self, connector_id: str) -> None:
        """Promote a connector from sandbox to production mode.

        Args:
            connector_id: Connector identifier.

        Raises:
            ValueError: If connector is not registered.
        """
        connector = self._connectors.get(connector_id)
        if connector is None:
            raise ValueError(f"Connector not registered: {connector_id}")

        connector.sandbox = False
        logger.debug(f"Promoted connector {connector_id} to production mode")
