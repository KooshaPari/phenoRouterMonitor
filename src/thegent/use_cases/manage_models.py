"""Model CRUD operations (add, remove, update, list models, routing)."""

from __future__ import annotations

from typing import Any

from thegent.provider_model_manager_io import (
    MODEL_DEFINITIONS_PATH,
    PROVIDER_DEFINITIONS_PATH,
    load_json,
    save_json,
)

__all__ = [
    "list_models",
    "add_model_alias",
    "remove_model_alias",
    "add_common_alias",
    "remove_common_alias",
]


def list_models(provider: str | None = None) -> list[dict[str, Any]]:
    """List all models, optionally filtered by provider.

    Args:
        provider: Optional provider name to filter by.

    Returns:
        List of model configurations across providers and common aliases.
    """
    models = load_json(MODEL_DEFINITIONS_PATH)
    providers = load_json(PROVIDER_DEFINITIONS_PATH)

    result = []
    common_aliases = models.get("common_aliases", [])

    # Get models from provider definitions
    for prov_name, prov_cfg in providers.items():
        if provider and prov_name.lower() != provider.lower():
            continue

        model_name = prov_cfg.get("model", "")
        aliases = prov_cfg.get("extra_aliases", [])

        result.append(
            {
                "provider": prov_name,
                "model": model_name,
                "aliases": aliases,
                "is_default": True,
            }
        )

    # Add common aliases
    if not provider:
        for alias in common_aliases:
            result.append(
                {
                    "provider": "common",
                    "model": alias,
                    "aliases": [],
                    "is_default": False,
                }
            )

    return result


def add_model_alias(provider: str, model: str, alias: str) -> tuple[bool, str]:
    """Add a model alias for a provider.

    Args:
        provider: Provider name.
        model: Base model name (for reference, not directly used in storage).
        alias: The alias to add.

    Returns:
        Tuple of (success: bool, message: str).
    """
    provider = provider.lower().strip()
    providers = load_json(PROVIDER_DEFINITIONS_PATH)

    if provider not in providers:
        return False, f"Provider '{provider}' not found"

    aliases = providers[provider].get("extra_aliases", [])
    if alias not in aliases:
        aliases.append(alias)
        providers[provider]["extra_aliases"] = aliases
        save_json(PROVIDER_DEFINITIONS_PATH, providers)
        return True, f"Alias '{alias}' added to provider '{provider}'"

    return False, f"Alias '{alias}' already exists for provider '{provider}'"


def remove_model_alias(provider: str, alias: str) -> tuple[bool, str]:
    """Remove a model alias from a provider.

    Args:
        provider: Provider name.
        alias: The alias to remove.

    Returns:
        Tuple of (success: bool, message: str).
    """
    provider = provider.lower().strip()
    providers = load_json(PROVIDER_DEFINITIONS_PATH)

    if provider not in providers:
        return False, f"Provider '{provider}' not found"

    aliases = providers[provider].get("extra_aliases", [])
    if alias in aliases:
        aliases.remove(alias)
        providers[provider]["extra_aliases"] = aliases
        save_json(PROVIDER_DEFINITIONS_PATH, providers)
        return True, f"Alias '{alias}' removed from provider '{provider}'"

    return False, f"Alias '{alias}' not found for provider '{provider}'"


def add_common_alias(alias: str) -> tuple[bool, str]:
    """Add a common model alias that works across providers.

    Args:
        alias: The alias to add.

    Returns:
        Tuple of (success: bool, message: str).
    """
    models = load_json(MODEL_DEFINITIONS_PATH)

    common_aliases = models.get("common_aliases", [])
    if alias not in common_aliases:
        common_aliases.append(alias)
        models["common_aliases"] = common_aliases
        save_json(MODEL_DEFINITIONS_PATH, models)
        return True, f"Common alias '{alias}' added"

    return False, f"Common alias '{alias}' already exists"


def remove_common_alias(alias: str) -> tuple[bool, str]:
    """Remove a common model alias.

    Args:
        alias: The alias to remove.

    Returns:
        Tuple of (success: bool, message: str).
    """
    models = load_json(MODEL_DEFINITIONS_PATH)

    common_aliases = models.get("common_aliases", [])
    if alias in common_aliases:
        common_aliases.remove(alias)
        models["common_aliases"] = common_aliases
        save_json(MODEL_DEFINITIONS_PATH, models)
        return True, f"Common alias '{alias}' removed"

    return False, f"Common alias '{alias}' not found"
