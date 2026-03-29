"""Google Search Grounding support via Gemini API passthrough."""

from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Any

import litellm

from thegent.agents.base import RunResult


# Agents that are Gemini-backed and eligible for grounding.
GEMINI_GROUNDING_AGENTS = frozenset({"gemini", "antigravity"})
GROUNDING_AGENTS = GEMINI_GROUNDING_AGENTS


@dataclass
class GroundingSource:
    """A single grounding source returned by the grounding API."""

    url: str
    title: str | None = None


def build_grounding_tools_arg() -> list[dict[str, dict[str, str]]]:
    """Return the grounding tools argument for Gemini search grounding."""
    return [{"google_search": {}}]


def extract_grounding_metadata_sources(response: Any) -> list[str]:
    """Extract grounding source URLs from a grounded response payload."""
    payload = response.get("groundingMetadata", {}) if isinstance(response, dict) else {}
    chunks = payload.get("groundingChunks", []) if isinstance(payload, dict) else []

    if not isinstance(chunks, list):
        return []

    urls: list[str] = []
    seen: set[str] = set()
    for chunk in chunks:
        if not isinstance(chunk, dict):
            continue
        web = chunk.get("web")
        if not isinstance(web, dict):
            continue
        uri = web.get("uri")
        if isinstance(uri, str) and uri not in seen:
            seen.add(uri)
            urls.append(uri)
    return urls


def _resolve_gemini_api_key() -> str:
    """Resolve the Gemini API key from environment."""
    key = os.getenv("GEMINI_API_KEY") or os.getenv("GOOGLE_API_KEY")
    if not key:
        raise ValueError("Gemini API key is required for grounding")
    return key


def _resolve_gemini_model(model: str | None) -> str:
    """Resolve the model to use for grounding calls."""
    if model:
        return model
    return "gemini/gemini-2.0-flash"


def _extract_grounding_metadata(response: Any) -> list[str] | None:
    """Extract grounding metadata URLs from litellm-style result objects."""
    if response is None:
        return None

    payload = {}
    if hasattr(response, "_hidden_params") and isinstance(response._hidden_params, dict):
        payload = response._hidden_params
    elif hasattr(response, "model_extra") and isinstance(response.model_extra, dict):
        payload = response.model_extra
    elif hasattr(response, "additional_kwargs") and isinstance(response.additional_kwargs, dict):
        payload = response.additional_kwargs

    sources = extract_grounding_metadata_sources(payload)
    if not sources:
        return None
    return sources


def _resolve_completion_response_text(response: Any) -> str:
    """Extract completion text from a litellm response."""
    if hasattr(response, "choices"):
        choices = response.choices
        if isinstance(choices, list) and choices:
            choice0 = choices[0]
            message = getattr(choice0, "message", None)
            if message is not None:
                content = getattr(message, "content", None)
                if isinstance(content, str):
                    return content
    if isinstance(response, dict):
        choices = response.get("choices", [])
        if isinstance(choices, list) and choices:
            message = choices[0].get("message", {}) if isinstance(choices[0], dict) else {}
            content = message.get("content") if isinstance(message, dict) else None
            if isinstance(content, str):
                return content
    return ""


def run_gemini_with_grounding(
    prompt: str,
    *,
    model: str | None = None,
    api_key: str | None = None,
    timeout: int = 120,
    use_mcp_grounding: bool = False,
) -> RunResult:
    """Run a prompt with Gemini grounding enabled."""
    resolved_model = _resolve_gemini_model(model)
    if not resolved_model.startswith("gemini/"):
        raise ValueError("Google grounding requires a Gemini model")

    resolved_key = api_key or _resolve_gemini_api_key()
    _ = resolved_key

    kwargs = {
        "model": resolved_model,
        "messages": [{"role": "user", "content": prompt}],
        "tools": build_grounding_tools_arg(),
        "timeout": timeout,
    }

    try:
        response = litellm.completion(**kwargs)
    except Exception as exc:
        raise RuntimeError(f"Gemini grounding API call failed: {exc}") from exc

    grounding_sources = _extract_grounding_metadata(response)
    text = _resolve_completion_response_text(response)

    return RunResult(
        exit_code=0,
        stdout=text,
        stderr="",
        grounding_sources=grounding_sources,
    )
