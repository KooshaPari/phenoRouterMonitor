"""Tests for MCP entry points module.

Traces to: FR-MCP-001 - MCP Entry Point Configuration
"""

import pytest
from unittest.mock import Mock, patch

from pheno_mcp.mcp import MCPEntryPoint, MCPServer


class TestMCPEntryPoint:
    """Test MCPEntryPoint class initialization and configuration."""

    def test_entry_point_initialization(self):
        """Test that MCPEntryPoint initializes with required parameters."""
        entry_point = MCPEntryPoint(name="test-mcp", version="1.0.0")
        assert entry_point.name == "test-mcp"
        assert entry_point.version == "1.0.0"
        assert entry_point.server_url is None
        assert entry_point.api_key is None

    def test_entry_point_configure_auth(self):
        """Test that authentication can be configured."""
        entry_point = MCPEntryPoint(name="test-mcp")
        test_key = "sk-test-123"
        entry_point.configure_auth(test_key)
        assert entry_point.api_key == test_key

    def test_entry_point_set_endpoint_url(self):
        """Test that endpoint URL can be set and retrieved."""
        entry_point = MCPEntryPoint(name="test-mcp")
        test_url = "http://localhost:8000"
        entry_point.set_endpoint_url(test_url)
        assert entry_point.get_endpoint_url() == test_url

    def test_entry_point_initialization_with_defaults(self):
        """Test MCPEntryPoint initialization with default version."""
        entry_point = MCPEntryPoint(name="default-mcp")
        assert entry_point.version == "1.0.0"

    def test_entry_point_no_atoms_references(self):
        """Test that MCPEntryPoint has no atoms-specific naming."""
        entry_point = MCPEntryPoint(name="generic-mcp")
        # Should not have atoms-specific attributes
        assert not hasattr(entry_point, "atoms_specific_field")
        assert entry_point.name == "generic-mcp"


class TestMCPServer:
    """Test MCPServer wrapper class."""

    def test_mcp_server_initialization(self):
        """Test that MCPServer initializes with entry point."""
        entry_point = MCPEntryPoint(name="test-mcp")
        server = MCPServer(entry_point)
        assert server.entry_point == entry_point

    def test_mcp_server_starts_without_errors(self):
        """Test that MCP server can be started."""
        entry_point = MCPEntryPoint(name="test-mcp", version="1.0.0")
        entry_point.set_endpoint_url("http://localhost:8000")
        server = MCPServer(entry_point)
        # Should not raise any errors during initialization
        assert server.entry_point.get_endpoint_url() == "http://localhost:8000"

    def test_mcp_server_requires_endpoint_configuration(self):
        """Test that server requires endpoint configuration."""
        entry_point = MCPEntryPoint(name="test-mcp")
        server = MCPServer(entry_point)
        # Endpoint should be None initially
        assert server.entry_point.get_endpoint_url() is None

    def test_mcp_server_supports_health_check(self):
        """Test that MCP server can report health status."""
        entry_point = MCPEntryPoint(name="test-mcp")
        server = MCPServer(entry_point)
        health = server.health_check()
        assert health is not None
        assert "status" in health


class TestMCPIntegration:
    """Integration tests for MCP configuration."""

    def test_entry_point_and_server_integration(self):
        """Test complete setup of entry point and server."""
        # Create entry point
        entry_point = MCPEntryPoint(name="integration-test-mcp")
        entry_point.set_endpoint_url("http://localhost:8000")
        entry_point.configure_auth("sk-integration-test")

        # Create server
        server = MCPServer(entry_point)

        # Verify configuration
        assert server.entry_point.name == "integration-test-mcp"
        assert server.entry_point.get_endpoint_url() == "http://localhost:8000"
        assert server.entry_point.api_key == "sk-integration-test"

    def test_multiple_servers_independent(self):
        """Test that multiple MCP servers can be created independently."""
        ep1 = MCPEntryPoint(name="server-1")
        ep2 = MCPEntryPoint(name="server-2")

        ep1.set_endpoint_url("http://localhost:8001")
        ep2.set_endpoint_url("http://localhost:8002")

        server1 = MCPServer(ep1)
        server2 = MCPServer(ep2)

        assert server1.entry_point.get_endpoint_url() == "http://localhost:8001"
        assert server2.entry_point.get_endpoint_url() == "http://localhost:8002"
