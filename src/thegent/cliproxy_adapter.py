# MIGRATION NOTE: Migrate to cliproxyapi-plusplus Go SDK
"""CLIProxy adapter: exposes /v1/responses (HTTP + WebSocket) for Codex compatibility.

DEPRECATED: This module is now a thin shim for backward compatibility.
New code should use the decomposed modules:
- thegent.adapters.driven.cliproxy_http — HTTP client adapter
- thegent.use_cases.manage_cliproxy — Business logic
- thegent.ports.driven.cliproxy — Port interfaces

cliproxyapi++ (kooshapari fork) may not implement /v1/responses. This adapter:
- Proxies all /v1/* to the backend
- For POST /v1/responses: tries backend first; on 404, translates to /v1/chat/completions
- For WebSocket /v1/responses: bridges WS to HTTP streaming (SSE)
"""

from __future__ import annotations

import contextlib
import orjson as json
import logging
import time
import uuid
from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import Any, cast

from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import Response, StreamingResponse
from starlette.routing import Route, WebSocketRoute
from starlette.types import Send

from thegent.config import ThegentSettings
from thegent.integrations.bifrost import get_bifrost, BifrostValidationError
from thegent.adapters.ports import AdapterRegistry
from thegent.cliproxy_error_utils import (
    _ERROR_MESSAGES,
    _RETRY_MAX_ATTEMPTS,
    _RetryableStreamError,
    InsufficientCreditsError,
    _make_error_body,
)
from thegent.cliproxy_header_utils import (
    extract_websocket_forward_headers,
    filter_inbound_response_headers,
    sanitize_outbound_request_headers,
)
from thegent.cliproxy_models_transform import (
    _compute_models_etag,
    transform_models_response,
)
from thegent.cliproxy_request_transform import (
    _extract_delta_content,
    _extract_delta_tool_calls,
    _extract_usage,
    _OR_PASSTHROUGH_FIELDS,
    build_openrouter_passthrough_body as _build_openrouter_passthrough_body,
    _responses_to_chat_completions as _request_transform_to_chat_completions,
    _map_model_for_backend,
    _process_sse_line,
)
from thegent.cliproxy_stream_state import ResponsesStreamState
from thegent.utils.routing_impl.cost_calculator import calculate_cost_from_response, format_cost_header_value

# Import decomposed adapter module
from thegent.adapters.driven.cliproxy_http import (
    CliproxyHTTPClient,
    CliproxyResponseTransformer,
    CliproxyHeaderManager,
)

_log = logging.getLogger(__name__)


# Backward-compatible symbol aliases for historical adapter import surface.
class _LegacyModelsTransformResult(bytes):
    """Legacy test compatibility object that supports both old and new unpack protocols."""

    _full_body: bytes
    _etag: str

    def __new__(cls, compact_body: bytes, full_body: bytes, etag: str):
        obj = super().__new__(cls, compact_body)
        obj._full_body = full_body
        obj._etag = etag
        return obj

    def __iter__(self):
        yield self._full_body
        yield self._etag


def _transform_models_response(content: bytes | memoryview, *, inject_openrouter: bool = False) -> bytes | None:
    """Return the legacy adapter helper output as response bytes only."""
    try:
        raw = bytes(content) if isinstance(content, memoryview) else content
        parsed = json.loads(raw.decode(errors="replace"))
        if isinstance(parsed, dict) and "models" in parsed and parsed.get("object") != "list" and "data" not in parsed:
            return None

        transformed = transform_models_response(content, inject_openrouter=inject_openrouter)
        if transformed is None:
            return None
        full_body, etag = transformed

        parsed = json.loads(full_body.decode())
        models = parsed.get("models", [])
        if not isinstance(models, list):
            return None
        compact_models = [{"id": model.get("id")} for model in models if isinstance(model, dict) and model.get("id")]
        compact_body = json.dumps({"models": compact_models}).decode().encode()
        return _LegacyModelsTransformResult(compact_body, full_body, etag)
    except (TypeError, json.JSONDecodeError):
        return None


def build_openrouter_passthrough_body(body: dict) -> dict:
    """Compatibility export for old cliproxy tests and callers."""
    return _build_openrouter_passthrough_body(body)


def _responses_to_chat_completions(body: dict[str, Any]) -> dict[str, Any]:
    """Compatibility wrapper with legacy message formatting."""
    collapse_text_content: list[bool] = []
    input_messages = body.get("input")
    if isinstance(input_messages, list):
        for msg in input_messages:
            collapse = False
            if isinstance(msg, dict):
                raw_content = msg.get("content")
                if (
                    isinstance(raw_content, list)
                    and len(raw_content) == 1
                    and isinstance(raw_content[0], dict)
                    and raw_content[0].get("type") == "text"
                    and "text" in raw_content[0]
                    and len(raw_content[0]) == 2
                ):
                    collapse = True
            collapse_text_content.append(collapse)

    transformed = _request_transform_to_chat_completions(body)
    messages = transformed.get("messages")
    if isinstance(messages, list):
        normalized: list[dict[str, Any]] = []
        for idx, message in enumerate(messages):
            if not isinstance(message, dict):
                continue
            content = message.get("content")
            if isinstance(content, list):
                if (
                    idx < len(collapse_text_content)
                    and collapse_text_content[idx]
                    and len(content) == 1
                    and isinstance(content[0], dict)
                    and content[0].get("type") == "text"
                    and "text" in content[0]
                    and len(content[0]) == 2
                ):
                    message["content"] = content[0].get("text", "")
            normalized.append(message)
        transformed["messages"] = normalized
    return transformed


