"""Tool registry for managing MCP tools.

Provides a generic registry for tools that can be used by any MCP server.
"""

from typing import Any, Callable, Dict, List, Optional
import inspect


class ToolRegistry:
    """Registry for MCP tools.

    Manages tool registration and retrieval independent of specific MCP server implementation.
    """

    def __init__(self):
        """Initialize tool registry."""
        self._tools: Dict[str, Any] = {}
        self._tool_info: Dict[str, Dict[str, Any]] = {}

    def register(self, name: str, callable_obj: Callable, **metadata) -> None:
        """Register a tool in the registry.

        Args:
            name: Name of the tool
            callable_obj: The callable that implements the tool
            **metadata: Additional metadata about the tool
        """
        self._tools[name] = callable_obj

        # Extract metadata from function
        sig = inspect.signature(callable_obj)
        docstring = inspect.getdoc(callable_obj) or ""

        self._tool_info[name] = {
            "name": name,
            "description": docstring.split("\n")[0] if docstring else "",
            "parameters": self._extract_parameters(sig),
            **metadata,
        }

    def get_tool(self, name: str) -> Optional[Callable]:
        """Get a tool by name.

        Args:
            name: Name of the tool

        Returns:
            The callable or None if not found
        """
        return self._tools.get(name)

    def get_tool_info(self, name: str) -> Optional[Dict[str, Any]]:
        """Get information about a tool.

        Args:
            name: Name of the tool

        Returns:
            Tool information dictionary or None if not found
        """
        return self._tool_info.get(name)

    def has_tool(self, name: str) -> bool:
        """Check if a tool is registered.

        Args:
            name: Name of the tool

        Returns:
            True if tool is registered, False otherwise
        """
        return name in self._tools

    def list_tools(self) -> List[str]:
        """List all registered tool names.

        Returns:
            List of tool names
        """
        return list(self._tools.keys())

    def count(self) -> int:
        """Get count of registered tools.

        Returns:
            Number of registered tools
        """
        return len(self._tools)

    def list_tool_info(self) -> List[Dict[str, Any]]:
        """Get information about all tools.

        Returns:
            List of tool information dictionaries
        """
        return list(self._tool_info.values())

    def is_atoms_specific(self) -> bool:
        """Check if registry has atoms-specific tools.

        Returns:
            False - this registry supports generic MCP tools
        """
        return False

    def _extract_parameters(self, sig: inspect.Signature) -> Dict[str, Any]:
        """Extract parameter information from function signature.

        Args:
            sig: Function signature

        Returns:
            Dictionary of parameter information
        """
        params = {}
        for param_name, param in sig.parameters.items():
            if param_name == "self":
                continue
            params[param_name] = {
                "type": str(param.annotation) if param.annotation != inspect.Parameter.empty else "Any",
                "required": param.default == inspect.Parameter.empty,
                "default": str(param.default) if param.default != inspect.Parameter.empty else None,
            }
        return params
