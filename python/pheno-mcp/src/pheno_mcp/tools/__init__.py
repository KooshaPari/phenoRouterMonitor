"""Tools module for pheno-mcp.

Provides FastMCP decorators and tool registry abstraction.
"""

from .decorators import mcp_tool
from . import tool_registry

__all__ = ["mcp_tool", "tool_registry"]
