"""Model-related configuration settings for thegent.

Settings related to which models/agents to use for different tasks.
"""

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class ModelConfig(BaseSettings):
    """Model selection and routing configuration."""

    model_config = SettingsConfigDict(
        env_prefix="THGENT_",
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    # Model selection
    default_cursor_model: str = Field(
        default="gemini-3-flash",
        description="Default model for cursor agent (gemini-3-flash, composer-1.5, auto)",
    )
    default_gemini_model: str = Field(
        default="gemini-3-flash",
        description="Default model for gemini CLI (-m/--model)",
    )
    default_copilot_model: str = Field(
        default="gpt-5-mini",
        description="Default model for copilot (--model); copilot doesn't support flash",
    )
    default_claude_model: str = Field(
        default="claude-opus-4.6",
        description="Default model for claude (--model)",
    )
    default_codex_model: str = Field(
        default="gpt-5.3-codex-spark",
        description="Default model for codex",
    )
    default_codex_model_high: str = Field(
        default="gpt-5.3-codex-high",
        description="Codex high-power model (use with --mode full or --model override)",
    )
    default_antigravity_model: str = Field(
        default="gemini-3-flash",
        description="Default model for antigravity (via CLIProxyAPIPlus); was tstars2.0",
    )
    default_kiro_model: str = Field(
        default="claude-haiku-4.5",
        description="Default model for kiro (claude-haiku-4.5, claude-opus-4.6 via CLIProxyAPIPlus)",
    )

    # Timeouts (model-related)
    default_timeout: int = Field(
        default=1800,
        ge=10,
        le=3600,
        description="Default agent timeout in seconds (30m default; agents truly need to run indefinitely, up to 30m)",
    )
    default_timeout_claude: int = Field(
        default=1800,
        ge=60,
        le=3600,
        description="Claude agent timeout (slower API); use THGENT_DEFAULT_TIMEOUT_CLAUDE to override",
    )
    default_timeout_free: int = Field(
        default=300,
        ge=60,
        le=3600,
        description="Free agent (copilot) timeout for WP tasks; use THGENT_DEFAULT_TIMEOUT_FREE to override",
    )
    max_idle_seconds: int = Field(
        default=180,
        ge=60,
        le=600,
        description="Activity-based hang detection: kill only when no stdout/stderr for this many seconds (THGENT_MAX_IDLE_SECONDS)",
    )
    max_wall_time: int = Field(
        default=0,
        ge=0,
        le=86400,
        description="Optional absolute cap in seconds (0=unbounded). Only idle detection kills by default (THGENT_MAX_WALL_TIME)",
    )

    # Routing
    default_routing: str = Field(
        default="prefer_direct",
        description="Default routing policy: prefer_direct | prefer_proxy | failover (THGENT_DEFAULT_ROUTING)",
    )
    routing_enabled: bool = Field(
        default=True,
        description="Enable task routing based on Terminal Bench 2.0 Pareto frontier (THGENT_ROUTING_ENABLED)",
    )
    routing_constraints_enabled: bool = Field(
        default=True,
        description="Enable hard constraint validation for routing (quality, cost, speed) (THGENT_ROUTING_CONSTRAINTS_ENABLED)",
    )
    routing_budget_warning_threshold: float = Field(
        default=0.80,
        ge=0.0,
        le=1.0,
        description="Budget utilization threshold for warnings (0.8 = 80%) (THGENT_ROUTING_BUDGET_WARNING_THRESHOLD)",
    )
    routing_parser_quality_enabled: bool = Field(
        default=True,
        description="Enable parser-quality based routing (THGENT_ROUTING_PARSER_QUALITY_ENABLED)",
    )
    routing_cost_aware_enabled: bool = Field(
        default=True,
        description="Enable cost_quality routing when budget pressure (WP-5003)",
    )
    cost_quality_min_weight: float = Field(
        default=0.1,
        ge=0.0,
        le=2.0,
        description="Minimum cost_weight for cost_quality policy (0.1=gemini flash; WP-5003)",
    )
    cost_quality_budget_tighten_threshold: float = Field(
        default=0.80,
        ge=0.5,
        le=1.0,
        description="Budget utilization above this tightens quality floor (WP-5003)",
    )

    # Auto router configuration
    auto_router_enabled: bool = Field(
        default=True,
        description="Enable auto router when agent/model is 'auto' (THGENT_AUTO_ROUTER_ENABLED)",
    )
    auto_router_classifier_model: str = Field(
        default="gemini-3-flash",
        description="Model for headless task complexity classification (THGENT_AUTO_ROUTER_CLASSIFIER_MODEL)",
    )
    auto_router_use_classifier: bool = Field(
        default=True,
        description="Use Gemini Flash to classify prompt; if False, assume moderate (THGENT_AUTO_ROUTER_USE_CLASSIFIER)",
    )
    auto_router_min_quality: float = Field(
        default=0.0,
        ge=0.0,
        le=1.0,
        description="Minimum quality floor for Pareto selection (THGENT_AUTO_ROUTER_MIN_QUALITY)",
    )
    auto_router_max_cost_weight: float = Field(
        default=2.0,
        ge=0.1,
        le=10.0,
        description="Maximum cost weight for Pareto selection (THGENT_AUTO_ROUTER_MAX_COST_WEIGHT)",
    )
