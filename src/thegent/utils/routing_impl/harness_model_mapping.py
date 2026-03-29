"""Provider-harness-model mapping for universal parity across Codex, LiteLLM, and CLIProxy.

Ensures consistent model resolution and metadata when requests flow through:
- Codex harness (dex) -> CLIProxy adapter -> CLIProxyAPIPlus
- LiteLLM Router -> CLIProxyAPIPlus
- Direct CLIProxy API

When clode harness pairs with minimax/kilo + MiniMax-M2.5, see Minimax clode guidance:
https://platform.minimax.io/docs/coding-plan/claude-code
"""

from __future__ import annotations

# Codex/MiniMax/GLM model aliases -> CLIProxy backend (catalog) model IDs
# Used by cliproxy_adapter for request translation and /v1/models enrichment
CODEX_TO_BACKEND_MODEL: dict[str, str] = {
    # MiniMax (Codex CLI guide: codex-MiniMax-M2.5)
    "codex-MiniMax-M2.5": "minimax-m2.5",
    "codex-minimax-m2.5": "minimax-m2.5",
    "MiniMax-M2.5": "minimax-m2.5",
    # GLM
    "codex-GLM-5": "glm-5",
    "codex-glm-5": "glm-5",
    "GLM-5": "glm-5",
    # Kilo, Roo
    "codex-kilo-default": "kilo-default",
    "codex-roo-default": "roo-default",
    # Claude thinking aliases
    "claude-opus-4-6-thinking": "claude-opus-4-6",
    "claude-opus-4.6-thinking": "claude-opus-4-6",
}

# Thegent canonical alias -> OpenRouter model ID
# OpenRouter format: provider/model-name
CANONICAL_TO_OPENROUTER: dict[str, str] = {
    # Anthropic Claude — dash variants (canonical thegent style)
    "claude-opus-4-6": "anthropic/claude-opus-4-6",
    "claude-sonnet-4-6": "anthropic/claude-sonnet-4-6",
    "claude-sonnet-4-5": "anthropic/claude-sonnet-4-5",
    "claude-haiku-4-5": "anthropic/claude-haiku-4-5-20251001",
    # Anthropic Claude — dot variants (model_metadata style)
    "claude-opus-4.6": "anthropic/claude-opus-4-6",
    "claude-sonnet-4.6": "anthropic/claude-sonnet-4-6",
    "claude-sonnet-4.5": "anthropic/claude-sonnet-4-5",
    "claude-haiku-4.5": "anthropic/claude-haiku-4-5-20251001",
    # OpenAI
    "gpt-4o": "openai/gpt-4o",
    "gpt-4o-mini": "openai/gpt-4o-mini",
    "gpt-4-turbo": "openai/gpt-4-turbo",
    "gpt-5": "openai/gpt-4o",
    "gpt-5-mini": "openai/gpt-4o-mini",
    "o3": "openai/o3",
    "o3-mini": "openai/o3-mini",
    "o4-mini": "openai/o4-mini",
    # Google Gemini
    "gemini-2.0-flash": "google/gemini-2.0-flash-001",
    "gemini-2.5-flash": "google/gemini-2.5-flash-preview",
    "gemini-3-flash": "google/gemini-2.0-flash-001",
    "gemini-3.1-pro": "google/gemini-pro-1.5",
    # DeepSeek
    "deepseek-v3.2": "deepseek/deepseek-chat",
    "deepseek-r2": "deepseek/deepseek-r1",
    # Meta Llama via NVIDIA NIM
    "llama-nemotron-ultra": "nvidia/llama-3.1-nemotron-ultra-253b-v1",
    # Qwen
    "qwen3-coder": "qwen/qwen-2.5-coder-32b-instruct",
    # MiniMax
    "minimax-m2.5": "minimax/minimax-01",
    # Zhipu GLM
    "glm-5": "zhipu/glm-4-9b",
    # Kimi (Moonshot)
    "kimi-k2.5": "moonshot/moonshot-v1-128k",
}

# Ollama local model aliases: thegent short name -> Ollama model name
# Used for --provider ollama --model <alias> CLI resolution
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

# Case-insensitive alias normalization. Keys must be lowercase.
_LOWERCASE_BACKEND_MODEL_ALIASES: dict[str, str] = {key.lower(): value for key, value in CODEX_TO_BACKEND_MODEL.items()}
_LOWERCASE_BACKEND_MODEL_ALIASES.update(
    {
        "minimax-m2.5": "minimax-m2.5",
        "minimax/minimax-m2.5": "minimax-m2.5",
        "minimax/minimax-01": "minimax-m2.5",
    }
)


def resolve_model_for_backend(model: str) -> str:
    """Map Codex/provider-specific model ID to CLIProxy backend model ID."""
    mapped = CODEX_TO_BACKEND_MODEL.get(model)
    if mapped:
        return mapped
    normalized = _LOWERCASE_BACKEND_MODEL_ALIASES.get(model.lower())
    return normalized if normalized else model


def resolve_openrouter_model(model: str) -> str:
    """Map any thegent model alias to OpenRouter provider/model format.

    Tries CANONICAL_TO_OPENROUTER first. If not found and model contains '/',
    returns as-is (already in provider/model format). Otherwise returns model unchanged.
    """
    if model in CANONICAL_TO_OPENROUTER:
        return CANONICAL_TO_OPENROUTER[model]
    if "/" in model:
        return model
    return model


def is_openrouter_model_id(model: str) -> bool:
    """Return True if model string is in OpenRouter provider/model format (contains '/')."""
    return "/" in model


def get_openrouter_models() -> list[str]:
    """Return list of all OpenRouter model IDs we can route to."""
    return list(CANONICAL_TO_OPENROUTER.values())


def resolve_ollama_model_alias(model: str) -> str:
    """Map a thegent short model name to the canonical Ollama model name.

    Strips an ``ollama/`` prefix first, then looks up in ``OLLAMA_MODEL_ALIASES``.
    Falls back to the raw (stripped) name if no alias is registered.

    Args:
        model: Short alias (e.g. ``"llama3.3"``) or prefixed form
               (e.g. ``"ollama/llama3.3"``).

    Returns:
        Canonical Ollama model name (e.g. ``"llama3.3"``).
    """
    stripped = model.removeprefix("ollama/")
    return OLLAMA_MODEL_ALIASES.get(stripped, stripped)


def get_ollama_models() -> list[str]:
    """Return list of all registered Ollama model aliases."""
    return list(OLLAMA_MODEL_ALIASES.keys())
