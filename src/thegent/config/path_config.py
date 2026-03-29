"""Path and directory configuration settings for thegent.

Settings related to filesystem paths and directory locations.
"""

from pathlib import Path

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict

from thegent.config_defaults import expanded_path_factory


class PathConfig(BaseSettings):
    """Filesystem path and directory configuration."""

    model_config = SettingsConfigDict(
        env_prefix="THGENT_",
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    # Factory directories
    factory_skills_dir: Path = Field(
        default_factory=expanded_path_factory("~/.factory/skills"),
        description="Factory skills directory",
    )
    factory_droids_dir: Path = Field(
        default_factory=expanded_path_factory("~/.factory/droids"),
        description="Factory droids directory",
    )

    # Cache and session directories
    cache_dir: Path = Field(
        default_factory=expanded_path_factory("~/.cache/thegent"),
        description="Global cache directory for thegent",
    )
    session_dir: Path = Field(
        default_factory=expanded_path_factory("~/.cache/thegent/sessions"),
        description="Session metadata/log directory for background runs",
    )

    # Working directory
    cwd: Path | None = Field(
        default=None,
        description="Working directory (inferred or explicit)",
    )

    # Session management paths
    session_meta_path: Path | None = Field(
        default=None,
        description="Session metadata path override (THGENT_SESSION_META_PATH)",
    )
    session_rc_path: Path | None = Field(
        default=None,
        description="Session rc path override (THGENT_SESSION_RC_PATH)",
    )
    health_snapshot_path: Path | None = Field(
        default=None,
        description="Health snapshot path (THGENT_HEALTH_SNAPSHOT_PATH)",
    )
    fifo_path: Path | None = Field(
        default=None,
        description="Override path for stdin FIFO (default: session_dir/ID.in); THGENT_FIFO_PATH",
    )
    holdpty_socket_dir: Path | None = Field(
        default=None,
        description="Directory for holdpty sockets; defaults to session_dir/ID.sock",
    )

    # MCP storage
    mcp_storage_dir: Path | None = Field(
        default=None,
        description="MCP storage directory override (THGENT_MCP_STORAGE_DIR)",
    )

    # Connector mapping cache
    connector_mapping_cache_path: Path | None = Field(
        default=None,
        description="Path override for connector mapping cache (THGENT_CONNECTOR_MAPPING_CACHE_PATH)",
    )
