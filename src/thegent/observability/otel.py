from __future__ import annotations

# @trace FR-OBS-037
"""GW-37: OTel OTLP span export for LLM gateway observability.

Exports LLM call spans to an OTLP collector using the OpenTelemetry SDK.
When opentelemetry is not installed, all functions are no-ops.

GenAI semantic convention attributes:
  gen_ai.system          — provider (e.g. "openai", "anthropic")
  gen_ai.request.model   — model name
  gen_ai.request.max_tokens
  gen_ai.usage.prompt_tokens
  gen_ai.usage.completion_tokens
  gen_ai.response.finish_reasons  — list as comma-joined string
  thegent.event_id       — tg-event-id header value
  thegent.cost_usd       — float, request cost
  thegent.cache_hit      — bool
  http.status_code       — response status
"""

import logging
import threading
from importlib import import_module
from dataclasses import dataclass
from typing import Any

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Optional dependency detection
# ---------------------------------------------------------------------------

try:
    _otel_trace = import_module("opentelemetry.trace")
    OTLPSpanExporter = import_module("opentelemetry.exporter.otlp.proto.grpc.trace_exporter").OTLPSpanExporter
    Resource = import_module("opentelemetry.sdk.resources").Resource
    TracerProvider = import_module("opentelemetry.sdk.trace").TracerProvider
    BatchSpanProcessor = import_module("opentelemetry.sdk.trace.export").BatchSpanProcessor

    _OTEL_AVAILABLE = True
except Exception:
    _OTEL_AVAILABLE = False


# ---------------------------------------------------------------------------
# Config dataclass
# ---------------------------------------------------------------------------


@dataclass
class OtelConfig:
    """Configuration for the OTLP exporter."""

    endpoint: str = "http://localhost:4317"
    service_name: str = "thegent-gateway"
    enabled: bool = True
    insecure: bool = True


# ---------------------------------------------------------------------------
# Singleton state
# ---------------------------------------------------------------------------

_config_lock = threading.Lock()
_config: OtelConfig | None = None

# Module-level tracer (set by configure_otel; no-op object when unavailable)
_tracer: Any = None
_provider: Any = None


# ---------------------------------------------------------------------------
# _NoOpSpan — safe dummy used when otel is unavailable or disabled
# ---------------------------------------------------------------------------


class _NoOpSpan:
    """Dummy span that silently accepts all attribute calls and context methods."""

    def set_attribute(self, _key: str, _value: Any) -> None:  # noqa: D102
        pass

    def record_exception(self, _exc: BaseException, **_kwargs: Any) -> None:  # noqa: D102
        pass

    def set_status(self, *_args: Any, **_kwargs: Any) -> None:  # noqa: D102
        pass

    def end(self) -> None:  # noqa: D102
        pass

    # Context manager support so callers can use `with start_llm_span(...) as s:`
    def __enter__(self) -> _NoOpSpan:
        return self

    def __exit__(self, *_args: Any) -> None:
        pass


_NOOP_SPAN = _NoOpSpan()


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def configure_otel(config: OtelConfig) -> None:
    """Configure the global TracerProvider with OTLP exporter.

    OTEL bootstrap is fail-loud by default: if enabled and OTEL packages are
    unavailable, this raises RuntimeError. Explicit opt-out remains available
    via config.enabled=False.
    """
    global _config, _tracer, _provider

    with _config_lock:
        _config = config

    if not config.enabled:
        _log.debug("OTel disabled via config; otel.py operating in no-op mode")
        return

    if not _OTEL_AVAILABLE:
        raise RuntimeError(
            "OTel bootstrap failed: opentelemetry OTLP exporter packages are unavailable. "
            "Install OTEL dependencies or set enabled=False to opt out explicitly."
        )

    resource = Resource.create({"service.name": config.service_name})
    exporter = OTLPSpanExporter(
        endpoint=config.endpoint,
        insecure=config.insecure,
    )
    processor = BatchSpanProcessor(exporter)
    provider = TracerProvider(resource=resource)
    provider.add_span_processor(processor)
    _otel_trace.set_tracer_provider(provider)

    _provider = provider
    _tracer = _otel_trace.get_tracer("thegent.gateway")
    _log.info("OTel OTLP exporter configured: endpoint=%s service=%s", config.endpoint, config.service_name)


