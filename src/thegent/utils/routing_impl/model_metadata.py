"""Model metadata registry for all models."""

from datetime import datetime, timedelta, UTC
from typing import Any
from thegent.utils.routing_impl.harness_model_mapping import resolve_model_for_backend

# Comprehensive model metadata registry
MODEL_METADATA: dict[str, dict[str, Any]] = {
    # Anthropic Claude
    "claude-haiku-4.5": {
        "context_window": 200000,
        "cost_per_mtok": 0.50,
        "provider": "claude",
        "backend": "direct",
    },
    "claude-sonnet-4.5": {
        "context_window": 200000,
        "cost_per_mtok": 3.00,
        "provider": "claude",
        "backend": "direct",
    },
    "claude-sonnet-4.6": {
        "context_window": 200000,
        "cost_per_mtok": 3.00,
        "provider": "claude",
        "backend": "direct",
    },
    "claude-opus-4.6": {
        "context_window": 200000,
        "cost_per_mtok": 15.00,
        "provider": "claude",
        "backend": "direct",
    },
    "claude-opus-4.6-1m": {
        "context_window": 1000000,
        "cost_per_mtok": 18.00,
        "provider": "claude",
        "backend": "direct",
    },
    # Google Gemini
    "gemini-2.0-flash": {
        "context_window": 1000000,
        "cost_per_mtok": 0.15,
        "provider": "gemini",
        "backend": "direct",
    },
    "gemini-2.5-flash": {
        "context_window": 1000000,
        "cost_per_mtok": 0.15,
        "provider": "gemini",
        "backend": "direct",
    },
    "gemini-3-flash": {
        "context_window": 1000000,
        "cost_per_mtok": 0.15,
        "provider": "gemini",
        "backend": "direct",
    },
    "gemini-3.1-pro": {
        "context_window": 200000,
        "cost_per_mtok": 3.50,
        "provider": "gemini",
        "backend": "direct",
    },
    # OpenAI / Codex
    "gpt-4o": {
        "context_window": 128000,
        "cost_per_mtok": 2.50,
        "provider": "openai",
        "backend": "direct",
    },
    "gpt-4o-mini": {
        "context_window": 128000,
        "cost_per_mtok": 0.15,
        "provider": "openai",
        "backend": "direct",
    },
    "gpt-5-mini": {
        "context_window": 128000,
        "cost_per_mtok": 0.15,
        "provider": "copilot",
        "backend": "direct",
    },
    "gpt-5.3-codex-spark": {
        "context_window": 128000,
        "cost_per_mtok": 1.20,
        "provider": "codex",
        "backend": "direct",
    },
    "gpt-5.3-codex": {
        "context_window": 128000,
        "cost_per_mtok": 3.00,
        "provider": "codex",
        "backend": "direct",
    },
    "gpt-5.3-codex-high": {
        "context_window": 128000,
        "cost_per_mtok": 5.00,
        "provider": "codex",
        "backend": "direct",
    },
    "gpt-5.3-codex-max": {
        "context_window": 128000,
        "cost_per_mtok": 10.00,
        "provider": "codex",
        "backend": "direct",
    },
    # Zhipu GLM
    "glm-5": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "glm",
        "backend": "proxy",
    },
    "GLM-5": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "glm",
        "backend": "proxy",
    },
    "z-ai/glm-5": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "nim",
        "backend": "proxy",
    },
    # MiniMax
    "minimax-m2.5": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "minimax",
        "backend": "proxy",
    },
    "MiniMax-M2.5": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "minimax",
        "backend": "proxy",
    },
    # Kilo
    "kilo-default": {
        "context_window": 128000,
        "cost_per_mtok": 0.50,
        "provider": "kilo",
        "backend": "proxy",
    },
    # Roo
    "roo-default": {
        "context_window": 128000,
        "cost_per_mtok": 0.50,
        "provider": "roo",
        "backend": "proxy",
    },
    # DeepSeek
    "deepseek-v3.2": {
        "context_window": 64000,
        "cost_per_mtok": 0.50,
        "provider": "deepseek",
        "backend": "direct",
    },
    # Kimi
    "kimi-k2.5": {
        "context_window": 200000,
        "cost_per_mtok": 0.50,
        "provider": "kimi",
        "backend": "proxy",
    },
    # Qwen
    "qwen3-coder": {
        "context_window": 32000,
        "cost_per_mtok": 0.30,
        "provider": "qwen",
        "backend": "proxy",
    },
    # Meta
    "llama-nemotron-ultra": {
        "context_window": 128000,
        "cost_per_mtok": 0.20,
        "provider": "meta",
        "backend": "proxy",
    },
    # Cursor
    "composer-1": {
        "context_window": 128000,
        "cost_per_mtok": 0.25,
        "provider": "cursor-agent",
        "backend": "direct",
    },
    "composer-1.5": {
        "context_window": 128000,
        "cost_per_mtok": 0.30,
        "provider": "cursor-agent",
        "backend": "direct",
    },
    # OpenRouter aliases (canonical thegent alias -> OpenRouter model ID)
    "anthropic/claude-opus-4-6": {
        "context_window": 200000,
        "cost_per_mtok": 15.00,
        "provider": "openrouter",
        "backend": "direct",
    },
    "anthropic/claude-sonnet-4-6": {
        "context_window": 200000,
        "cost_per_mtok": 3.00,
        "provider": "openrouter",
        "backend": "direct",
    },
    "anthropic/claude-haiku-4-5-20251001": {
        "context_window": 200000,
        "cost_per_mtok": 0.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    "openai/gpt-4o": {
        "context_window": 128000,
        "cost_per_mtok": 2.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    "google/gemini-2.0-flash-001": {
        "context_window": 1000000,
        "cost_per_mtok": 0.15,
        "provider": "openrouter",
        "backend": "direct",
    },
    "google/gemini-2.5-flash-preview": {
        "context_window": 1000000,
        "cost_per_mtok": 0.15,
        "provider": "openrouter",
        "backend": "direct",
    },
    "google/gemini-pro-1.5": {
        "context_window": 200000,
        "cost_per_mtok": 3.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    # OpenAI via OpenRouter
    "openai/gpt-4o-mini": {
        "context_window": 128000,
        "cost_per_mtok": 0.15,
        "provider": "openrouter",
        "backend": "direct",
    },
    "openai/gpt-4-turbo": {
        "context_window": 128000,
        "cost_per_mtok": 3.00,
        "provider": "openrouter",
        "backend": "direct",
    },
    "openai/o3": {
        "context_window": 200000,
        "cost_per_mtok": 10.00,
        "provider": "openrouter",
        "backend": "direct",
    },
    "openai/o3-mini": {
        "context_window": 200000,
        "cost_per_mtok": 1.00,
        "provider": "openrouter",
        "backend": "direct",
    },
    "openai/o4-mini": {
        "context_window": 200000,
        "cost_per_mtok": 0.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    # DeepSeek via OpenRouter
    "deepseek/deepseek-chat": {
        "context_window": 64000,
        "cost_per_mtok": 0.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    "deepseek/deepseek-r1": {
        "context_window": 64000,
        "cost_per_mtok": 0.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    # NVIDIA NIM via OpenRouter
    "nvidia/llama-3.1-nemotron-ultra-253b-v1": {
        "context_window": 128000,
        "cost_per_mtok": 0.20,
        "provider": "openrouter",
        "backend": "direct",
    },
    # Qwen via OpenRouter
    "qwen/qwen-2.5-coder-32b-instruct": {
        "context_window": 32000,
        "cost_per_mtok": 0.30,
        "provider": "openrouter",
        "backend": "direct",
    },
    # MiniMax via OpenRouter
    "minimax/minimax-01": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "openrouter",
        "backend": "direct",
    },
    # Zhipu GLM via OpenRouter
    "zhipu/glm-4-9b": {
        "context_window": 128000,
        "cost_per_mtok": 0.40,
        "provider": "openrouter",
        "backend": "direct",
    },
    # Kimi/Moonshot via OpenRouter
    "moonshot/moonshot-v1-128k": {
        "context_window": 128000,
        "cost_per_mtok": 0.50,
        "provider": "openrouter",
        "backend": "direct",
    },
    # Anthropic Claude via OpenRouter (additional variants)
    "anthropic/claude-sonnet-4-5": {
        "context_window": 200000,
        "cost_per_mtok": 3.00,
        "provider": "openrouter",
        "backend": "direct",
    },
    # Ollama local models — zero cloud cost
    "llama3.3": {
        "context_window": 128000,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "llama3.2": {
        "context_window": 128000,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "llama3.1": {
        "context_window": 128000,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "llama3": {
        "context_window": 8192,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "qwen2.5-coder": {
        "context_window": 32768,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "mistral": {
        "context_window": 32768,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "codellama": {
        "context_window": 16384,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "deepseek-coder-v2": {
        "context_window": 131072,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "phi4": {
        "context_window": 16384,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
    "gemma3": {
        "context_window": 8192,
        "cost_per_mtok": 0.0,
        "provider": "ollama",
        "backend": "direct",
    },
}


def stamp_metadata_freshness(
    metadata: dict[str, Any],
    *,
    fetched_at: datetime | None = None,
    ttl_seconds: int = 3600,
) -> dict[str, Any]:
    """Return metadata with freshness envelope fields."""
    if ttl_seconds <= 0:
        raise ValueError("ttl_seconds must be > 0")
    now = fetched_at or datetime.now(UTC)
    envelope = dict(metadata)
    envelope["fetched_at"] = now.isoformat()
    envelope["expires_at"] = (now + timedelta(seconds=ttl_seconds)).isoformat()
    envelope["ttl_seconds"] = ttl_seconds
    envelope["freshness_status"] = "fresh"
    return envelope


def mark_metadata_stale(metadata: dict[str, Any]) -> dict[str, Any]:
    """Mark metadata envelope as stale."""
    stale = dict(metadata)
    stale["freshness_status"] = "stale"
    return stale


def validate_metadata_freshness(
    metadata: dict[str, Any],
    *,
    now: datetime | None = None,
) -> dict[str, Any]:
    """Return metadata with explicit freshness marker based on expires_at."""
    current = now or datetime.now(UTC)
    expires_at_raw = metadata.get("expires_at")
    if not isinstance(expires_at_raw, str):
        return mark_metadata_stale(metadata)
    expires_at = datetime.fromisoformat(expires_at_raw.replace("Z", "+00:00"))
    if current > expires_at:
        return mark_metadata_stale(metadata)
    fresh = dict(metadata)
    fresh["freshness_status"] = "fresh"
    return fresh


def _normalize_model_id_token(model_id: str) -> str:
    """Normalize model IDs to token-comparable form."""
    return model_id.lower().replace("-", "").replace(".", "").replace("/", "").replace("_", "")


def _model_id_candidates(model_id: str) -> list[str]:
    """Return increasingly specific candidate IDs for lookup.

    Handles provider/custom prefixes and mixed namespace styles
    without requiring a strict one-to-one match.
    """
    raw = model_id.strip()
    if not raw:
        return []

    candidates: list[str] = []
    queue: list[str] = [raw]
    seen: set[str] = set()

    while queue:
        candidate = queue.pop(0)
        if candidate in seen:
            continue
        seen.add(candidate)
        candidates.append(candidate)

        lower = candidate.strip().lower()

        # Prefix-stripping for transport wrappers (custom:, provider:, openrouter/, etc.)
        if ":" in candidate:
            queue.append(candidate.split(":", 1)[1].strip())
        if "/" in candidate:
            queue.append(candidate.split("/", 1)[1].strip())

        for sep in ("-", "_"):
            for prefix in ("openrouter", "custom", "provider", "thegent", "codex"):
                marker = f"{prefix}{sep}"
                if lower.startswith(marker):
                    queue.append(candidate[len(marker) :].strip())

    return candidates


def get_model_metadata(model_id: str) -> dict[str, Any] | None:
    """Get comprehensive metadata for a model.

    Args:
        model_id: Model identifier (may be alias or canonical name)

    Returns:
        Model metadata dict or None if not found
    """
    resolved_model_id = resolve_model_for_backend(model_id)
    candidates = _model_id_candidates(resolved_model_id)
    if resolved_model_id != model_id:
        candidates.extend(_model_id_candidates(model_id))

    for candidate in candidates:
        if candidate in MODEL_METADATA:
            return MODEL_METADATA[candidate]

    normalized_map = {_normalize_model_id_token(key): metadata for key, metadata in MODEL_METADATA.items()}
    for candidate in candidates:
        normalized_candidate = _normalize_model_id_token(candidate)
        if normalized_candidate in normalized_map:
            return normalized_map[normalized_candidate]

    return None


def has_model_metadata(model_id: str) -> bool:
    """Check if model has metadata available.

    Args:
        model_id: Model identifier

    Returns:
        True if metadata exists, False otherwise
    """
    return get_model_metadata(model_id) is not None


def get_all_models_with_metadata() -> list[str]:
    """Get list of all models with metadata.

    Returns:
        List of model IDs
    """
    return list(MODEL_METADATA.keys())
