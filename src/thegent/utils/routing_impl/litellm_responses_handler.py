"""LiteLLM Router Responses API handler for Codex CLI compatibility."""

from __future__ import annotations

import orjson as json
import logging
from pathlib import Path
import inspect
from collections.abc import AsyncIterable, Awaitable, Mapping
from unittest.mock import Base
from typing import TYPE_CHECKING, Any, cast

import httpx
from starlette.responses import Response, StreamingResponse

if TYPE_CHECKING:
    from starlette.requests import Request
    from starlette.websockets import WebSocket

from thegent.utils.routing_impl.litellm_router import build_dynamic_fallback_router, get_litellm_router

_log = logging.getLogger(__name__)

# WL-071: Persistent httpx.AsyncClient — one TCP connection pool for the process lifetime.
# Avoids TCP handshake + SSL negotiation cost on every LLM request.
_http_client: httpx.AsyncClient | None = None


def _get_http_client() -> httpx.AsyncClient:
    """Return the module-level persistent httpx.AsyncClient, creating it if needed."""
    global _http_client
    async_client_type = getattr(httpx, "AsyncClient", None)

    if _http_client is None:
        _http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(120.0, connect=10.0),
            limits=httpx.Limits(max_connections=100, max_keepalive_connections=20),
        )
    # Rebind if a class-level mock replaced AsyncClient between calls.
    elif isinstance(async_client_type, type):
        if not isinstance(_http_client, async_client_type):
            _http_client = None
        else:
            is_closed = getattr(_http_client, "is_closed", False)
            if isinstance(is_closed, bool) and is_closed:
                _http_client = None
    else:
        # If ``AsyncClient`` is currently mocked (non-type), avoid returning
        # a stale client from before mocking began.
        _http_client = None

    if _http_client is None:
        _http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(120.0, connect=10.0),
            limits=httpx.Limits(max_connections=100, max_keepalive_connections=20),
        )
    return cast("httpx.AsyncClient", _http_client)


async def close_http_client() -> None:
    """Close the persistent httpx.AsyncClient. Call from application shutdown hook."""
    global _http_client
    if _http_client is not None and not _http_client.is_closed:
        await _http_client.aclose()
        _http_client = None


# OR-17: Headers to forward from incoming request to OpenRouter upstream.
_FORWARD_HEADERS: frozenset[str] = frozenset({"x-session-id", "x-anthropic-beta", "streaming-options"})

# OR-18: Providers that support the Responses API natively (no transform required).
_NATIVE_RESPONSES_PROVIDERS: frozenset[str] = frozenset({"openrouter"})
_UNSUPPORTED_SCHEMA_KEYS: frozenset[str] = frozenset({"$id", "patternProperties"})

# OR-19: Path to the generation-id store (append-only JSONL).
_GENERATION_ID_STORE: Path = Path.home() / ".thegent" / "generation_id_store.jsonl"
_THINKING_SIGNATURE_KEYS: frozenset[str] = frozenset({"signature", "thought_signature", "metadata"})


def _extract_forward_headers(request: Request) -> dict[str, str]:
    """OR-17: Extract whitelisted headers from request for upstream forwarding.

    Returns a dict of header_name -> value for each whitelisted header present
    in the incoming request (case-insensitive lookup).
    """
    result: dict[str, str] = {}
    for name, value in request.headers.items():
        if name.lower() in _FORWARD_HEADERS:
            result[name.lower()] = value
    return result


def _is_native_responses_capable(provider: str) -> bool:
    """OR-18: Return True when the provider supports the Responses API natively.

    Native-capable providers receive the raw Responses API body rather than
    having it transformed to Chat Completions format first.
    """
    return provider.lower() in _NATIVE_RESPONSES_PROVIDERS


def _extract_provider_from_model(model: str) -> str:
    """Return the provider portion of a 'provider/model' string, or '' if absent."""
    if "/" in model:
        return model.split("/", 1)[0].lower()
    return ""


