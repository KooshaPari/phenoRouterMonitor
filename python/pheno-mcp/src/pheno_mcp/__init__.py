"""pheno-mcp: Standalone MCP tooling package for Phenotype.

Provides FastMCP wrappers, tool registry, and CrewAI agent orchestration.
Designed to work with any MCP server, not atoms-specific.

Features:
- MCP entry points and server abstraction
- Tool registration and decorator abstraction
- CrewAI agent orchestration
- Server-agnostic design for maximum flexibility

Example:
    >>> from pheno_mcp.mcp import MCPEntryPoint
    >>> from pheno_mcp.tools import mcp_tool, tool_registry
    >>> from pheno_mcp.agents import Agent, AgentOrchestrator
    >>>
    >>> # Configure MCP entry point
    >>> ep = MCPEntryPoint(name="my-mcp-server")
    >>> ep.set_endpoint_url("http://localhost:8000")
    >>>
    >>> # Register tools
    >>> registry = tool_registry.ToolRegistry()
    >>> @mcp_tool(registry=registry, name="my_tool")
    >>> def my_tool(x: int) -> int:
    ...     return x * 2
    >>>
    >>> # Orchestrate agents
    >>> orchestrator = AgentOrchestrator()
    >>> orchestrator.add_agent(Agent(role=AgentRole.WORKER, name="worker"))
"""

__version__ = "1.0.0"
__author__ = "Phenotype Team"
__license__ = "MIT"

from .mcp import MCPEntryPoint, MCPServer
from .tools import mcp_tool, tool_registry
from .agents import Agent, AgentRole, TaskDefinition, AgentOrchestrator

__all__ = [
    "MCPEntryPoint",
    "MCPServer",
    "mcp_tool",
    "tool_registry",
    "Agent",
    "AgentRole",
    "TaskDefinition",
    "AgentOrchestrator",
]
