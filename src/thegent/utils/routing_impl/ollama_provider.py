"""Ollama local model provider for zero-cost execution.

Ollama exposes an OpenAI-compatible REST API at http://localhost:11434/v1.
This module provides availability detection and model discovery; actual
inference routing goes through LiteLLM with the ``ollama`` provider prefix.

WL-118: Ollama as a local model provider.
"""

from __future__ import annotations

import logging

import httpx

logger = logging.getLogger(__name__)

OLLAMA_BASE_URL = "http://localhost:11434"
OLLAMA_TAGS_ENDPOINT = f"{OLLAMA_BASE_URL}/api/tags"
OLLAMA_OPENAI_BASE = f"{OLLAMA_BASE_URL}/v1"
OLLAMA_PROBE_TIMEOUT = 2.0

# Well-known model aliases for thegent -> Ollama model name resolution
OLLAMA_MODEL_ALIASES: dict[str, str] = {
    "llama3.3": "llama3.3",
    "llama3.2": "llama3.2",
    "llama3.1": "llama3.1",
    "llama3": "llama3",
    "llama2": "llama2",
    "qwen2.5-coder": "qwen2.5-coder",
    "qwen2.5": "qwen2.5",
    "qwen2": "qwen2",
    "mistral": "mistral",
    "mistral-nemo": "mistral-nemo",
    "codellama": "codellama",
    "deepseek-coder-v2": "deepseek-coder-v2",
    "deepseek-r1": "deepseek-r1",
    "phi4": "phi4",
    "phi3.5": "phi3.5",
    "gemma3": "gemma3",
    "gemma2": "gemma2",
}

# Provider name constants
OLLAMA_PROVIDER_NAME = "ollama"
OLLAMA_LITELLM_PREFIX = "ollama"


def is_ollama_available() -> bool:
    """Check if Ollama daemon is running at localhost:11434.

    Performs a GET /api/tags probe with a short timeout.  Returns False for
    any network or HTTP error so callers can treat Ollama as unavailable
    without crashing.

    Returns:
        True if the daemon responded with HTTP 200, False otherwise.
    """
    try:
        resp = httpx.get(OLLAMA_TAGS_ENDPOINT, timeout=OLLAMA_PROBE_TIMEOUT)
        return resp.status_code == 200
    except (httpx.ConnectError, httpx.TimeoutException, httpx.RequestError):
        return False


def get_available_models() -> list[str]:
    """Return list of locally available Ollama model names.

    Calls GET /api/tags and extracts the ``name`` field from each entry in the
    ``models`` array.  Raises ``OllamaUnavailableError`` if the daemon is not
    reachable so routing code can fail loudly when Ollama was explicitly
    requested.

    Returns:
        Sorted list of model name strings (e.g. ``["llama3.3", "mistral"]``).

    Raises:
        OllamaUnavailableError: If daemon is not reachable or returns non-200.
    """
    try:
        resp = httpx.get(OLLAMA_TAGS_ENDPOINT, timeout=OLLAMA_PROBE_TIMEOUT)
    except (httpx.ConnectError, httpx.TimeoutException, httpx.RequestError) as exc:
        msg = f"Ollama daemon is not reachable at {OLLAMA_BASE_URL}: {exc}"
        raise OllamaUnavailableError(msg) from exc

    if resp.status_code != 200:
        msg = f"Ollama /api/tags returned HTTP {resp.status_code}"
        raise OllamaUnavailableError(msg)

    body = resp.json() if resp.content else {}
    models_raw = body.get("models") if isinstance(body, dict) else None
    if not isinstance(models_raw, list):
        return []

    names: list[str] = []
    for entry in models_raw:
        if isinstance(entry, dict):
            name = entry.get("name")
            if isinstance(name, str) and name:
                # Strip tag suffix (e.g. "llama3.3:latest" -> "llama3.3")
                names.append(name.split(":")[0])
    return sorted(set(names))


def assert_ollama_available() -> None:
    """Raise OllamaUnavailableError if Ollama daemon is not reachable.

    Use this at the start of any code path where ``--provider ollama`` was
    explicitly requested so we fail loudly rather than silently falling back.

    Raises:
        OllamaUnavailableError: If the daemon is not reachable.
    """
    if not is_ollama_available():
        msg = (
            f"Ollama provider was explicitly requested but the daemon is not "
            f"reachable at {OLLAMA_BASE_URL}. "
            f"Start it with `ollama serve` and ensure at least one model is "
            f"installed (`ollama pull llama3.3`)."
        )
        raise OllamaUnavailableError(msg)


def resolve_ollama_model(model: str) -> str:
    """Resolve a thegent model alias to a canonical Ollama model name.

    Strips an ``ollama/`` prefix if present, then looks up the alias in
    ``OLLAMA_MODEL_ALIASES``.  Falls back to the raw name if no alias exists.

    Args:
        model: Model identifier (e.g. ``"llama3.3"``, ``"ollama/mistral"``).

    Returns:
        Canonical Ollama model name (e.g. ``"llama3.3"``).
    """
    stripped = model.removeprefix(f"{OLLAMA_LITELLM_PREFIX}/")
    return OLLAMA_MODEL_ALIASES.get(stripped, stripped)


def build_litellm_entry(model: str) -> dict[str, object]:
    """Build a LiteLLM model_list entry for a local Ollama model.

    Args:
        model: Ollama model name (e.g. ``"llama3.3"``).

    Returns:
        LiteLLM model_list entry dict with ``model_name`` and
        ``litellm_params`` (including ``api_base`` and ``api_key``).
    """
    canonical = resolve_ollama_model(model)
    return {
        "model_name": canonical,
        "litellm_params": {
            "model": f"{OLLAMA_LITELLM_PREFIX}/{canonical}",
            "api_base": OLLAMA_OPENAI_BASE,
            # Ollama does not require a real API key; use a sentinel so LiteLLM
            # does not refuse to build the config.
            "api_key": "ollama-no-key",
        },
    }


class OllamaUnavailableError(RuntimeError):
    """Raised when Ollama provider is explicitly requested but unreachable."""