def _chat_completions_to_responses(chunk: dict[str, Any]) -> dict[str, Any] | None:
    """Compatibility wrapper: legacy output shape for adapter callers."""
    text = _extract_delta_content(chunk)
    if not text:
        return None
    return {
        "type": "response.output_item.added",
        "response_id": "resp_legacy",
        "item_id": "item_legacy",
        "output_index": 0,
        "item": {
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": text}],
        },
    }


_COMPAT_ADAPTER_EXPORTS = (
    _compute_models_etag,
    _OR_PASSTHROUGH_FIELDS,
    _map_model_for_backend,
    _transform_models_response,
    _chat_completions_to_responses,
)


# @trace FR-CACHE-024 FR-CACHE-025 FR-CACHE-027
@dataclass
class CacheControl:
    """Per-request cache control extracted from tg-* headers."""

    namespace: str = "default"
    force_refresh: bool = False
    skip_cache: bool = False  # set by tg-skip-cache: true


def extract_cache_control(request: Request) -> CacheControl:
    """Extract cache control directives from tg-* request headers.

    Headers:
        tg-cache-namespace: <string>   (default: "default")
        tg-cache-force-refresh: true   (default: false)
        tg-skip-cache: true            (default: false)
    """
    headers = request.headers
    return CacheControl(
        namespace=headers.get("tg-cache-namespace", "default"),
        force_refresh=headers.get("tg-cache-force-refresh", "").lower() == "true",
        skip_cache=headers.get("tg-skip-cache", "").lower() == "true",
    )


# GW-20: tg-* per-request control headers namespace
# Each header controls a specific gateway behavior.
_TG_HEADER_NAMES: frozenset[str] = frozenset(
    {
        "tg-cache-ttl",
        "tg-skip-cache",
        "tg-cache-namespace",
        "tg-cache-force-refresh",
        "tg-event-id",
        "tg-fallback-step",
        "tg-ttft-ms",
        "tg-response-cost",
        "tg-custom-cost",
    }
)


@dataclass
class TgHeaders:
    """Parsed tg-* per-request control headers (GW-20).

    # @trace FR-ROUTE-020
    """

    cache_ttl: float | None = None  # tg-cache-ttl: <seconds>
    skip_cache: bool = False  # tg-skip-cache: true
    cache_namespace: str = "default"  # tg-cache-namespace: <ns>
    cache_force_refresh: bool = False  # tg-cache-force-refresh: true
    custom_cost: float | None = None  # tg-custom-cost: <usd>


def extract_tg_headers(request_headers: dict) -> TgHeaders:
    """Extract and parse all tg-* control headers from the request (GW-20).

    # @trace FR-ROUTE-020
    """
    headers = {k.lower(): v for k, v in request_headers.items()}

    cache_ttl = None
    raw_ttl = headers.get("tg-cache-ttl")
    if raw_ttl is not None:
        try:
            cache_ttl = float(raw_ttl)
        except ValueError:
            pass

    custom_cost = None
    raw_cost = headers.get("tg-custom-cost")
    if raw_cost is not None:
        try:
            custom_cost = float(raw_cost)
        except ValueError:
            pass

    return TgHeaders(
        cache_ttl=cache_ttl,
        skip_cache=headers.get("tg-skip-cache", "").lower() == "true",
        cache_namespace=headers.get("tg-cache-namespace", "default"),
        cache_force_refresh=headers.get("tg-cache-force-refresh", "").lower() == "true",
        custom_cost=custom_cost,
    )


def build_cache_response_headers(hit: bool, ttl: float, namespace: str) -> dict[str, str]:
    """Build x-cache-* response headers for a cache HIT or MISS.

    Args:
        hit: True if response came from cache.
        ttl: Remaining TTL in seconds (for HIT) or configured TTL (for MISS).
        namespace: Cache namespace used.

    Returns:
        Dict of header name -> value to merge into response headers.
    """
    return {
        "x-cache-status": "HIT" if hit else "MISS",
        "x-cache-ttl": str(int(ttl)),
        "x-cache-namespace": namespace,
    }


# @trace FR-COST-032 FR-COST-033
def build_cost_response_header(response_body: dict) -> dict[str, str]:
    """Build tg-response-cost header from response JSON body.

    Returns {'tg-response-cost': '<cost>'} or empty dict if cost unavailable.

    On the hot response path — exceptions are swallowed to prevent crashing the gateway.

    # @trace FR-COST-032
    """
    try:
        cost = calculate_cost_from_response(response_body)
        if cost > 0.0:
            return {"tg-response-cost": format_cost_header_value(cost)}
    except Exception:
        pass
    return {}


# ---------------------------------------------------------------------------
# GW-48: inject usage.cost into every response body
# ---------------------------------------------------------------------------


