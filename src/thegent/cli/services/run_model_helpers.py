"""Run model resolution helpers extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

from typing import Any


def resolve_agent_model(
    *,
    agent: str,
    model: str | None,
    mode: str,
    settings: Any,
) -> str | None:
    """Resolve effective model for an agent given runtime mode and settings."""
    if model:
        return model
    if agent in ("cursor-agent", "cursor"):
        return settings.default_cursor_model
    if agent == "gemini":
        return settings.default_gemini_model
    if agent == "copilot":
        return settings.default_copilot_model
    if agent == "claude":
        return settings.default_claude_model
    if agent == "codex":
        return settings.default_codex_model_high if mode == "full" else settings.default_codex_model
    if agent == "antigravity":
        return settings.default_antigravity_model
    if agent == "minimax":
        return "minimax-m2.5"
    if agent == "glm":
        return "glm-5"
    if agent == "roo":
        return "roo-default"
    if agent == "kilo":
        return "kilo-default"
    return None


def validate_explicit_ollama_provider(*, provider: str | None, model: str | None) -> str | None:
    """Validate explicit --provider ollama request for daemon/model readiness.

    Returns an actionable error string when validation fails, otherwise ``None``.
    """
    if not provider or not model:
        return None

    from thegent.utils.routing_impl.provider_types import normalize_provider_name

    if normalize_provider_name(provider) != "ollama":
        return None

    from thegent.models import normalize_model_id
    from thegent.utils.routing_impl.ollama_provider import (
        OllamaUnavailableError,
        assert_ollama_available,
        get_available_models,
        resolve_ollama_model,
    )

    try:
        assert_ollama_available()
        available_models = get_available_models()
    except OllamaUnavailableError as exc:
        return str(exc)

    if not available_models:
        return (
            "Ollama provider is reachable but no local models are installed. Install one with `ollama pull llama3.3`."
        )

    requested = resolve_ollama_model(normalize_model_id(model))
    if requested not in set(available_models):
        visible = ", ".join(available_models[:8])
        suffix = " ..." if len(available_models) > 8 else ""
        return (
            f"Model '{model}' is not installed in local Ollama. "
            f"Installed models: {visible}{suffix}. "
            f"Install with `ollama pull {requested}`."
        )

    return None


__all__ = ["resolve_agent_model", "validate_explicit_ollama_provider"]
