"""Output schema validation for agent runs.

Validates agent output against a JSON Schema. Used by WL-113 to enforce
structured output constraints across all agent harnesses.

# @trace WL-113
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


class OutputSchemaValidator:
    """Validates agent output against a JSON Schema.

    Loads a JSON Schema from a file path and provides:
    - Output validation against that schema (fail loudly on mismatch)
    - System prompt injection text for Claude Code harness

    # @trace WL-113
    """

    def __init__(self, schema_path: Path | str) -> None:
        """Load and store the JSON Schema from *schema_path*.

        Args:
            schema_path: Path to the JSON Schema file.

        Raises:
            FileNotFoundError: If the schema file does not exist.
            ValueError: If the schema file cannot be parsed as JSON.
        """
        schema_path = Path(schema_path)
        if not schema_path.exists():
            raise FileNotFoundError(f"Schema file not found: {schema_path}")
        raw = schema_path.read_text(encoding="utf-8")
        try:
            self.schema: dict[str, Any] = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise ValueError(f"Schema file is not valid JSON: {schema_path}: {exc}") from exc
        self.schema_path = schema_path

    def validate(self, output: str) -> dict[str, Any]:
        """Parse and validate *output* against the stored schema.

        Args:
            output: The raw string output from the agent run.

        Returns:
            The parsed JSON object if valid.

        Raises:
            ValueError: If *output* is not valid JSON or fails schema validation.
        """
        try:
            data = json.loads(output)
        except json.JSONDecodeError as exc:
            raise ValueError(f"Agent output is not valid JSON: {exc}") from exc

        import fastjsonschema

        try:
            validate_fn: Any = fastjsonschema.compile(self.schema)
            validate_fn(data)
        except fastjsonschema.JsonSchemaValueException as exc:
            raise ValueError(f"Output schema validation failed: {exc.message}") from exc
        except fastjsonschema.JsonSchemaDefinitionException as exc:
            raise ValueError(f"Output schema definition error: {exc}") from exc

        return data  # type: ignore[return-value]

    def get_system_prompt_injection(self) -> str:
        """Return schema injection text for Claude Code system prompt.

        Returns:
            A string containing the schema injection instruction.
        """
        schema_json = json.dumps(self.schema, indent=2)
        return f"\n\nYou MUST respond with valid JSON matching this schema:\n{schema_json}"

    def get_codex_args(self) -> list[str]:
        """Return the CLI args to pass to the Codex harness.

        Returns:
            A list of CLI arguments to append to the codex command.
        """
        return ["--output-schema", str(self.schema_path)]