def inject_usage_cost(response_body: dict) -> dict:
    """Inject cost into response body's usage object (GW-48).

    Adds usage.cost = <usd_float> computed from token counts and model pricing.
    If usage or pricing unavailable, returns body unchanged.

    Returns modified copy of response_body (does not mutate).

    # @trace FR-REQEXT-048
    """
    try:
        from thegent.utils.routing_impl.cost_calculator import calculate_cost_from_response

        cost = calculate_cost_from_response(response_body)
        if cost <= 0.0:
            return response_body
        result = dict(response_body)
        usage = dict(result.get("usage", {}))
        usage["cost"] = cost
        result["usage"] = usage
        return result
    except Exception:
        return response_body


# ---------------------------------------------------------------------------
# GW-49: native_finish_reason alongside normalized finish_reason
# ---------------------------------------------------------------------------

# Mapping of provider-native finish reasons to normalized OpenAI-compatible reasons
_FINISH_REASON_NORMALIZATION: dict[str, str] = {
    # Anthropic
    "end_turn": "stop",
    "max_tokens": "length",
    "stop_sequence": "stop",
    "tool_use": "tool_calls",
    # Google Gemini
    "STOP": "stop",
    "MAX_TOKENS": "length",
    "SAFETY": "content_filter",
    "RECITATION": "content_filter",
    "OTHER": "stop",
    # OpenAI (already normalized — identity mapping)
    "stop": "stop",
    "length": "length",
    "content_filter": "content_filter",
    "tool_calls": "tool_calls",
    "function_call": "tool_calls",
}


def normalize_finish_reason(native_reason: str | None) -> str:
    """Normalize a provider-native finish reason to OpenAI-compatible value (GW-49).

    Returns the normalized reason, or "stop" as default for unknown values.

    # @trace FR-REQEXT-049
    """
    if native_reason is None:
        return "stop"
    return _FINISH_REASON_NORMALIZATION.get(native_reason, "stop")


def inject_native_finish_reason(response_body: dict) -> dict:
    """Inject native_finish_reason alongside normalized finish_reason in choices (GW-49).

    For each choice in response_body["choices"]:
    - Reads the existing finish_reason
    - Sets native_finish_reason = finish_reason (the original from provider)
    - Sets finish_reason = normalize_finish_reason(native_finish_reason)

    Returns modified copy of response_body (does not mutate).

    # @trace FR-REQEXT-049
    """
    choices = response_body.get("choices")
    if not choices:
        return response_body
    result = dict(response_body)
    new_choices = []
    for choice in choices:
        new_choice = dict(choice)
        native = choice.get("finish_reason")
        new_choice["native_finish_reason"] = native
        new_choice["finish_reason"] = normalize_finish_reason(native)
        new_choices.append(new_choice)
    result["choices"] = new_choices
    return result


# ---------------------------------------------------------------------------
# GW-35: tg-event-id — unique per-request trace ID
# ---------------------------------------------------------------------------


def generate_event_id() -> str:
    """Generate a unique event ID for request tracing (GW-35).

    Format: tg-<8-char-hex> for brevity and readability.
    # @trace FR-OBS-035
    """
    return f"tg-{uuid.uuid4().hex[:8]}"


def build_event_id_header() -> dict[str, str]:
    """Build tg-event-id response header (GW-35).

    Returns {'tg-event-id': 'tg-<8hex>'}.
    """
    return {"tg-event-id": generate_event_id()}


# ---------------------------------------------------------------------------
# GW-36: tg-fallback-step — which fallback provider was used
# ---------------------------------------------------------------------------


def build_fallback_step_header(step: int) -> dict[str, str]:
    """Build tg-fallback-step response header (GW-36).

    step=0 means primary model succeeded.
    step=1+ means Nth fallback was used.

    Returns {'tg-fallback-step': '<step>'}.
    # @trace FR-OBS-036
    """
    return {"tg-fallback-step": str(step)}


# ---------------------------------------------------------------------------
# GW-38: TTFTTracker — time-to-first-token for streaming responses
# ---------------------------------------------------------------------------


class TTFTTracker:
    """Time-to-First-Token tracker for streaming LLM responses (GW-38).

    Usage:
        tracker = TTFTTracker()
        tracker.start()
        # ... receive first SSE chunk ...
        tracker.record_first_token()
        ttft = tracker.ttft_seconds  # float

    # @trace FR-OBS-038
    """

    def __init__(self) -> None:
        self._start: float | None = None
        self._first_token: float | None = None

    def start(self) -> None:
        """Record the request start time."""
        self._start = time.monotonic()

    def record_first_token(self) -> None:
        """Record when the first token arrived (idempotent — only first call counts)."""
        if self._first_token is None and self._start is not None:
            self._first_token = time.monotonic()

    @property
    def ttft_seconds(self) -> float | None:
        """Return TTFT in seconds, or None if not yet measured."""
        if self._start is None or self._first_token is None:
            return None
        return self._first_token - self._start

    def build_ttft_header(self) -> dict[str, str]:
        """Build tg-ttft-ms header with TTFT in milliseconds.

        Returns {} if TTFT not yet measured.
        """
        ttft = self.ttft_seconds
        if ttft is None:
            return {}
        return {"tg-ttft-ms": f"{ttft * 1000:.1f}"}


# ---------------------------------------------------------------------------
# GW-43: Native Anthropic endpoint /v1/messages request/response conversion
# ---------------------------------------------------------------------------


