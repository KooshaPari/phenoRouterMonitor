"""OpenTelemetry helpers for agent execution spans."""

from __future__ import annotations

from collections.abc import Iterator
from contextlib import contextmanager

from opentelemetry import trace
from opentelemetry.trace import Span, SpanKind

_TRACER = trace.get_tracer(__name__)


@contextmanager
def instrument_genai_call(
    *,
    agent_name: str,
    model: str,
    system: str | None = None,
) -> Iterator[Span]:
    """Open a CLIENT span for a single GenAI tool invocation."""
    with _TRACER.start_as_current_span("thegent.genai.call", kind=SpanKind.CLIENT) as span:
        span.set_attribute("gen_ai.agent.name", agent_name)
        span.set_attribute("gen_ai.request.model", model)
        if system:
            span.set_attribute("gen_ai.system", system)
        yield span
