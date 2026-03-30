"""MCP integration module for pheno-mcp.

Provides entry point definitions and server abstraction for Model Context Protocol.
"""

from .entry_points import MCPEntryPoint, MCPServer

__all__ = ["MCPEntryPoint", "MCPServer"]
