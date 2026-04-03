"""FastMCP 3.0 server for AgilePlus."""

from fastmcp import FastMCP

from agileplus_mcp.tools import features, governance, status

mcp = FastMCP(
    name="agileplus",
    instructions="Spec-driven development engine with governance",
)

# Register tool modules
features.register(mcp)
governance.register(mcp)
status.register(mcp)


def main() -> None:
    """Start the MCP server."""
    mcp.run()