def anthropic_messages_to_chat_completions(body: dict) -> dict:
    """Convert Anthropic /v1/messages request body to OpenAI chat completions format.

    Maps:
    - body["model"] -> body["model"] (unchanged)
    - body["messages"] -> body["messages"] (format is compatible)
    - body["max_tokens"] -> body["max_tokens"]
    - body["system"] string -> prepend {"role": "system", "content": system} to messages
    - body["stream"] -> body["stream"] (if present)
    - body["temperature"] -> body["temperature"] (if present)

    Returns a new dict in chat completions format.

    # @trace FR-REQEXT-043
    """
    result: dict = {}
    result["model"] = body.get("model", "")
    result["max_tokens"] = body.get("max_tokens", 1024)

    messages = list(body.get("messages", []))
    system = body.get("system")
    if system:
        messages = [{"role": "system", "content": system}] + messages
    result["messages"] = messages

    for key in ("stream", "temperature", "top_p", "stop"):
        if key in body:
            result[key] = body[key]

    return result


def anthropic_response_to_messages_format(response: dict) -> dict:
    """Convert OpenAI chat completions response to Anthropic /v1/messages format.

    Maps:
    - response["choices"][0]["message"]["content"] -> content[0]["text"]
    - response["model"] -> model
    - response["usage"]["prompt_tokens"] -> usage.input_tokens
    - response["usage"]["completion_tokens"] -> usage.output_tokens

    Returns dict in Anthropic messages format.

    # @trace FR-REQEXT-043
    """
    choices = response.get("choices", [])
    content_text = ""
    stop_reason = "end_turn"
    if choices:
        msg = choices[0].get("message", {})
        content_text = msg.get("content", "")
        fr = choices[0].get("finish_reason", "stop")
        stop_reason = "end_turn" if fr == "stop" else fr

    usage = response.get("usage", {})

    return {
        "id": response.get("id", ""),
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": content_text}],
        "model": response.get("model", ""),
        "stop_reason": stop_reason,
        "stop_sequence": None,
        "usage": {
            "input_tokens": usage.get("prompt_tokens", 0),
            "output_tokens": usage.get("completion_tokens", 0),
        },
    }


# ---------------------------------------------------------------------------
# GW-44: Vercel providerOptions.gateway passthrough
# ---------------------------------------------------------------------------


def extract_provider_gateway_options(body: dict) -> dict:
    """Extract Vercel AI SDK providerOptions.gateway object (GW-44).

    Reads body.get("providerOptions", {}).get("gateway", {}).
    These control gateway behavior: cache, retry, timeout, etc.
    Returns empty dict if not present.

    # @trace FR-REQEXT-044
    """
    provider_options = body.get("providerOptions")
    if not isinstance(provider_options, dict):
        return {}
    gateway_options = provider_options.get("gateway")
    if not isinstance(gateway_options, dict):
        return {}
    return gateway_options


# ---------------------------------------------------------------------------
# GW-45: Forward special headers
# ---------------------------------------------------------------------------

_SPECIAL_FORWARD_HEADERS: frozenset[str] = frozenset(
    {
        "x-session-id",
        "x-request-id",
        "x-anthropic-beta",
        "x-stainless-os",
        "x-stainless-arch",
        "structured-outputs-2025-11-13",
    }
)


def extract_special_headers(request_headers: dict) -> dict[str, str]:
    """Extract special headers to forward to the backend (GW-45).

    Returns a dict of header_name -> value for headers in _SPECIAL_FORWARD_HEADERS
    that are present in request_headers.

    # @trace FR-REQEXT-045
    """
    return {k: v for k, v in request_headers.items() if k.lower() in _SPECIAL_FORWARD_HEADERS}


# ---------------------------------------------------------------------------
# GW-46: Enrich /v1/models response entry
# ---------------------------------------------------------------------------


def enrich_model_entry(entry: dict) -> dict:
    """Enrich a /v1/models entry with context_length and supported_parameters (GW-46).

    Adds:
    - context_length: int (max input tokens)
    - supported_parameters: list of supported params

    Uses thegent.routing.model_metadata if available; returns entry unchanged if not.

    # @trace FR-REQEXT-046
    """
    try:
        from thegent.utils.routing_impl.model_metadata import get_model_metadata, has_model_metadata

        model_id = entry.get("id", "")
        if not has_model_metadata(model_id):
            return entry
        meta = get_model_metadata(model_id)
        if meta is None:
            return entry
        result = dict(entry)
        ctx_len = meta.get("context_length")
        if ctx_len:
            result["context_length"] = ctx_len
        sup_params = meta.get("supported_parameters")
        if sup_params:
            result["supported_parameters"] = sup_params
        return result
    except Exception:
        return entry


# ---------------------------------------------------------------------------
# GW-47: Inject missing proxy models into /v1/models
# ---------------------------------------------------------------------------


def inject_proxy_models(models_list: list[dict]) -> list[dict]:
    """Inject proxy model entries for models that may be missing from the backend (GW-47).

    Adds entries for thegent-known canonical model aliases if not already present.
    This fixes "Model metadata not found" errors for models served through the proxy.

    # @trace FR-REQEXT-047
    """
    try:
        from thegent.utils.routing_impl.harness_model_mapping import CANONICAL_TO_OPENROUTER

        existing_ids = {m.get("id", "") for m in models_list}
        result = list(models_list)
        for alias in CANONICAL_TO_OPENROUTER:
            if alias not in existing_ids:
                result.append(
                    {
                        "id": alias,
                        "object": "model",
                        "created": 0,
                        "owned_by": "thegent-proxy",
                    }
                )
        return result
    except Exception:
        return models_list


