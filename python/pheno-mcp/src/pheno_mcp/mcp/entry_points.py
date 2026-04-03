"""MCP Entry Points for pheno-mcp.

This module provides entry point definitions for Model Context Protocol integration.
Generalized to work with any MCP server, not atoms-specific.
"""

from typing import Optional, Dict, Any


class MCPEntryPoint:
    """Entry point for MCP server.

    Provides MCP protocol integration for infrastructure operations.
    This implementation is server-agnostic and supports any MCP-compatible server.
    """

    def __init__(self, name: str, version: str = "1.0.0"):
        """Initialize the MCP entry point.

        Args:
            name: The name of the MCP endpoint
            version: Version string (default: 1.0.0)
        """
        self.name = name
        self.version = version
        self.server_url: Optional[str] = None
        self.api_key: Optional[str] = None
        self._metadata: Dict[str, Any] = {}

    def configure_auth(self, api_key: str) -> None:
        """Configure authentication for MCP server.

        Args:
            api_key: API key for MCP server integration
        """
        self.api_key = api_key

    def set_endpoint_url(self, url: str) -> None:
        """Set the MCP server endpoint URL.

        Args:
            url: The MCP server endpoint URL
        """
        self.server_url = url

    def get_endpoint_url(self) -> Optional[str]:
        """Get the MCP server endpoint URL.

        Returns:
            The configured endpoint URL or None if not set
        """
        return self.server_url

    def set_metadata(self, key: str, value: Any) -> None:
        """Set arbitrary metadata on this entry point.

        Args:
            key: Metadata key
            value: Metadata value
        """
        self._metadata[key] = value

    def get_metadata(self, key: str, default: Any = None) -> Any:
        """Get metadata by key.

        Args:
            key: Metadata key
            default: Default value if key not found

        Returns:
            Metadata value or default
        """
        return self._metadata.get(key, default)

    def to_dict(self) -> Dict[str, Any]:
        """Serialize entry point to dictionary.

        Returns:
            Dictionary representation of the entry point
        """
        return {
            "name": self.name,
            "version": self.version,
            "server_url": self.server_url,
            "api_key": self.api_key if self.api_key else None,
            "metadata": self._metadata,
        }


class MCPServer:
    """MCP Server wrapper.

    Wraps FastMCP or other MCP server implementations with a generic interface.
    """

    def __init__(self, entry_point: MCPEntryPoint):
        """Initialize MCP Server with an entry point.

        Args:
            entry_point: The MCPEntryPoint to use
        """
        self.entry_point = entry_point
        self._is_running = False
        self._tools = []

    def add_tool(self, tool_name: str, tool_callable: Any) -> None:
        """Register a tool with this MCP server.

        Args:
            tool_name: Name of the tool
            tool_callable: Callable that implements the tool
        """
        self._tools.append({"name": tool_name, "callable": tool_callable})

    def get_tools(self) -> list:
        """Get all registered tools.

        Returns:
            List of registered tools
        """
        return self._tools

    def health_check(self) -> Dict[str, Any]:
        """Check the health of the MCP server.

        Returns:
            Health status dictionary
        """
        return {
            "status": "healthy" if self.entry_point.get_endpoint_url() else "unconfigured",
            "name": self.entry_point.name,
            "version": self.entry_point.version,
            "running": self._is_running,
            "tools_registered": len(self._tools),
        }

    def start(self) -> None:
        """Start the MCP server.

        Raises:
            ValueError: If endpoint URL is not configured
        """
        if not self.entry_point.get_endpoint_url():
            raise ValueError("Endpoint URL must be configured before starting server")
        self._is_running = True

    def stop(self) -> None:
        """Stop the MCP server."""
        self._is_running = False

    def is_running(self) -> bool:
        """Check if server is running.

        Returns:
            True if server is running, False otherwise
        """
        return self._is_running
