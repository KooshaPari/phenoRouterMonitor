"""Provider CRUD operations (add, remove, update, list providers)."""

from __future__ import annotations

import logging
from typing import Any

from thegent.config import ThegentSettings
from thegent.domain.provider_config import OAUTH_ONLY_PROVIDERS
from thegent.agents.cliproxy_manager import _ensure_config
from thegent.provider_model_manager_cliproxy import (
    remove_openai_compat_entry,
    upsert_openai_compat_entry,
)
from thegent.provider_model_manager_io import (
    PROVIDER_DEFINITIONS_PATH,
    PROVIDER_MAPPING_PATH,
    load_json,
    load_yaml,
    save_json,
    save_yaml,
    update_provider_mapping as update_provider_mapping_file,
)

_LOG = logging.getLogger(__name__)


def list_providers(include_credentials: bool = False) -> list[dict[str, Any]]:
    """List all configured providers.

    Args:
        include_credentials: If False, strips sensitive API keys and credentials.

    Returns:
        List of provider configurations.
    """
    providers = load_json(PROVIDER_DEFINITIONS_PATH)
    result = []
    for name, cfg in providers.items():
        entry = {"name": name, **cfg}
        if not include_credentials:
            # Remove sensitive info
            entry.pop("api_key", None)
            if "login" in entry:
                entry["login"] = {k: v for k, v in entry["login"].items() if k != "credentials"}
        result.append(entry)
    return result


def get_provider(name: str) -> dict[str, Any] | None:
    """Get a specific provider by name.

    Args:
        name: Provider name (case-insensitive).

    Returns:
        Provider configuration dict, or None if not found.
    """
    providers = load_json(PROVIDER_DEFINITIONS_PATH)
    return providers.get(name.lower())


def add_provider(
    name: str,
    base_url: str,
    model: str,
    login_url: str | None = None,
    login_instructions: list[str] | None = None,
    display_name: str | None = None,
    extra_aliases: list[str] | None = None,
    api_key: str | None = None,
    base_url_env: str | None = None,
) -> tuple[bool, str]:
    """Add a new provider.

    Args:
        name: Provider name.
        base_url: Base URL for the provider API.
        model: Default model name.
        login_url: Optional login/authentication URL.
        login_instructions: Optional list of login instruction steps.
        display_name: Display name for login UI.
        extra_aliases: Additional model aliases.
        api_key: API key to store in CLIProxy config.
        base_url_env: Environment variable name for base_url.

    Returns:
        Tuple of (success: bool, message: str).
    """
    name = name.lower().strip()
    if name in OAUTH_ONLY_PROVIDERS:
        return False, f"Provider '{name}' uses OAuth only. Use: thegent cliproxy login {name}"
    providers = load_json(PROVIDER_DEFINITIONS_PATH)

    if name in providers:
        return False, f"Provider '{name}' already exists"

    provider_cfg: dict[str, Any] = {
        "base_url": base_url,
        "model": model,
    }

    if base_url_env:
        provider_cfg["base_url_env"] = base_url_env

    if extra_aliases:
        provider_cfg["extra_aliases"] = extra_aliases

    if login_url or login_instructions:
        provider_cfg["login"] = {
            "url": login_url or "",
            "display_name": display_name or name.title(),
            "instructions": login_instructions or [],
        }

    # Add API key to CLIProxy config if provided
    if api_key:
        settings = ThegentSettings()
        config_path = _ensure_config(settings)
        config = load_yaml(config_path)

        # Add to openai-compatibility
        compat = config.get("openai-compatibility", [])
        if not isinstance(compat, list):
            compat = []

        upsert_openai_compat_entry(
            compat,
            name=name,
            base_url=base_url,
            model=model,
            api_key=api_key,
        )

        config["openai-compatibility"] = compat
        save_yaml(config_path, config)

    providers[name] = provider_cfg
    save_json(PROVIDER_DEFINITIONS_PATH, providers)

    # Update provider_mapping.json
    _update_provider_mapping(name, is_openai_compat=True)

    return True, f"Provider '{name}' added successfully"