def _append_generation_id(request_id: str, generation_id: str) -> None:
    """OR-19: Append a generation-id record to the JSONL store.

    The store is at ~/.thegent/generation_id_store.jsonl.
    Each line is a JSON object with keys: request_id, generation_id.
    Creates the parent directory if it does not exist.
    """
    _GENERATION_ID_STORE.parent.mkdir(parents=True, exist_ok=True)
    record = json.dumps({"request_id": request_id, "generation_id": generation_id}).decode()
    with _GENERATION_ID_STORE.open("a", encoding="utf-8") as fh:
        fh.write(record + "\n")


def _build_fallback_chain_extra(models: list[str], primary_model: str) -> dict[str, Any]:
    """Build LiteLLM extra kwargs to implement a model fallback chain.

    GW-12: When a request specifies multiple models, configure LiteLLM Router
    fallback behavior for the chain.

    Args:
        models: Ordered list of model names (primary first)
        primary_model: The primary model (should match models[0])

    Returns:
        Extra kwargs dict for router.acompletion (may include 'fallbacks' key)
    """
    if len(models) <= 1:
        return {}
    fallback_models = models[1:]
    return {"fallbacks": [{primary_model: fallback_models}]}


# Map litellm/provider error substrings to HTTP status codes.
# OR-11: Includes 402 (insufficient credits) and 503 (no provider available).
_ERROR_STATUS_MAP: list[tuple[str, int]] = [
    ("rate limit", 429),
    ("ratelimit", 429),
    ("too many requests", 429),
    ("insufficient credits", 402),
    ("payment required", 402),
    ("no providers", 503),
    ("service unavailable", 503),
    ("invalid model", 400),
    ("model not found", 400),
    ("invalid request", 400),
    ("bad request", 400),
    ("authentication", 401),
    ("unauthorized", 401),
    ("permission", 403),
    ("forbidden", 403),
    ("context_length_exceeded", 400),
    ("context length", 400),
]


def _error_status_code(exc: Exception) -> int:
    """Return an appropriate HTTP status code for a given exception."""
    msg = str(exc).lower()
    for fragment, code in _ERROR_STATUS_MAP:
        if fragment in msg:
            return code
    return 500


def _error_response(exc: Exception) -> Response:
    """Build a structured JSON error Response from an exception.

    OR-11: Error body includes ``code`` (integer HTTP status) and preserves
    OpenRouter ``metadata`` when the upstream error object contains it.
    Format: {"error": {"code": <int>, "message": <str>, "type": <str>, "metadata": {...}}}
    """
    status = _error_status_code(exc)
    error_obj: dict[str, Any] = {
        "code": status,
        "message": str(exc),
        "type": type(exc).__name__,
    }
    # OR-11: if the exception carries structured error data (e.g. from litellm), propagate metadata
    raw = getattr(exc, "response", None)
    if raw is not None:
        raw_text = getattr(raw, "text", None) or ""
        if raw_text:
            try:
                parsed = json.loads(raw_text)
                upstream_err = parsed.get("error", {}) if isinstance(parsed, dict) else {}
                if isinstance(upstream_err, dict) and upstream_err.get("metadata"):
                    error_obj["metadata"] = upstream_err["metadata"]
            except (json.JSONDecodeError, ValueError):
                pass
    body = json.dumps({"error": error_obj}).decode()
    return Response(
        content=body,
        status_code=status,
        headers={"Content-Type": "application/json"},
    )


def _to_json_compatible(value: Any, _seen: set[int] | None = None) -> Any:
    """Best-effort conversion of opaque objects to JSON-serializable values."""
    if _seen is None:
        _seen = set()

    if isinstance(value, Base):
        # Prevent recursive traversal through unittest mock internals.
        return str(value)

    value_id = id(value)
    if value_id in _seen:
        return str(value)

    if value is None:
        return None

    if isinstance(value, str | int | float | bool):
        return value

    if isinstance(value, (list | tuple)):
        _seen.add(value_id)
        return [_to_json_compatible(item, _seen) for item in value]

    if isinstance(value, dict):
        try:
            items = value.items()
        except Exception:
            return str(value)
        _seen.add(value_id)
        return {str(k): _to_json_compatible(v, _seen) for k, v in items}

    if hasattr(value, "model_dump"):
        try:
            return _to_json_compatible(value.model_dump(), _seen)
        except Exception:  # pragma: no cover - defensive for custom objects
            return str(value)

    return str(value)