# OR-08: App identification headers for OpenRouter attribution
_OPENROUTER_REFERER = "https://thegent.dev"
_OPENROUTER_TITLE = "thegent"


def _is_openrouter_backend(url: str) -> bool:
    """Return True when the backend URL targets OpenRouter."""
    return "openrouter.ai" in url


def _inject_openrouter_headers(headers: dict[str, str], backend_url: str) -> None:
    """OR-08: Inject HTTP-Referer and X-Title when routing to OpenRouter."""
    if _is_openrouter_backend(backend_url):
        headers.setdefault("HTTP-Referer", _OPENROUTER_REFERER)
        headers.setdefault("X-Title", _OPENROUTER_TITLE)


async def _proxy_request(
    request: Request,
    backend_url: str,
    path: str,
    *,
    transform_responses: bool = False,
) -> Response:
    """Forward request to backend. Optionally transform Responses <-> Chat Completions.

    OR-11: Normalize OpenRouter JSON error envelopes while preserving metadata.
    OR-13: Retry transient OpenRouter HTTP statuses (408/502/503) for non-streaming paths.
    """
    import asyncio
    import httpx

    body = b""
    if request.method in ("POST", "PUT", "PATCH"):
        body = await request.body()
    url = f"{backend_url.rstrip('/')}{path}" if path.startswith("/") else f"{backend_url.rstrip('/')}/{path}"
    if request.url.query:
        url = f"{url}?{request.url.query}"

    headers = sanitize_outbound_request_headers(dict(request.headers))
    # OR-08: inject attribution headers for OpenRouter backends
    _inject_openrouter_headers(headers, backend_url)

    if transform_responses and body and path == "/v1/responses":
        try:
            data = json.loads(body)
            transformed = _responses_to_chat_completions(data)
            body = json.dumps(transformed).decode().encode()
            url = f"{backend_url.rstrip('/')}/v1/chat/completions"
        except (json.JSONDecodeError, KeyError) as e:
            _log.warning("responses->chat transform failed: %s", e)

    try:
        attempts = 0
        async with httpx.AsyncClient(timeout=120.0) as client:
            while True:
                resp = await client.request(
                    request.method,
                    url,
                    content=body,
                    headers=headers,
                )
                is_openrouter = _is_openrouter_backend(backend_url)
                max_attempts = _RETRY_MAX_ATTEMPTS.get(resp.status_code, 0) if is_openrouter else 0
                if max_attempts and attempts < max_attempts:
                    attempts += 1
                    delay = 2.0**attempts
                    _log.warning(
                        "OR-13 non-stream: backend %s retry %d/%d in %.0fs",
                        resp.status_code,
                        attempts,
                        max_attempts,
                        delay,
                    )
                    await asyncio.sleep(delay)
                    continue
                break

        filtered_headers = filter_inbound_response_headers(dict(resp.headers))

        if resp.status_code >= 400 and _is_openrouter_backend(backend_url):
            # OR-11: preserve provider metadata and always provide semantic error.code.
            err_obj = _make_error_body(resp.status_code, resp.content)
            filtered_headers["Content-Type"] = "application/json"
            return Response(
                content=json.dumps(err_obj).decode().encode(),
                status_code=resp.status_code,
                headers=filtered_headers,
            )

        return Response(
            content=resp.content,
            status_code=resp.status_code,
            headers=filtered_headers,
        )
    except (httpx.ConnectError, httpx.ConnectTimeout) as e:
        _log.error("Backend proxy (%s) unreachable: %s", backend_url, e)
        return Response(
            content=json.dumps(
                {"error": {"message": f"Backend proxy ({backend_url}) unreachable. Restart with: thegent mcp restart"}}
            ),
            status_code=503,
            headers={"Content-Type": "application/json"},
        )


