"""Hexagonal configuration layer for thegent.

This module provides configuration management split into logical domains:
- ModelConfig: Model selection and routing configuration
- PathConfig: Filesystem paths and directories
- RuntimeConfig: Sandbox, budgets, retention, and execution behavior
- ThegentSettings: Composite settings class combining all domains

The main ThegentSettings class is the primary entry point and is designed to be
backward compatible with the original monolithic configuration class.
"""

from thegent.config.settings import ThegentSettings, get_settings
from thegent.config.model_config import ModelConfig
from thegent.config.path_config import PathConfig
from thegent.config.runtime_config import RuntimeConfig

__all__ = [
    "ThegentSettings",
    "ModelConfig",
    "PathConfig",
    "RuntimeConfig",
    "get_settings",
]
