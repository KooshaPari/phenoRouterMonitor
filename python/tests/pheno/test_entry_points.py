"""Tests for sanitized MCP entry points.

Verifies that all atoms.tech and ATOMS identifiers have been properly removed.
"""

import pytest
import sys
from pathlib import Path

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from pheno.mcp.entry_points import MCPEntryPoint, MCPCLI
from pheno.shared.mcp_entry_points import MCPConfiguration, MCPEntryPointRegistry


class TestMCPEntryPointSanitization:
    """Test that ATOMS identifiers have been removed."""

    def test_no_atoms_class_names(self):
        """Verify ATOMS class names don't exist."""
        # These should not exist in the module
        mcp_module = __import__("pheno.mcp.entry_points", fromlist=[])
        assert not hasattr(mcp_module.mcp.entry_points, "AtomsMCPEntryPoint")
        assert not hasattr(mcp_module.mcp.entry_points, "AtomsMCPCLI")

    def test_generic_mcp_entry_point_exists(self):
        """Verify generic MCPEntryPoint class exists."""
        assert MCPEntryPoint is not None
        entry = MCPEntryPoint("test-endpoint")
        assert entry.name == "test-endpoint"

    def test_generic_mcp_cli_exists(self):
        """Verify generic MCPCLI class exists."""
        assert MCPCLI is not None
        cli = MCPCLI()
        assert cli is not None

    def test_no_atoms_domain_in_endpoint(self):
        """Verify no atoms.tech domain hardcoding."""
        entry = MCPEntryPoint("test")
        # Should be None initially, not hardcoded to atoms.tech
        assert entry.server_url is None
        # Should be configurable
        entry.set_endpoint_url("https://custom.example.com")
        assert entry.get_endpoint_url() == "https://custom.example.com"

    def test_auth_is_generic(self):
        """Verify authentication configuration is generic."""
        entry = MCPEntryPoint("test")
        assert entry.api_key is None
        entry.configure_auth("test-key")
        assert entry.api_key == "test-key"


class TestSharedMCPConfigSanitization:
    """Test that shared configuration is properly sanitized."""

    def test_no_atoms_configuration_class(self):
        """Verify AtomsMCPConfiguration doesn't exist."""
        shared_module = __import__("pheno.shared.mcp_entry_points", fromlist=[])
        assert not hasattr(shared_module.shared.mcp_entry_points, "AtomsMCPConfiguration")

    def test_generic_mcp_configuration_exists(self):
        """Verify generic MCPConfiguration class exists."""
        config = MCPConfiguration()
        assert config is not None
        assert config.endpoints == []

    def test_no_atoms_endpoints_in_config(self):
        """Verify no hardcoded atoms.tech endpoints."""
        config = MCPConfiguration()
        # Should start empty
        assert len(config.endpoints) == 0
        # Add a custom endpoint
        config.add_endpoint("https://my-mcp.example.com")
        assert "https://my-mcp.example.com" in config.endpoints
        # Should NOT have atoms.tech endpoints
        assert not any("atoms.tech" in ep for ep in config.endpoints)

    def test_registry_is_generic(self):
        """Verify MCPEntryPointRegistry is properly generic."""
        registry = MCPEntryPointRegistry()
        assert registry is not None
        # Should not reference atoms
        assert not hasattr(registry, "atoms_primary_endpoint")
        assert hasattr(registry, "_primary_endpoint")


class TestNoAtomsReferences:
    """Test that no atoms.tech references remain in code."""

    def test_no_atoms_tech_domain_in_source(self):
        """Scan source files for atoms.tech domain."""
        src_path = Path(__file__).parent.parent / "src"
        for py_file in src_path.rglob("*.py"):
            content = py_file.read_text()
            assert "atoms.tech" not in content, f"Found atoms.tech in {py_file}"
            assert "atoms_mcp" not in content.lower(), f"Found atoms_mcp in {py_file}"

    def test_pyproject_sanitized(self):
        """Verify pyproject.toml is sanitized."""
        pyproject_path = Path(__file__).parent.parent / "pyproject.toml"
        content = pyproject_path.read_text()
        # Check that ATOMS identifiers are removed
        assert "ATOMS-PHENO" not in content
        assert "atoms@atoms.tech" not in content
        # Check that generic identifiers are present
        assert "Phenotype Team" in content
        assert "phenotype" in content.lower()


class TestFunctionalityPreserved:
    """Test that sanitization didn't break functionality."""

    def test_entry_point_initialization(self):
        """Test MCPEntryPoint initialization."""
        entry = MCPEntryPoint("test-ep", version="2.0.0")
        assert entry.name == "test-ep"
        assert entry.version == "2.0.0"

    def test_entry_point_configuration(self):
        """Test MCPEntryPoint configuration."""
        entry = MCPEntryPoint("test")
        entry.configure_auth("my-key")
        entry.set_endpoint_url("https://mcp.example.com")
        assert entry.api_key == "my-key"
        assert entry.get_endpoint_url() == "https://mcp.example.com"

    def test_cli_deployment(self):
        """Test MCPCLI deployment."""
        cli = MCPCLI()
        cli.entry_point.set_endpoint_url("https://mcp.example.com")
        assert cli.deploy_mcp() is True

    def test_cli_validation(self):
        """Test MCPCLI validation."""
        cli = MCPCLI()
        # Should fail without API key
        assert cli.validate_config() is False
        # Should succeed with API key
        cli.entry_point.configure_auth("test-key")
        assert cli.validate_config() is True

    def test_registry_operations(self):
        """Test MCPEntryPointRegistry operations."""
        registry = MCPEntryPointRegistry()
        registry.set_primary_endpoint("https://mcp.example.com")
        assert registry._primary_endpoint == "https://mcp.example.com"
        assert len(registry.get_entries()) == 0

    def test_configuration_operations(self):
        """Test MCPConfiguration operations."""
        config = MCPConfiguration()
        config.add_endpoint("https://ep1.example.com")
        config.add_endpoint("https://ep2.example.com")
        cfg = config.get_config()
        assert len(cfg["endpoints"]) == 2
        assert "https://ep1.example.com" in cfg["endpoints"]