async def _proxy_stream(
    body: bytes,
    headers: dict[str, str],
    backend_url: str,
    path: str,
    *,
    transform_responses: bool = False,
    model: str = "proxy",
) -> Response:
    """Stream proxy with optional response transformation. Buffers SSE by line to handle chunk boundaries.

    OR-08: Injects HTTP-Referer + X-Title when backend is OpenRouter.
    OR-13: 402 raises InsufficientCreditsError (no retry). 408/502/503 retried with backoff.
    """
    import asyncio

    import httpx

    # Track the routed model discovered from upstream SSE model field.
    routed_model = model

    class _StreamingResponseWithModel(StreamingResponse):
        """StreamingResponse that refreshes the openai-model header from upstream SSE."""

        async def stream_response(self, send: Send) -> None:  # noqa: PLR0912 -- stream startup state machine
            try:
                iterator = cast("AsyncIterator[bytes | memoryview | str]", self.body_iterator)
                first_chunk = await iterator.__anext__()
            except StopAsyncIteration:
                await send(
                    {
                        "type": "http.response.start",
                        "status": self.status_code,
                        "headers": self.raw_headers,
                    }
                )
                await send({"type": "http.response.body", "body": b"", "more_body": False})
                return

            self.headers["openai-model"] = routed_model
            await send(
                {
                    "type": "http.response.start",
                    "status": self.status_code,
                    "headers": self.raw_headers,
                }
            )
            if not isinstance(first_chunk, bytes | memoryview):
                first_chunk = first_chunk.encode(self.charset)
            await send({"type": "http.response.body", "body": first_chunk, "more_body": True})
            async for chunk in self.body_iterator:
                if not isinstance(chunk, bytes | memoryview):
                    chunk = chunk.encode(self.charset)
                await send({"type": "http.response.body", "body": chunk, "more_body": True})
            await send({"type": "http.response.body", "body": b"", "more_body": False})

    if transform_responses:
        try:
            data = json.loads(body)
            model = data.get("model", model)
            transformed = _responses_to_chat_completions(data)
            body = json.dumps(transformed).decode().encode()
        except (json.JSONDecodeError, KeyError):
            pass
        url = f"{backend_url.rstrip('/')}/chat/completions"
    else:
        url = f"{backend_url.rstrip('/')}{path}" if path.startswith("/") else f"{backend_url.rstrip('/')}/{path}"

    # OR-08: inject OpenRouter attribution headers before streaming begins
    _inject_openrouter_headers(headers, backend_url)

    async def _do_stream(attempt: int):  # noqa: PLR0912 -- streaming state machine; complexity justified
        """Inner generator for a single stream attempt."""
        nonlocal routed_model
        buffer = b""
        state = ResponsesStreamState(model=model) if transform_responses else None
        preamble_emitted = False
        done_received = False
        async with httpx.AsyncClient(timeout=120.0) as client:
            async with client.stream("POST", url, content=body, headers=headers) as resp:
                if resp.status_code != 200:
                    # GW-05 + GW-06: preserve backend error body with semantic code context
                    err_body = await resp.aread()
                    _log.warning("backend stream error %s: %s", resp.status_code, err_body[:200])

                    # OR-13: 402 = InsufficientCredits — hard-stop, never retry
                    if resp.status_code == 402:
                        raise InsufficientCreditsError(_ERROR_MESSAGES.get(402, "Payment required"))

                    # OR-13: transient errors — signal caller to retry
                    max_attempts = _RETRY_MAX_ATTEMPTS.get(resp.status_code, 0)
                    if max_attempts and attempt < max_attempts:
                        raise _RetryableStreamError(resp.status_code, err_body)

                    err_obj = _make_error_body(resp.status_code, err_body)
                    yield f"data: {json.dumps(err_obj).decode()}\n\n".encode()
                    return

                async for chunk in resp.aiter_bytes():
                    buffer += chunk
                    while b"\n" in buffer or (buffer and buffer.endswith(b"\n")):
                        idx = buffer.find(b"\n")
                        if idx < 0:
                            break
                        line = buffer[:idx]
                        buffer = buffer[idx + 1 :]
                        if state is not None:
                            line = line.strip()
                            # WL-005: skip SSE comment lines (e.g. ": OPENROUTER PROCESSING")
                            if not line or line.startswith(b":"):
                                continue
                            if not line.startswith(b"data:"):
                                continue
                            data_part = line[5:].strip()
                            if not data_part:
                                continue
                            if data_part == b"[DONE]":
                                done_received = True
                                break
                            try:
                                obj = json.loads(data_part.decode(errors="replace"))
                            except (json.JSONDecodeError, UnicodeDecodeError):
                                continue
                            # GW-09 / OR-12: capture actual routed model from SSE chunk
                            chunk_model = obj.get("model")
                            if chunk_model and chunk_model != state.model:
                                state.model = chunk_model
                                routed_model = chunk_model
                            usage = _extract_usage(obj)
                            if usage:
                                state.set_usage(usage)
                            text = _extract_delta_content(obj)
                            tool_calls = _extract_delta_tool_calls(obj)
                            if text:
                                if not preamble_emitted:
                                    for ev in state.preamble_events():
                                        yield f"data: {json.dumps(ev).decode()}\n\n".encode()
                                    preamble_emitted = True
                                yield f"data: {json.dumps(state.delta_event(text)).decode()}\n\n".encode()
                            # GW-07: forward tool call deltas
                            if tool_calls:
                                for ev in state.tool_call_delta_events(tool_calls):
                                    yield f"data: {json.dumps(ev).decode()}\n\n".encode()
                        else:
                            out = _process_sse_line(line, False)
                            if out:
                                yield out
                    if done_received:
                        break
                if buffer.strip() and state is None:
                    out = _process_sse_line(buffer, False)
                    if out:
                        yield out
                if state is not None:
                    if not preamble_emitted:
                        for ev in state.preamble_events():
                            yield f"data: {json.dumps(ev).decode()}\n\n".encode()
                    # GW-07: emit tool call done events before text closing
                    for ev in state.tool_call_closing_events():
                        yield f"data: {json.dumps(ev).decode()}\n\n".encode()
                    for ev in state.closing_events():
                        yield f"data: {json.dumps(ev).decode()}\n\n".encode()
                    yield b"data: [DONE]\n\n"

    async def _stream_with_retries(attempt: int):
        """Execute a single stream attempt and recurse on retryable failures."""
        try:
            async for chunk in _do_stream(attempt):
                yield chunk
            return
        except InsufficientCreditsError as stream_error:
            # OR-13: 402 — hard-stop, no retry
            _log.error("OR-13: insufficient credits: %s", stream_error)
            err_obj = {"error": {"message": str(stream_error), "code": 402}}
            yield f"data: {json.dumps(err_obj).decode()}\n\n".encode()
            return
        except _RetryableStreamError as stream_error:
            # OR-13: transient error — retry with exponential backoff
            next_attempt = attempt + 1
            delay = 2.0**next_attempt  # 2s, 4s, 8s
            _log.warning(
                "OR-13: backend %s on attempt %d/%d; retrying in %.0fs",
                stream_error.status_code,
                next_attempt,
                _RETRY_MAX_ATTEMPTS.get(stream_error.status_code, 1),
                delay,
            )
            await asyncio.sleep(delay)
            async for chunk in _stream_with_retries(next_attempt):
                yield chunk
            return
        except (httpx.ConnectError, httpx.ConnectTimeout) as stream_error:
            # Connection error — surface and stop
            _log.error("Backend stream connection failed: %s", stream_error)
            yield f'data: {{"error":{{"message":"Backend proxy ({url}) unreachable."}}}}\n\n'.encode()
            return

    async def stream():
        """Outer stream generator: handles OR-13 retry/error routing."""
        async for chunk in _stream_with_retries(0):
            yield chunk

    return _StreamingResponseWithModel(
        stream(),
        status_code=200,
        headers={
            "Content-Type": "text/event-stream",
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "openai-model": routed_model,
        },
    )


