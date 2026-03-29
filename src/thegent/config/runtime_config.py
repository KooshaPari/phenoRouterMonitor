"""Runtime execution configuration settings for thegent.

Settings related to sandboxing, budgets, retention, and execution behavior.
"""

from typing import Literal

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict

from thegent.config_defaults import (
    default_cost_budget_by_category,
    default_mac_keep_awake_agents,
    default_sandbox_env_allowlist,
)
from thegent.config_parsers import parse_retention_by_domain


class RuntimeConfig(BaseSettings):
    """Runtime execution, sandboxing, and budget configuration."""

    model_config = SettingsConfigDict(
        env_prefix="THGENT_",
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    # Session backend configuration
    session_backend: Literal["auto", "zmx", "tmux", "none"] = Field(
        default="auto",
        description=(
            "Session persistence backend for agent sessions. "
            "'auto' probes for zmx then falls back to tmux/none. "
            "'zmx' requires zmx (Zig muxless) on PATH. "
            "'tmux' uses existing tmux tooling. "
            "'none' disables session persistence. "
            "(THGENT_SESSION_BACKEND)"
        ),
    )
    zmx_bin: str = Field(
        default="zmx",
        description="Path or command name for the zmx binary (THGENT_ZMX_BIN)",
    )
    use_fifo: bool = Field(
        default=False,
        description="Enable stdin FIFO for background sessions (WP-9004); THGENT_USE_FIFO",
    )
    use_holdpty: bool = Field(
        default=False,
        description="Wrap background sessions with holdpty for interactive attach (WP-9007); THGENT_USE_HOLDPTY",
    )

    # Retention policy
    retention_days_sessions: int = Field(
        default=30,
        ge=7,
        le=365,
        description="Retention for session dirs (WP-3006 tiered); THGENT_RETENTION_DAYS_SESSIONS",
    )
    retention_default_days: int = Field(
        default=30,
        ge=1,
        le=365,
        description="Default retention in days for history (WP-3006); THGENT_RETENTION_DEFAULT_DAYS",
    )
    retention_days_registry: int = Field(
        default=90,
        ge=30,
        le=730,
        description="Retention for run registry (audit trail); THGENT_RETENTION_DAYS_REGISTRY",
    )
    retention_days_health: int = Field(
        default=90,
        ge=7,
        le=365,
        description="Retention for health snapshots; THGENT_RETENTION_DAYS_HEALTH",
    )
    retention_by_domain: dict[str, int] = Field(
        default_factory=dict,
        description='Per-domain retention days (WP-3006); THGENT_RETENTION_BY_DOMAIN JSON e.g. {"gdpr":365,"soc2":2555}',
    )
    default_domain_tag: str | None = Field(
        default=None,
        description="Default domain tag for runs if not specified (G-GP-07)",
    )
    retention_policy: str | None = Field(
        default=None,
        description="Retention policy string (WP-3006); format: default:30,domain1:10",
    )

    # Budget settings (WP-Y4)
    budget_hourly_limit: float = Field(
        default=10.0,
        ge=0.0,
        description="Hourly budget limit in USD (THGENT_BUDGET_HOURLY_LIMIT)",
    )
    budget_daily_limit: float = Field(
        default=100.0,
        ge=0.0,
        description="Daily budget limit in USD (THGENT_BUDGET_DAILY_LIMIT)",
    )
    budget_run_limit: float = Field(
        default=5.0,
        ge=0.0,
        description="Per-run budget limit in USD (THGENT_BUDGET_RUN_LIMIT)",
    )
    budget_warning_threshold: float = Field(
        default=0.8,
        ge=0.0,
        le=1.0,
        description="Budget warning threshold (0.8 = 80%; THGENT_BUDGET_WARNING_THRESHOLD)",
    )

    # Cost tracking
    cost_tracking_enabled: bool = Field(
        default=True,
        description="Enable cost tracking per run (THGENT_COST_TRACKING_ENABLED)",
    )
    cost_tracking: bool = Field(
        default=False,
        description="Legacy: enable cost tracking via THGENT_COST_TRACKING=1",
    )
    cost_budget_mtd: float = Field(
        default=100.0,
        ge=0.0,
        description="MTD budget for AI providers (THGENT_COST_BUDGET_MTD)",
    )
    cost_budget_by_category: dict[str, float] = Field(
        default_factory=default_cost_budget_by_category,
        description="Per-category MTD budgets in USD (THGENT_COST_BUDGET_BY_CATEGORY JSON)",
    )

    # Sandbox configuration
    sandbox_level: str = Field(
        default="none",
        description="macOS sandbox level: none, readonly, restricted, networked, full (THGENT_SANDBOX_LEVEL)",
    )
    sandbox_env_filter: bool = Field(
        default=False,
        description="Filter environment variables in sandbox (THGENT_SANDBOX_ENV_FILTER)",
    )
    sandbox_env_allowlist: list[str] = Field(
        default_factory=default_sandbox_env_allowlist,
        description="Environment variables allowed in sandbox (THGENT_SANDBOX_ENV_ALLOWLIST)",
    )

    # Keep-alive agents (macOS)
    mac_keep_awake_agents: list[str] = Field(
        default_factory=default_mac_keep_awake_agents,
        description="Agents to keep awake on macOS (THGENT_MAC_KEEP_AWAKE_AGENTS)",
    )

    # Validation and retention parsing
    @staticmethod
    def parse_retention_by_domain_validator(v: object) -> dict[str, int]:
        """Validate retention_by_domain."""
        return parse_retention_by_domain(v)

    retention_policy_parse: str | None = Field(
        default=None,
        description="Parsed retention policy for compatibility",
    )