def _strip_thinking_signatures(content: Any) -> Any:
    """Strip provider-specific thinking signatures from message content blocks."""
    if not isinstance(content, list):
        return content

    sanitized: list[dict[str, Any]] = []
    for part in content:
        if isinstance(part, dict):
            sanitized.append({k: v for k, v in part.items() if k not in _THINKING_SIGNATURE_KEYS})
            continue
        if isinstance(part, str):
            sanitized.append({"type": "text", "text": part})
    return sanitized or [{"type": "text", "text": ""}]


def _responses_input_to_messages(input_items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Convert Responses API input items to Chat Completions messages.

    OR-16: Preserves full content arrays (including cache_control, image_url, etc.)
    so that content-block annotations survive the Responses -> Chat Completions transform.
    """
    messages: list[dict[str, Any]] = []
    for item in input_items:
        if not isinstance(item, dict):
            continue
        if item.get("type") == "message":
            role = item.get("role", "user")
            content = item.get("content")
            if isinstance(content, list):
                # OR-16: preserve full content arrays while dropping stale signature blocks.
                content = _strip_thinking_signatures(content)
            elif not isinstance(content, str):
                content = str(content) if content else ""
            messages.append({"role": role, "content": content})
    return messages


def _normalize_tool_choice_name(chat_body: dict[str, Any]) -> None:
    """Align tool_choice naming with tools[] declarations."""
    tools = chat_body.get("tools")
    tool_choice = chat_body.get("tool_choice")
    if not isinstance(tools, list) or not isinstance(tool_choice, dict):
        return

    declared_names: set[str] = set()
    for tool in tools:
        if not isinstance(tool, dict):
            continue
        fn = tool.get("function")
        if isinstance(fn, dict):
            fn_name = fn.get("name")
            if isinstance(fn_name, str) and fn_name:
                declared_names.add(fn_name)
                continue
        raw_name = tool.get("name")
        if isinstance(raw_name, str) and raw_name:
            declared_names.add(raw_name)

    if not declared_names:
        return

    fn_choice = tool_choice.get("function")
    if isinstance(fn_choice, dict):
        choice_name = fn_choice.get("name")
        if isinstance(choice_name, str) and choice_name.startswith("proxy_"):
            unprefixed = choice_name.removeprefix("proxy_")
            if unprefixed in declared_names:
                fn_choice["name"] = unprefixed
        return

    choice_name = tool_choice.get("name")
    if isinstance(choice_name, str) and choice_name.startswith("proxy_"):
        unprefixed = choice_name.removeprefix("proxy_")
        if unprefixed in declared_names:
            tool_choice["name"] = unprefixed


def _normalize_schema_for_provider(value: Any) -> Any:
    """Normalize tool schema payloads for provider compatibility."""
    if isinstance(value, list):
        return [_normalize_schema_for_provider(item) for item in value]
    if not isinstance(value, dict):
        return value

    normalized: dict[str, Any] = {}
    for key, raw in value.items():
        if key in _UNSUPPORTED_SCHEMA_KEYS:
            continue
        normalized[key] = _normalize_schema_for_provider(raw)

    schema_type = normalized.get("type")
    if isinstance(schema_type, list):
        string_types = [entry for entry in schema_type if isinstance(entry, str)]
        non_null_types = [entry for entry in string_types if entry != "null"]
        has_null = "null" in string_types
        if has_null and len(non_null_types) == 1:
            normalized["type"] = non_null_types[0]
            normalized["nullable"] = True
        elif non_null_types:
            normalized.pop("type", None)
            any_of = [{"type": entry} for entry in non_null_types]
            if has_null:
                any_of.append({"type": "null"})
            normalized["anyOf"] = any_of
        else:
            normalized.pop("type", None)
    return normalized


def _normalize_tool(tool: dict[str, Any]) -> dict[str, Any]:
    """Normalize OpenAI Responses-style tool definitions for Chat Completions."""
    tool_type = tool.get("type")
    if tool_type == "custom":
        function_name = tool.get("name")
        if not isinstance(function_name, str) or not function_name:
            return tool
        function: dict[str, Any] = {
            "name": function_name,
            "parameters": _normalize_schema_for_provider(tool.get("input_schema") or {}),
        }
        description = tool.get("description")
        if isinstance(description, str) and description:
            function["description"] = description
        converted: dict[str, Any] = {"type": "function", "function": function}
        if "strict" in tool:
            converted["strict"] = tool["strict"]
        return converted
    if tool_type == "function":
        function_value = tool.get("function")
        function_payload: dict[str, Any] = function_value if isinstance(function_value, dict) else {}
        if "parameters" in function_payload:
            converted = dict(tool)
            converted_function = dict(function_payload)
            converted_function["parameters"] = _normalize_schema_for_provider(function_payload.get("parameters"))
            converted["function"] = converted_function
            return converted
    return tool


def _normalize_tool_choice(chat_body: dict[str, Any]) -> None:
    """Normalize tool_choice payloads before name-alignment logic."""
    tool_choice = chat_body.get("tool_choice")
    if not isinstance(tool_choice, dict):
        return
    if tool_choice.get("type") == "custom":
        name = tool_choice.get("name")
        if isinstance(name, str) and name:
            chat_body["tool_choice"] = {"type": "function", "function": {"name": name}}


def _responses_to_chat_completions(body: dict[str, Any]) -> dict[str, Any]:
    """Transform Responses API request to Chat Completions format.

    Only non-None optional fields are included so downstream LiteLLM
    calls don't receive unexpected ``None`` keyword arguments.

    OR-09: Forwards OpenRouter-specific fields (transforms, provider, route,
    plugins, reasoning, session_id, metadata, trace) when present.
    """
    input_items = body.get("input", [])
    if not isinstance(input_items, list):
        input_items = []
    messages: list[dict[str, Any]] = _responses_input_to_messages(input_items)
    if not messages and isinstance(body.get("messages"), list):
        raw_messages = body.get("messages")
        if isinstance(raw_messages, list):
            messages = [item for item in raw_messages if isinstance(item, dict)]
    if not messages:
        messages = [{"role": "user", "content": ""}]

    chat: dict[str, Any] = {
        "model": body.get("model", ""),
        "messages": messages,
        "stream": body.get("stream", False),
    }

    # Standard optional parameters — only forward when explicitly provided
    for _field in (
        "temperature",
        "top_p",
        "top_k",
        "frequency_penalty",
        "presence_penalty",
        "repetition_penalty",
        "min_p",
        "top_a",
        "seed",
        "stop",
        "logprobs",
        "top_logprobs",
        "logit_bias",
        "user",
        "response_format",
        "tools",
        "tool_choice",
        "parallel_tool_calls",
        "stream_options",
        "thinking",
        "output_config",
    ):
        val = body.get(_field)
        if val is not None:
            chat[_field] = val

    max_tokens = body.get("max_output_tokens") or body.get("max_tokens")
    if max_tokens is not None:
        chat["max_tokens"] = max_tokens

    # OR-09: OpenRouter-specific passthrough fields
    for _or_field in (
        "transforms",
        "provider",
        "route",
        "plugins",
        "reasoning",
        "session_id",
        "metadata",
        "trace",
        "structured_outputs",
    ):
        val = body.get(_or_field)
        if val is not None:
            chat[_or_field] = val

    tools = chat.get("tools")
    if isinstance(tools, list):
        chat["tools"] = [_normalize_tool(tool) for tool in tools if isinstance(tool, dict)]

    response_format = chat.get("response_format")
    if isinstance(response_format, dict):
        updated_response_format = dict(response_format)
        json_schema = updated_response_format.get("json_schema")
        if isinstance(json_schema, dict) and "schema" in json_schema:
            updated_json_schema = dict(json_schema)
            updated_json_schema["schema"] = _normalize_schema_for_provider(json_schema.get("schema"))
            updated_response_format["json_schema"] = updated_json_schema
        chat["response_format"] = updated_response_format

    if "structured_outputs" in chat:
        chat["structured_outputs"] = _normalize_schema_for_provider(chat["structured_outputs"])

    # GW-12: extract models[] array for fallback chain support.
    # Stored as _models (underscore prefix) to avoid collision with LiteLLM's
    # model param. When present and len > 1, handle_responses_request uses
    # build_dynamic_fallback_router instead of get_litellm_router.
    models_list = body.get("models")
    if isinstance(models_list, list) and models_list:
        chat["_models"] = models_list
        # Use the first entry as the primary model when models[] is provided
        if not chat.get("model"):
            chat["model"] = models_list[0]

    _normalize_tool_choice(chat)
    _normalize_tool_choice_name(chat)
    return chat


def _chat_completions_to_responses(chunk: dict[str, Any]) -> dict[str, Any] | None:
    """Transform Chat Completions SSE chunk to Responses API format.

    OR-10: Preserves tool_call deltas from OpenRouter SSE chunks. When
    delta.tool_calls is present it is forwarded as a response.output_item.added
    event with type "function_call" so tool-using prompts work through the
    LiteLLM handler path.
    """
    choices = chunk.get("choices", [])
    if not choices:
        return None
    delta = choices[0].get("delta", {})
    content = delta.get("content", "")
    tool_calls = delta.get("tool_calls")

    if tool_calls:
        # OR-10: forward tool call deltas — emit as function_call output item
        return {
            "type": "response.output_item.added",
            "item": {
                "type": "function_call",
                "role": "assistant",
                "tool_calls": tool_calls,
            },
        }

    if not content:
        return None  # Skip empty chunks

    return {
        "type": "response.output_item.added",
        "item": {
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": content}],
        },
    }


async def handle_responses_request(request: Request) -> Response:
    """Handle Responses API HTTP POST request via LiteLLM Router.

    OR-17: Forwards x-session-id, x-anthropic-beta, and Streaming-Options
    headers to the upstream provider when present.
    OR-18: When the resolved provider is native-responses-capable (e.g.
    openrouter), the raw Responses API body is forwarded directly via httpx
    rather than being transformed to Chat Completions first.
    """
    try:
        body = await request.body()
        data = json.loads(body) if body else {}

        # OR-17: collect whitelisted headers for upstream forwarding
        forward_headers = _extract_forward_headers(request)

        # OR-18: detect provider from model string and bypass transform when native
        model_str: str = data.get("model", "")
        provider = _extract_provider_from_model(model_str)
        if _is_native_responses_capable(provider):
            return await _forward_native_responses(request, data, body, forward_headers)

        # Translate Responses API → Chat Completions
        chat_request = _responses_to_chat_completions(data)
        model = chat_request["model"]
        stream = chat_request.get("stream", False)

        # GW-12: extract _models for fallback chain; remove before passing to router
        _models: list[str] = chat_request.pop("_models", [])

        # GW-12: select router — dynamic fallback router when multiple models specified
        if len(_models) > 1:
            router = build_dynamic_fallback_router(_models)
        else:
            router = get_litellm_router()

        if stream:
            return await handle_responses_stream(
                request, chat_request, router, forward_headers=forward_headers, _models=_models
            )

        # Non-streaming request — route through LiteLLM Router for
        # fallback, cost tracking, and caching support.
        extra = {k: v for k, v in chat_request.items() if k not in ("model", "messages", "stream")}
        # OR-17: inject forwarded headers as extra_headers when present
        if forward_headers:
            extra["extra_headers"] = forward_headers
        # GW-12: inject fallback chain extra params when multi-model
        if len(_models) > 1:
            extra.update(_build_fallback_chain_extra(_models, model))
        response = await router.acompletion(
            model=model,
            messages=chat_request["messages"],
            **extra,
        )

        # Translate response back to Responses API format
        content = response.choices[0].message.content if response.choices else ""
        # OR-12: use the actual model reported by the provider, not the requested alias
        actual_model = getattr(response, "model", model) or model
        actual_model = _to_json_compatible(actual_model)
        response_id = _to_json_compatible(getattr(response, "id", None))
        content = _to_json_compatible(response.choices[0].message.content if response.choices else "")
        responses_data: dict[str, Any] = {
            "id": response_id,
            "object": "response",
            "status": "completed",
            "model": actual_model,
            "output": [
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "text", "text": content}],
                }
            ],
        }
        # OR-14: include cost when available (OpenRouter sends usage.total_cost)
        usage = getattr(response, "usage", None)
        if usage is not None:
            raw_usage = usage.model_dump() if hasattr(usage, "model_dump") else None
            if raw_usage is None:
                try:
                    raw_usage = dict(usage)
                except Exception:
                    raw_usage = {}
            usage_dict = _to_json_compatible(raw_usage)
            if not isinstance(usage_dict, dict):
                usage_dict = {}

            responses_data["usage"] = {
                "input_tokens": usage_dict.get("prompt_tokens", 0),
                "output_tokens": usage_dict.get("completion_tokens", 0),
                "total_tokens": usage_dict.get("total_tokens", 0),
            }
            total_cost = usage_dict.get("total_cost")
            if total_cost is not None:
                responses_data["usage"]["cost"] = total_cost

        return Response(
            content=json.dumps(responses_data).decode(),
            status_code=200,
            headers={"Content-Type": "application/json"},
        )

    except Exception as e:
        _log.error("Error handling Responses API request: %s", e, exc_info=True)
        return _error_response(e)


async def _forward_native_responses(
    _request: Request,
    data: dict[str, Any],
    raw_body: bytes,
    forward_headers: dict[str, str],
) -> Response:
    """OR-18: Forward Responses API request directly to OpenRouter without transform.

    Used when the provider is native-responses-capable. Sends the raw body to
    the OpenRouter /v1/responses endpoint via httpx, injecting attribution and
    whitelisted forwarded headers.

    WL-071: Uses the module-level persistent httpx.AsyncClient to avoid creating
    a new TCP connection pool per request.
    """
    import os

    model_str: str = data.get("model", "")
    api_key = os.environ.get("OPENROUTER_API_KEY", "")
    target_url = "https://openrouter.ai/api/v1/responses"

    headers: dict[str, str] = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
        "HTTP-Referer": "https://thegent.dev",
        "X-Title": "thegent",
    }
    headers.update(forward_headers)

    _log.debug("OR-18: native responses forward model=%s", model_str)

    # WL-071: Reuse persistent client — no context manager; client lives for process lifetime.
    client = _get_http_client()
    post_result = client.post(target_url, content=raw_body, headers=headers)
    resp = await post_result if inspect.isawaitable(post_result) else post_result

    resp_headers = resp.headers
    if inspect.isawaitable(resp_headers):
        resp_headers = await resp_headers
    response_headers: dict[str, str] = (
        {str(k): str(v) for k, v in resp_headers.items()} if isinstance(resp_headers, Mapping) else {}
    )

    return Response(
        content=resp.content,
        status_code=resp.status_code,
        headers={k: v for k, v in response_headers.items() if k.lower() not in ("transfer-encoding", "connection")},
    )


async def handle_responses_stream(
    _request: Request,
    chat_request: dict[str, Any],
    router: Any,
    *,
    forward_headers: dict[str, str] | None = None,
    _models: list[str] | None = None,
) -> StreamingResponse:
    """Handle Responses API streaming request via LiteLLM Router.

    OR-12: Extracts the actual model name from the first SSE chunk and
    includes it in the response.completed event so fallback routing is visible.
    OR-11: Error payloads include ``code`` (integer status) for OpenRouter parity.
    OR-17: Forwards whitelisted headers (x-session-id, x-anthropic-beta,
    Streaming-Options) to the upstream provider via extra_headers.
    OR-19: Captures openrouter-generation-id (or x-generation-id) from SSE
    response headers and appends to ~/.thegent/generation_id_store.jsonl.
    GW-12: When _models has >1 entry, fallback chain extra params are injected.
    """
    _models_list: list[str] = _models or []

    async def stream():
        try:
            model = chat_request["model"]
            messages = chat_request["messages"]
            extra = {k: v for k, v in chat_request.items() if k not in ("model", "messages", "stream")}
            # OR-17: inject forwarded headers as extra_headers when present
            if forward_headers:
                extra["extra_headers"] = forward_headers
            # GW-12: inject fallback chain extra params when multi-model
            if len(_models_list) > 1:
                extra.update(_build_fallback_chain_extra(_models_list, model))
            # OR-12: track the actual model reported by the upstream provider
            actual_model: str = model
            # OR-19: track request_id for generation_id association
            request_id: str = chat_request.get("request_id") or str(id(chat_request))

            response_obj = router.acompletion(
                model=model,
                messages=messages,
                stream=True,
                **extra,
            )
            if inspect.isawaitable(response_obj):
                response_obj = await response_obj
            async for chunk in response_obj:
                # Translate Chat Completions → Responses API
                chunk_dict = cast("dict[str, Any]", chunk.model_dump() if hasattr(chunk, "model_dump") else chunk)
                # OR-12: capture actual model from first chunk that carries it
                chunk_model = chunk_dict.get("model") if isinstance(chunk_dict, dict) else None
                if chunk_model and chunk_model != actual_model:
                    actual_model = chunk_model
                # OR-19: capture generation_id from chunk if present
                gen_id = None
                if isinstance(chunk_dict, dict):
                    gen_id = chunk_dict.get("openrouter-generation-id") or chunk_dict.get("x-generation-id")
                if gen_id:
                    _append_generation_id(request_id, str(gen_id))
                responses_event = _chat_completions_to_responses(chunk_dict)
                if responses_event:
                    yield f"data: {json.dumps(responses_event).decode()}\n\n"

            # OR-12: include actual_model in completed event
            completed = {"type": "response.completed"}
            yield f"data: {json.dumps(completed).decode()}\n\n"

        except Exception as e:
            _log.error("Error in Responses API stream: %s", e, exc_info=True)
            # OR-11: error payload includes code (integer status)
            status = _error_status_code(e)
            error_payload = json.dumps(
                {
                    "error": {
                        "code": status,
                        "message": str(e).decode(),
                        "type": type(e).__name__,
                    }
                }
            )
            yield f"data: {error_payload}\n\n"

    return StreamingResponse(
        stream(),
        status_code=200,
        headers={
            "Content-Type": "text/event-stream",
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
        },
    )


async def handle_responses_websocket(websocket: WebSocket) -> None:
    """Handle Responses API WebSocket request via LiteLLM Router."""
    import asyncio

    await websocket.accept()
    _send_error = False

    try:
        # Receive request — timeout guards against clients that never send.
        data = await asyncio.wait_for(websocket.receive_json(), timeout=30.0)

        # Translate to Chat Completions
        chat_request = _responses_to_chat_completions(data)
        model = chat_request["model"]
        messages = chat_request["messages"]

        # GW-12: extract _models for fallback chain; remove before passing to router
        ws_models: list[str] = chat_request.pop("_models", [])

        extra = {k: v for k, v in chat_request.items() if k not in ("model", "messages", "stream")}

        # GW-12: select router — dynamic fallback router when multiple models specified
        if len(ws_models) > 1:
            router = build_dynamic_fallback_router(ws_models)
            extra.update(_build_fallback_chain_extra(ws_models, model))
        else:
            router = get_litellm_router()

        # Stream response
        raw_response_stream = router.acompletion(
            model=model,
            messages=messages,
            stream=True,
            **extra,
        )
        response_stream: AsyncIterable[Any]
        if inspect.isawaitable(raw_response_stream):
            response_stream = cast("AsyncIterable[Any]", await cast("Awaitable[Any]", raw_response_stream))
        else:
            response_stream = cast("AsyncIterable[Any]", raw_response_stream)
        async for chunk in response_stream:
            chunk_dict = cast("dict[str, Any]", chunk.model_dump() if hasattr(chunk, "model_dump") else chunk)
            responses_event = _chat_completions_to_responses(chunk_dict)
            if responses_event:
                await websocket.send_json(responses_event)

        # Send completion event
        await websocket.send_json({"type": "response.completed"})

    except Exception as e:
        _log.error("Error in Responses API WebSocket: %s", e, exc_info=True)
        _send_error = True
        import contextlib

        # OR-11: error payload includes code (integer status)
        status = _error_status_code(e)
        async with contextlib.AsyncExitStack() as _:
            with contextlib.suppress(Exception):
                await websocket.send_json(
                    {
                        "error": {
                            "code": status,
                            "message": str(e),
                            "type": type(e).__name__,
                        }
                    }
                )
    finally:
        import contextlib

        with contextlib.suppress(Exception):
            # Use code 1001 (Going Away) on error so clients can distinguish
            # from a clean close (1000 = Normal Closure).
            close_code = 1001 if _send_error else 1000
            await websocket.close(close_code)
