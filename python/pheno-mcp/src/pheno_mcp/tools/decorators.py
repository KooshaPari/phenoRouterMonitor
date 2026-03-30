"""FastMCP decorators for tool registration.

Provides a generic @mcp_tool decorator that abstracts FastMCP specifics.
Designed to work with any FastMCP-compatible server without atoms-specific code.
"""

from typing import Callable, Optional, Any
from functools import wraps
from .tool_registry import ToolRegistry


def mcp_tool(
    registry: ToolRegistry,
    name: Optional[str] = None,
    description: Optional[str] = None,
    **extra_metadata
) -> Callable:
    """Decorator to register a function as an MCP tool.

    Abstracts FastMCP decorator functionality to work with any MCP server.
    Not atoms-specific - works with any compatible MCP implementation.

    Args:
        registry: The ToolRegistry to register with
        name: Tool name (defaults to function name)
        description: Tool description (defaults to docstring)
        **extra_metadata: Additional metadata to attach to the tool

    Returns:
        Decorator function

    Example:
        >>> registry = ToolRegistry()
        >>> @mcp_tool(registry=registry, name="my_tool")
        >>> def my_tool(x: int) -> int:
        >>>     return x * 2
    """

    def decorator(func: Callable) -> Callable:
        # Use provided name or default to function name
        tool_name = name or func.__name__

        # Use provided description or default to docstring
        tool_description = description or (func.__doc__ or "")

        # Register in the registry
        registry.register(
            tool_name,
            func,
            description=tool_description,
            **extra_metadata
        )

        # Wrap function to preserve original behavior
        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)

        # Attach metadata to wrapper for introspection
        wrapper._mcp_name = tool_name
        wrapper._mcp_description = tool_description
        wrapper._registry = registry

        return wrapper

    return decorator


def mcp_parameter(
    name: str,
    param_type: str,
    description: str = "",
    required: bool = True,
) -> Any:
    """Decorator to specify MCP tool parameter metadata.

    Provides detailed parameter information for MCP tool discovery.

    Args:
        name: Parameter name
        param_type: Parameter type (e.g., "string", "int", "object")
        description: Parameter description
        required: Whether parameter is required

    Returns:
        Parameter metadata dictionary
    """
    return {
        "name": name,
        "type": param_type,
        "description": description,
        "required": required,
    }


class ToolMetadata:
    """Metadata container for MCP tools.

    Holds tool information for introspection and documentation.
    """

    def __init__(
        self,
        name: str,
        description: str = "",
        input_schema: Optional[dict] = None,
        output_schema: Optional[dict] = None,
    ):
        """Initialize tool metadata.

        Args:
            name: Tool name
            description: Tool description
            input_schema: JSON schema for inputs
            output_schema: JSON schema for outputs
        """
        self.name = name
        self.description = description
        self.input_schema = input_schema or {}
        self.output_schema = output_schema or {}

    def to_dict(self) -> dict:
        """Convert metadata to dictionary.

        Returns:
            Dictionary representation of metadata
        """
        return {
            "name": self.name,
            "description": self.description,
            "input_schema": self.input_schema,
            "output_schema": self.output_schema,
        }
