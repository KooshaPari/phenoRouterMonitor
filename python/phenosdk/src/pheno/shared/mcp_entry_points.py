"""
Shared MCP entry point utilities for phenoSDK.

This module contains common utilities and base classes for MCP entry points.
"""

from abc import ABC, abstractmethod
from typing import Dict, Any, List, Optional


class BaseMCPEntryPoint(ABC):
    """Abstract base class for MCP entry points.

    Defines the interface for MCP entry point implementations.
    """

    @abstractmethod
    def initialize(self) -> None:
        """Initialize the entry point."""
        pass

    @abstractmethod
    def validate(self) -> bool:
        """Validate the entry point configuration."""
        pass


class MCPConfiguration:
    """Configuration handler for MCP integration.

    Manages settings for MCP server connectivity and authentication.
    """

    def __init__(self):
        """Initialize MCP configuration."""
        self.endpoints: List[str] = []
        self.credentials: Dict[str, Any] = {}
        self.feature_flags: Dict[str, bool] = {
            "migration_mode": False,
            "legacy_support": True,
        }

    def add_endpoint(self, endpoint: str) -> None:
        """Add an MCP server endpoint URL.

        Args:
            endpoint: URL for MCP server
        """
        if endpoint not in self.endpoints:
            self.endpoints.append(endpoint)

    def get_config(self) -> Dict[str, Any]:
        """Get the current MCP configuration.

        Returns:
            Dictionary containing MCP configuration
        """
        return {
            "endpoints": self.endpoints,
            "credentials": self.credentials,
            "flags": self.feature_flags,
        }


class MCPEntryPointRegistry:
    """Registry for managing multiple MCP entry points.

    Provides centralized management of MCP entry points
    for the phenoSDK infrastructure system.
    """

    def __init__(self):
        """Initialize the MCP entry point registry."""
        self._entries: Dict[str, BaseMCPEntryPoint] = {}
        self._primary_endpoint: Optional[str] = None

    def register_entry(self, name: str, entry: BaseMCPEntryPoint) -> None:
        """Register an MCP entry point.

        Args:
            name: Name of the entry point
            entry: The entry point instance
        """
        self._entries[name] = entry

    def set_primary_endpoint(self, endpoint: str) -> None:
        """Set the primary MCP endpoint.

        Args:
            endpoint: The primary endpoint URL
        """
        self._primary_endpoint = endpoint

    def get_entries(self) -> Dict[str, BaseMCPEntryPoint]:
        """Get all registered MCP entry points.

        Returns:
            Dictionary of registered MCP entries
        """
        return dict(self._entries)
