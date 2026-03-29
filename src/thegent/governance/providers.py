"""Provider registry for economic governance (WP-5003).

Centralized registry of provider configurations with fallback chains
and cost/reliability metadata for routing decisions.

See: docs/changes/research-economic-governance/design.md § 2.1
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import ClassVar

# Re-export ProviderMetrics from scoring for backwards compatibility
from thegent.governance.scoring import ProviderMetrics


class ProviderType(Enum):
    """Provider deployment type."""

    DIRECT = "direct"  # Direct API connection
    PROXY = "proxy"  # Proxy/gateway connection


@dataclass
class ProviderConfig:
    """Provider configuration.

    Attributes:
        provider_id: Unique provider identifier (e.g., "gemini-flash")
        name: Display name
        provider_type: DIRECT or PROXY
        api_endpoint: API base URL
        auth_method: "api_key", "oauth", etc.
        cost_per_1m_tokens: USD cost per million tokens
        max_rpm: Maximum requests per minute
        max_tpm: Maximum tokens per minute
        fallback_order: List of provider IDs to try on failure
    """

    provider_id: str
    name: str
    provider_type: ProviderType
    api_endpoint: str
    auth_method: str
    cost_per_1m_tokens: float
    max_rpm: int  # Requests per minute
    max_tpm: int  # Tokens per minute
    fallback_order: list[str] = field(default_factory=list)


class ProviderRegistry:
    """Centralized provider configuration and lookup.

    Manages provider definitions, routing metadata, and fallback chains.
    Implements singleton pattern with class-level registry.
    """

    _registry: ClassVar[dict[str, ProviderConfig]] = {}
    _initialized: ClassVar[bool] = False

    @classmethod
    def register(cls, config: ProviderConfig) -> None:
        """Register a provider configuration.

        Args:
            config: Provider configuration
        """
        cls._registry[config.provider_id] = config

    @classmethod
    def get(cls, provider_id: str) -> ProviderConfig | None:
        """Get provider configuration by ID.

        Args:
            provider_id: Provider identifier

        Returns:
            Provider config or None if not found
        """
        return cls._registry.get(provider_id)

    @classmethod
    def list_providers(cls) -> list[ProviderConfig]:
        """List all registered providers.

        Returns:
            List of provider configurations
        """
        return list(cls._registry.values())

    @classmethod
    def get_fallback_order(cls, provider_id: str) -> list[str]:
        """Get fallback chain for a provider.

        Args:
            provider_id: Provider identifier

        Returns:
            Ordered list of fallback provider IDs
        """
        config = cls.get(provider_id)
        return config.fallback_order if config else []

    @classmethod
    def unregister(cls, provider_id: str) -> None:
        """Unregister a provider (for testing).

        Args:
            provider_id: Provider identifier to remove
        """
        cls._registry.pop(provider_id, None)

    @classmethod
    def clear(cls) -> None:
        """Clear all registered providers (for testing).

        WARNING: This should only be called during tests.
        """
        cls._registry.clear()

    @classmethod
    def count(cls) -> int:
        """Get number of registered providers.

        Returns:
            Provider count
        """
        return len(cls._registry)


# Built-in provider configurations
_BUILTIN_PROVIDERS = [
    ProviderConfig(
        provider_id="gemini-3-flash",
        name="Google Gemini 3 Flash",
        provider_type=ProviderType.DIRECT,
        api_endpoint="https://generativelanguage.googleapis.com",
        auth_method="api_key",
        cost_per_1m_tokens=0.10,
        max_rpm=1500,
        max_tpm=1000000,
        fallback_order=["claude-haiku-4.5", "gpt-4o-mini"],
    ),
    ProviderConfig(
        provider_id="claude-haiku-4.5",
        name="Anthropic Claude Haiku 4.5",
        provider_type=ProviderType.DIRECT,
        api_endpoint="https://api.anthropic.com/v1",
        auth_method="api_key",
        cost_per_1m_tokens=0.25,
        max_rpm=1000,
        max_tpm=500000,
        fallback_order=["gemini-3-flash", "gpt-4o-mini"],
    ),
    ProviderConfig(
        provider_id="gpt-4o-mini",
        name="OpenAI GPT-4o Mini",
        provider_type=ProviderType.DIRECT,
        api_endpoint="https://api.openai.com/v1",
        auth_method="api_key",
        cost_per_1m_tokens=0.15,
        max_rpm=3500,
        max_tpm=2000000,
        fallback_order=["claude-haiku-4.5", "gemini-3-flash"],
    ),
    ProviderConfig(
        provider_id="claude-sonnet-4.5",
        name="Anthropic Claude Sonnet 4.5",
        provider_type=ProviderType.DIRECT,
        api_endpoint="https://api.anthropic.com/v1",
        auth_method="api_key",
        cost_per_1m_tokens=3.00,
        max_rpm=500,
        max_tpm=250000,
        fallback_order=["claude-haiku-4.5"],
    ),
    ProviderConfig(
        provider_id="gemini-3.1-pro",
        name="Google Gemini 3.1 Pro",
        provider_type=ProviderType.DIRECT,
        api_endpoint="https://generativelanguage.googleapis.com",
        auth_method="api_key",
        cost_per_1m_tokens=3.50,
        max_rpm=500,
        max_tpm=250000,
        fallback_order=["claude-sonnet-4.5", "gpt-4o-mini"],
    ),
]


def _initialize_registry() -> None:
    """Initialize registry with built-in providers.

    Called once during module import to populate the default registry.
    """
    if ProviderRegistry._initialized:  # noqa: SLF001 -- intentional access to ClassVar sentinel on own class
        return

    for provider_config in _BUILTIN_PROVIDERS:
        ProviderRegistry.register(provider_config)

    ProviderRegistry._initialized = True  # noqa: SLF001 -- intentional access to ClassVar sentinel on own class


# Initialize registry on module import
_initialize_registry()
