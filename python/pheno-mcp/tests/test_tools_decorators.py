"""Tests for FastMCP decorator abstraction.

Traces to: FR-MCP-002 - FastMCP Tool Decorator Abstraction
"""

import pytest
from typing import Any, Callable
from pheno_mcp.tools import mcp_tool, tool_registry


class TestMCPToolDecorator:
    """Test the @mcp_tool decorator for registering MCP tools."""

    def test_decorator_registers_tool(self):
        """Test that @mcp_tool decorator registers a function as MCP tool."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="test_tool")
        def sample_tool(param: str) -> str:
            """A sample tool."""
            return f"processed: {param}"

        assert registry.has_tool("test_tool")
        assert callable(sample_tool)

    def test_decorator_preserves_function_signature(self):
        """Test that decorator preserves original function behavior."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="echo_tool")
        def echo(text: str) -> str:
            """Echo the input."""
            return text

        result = echo("hello")
        assert result == "hello"

    def test_decorator_with_description(self):
        """Test that decorator accepts description parameter."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(
            registry=registry,
            name="described_tool",
            description="This is a test tool"
        )
        def tool_with_desc() -> str:
            """Tool with description."""
            return "success"

        tool_info = registry.get_tool_info("described_tool")
        assert tool_info is not None
        assert tool_info.get("description") == "This is a test tool"

    def test_decorator_with_parameters(self):
        """Test that decorator captures function parameters."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="param_tool")
        def tool_with_params(name: str, age: int) -> str:
            """Tool with multiple parameters."""
            return f"{name} is {age} years old"

        tool_info = registry.get_tool_info("param_tool")
        assert tool_info is not None
        assert "parameters" in tool_info

    def test_decorator_without_atoms_specifics(self):
        """Test that decorator has no atoms-specific implementation."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="generic_tool")
        def generic_tool() -> str:
            """A generic tool."""
            return "generic result"

        # Should work with any MCP server, not atoms-specific
        assert not registry.is_atoms_specific()


class TestToolRegistry:
    """Test the tool registry for managing decorated tools."""

    def test_registry_initialization(self):
        """Test that ToolRegistry initializes empty."""
        registry = tool_registry.ToolRegistry()
        assert registry.count() == 0

    def test_registry_register_tool(self):
        """Test registering a tool in the registry."""
        registry = tool_registry.ToolRegistry()

        def sample_func():
            """Sample function."""
            pass

        registry.register("sample", sample_func)
        assert registry.has_tool("sample")
        assert registry.count() == 1

    def test_registry_get_tool(self):
        """Test retrieving a registered tool."""
        registry = tool_registry.ToolRegistry()

        def my_tool():
            """My tool."""
            return "result"

        registry.register("my_tool", my_tool)
        retrieved = registry.get_tool("my_tool")
        assert retrieved == my_tool
        assert retrieved() == "result"

    def test_registry_list_tools(self):
        """Test listing all registered tools."""
        registry = tool_registry.ToolRegistry()

        registry.register("tool1", lambda: None)
        registry.register("tool2", lambda: None)
        registry.register("tool3", lambda: None)

        tools = registry.list_tools()
        assert len(tools) == 3
        assert "tool1" in tools
        assert "tool2" in tools
        assert "tool3" in tools

    def test_registry_get_nonexistent_tool(self):
        """Test that getting nonexistent tool returns None."""
        registry = tool_registry.ToolRegistry()
        assert registry.get_tool("nonexistent") is None

    def test_registry_tool_info(self):
        """Test retrieving metadata about a tool."""
        registry = tool_registry.ToolRegistry()

        def documented_tool(x: int, y: int) -> int:
            """Add two numbers.

            Args:
                x: First number
                y: Second number

            Returns:
                Sum of x and y
            """
            return x + y

        registry.register("add", documented_tool)
        info = registry.get_tool_info("add")

        assert info is not None
        assert info.get("name") == "add"
        assert "Add two numbers" in info.get("description", "")


class TestToolDecoratorIntegration:
    """Integration tests for tool decorators with registry."""

    def test_multiple_tools_independent(self):
        """Test that multiple decorated tools are independent."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="tool_a")
        def tool_a(x: int) -> int:
            return x * 2

        @mcp_tool(registry=registry, name="tool_b")
        def tool_b(x: int) -> int:
            return x * 3

        assert tool_a(5) == 10
        assert tool_b(5) == 15

    def test_tools_with_state(self):
        """Test that tools can maintain state independently."""
        registry = tool_registry.ToolRegistry()

        state = {"count": 0}

        @mcp_tool(registry=registry, name="counter")
        def counter() -> int:
            state["count"] += 1
            return state["count"]

        assert counter() == 1
        assert counter() == 2
        assert counter() == 3

    def test_registry_isolation(self):
        """Test that registries are isolated from each other."""
        registry1 = tool_registry.ToolRegistry()
        registry2 = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry1, name="unique_tool")
        def tool():
            return "result"

        assert registry1.has_tool("unique_tool")
        assert not registry2.has_tool("unique_tool")
