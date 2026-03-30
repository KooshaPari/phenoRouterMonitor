"""Configuration management for Phenotype Core."""

import json
from pathlib import Path
from typing import Any, Type, TypeVar, Optional

from pydantic import BaseModel, ConfigDict, ValidationError as PydanticValidationError
from pydantic_settings import BaseSettings

from .errors import ConfigurationError, ValidationError as CoreValidationError


T = TypeVar("T", bound="BaseConfig")


class BaseConfig(BaseSettings):
    """
    Base configuration class using Pydantic.

    Provides unified configuration handling with environment variable support
    and validation.
    """

    model_config = ConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )


def from_env(
    config_cls: Type[T],
    env_prefix: Optional[str] = None,
) -> T:
    """
    Load configuration from environment variables.

    Args:
        config_cls: Configuration class to instantiate.
        env_prefix: Optional prefix for environment variable names.

    Returns:
        Configured instance.

    Raises:
        ConfigurationError: If configuration is invalid or required fields are missing.
    """
    try:
        if env_prefix:
            return config_cls(_env_file=None, _case_sensitive=False, **__get_env_vars(env_prefix))
        return config_cls(_env_file=None)  # type: ignore
    except PydanticValidationError as e:
        raise ConfigurationError(
            f"Configuration validation failed: {str(e)}",
            code="ERR_CONFIG_INVALID",
            context={"validation_errors": e.errors()},
        ) from e
    except Exception as e:
        raise ConfigurationError(
            f"Failed to load configuration from environment: {str(e)}",
            code="ERR_CONFIG_LOAD_ENV",
        ) from e


def from_file(
    config_cls: Type[T],
    filepath: str,
) -> T:
    """
    Load configuration from a file (JSON, TOML, YAML, etc.).

    Args:
        config_cls: Configuration class to instantiate.
        filepath: Path to configuration file.

    Returns:
        Configured instance.

    Raises:
        ConfigurationError: If file not found or configuration is invalid.
    """
    path = Path(filepath)

    if not path.exists():
        raise ConfigurationError(
            f"Configuration file not found: {filepath}",
            code="ERR_CONFIG_NOT_FOUND",
            context={"filepath": filepath},
        )

    try:
        with open(path, "r") as f:
            if path.suffix == ".json":
                data = json.load(f)
            elif path.suffix in [".toml", ".tml"]:
                try:
                    import tomllib  # type: ignore
                except ImportError:
                    import tomli as tomllib  # type: ignore
                data = tomllib.loads(f.read())
            elif path.suffix in [".yaml", ".yml"]:
                try:
                    import yaml
                    data = yaml.safe_load(f)
                except ImportError:
                    raise ConfigurationError(
                        "YAML support requires PyYAML package",
                        code="ERR_CONFIG_YAML_UNSUPPORTED",
                    ) from None
            else:
                raise ConfigurationError(
                    f"Unsupported file format: {path.suffix}",
                    code="ERR_CONFIG_UNSUPPORTED_FORMAT",
                    context={"filepath": filepath, "suffix": path.suffix},
                )

        return config_cls(**data)  # type: ignore

    except PydanticValidationError as e:
        raise ConfigurationError(
            f"Configuration validation failed: {str(e)}",
            code="ERR_CONFIG_INVALID",
            context={"filepath": filepath, "validation_errors": e.errors()},
        ) from e
    except ConfigurationError:
        raise
    except Exception as e:
        raise ConfigurationError(
            f"Failed to load configuration from file: {str(e)}",
            code="ERR_CONFIG_LOAD_FILE",
            context={"filepath": filepath},
        ) from e


def load(
    config_cls: Type[T],
    filepath: Optional[str] = None,
    env_prefix: Optional[str] = None,
) -> T:
    """
    Load configuration with fallback chain.

    Loads configuration in this order:
    1. File (if provided)
    2. Environment variables (if prefix provided)
    3. Defaults

    Args:
        config_cls: Configuration class to instantiate.
        filepath: Optional path to configuration file.
        env_prefix: Optional prefix for environment variable names.

    Returns:
        Configured instance.

    Raises:
        ConfigurationError: If all loading methods fail.
    """
    if filepath:
        return from_file(config_cls, filepath)

    if env_prefix:
        return from_env(config_cls, env_prefix)

    try:
        return config_cls()  # type: ignore
    except Exception as e:
        raise ConfigurationError(
            "Failed to load configuration",
            code="ERR_CONFIG_LOAD_FAILED",
        ) from e


class ConfigLoader:
    """
    Fluent configuration loader with method chaining.

    Allows flexible configuration composition through chaining.
    """

    def __init__(self, config_cls: Type[T]) -> None:
        """
        Initialize ConfigLoader.

        Args:
            config_cls: Configuration class to instantiate.
        """
        self.config_cls = config_cls
        self._data: dict[str, Any] = {}
        self._env_prefix: Optional[str] = None

    def from_file(self, filepath: str) -> "ConfigLoader":
        """
        Load configuration from file.

        Args:
            filepath: Path to configuration file.

        Returns:
            Self for method chaining.
        """
        config = from_file(self.config_cls, filepath)
        self._data.update(config.model_dump())
        return self

    def from_env(self, env_prefix: Optional[str] = None) -> "ConfigLoader":
        """
        Load configuration from environment variables.

        Args:
            env_prefix: Optional prefix for environment variable names.

        Returns:
            Self for method chaining.
        """
        config = from_env(self.config_cls, env_prefix)
        self._data.update(config.model_dump())
        return self

    def build(self) -> T:
        """
        Build final configuration.

        Returns:
            Configured instance.

        Raises:
            ConfigurationError: If configuration is invalid.
        """
        try:
            return self.config_cls(**self._data)  # type: ignore
        except PydanticValidationError as e:
            raise ConfigurationError(
                f"Configuration validation failed: {str(e)}",
                code="ERR_CONFIG_INVALID",
                context={"validation_errors": e.errors()},
            ) from e


def __get_env_vars(prefix: str) -> dict[str, Any]:
    """
    Get environment variables with given prefix.

    Args:
        prefix: Environment variable prefix.

    Returns:
        Dictionary of environment variables with prefix removed.
    """
    import os

    result = {}
    for key, value in os.environ.items():
        if key.startswith(prefix):
            clean_key = key[len(prefix):].lower()
            result[clean_key] = value

    return result
