"""GW-52: JSON schema output validation guardrail.

Validates LLM response content against a JSON schema provided in the request.

# @trace FR-GUARD-052
"""

from __future__ import annotations

import orjson as json
from dataclasses import dataclass
from typing import Any

try:
    import jsonschema as _jsonschema

    _JSONSCHEMA_AVAILABLE = True
except ModuleNotFoundError:
    _JSONSCHEMA_AVAILABLE = False


@dataclass
class SchemaValidationResult:
    valid: bool
    errors: list[str]  # validation error messages
    parsed: Any  # the parsed JSON if valid, None otherwise


def validate_json_output(content: str, schema: dict) -> SchemaValidationResult:
    """Validate LLM response content string against a JSON schema.

    Parses ``content`` as JSON.  If parsing succeeds and ``jsonschema`` is
    available, validates the parsed object against ``schema``.  When
    ``jsonschema`` is not installed only JSON-parse validity is checked.

    Args:
        content: Raw text from the LLM response.
        schema: JSON Schema dict to validate against.

    Returns:
        SchemaValidationResult with ``valid``, ``errors``, and ``parsed``.
    """
    # Step 1 — parse JSON
    try:
        parsed = json.loads(content)
    except (json.JSONDecodeError, ValueError) as exc:
        return SchemaValidationResult(
            valid=False,
            errors=[f"JSON parse error: {exc}"],
            parsed=None,
        )

    # Step 2 — schema validation (if available)
    if _JSONSCHEMA_AVAILABLE:
        validator = _jsonschema.Draft7Validator(schema)
        validation_errors = sorted(validator.iter_errors(parsed), key=lambda e: e.path)
        if validation_errors:
            return SchemaValidationResult(
                valid=False,
                errors=[e.message for e in validation_errors],
                parsed=None,
            )

    return SchemaValidationResult(valid=True, errors=[], parsed=parsed)


def extract_response_schema(body: dict) -> dict | None:
    """Extract a JSON schema from an OpenAI-style request body.

    Returns the schema dict if ``response_format.type == "json_schema"`` and a
    ``schema`` key is present.  Returns ``None`` otherwise.

    Args:
        body: The parsed request body dict.

    Returns:
        Schema dict or ``None``.
    """
    response_format = body.get("response_format", {})
    if not isinstance(response_format, dict):
        return None
    if response_format.get("type") != "json_schema":
        return None
    return response_format.get("schema") or None


def extract_llm_response_content(response: dict) -> str:
    """Extract the assistant message content from an OpenAI-style response dict.

    Returns the content string from ``response["choices"][0]["message"]["content"]``
    or an empty string if the path does not exist.

    Args:
        response: Parsed LLM response dict.

    Returns:
        Content string, or ``""`` on any missing key.
    """
    try:
        return response["choices"][0]["message"]["content"] or ""
    except (KeyError, IndexError, TypeError):
        return ""
