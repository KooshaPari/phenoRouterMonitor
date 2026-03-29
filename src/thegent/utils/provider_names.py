"""Provider name normalization and type utilities.

This module is intentionally decoupled from routing_impl to avoid circular imports
with models.catalog.
"""

from typing import Final

_PROVIDER_ALIASES: Final[dict[str, str]] = {
    "ollama-local": "ollama",
    "local-ollama": "ollama",
    "ollama-localhost": "ollama",
    "ollama@localhost": "ollama",
}


def normalize_provider_name(provider: str) -> str:
    """Normalize provider aliases into canonical routing names."""
    value = (provider or "").strip().lower()
    return _PROVIDER_ALIASES.get(value, value)