def _backend_path(backend_url: str, request_path: str) -> str:
    """Path to append to backend base. Backend base is .../v1, so /v1/responses -> /responses."""
    base = backend_url.rstrip("/")
    if base.endswith("/v1") and request_path.startswith("/v1/"):
        return request_path[4:]  # /v1/responses -> /responses
    return request_path


async def proxy_handler(request: Request) -> Response:
    """Proxy /v1/* to backend. Transform /v1/responses to /v1/chat/completions."""
    # Bifrost validation
    bifrost = get_bifrost()
    if bifrost.is_enabled:
        try:
            claims = {"api_key": request.headers.get("authorization", ""), "identifier": request.client.host}
            bifrost.validate_claims(claims)
        except BifrostValidationError as e:
            return Response(
                content=json.dumps({"error": {"message": str(e)}}),
                status_code=403,
                headers={"Content-Type": "application/json"},
            )

    backend = getattr(request.app.state, "backend_url", "http://127.0.0.1:8318/v1")
    path = request.url.path or "/v1/models"

    # Check if LiteLLM Router should be used
    settings = ThegentSettings()
    use_litellm = settings.use_litellm_router

    if _log.isEnabledFor(logging.DEBUG) or __debug__:
        _log.debug("adapter request: %s %s (litellm=%s)", request.method, path, use_litellm)

    if not path.startswith("/v1/"):
        return Response("Not Found", status_code=404)

    # Route Responses API to LiteLLM Router if enabled
    if use_litellm and path == "/v1/responses" and request.method == "POST":
        try:
            from thegent.utils.routing_impl.litellm_responses_handler import handle_responses_request

            return await handle_responses_request(request)
        except Exception as e:
            _log.error("LiteLLM Router handler failed: %s", e, exc_info=True)
            # Fallback to CLIProxyAPIPlus

    backend_path = _backend_path(backend, path)

    if request.method == "GET" and path in ("/v1/models", "/v1/models/"):
        resp = await _proxy_request(request, backend, backend_path)
        # Codex expects {"models": [...]}; CLIProxy returns {"data": [...], "object": "list"}
        if resp.status_code == 200 and resp.body:
            # OR-15: inject OpenRouter proxy models when the backend is OpenRouter
            result = transform_models_response(resp.body, inject_openrouter=_is_openrouter_backend(backend))
            if result is not None:
                transformed_body, etag = result
                return Response(
                    content=transformed_body,
                    status_code=200,
                    headers={
                        "Content-Type": "application/json",
                        "x-models-etag": etag,
                    },
                )
        return resp
    if request.method == "POST":
        try:
            body = await request.body()
            data = json.loads(body) if body else {}
            stream_mode = data.get("stream", False)
        except json.JSONDecodeError:
            stream_mode = False

        if path == "/v1/responses":
            # Backend often lacks /v1/responses; translate to /v1/chat/completions
            req_model = data.get("model", "proxy") if isinstance(data, dict) else "proxy"
            if data:
                _log.debug("responses transform: model=%s stream=%s", req_model, data.get("stream"))
            req_headers = sanitize_outbound_request_headers(dict(request.headers))
            return await _proxy_stream(
                body, req_headers, backend, "/chat/completions", transform_responses=True, model=req_model
            )

        if stream_mode and path == "/v1/chat/completions":
            req_headers = sanitize_outbound_request_headers(dict(request.headers))
            return await _proxy_stream(body, req_headers, backend, backend_path)
        return await _proxy_request(request, backend, backend_path)
    return await _proxy_request(request, backend, backend_path)