def update_provider(
    name: str,
    base_url: str | None = None,
    model: str | None = None,
    login_url: str | None = None,
    login_instructions: list[str] | None = None,
    display_name: str | None = None,
    extra_aliases: list[str] | None = None,
    api_key: str | None = None,
    base_url_env: str | None = None,
) -> tuple[bool, str]:
    """Update an existing provider.

    Args:
        name: Provider name.
        base_url: New base URL (optional).
        model: New default model (optional).
        login_url: New login URL (optional).
        login_instructions: New login instructions (optional).
        display_name: New display name (optional).
        extra_aliases: New aliases (optional).
        api_key: New API key (optional).
        base_url_env: New env var name (optional).

    Returns:
        Tuple of (success: bool, message: str).
    """
    name = name.lower().strip()
    if name in OAUTH_ONLY_PROVIDERS:
        return False, f"Provider '{name}' uses OAuth only. Use: thegent cliproxy login {name}"
    providers = load_json(PROVIDER_DEFINITIONS_PATH)

    if name not in providers:
        return False, f"Provider '{name}' not found"

    if base_url:
        providers[name]["base_url"] = base_url
    if model:
        providers[name]["model"] = model
    if base_url_env:
        providers[name]["base_url_env"] = base_url_env
    if extra_aliases is not None:
        providers[name]["extra_aliases"] = extra_aliases

    if login_url or login_instructions or display_name:
        login = providers[name].get("login", {})
        if login_url:
            login["url"] = login_url
        if display_name:
            login["display_name"] = display_name
        if login_instructions:
            login["instructions"] = login_instructions
        providers[name]["login"] = login

    # Update API key if provided
    if api_key:
        settings = ThegentSettings()
        config_path = _ensure_config(settings)
        config = load_yaml(config_path)

        compat = config.get("openai-compatibility", [])
        upsert_openai_compat_entry(
            compat,
            name=name,
            base_url=base_url or providers[name].get("base_url", ""),
            model=model or providers[name].get("model", ""),
            api_key=api_key,
        )

        config["openai-compatibility"] = compat
        save_yaml(config_path, config)

    save_json(PROVIDER_DEFINITIONS_PATH, providers)
    return True, f"Provider '{name}' updated successfully"


def delete_provider(name: str, remove_credentials: bool = True) -> tuple[bool, str]:
    """Delete a provider.

    Args:
        name: Provider name.
        remove_credentials: If True, also removes credentials from CLIProxy config.

    Returns:
        Tuple of (success: bool, message: str).
    """
    name = name.lower().strip()
    providers = load_json(PROVIDER_DEFINITIONS_PATH)

    if name not in providers:
        return False, f"Provider '{name}' not found"

    del providers[name]
    save_json(PROVIDER_DEFINITIONS_PATH, providers)

    # Update provider_mapping.json
    _update_provider_mapping(name, remove=True)

    # Remove from CLIProxy config if requested
    if remove_credentials:
        settings = ThegentSettings()
        config_path = _ensure_config(settings)
        config = load_yaml(config_path)

        compat = config.get("openai-compatibility", [])
        config["openai-compatibility"] = remove_openai_compat_entry(compat, name)
        save_yaml(config_path, config)

    return True, f"Provider '{name}' deleted successfully"


def _update_provider_mapping(name: str, is_openai_compat: bool = False, remove: bool = False) -> None:
    """Update provider_mapping.json.

    Args:
        name: Provider name.
        is_openai_compat: If True, marks provider as openai-compatible.
        remove: If True, removes the provider from all lists.
    """
    update_provider_mapping_file(
        PROVIDER_MAPPING_PATH,
        name,
        is_openai_compat=is_openai_compat,
        remove=remove,
    )


__all__ = [
    "list_providers",
    "get_provider",
    "add_provider",
    "update_provider",
    "delete_provider",
    "_update_provider_mapping",
]