def get_tracer() -> Any:
    """Return the module-level tracer.

    Returns a no-op tracer proxy when opentelemetry is unavailable or
    configure_otel has not been called yet.
    """
    return _tracer


def get_otel_config() -> OtelConfig:
    """Return the current singleton OtelConfig, creating a default one if needed."""
    global _config
    with _config_lock:
        if _config is None:
            _config = OtelConfig()
        return _config


def reset_otel_config() -> None:
    """Reset the singleton config and tracer state. Intended for testing only."""
    global _config, _tracer, _provider
    with _config_lock:
        _config = None
    _tracer = None
    _provider = None


def start_llm_span(
    model: str,
    provider: str,
    event_id: str = "",
    max_tokens: int | None = None,
) -> Any:
    """Start a new LLM span.

    Returns an opentelemetry Span when otel is available and configured,
    otherwise returns a _NoOpSpan that is safe to pass to finish_llm_span.
    """
    if not _OTEL_AVAILABLE or _tracer is None:
        return _NoOpSpan()

    attributes: dict[str, Any] = {
        "gen_ai.system": provider,
        "gen_ai.request.model": model,
    }
    if event_id:
        attributes["thegent.event_id"] = event_id
    if max_tokens is not None:
        attributes["gen_ai.request.max_tokens"] = max_tokens

    span = _tracer.start_span(
        f"gen_ai.{provider}.chat",
        kind=_otel_trace.SpanKind.CLIENT,
        attributes=attributes,
    )
    return span


def finish_llm_span(
    span: Any,
    *,
    prompt_tokens: int = 0,
    completion_tokens: int = 0,
    finish_reason: str = "",
    cost_usd: float = 0.0,
    cache_hit: bool = False,
    status_code: int = 200,
    error: str = "",
) -> None:
    """Record completion attributes on *span* and end it.

    Always safe to call even when span is None or a _NoOpSpan.
    """
    if span is None:
        return

    try:
        span.set_attribute("gen_ai.usage.prompt_tokens", prompt_tokens)
        span.set_attribute("gen_ai.usage.completion_tokens", completion_tokens)
        span.set_attribute("http.status_code", status_code)
        span.set_attribute("thegent.cost_usd", cost_usd)
        span.set_attribute("thegent.cache_hit", cache_hit)

        if finish_reason:
            span.set_attribute("gen_ai.response.finish_reasons", finish_reason)

        if error:
            span.set_attribute("error.message", error)
            if _OTEL_AVAILABLE:
                from opentelemetry.trace import Status, StatusCode

                span.set_status(Status(StatusCode.ERROR, error))
        elif _OTEL_AVAILABLE and not isinstance(span, _NoOpSpan):
            from opentelemetry.trace import Status, StatusCode

            span.set_status(Status(StatusCode.OK))

        span.end()
    except Exception:
        _log.debug("finish_llm_span: error setting span attributes", exc_info=True)


def record_llm_call(
    model: str,
    provider: str,
    *,
    event_id: str = "",
    prompt_tokens: int = 0,
    completion_tokens: int = 0,
    finish_reason: str = "",
    cost_usd: float = 0.0,
    cache_hit: bool = False,
    status_code: int = 200,
    error: str = "",
    duration_sec: float = 0.0,
) -> None:
    """Convenience: create span, set all attributes, and end it immediately.

    This is the primary entry point for recording a complete LLM call.
    When opentelemetry is unavailable, this function is a silent no-op.
    """
    span = start_llm_span(model=model, provider=provider, event_id=event_id)

    if duration_sec and not isinstance(span, _NoOpSpan) and _OTEL_AVAILABLE:
        # Attach duration as a custom attribute for backends that support it
        span.set_attribute("thegent.duration_sec", duration_sec)

    finish_llm_span(
        span,
        prompt_tokens=prompt_tokens,
        completion_tokens=completion_tokens,
        finish_reason=finish_reason,
        cost_usd=cost_usd,
        cache_hit=cache_hit,
        status_code=status_code,
        error=error,
    )