async def websocket_responses_handler(websocket: Any) -> None:
    """Bridge WebSocket /v1/responses to HTTP streaming. Buffers SSE by line."""
    import asyncio

    # Check if LiteLLM Router should be used
    settings = ThegentSettings()
    use_litellm = settings.use_litellm_router

    if use_litellm:
        try:
            from thegent.utils.routing_impl.litellm_responses_handler import handle_responses_websocket

            await handle_responses_websocket(websocket)
            return
        except Exception as e:
            _log.error("LiteLLM Router WebSocket handler failed: %s", e, exc_info=True)
            # Fallback to CLIProxyAPIPlus

    import httpx

    await websocket.accept()
    backend = getattr(websocket.app.state, "backend_url", "http://127.0.0.1:8318/v1")
    base = backend.rstrip("/")
    url = f"{base}/chat/completions" if base.endswith("/v1") else f"{base}/v1/chat/completions"

    # Persistent connection: Codex sends multiple requests per WS connection.
    # After response.completed, the connection stays open for the next request.
    async with httpx.AsyncClient(timeout=120.0) as client:
        while True:
            try:
                data = await asyncio.wait_for(websocket.receive_json(), timeout=300.0)
            except TimeoutError:
                _log.debug("ws responses: idle timeout, closing")
                break
            except Exception as e:
                _log.debug("ws responses: receive_json ended: %s", e)
                break

            if not isinstance(data, dict):
                continue

            if "input" in data:
                transformed = _responses_to_chat_completions(data)
                # Always force streaming from backend for WebSocket transport
                transformed["stream"] = True
                body = json.dumps(transformed).decode().encode()
            else:
                body = json.dumps(data).decode().encode()
            # Forward auth and other upstream headers; WS upgrade headers carry Authorization (WL-001)
            # Explicitly include Authorization in forward_headers to fix WL-001: WS auth header drop
            headers = extract_websocket_forward_headers(dict(websocket.headers))

            model = data.get("model", "proxy")
            state = ResponsesStreamState(model=model)
            preamble_emitted = False
            done_received = False
            try:
                async with client.stream("POST", url, content=body, headers=headers) as resp:
                    if resp.status_code != 200:
                        # GW-05 + GW-06: preserve backend error body with semantic code context
                        err_body = await resp.aread()
                        _log.warning("ws backend error %s: %s", resp.status_code, err_body[:200])
                        err_obj = _make_error_body(resp.status_code, err_body)
                        await websocket.send_json({"type": "response.failed", **err_obj})
                        continue
                    buffer = b""
                    async for chunk in resp.aiter_bytes():
                        buffer += chunk
                        while b"\n" in buffer:
                            idx = buffer.find(b"\n")
                            line = buffer[:idx]
                            buffer = buffer[idx + 1 :]
                            line = line.strip()
                            # WL-005: skip SSE comment lines (e.g. ": OPENROUTER PROCESSING")
                            if not line or line.startswith(b":"):
                                continue
                            if not line.startswith(b"data:"):
                                continue
                            data_part = line[5:].strip()
                            if not data_part:
                                continue
                            if data_part == b"[DONE]":
                                done_received = True
                                break
                            try:
                                obj = json.loads(data_part.decode(errors="replace"))
                            except (json.JSONDecodeError, UnicodeDecodeError):
                                continue
                            # GW-09: capture actual routed model from SSE chunk
                            chunk_model = obj.get("model")
                            if chunk_model and chunk_model != state.model:
                                state.model = chunk_model
                            usage = _extract_usage(obj)
                            if usage:
                                state.set_usage(usage)
                            text = _extract_delta_content(obj)
                            tool_calls = _extract_delta_tool_calls(obj)
                            if text:
                                if not preamble_emitted:
                                    for ev in state.preamble_events():
                                        await websocket.send_json(ev)
                                    preamble_emitted = True
                                await websocket.send_json(state.delta_event(text))
                            # GW-07: forward tool call deltas
                            if tool_calls:
                                for ev in state.tool_call_delta_events(tool_calls):
                                    await websocket.send_json(ev)
                        if done_received:
                            break
                    # Emit closing sequence after stream ends
                    if not preamble_emitted:
                        for ev in state.preamble_events():
                            await websocket.send_json(ev)
                    # GW-07: emit tool call done events
                    for ev in state.tool_call_closing_events():
                        await websocket.send_json(ev)
                    for ev in state.closing_events():
                        await websocket.send_json(ev)
            except Exception as e:
                _log.warning("ws responses: stream error: %s", e)
                with contextlib.suppress(Exception):
                    await websocket.send_json({"type": "response.failed", "error": {"message": str(e)}})

    with contextlib.suppress(Exception):
        await websocket.close(1000)


def create_adapter_app(backend_url: str) -> Starlette:
    """Create the adapter Starlette app."""
    from thegent.config import ThegentSettings

    if ThegentSettings().debug:
        _log.setLevel(logging.DEBUG)
    app = Starlette(
        routes=[
            Route("/{path:path}", proxy_handler, methods=["GET", "POST", "OPTIONS"]),
            WebSocketRoute("/v1/responses", websocket_responses_handler),
        ],
    )
    app.state.backend_url = backend_url.rstrip("/")
    return app


# Register with unified adapter registry
class CliproxyAdapter:
    """HTTP proxy adapter for cliproxy"""

    def __init__(self, backend_url: str = "http://127.0.0.1:8318/v1"):
        self._app = create_adapter_app(backend_url)

    def call(self, request=None, **kwargs) -> dict:
        """Proxy request through adapter"""
        return {"status": "ready", "backend": self._app.state.backend_url}

    @property
    def app(self):
        return self._app


AdapterRegistry.register("cliproxy", CliproxyAdapter())
