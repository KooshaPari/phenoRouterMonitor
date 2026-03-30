"""Tests for pheno_core.config module."""

import os
import tempfile

import pytest
from pydantic import Field

from pheno_core.config import (
    BaseConfig,
    ConfigLoader,
    from_env,
    from_file,
)
from pheno_core.errors import ConfigurationError


class SampleConfig(BaseConfig):
    """Sample configuration class."""

    app_name: str = Field(default="test_app")
    debug: bool = Field(default=False)
    port: int = Field(default=8000)
    database_url: str = Field(default="sqlite:///test.db")


class TestBaseConfig:
    """Test BaseConfig class."""

    def test_base_config_creation(self) -> None:
        """Test creating a BaseConfig instance."""
        config = SampleConfig()
        assert config.app_name == "test_app"
        assert config.debug is False
        assert config.port == 8000

    def test_base_config_from_dict(self) -> None:
        """Test creating BaseConfig from dictionary."""
        data = {
            "app_name": "my_app",
            "debug": True,
            "port": 9000,
        }
        config = SampleConfig(**data)
        assert config.app_name == "my_app"
        assert config.debug is True
        assert config.port == 9000


class TestFromEnv:
    """Test from_env function."""

    def test_from_env_basic(self) -> None:
        """Test loading config from environment variables."""
        os.environ["APP_NAME"] = "env_app"
        os.environ["DEBUG"] = "true"
        os.environ["PORT"] = "5000"

        config = from_env(SampleConfig)

        assert config.app_name == "env_app"
        assert config.debug is True
        assert config.port == 5000

        # Cleanup
        del os.environ["APP_NAME"]
        del os.environ["DEBUG"]
        del os.environ["PORT"]

    def test_from_env_with_prefix(self) -> None:
        """Test loading config from environment with prefix."""
        os.environ["TEST_APP_NAME"] = "prefixed_app"
        os.environ["TEST_DEBUG"] = "true"

        config = from_env(SampleConfig, env_prefix="TEST_")

        assert config.app_name == "prefixed_app"
        assert config.debug is True

        # Cleanup
        del os.environ["TEST_APP_NAME"]
        del os.environ["TEST_DEBUG"]

    def test_from_env_missing_required(self) -> None:
        """Test from_env raises error for missing required fields."""
        # Create a config with required field without default
        class StrictConfig(BaseConfig):
            required_field: str

        with pytest.raises(ConfigurationError):
            from_env(StrictConfig)


class TestFromFile:
    """Test from_file function."""

    def test_from_file_json(self) -> None:
        """Test loading config from JSON file."""
        config_data = {
            "app_name": "file_app",
            "debug": True,
            "port": 3000,
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            import json
            json.dump(config_data, f)
            temp_path = f.name

        try:
            config = from_file(SampleConfig, temp_path)
            assert config.app_name == "file_app"
            assert config.debug is True
            assert config.port == 3000
        finally:
            os.unlink(temp_path)

    def test_from_file_toml(self) -> None:
        """Test loading config from TOML file."""
        config_content = """
app_name = "toml_app"
debug = true
port = 4000
database_url = "postgresql://localhost/test"
"""

        with tempfile.NamedTemporaryFile(mode='w', suffix='.toml', delete=False) as f:
            f.write(config_content)
            temp_path = f.name

        try:
            config = from_file(SampleConfig, temp_path)
            assert config.app_name == "toml_app"
            assert config.debug is True
            assert config.port == 4000
        finally:
            os.unlink(temp_path)

    def test_from_file_not_found(self) -> None:
        """Test from_file raises error for missing file."""
        with pytest.raises(ConfigurationError):
            from_file(SampleConfig, "/nonexistent/path/config.json")


class TestConfigLoader:
    """Test ConfigLoader class."""

    def test_config_loader_chain(self) -> None:
        """Test ConfigLoader with method chaining."""
        os.environ["APP_NAME"] = "chained_app"

        loader = ConfigLoader(SampleConfig)
        config = loader.from_env().build()

        assert config.app_name == "chained_app"

        # Cleanup
        del os.environ["APP_NAME"]

    def test_config_loader_file_override(self) -> None:
        """Test ConfigLoader with file loading."""
        config_data = {"app_name": "loader_app", "port": 7000}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            import json
            json.dump(config_data, f)
            temp_path = f.name

        try:
            loader = ConfigLoader(SampleConfig)
            config = loader.from_file(temp_path).build()

            assert config.app_name == "loader_app"
            assert config.port == 7000
        finally:
            os.unlink(temp_path)

    def test_config_loader_env_overlay(self) -> None:
        """Test ConfigLoader with env overlaying file."""
        config_data = {"app_name": "base_app", "port": 6000}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            import json
            json.dump(config_data, f)
            temp_path = f.name

        try:
            os.environ["APP_NAME"] = "override_app"

            loader = ConfigLoader(SampleConfig)
            config = loader.from_file(temp_path).from_env().build()

            # Environment should override file
            assert config.app_name == "override_app"
            assert config.port == 6000

            # Cleanup
            del os.environ["APP_NAME"]
        finally:
            os.unlink(temp_path)


class TestConfigValidation:
    """Test config validation."""

    def test_config_type_validation(self) -> None:
        """Test that config validates types."""
        with pytest.raises(ConfigurationError):
            from_env(SampleConfig)
            SampleConfig(port="not_a_number")  # type: ignore

    def test_config_field_validation(self) -> None:
        """Test custom field validation."""
        class ValidatedConfig(BaseConfig):
            port: int = Field(default=8000, ge=1024, le=65535)

        # Valid port
        config = ValidatedConfig(port=8080)
        assert config.port == 8080

        # Invalid port - should raise during validation
        with pytest.raises(ConfigurationError):
            ValidatedConfig(port=100)  # type: ignore
