"""WP-24002: Recursive Tool Discovery & Adaptation.
Enables agents to discover, wrap, and use new tools dynamically at runtime.
Includes automatic interface adaptation for foreign tool protocols.
"""

import orjson as json
import logging
import shlex
from thegent.infra.shim_subprocess import run as shim_run
from collections.abc import Callable
from importlib import import_module
from typing import Any

import httpx
from pydantic import BaseModel, TypeAdapter

_log = logging.getLogger(__name__)


class ToolDefinition(BaseModel):
    """Metadata for a dynamically discovered tool."""

    tool_id: str
    description: str
    parameters: dict[str, Any]
    protocol: str  # 'mcp', 'rest', 'python', 'cli'
    endpoint: str | None = None
    method: str = "POST"
    target: str | None = None
    command: str | None = None


# Pre-compile the TypeAdapter for ToolDefinition to speed up high-frequency validation (Phase 1: JIT Migration)
tool_definition_adapter = TypeAdapter(ToolDefinition)


class ToolAdapter:
    """Adapts foreign tool interfaces to thegent's canonical tool protocol."""

    def __init__(self, agent_id: str) -> None:
        self.agent_id = agent_id
        self.discovered_tools: dict[str, ToolDefinition] = {}

    @staticmethod
    def _extract_required_parameters(tool: ToolDefinition) -> set[str]:
        required: set[str] = set()
        for name, schema in tool.parameters.items():
            if isinstance(schema, dict):
                if schema.get("required", True):
                    required.add(name)
            else:
                required.add(name)
        return required

    @classmethod
    def _validate_kwargs(cls, tool: ToolDefinition, kwargs: dict[str, Any]) -> dict[str, Any] | None:
        expected = set(tool.parameters.keys())
        required = cls._extract_required_parameters(tool)
        provided = set(kwargs.keys())

        missing = sorted(required - provided)
        unexpected = sorted(provided - expected)
        if missing or unexpected:
            return {
                "status": "error",
                "error": {
                    "type": "contract_violation",
                    "tool_id": tool.tool_id,
                    "missing": missing,
                    "unexpected": unexpected,
                    "expected": sorted(expected),
                },
            }
        return None

    @staticmethod
    async def _execute_mcp(tool: ToolDefinition, kwargs: dict[str, Any]) -> dict[str, Any]:
        if not tool.target:
            raise ValueError(f"Tool {tool.tool_id} missing MCP target.")
        return {
            "status": "success",
            "tool": tool.tool_id,
            "protocol": "mcp",
            "target": tool.target,
            "data": {"arguments": kwargs},
        }

    @staticmethod
    async def _execute_rest(tool: ToolDefinition, kwargs: dict[str, Any]) -> dict[str, Any]:
        if not tool.endpoint:
            raise ValueError(f"Tool {tool.tool_id} missing REST endpoint.")
        method = (tool.method or "POST").upper()
        async with httpx.AsyncClient(timeout=10.0) as client:
            response = await client.request(method, tool.endpoint, json=kwargs)
            response.raise_for_status()
            payload = response.json()
        return {
            "status": "success",
            "tool": tool.tool_id,
            "protocol": "rest",
            "data": payload,
        }

    @staticmethod
    async def _execute_python(tool: ToolDefinition, kwargs: dict[str, Any]) -> dict[str, Any]:
        if not tool.target or ":" not in tool.target:
            raise ValueError(f"Tool {tool.tool_id} missing python target in module:function format.")
        module_name, attr = tool.target.split(":", 1)
        module = import_module(module_name)
        fn = getattr(module, attr, None)
        if not callable(fn):
            raise ValueError(f"Python target '{tool.target}' is not callable.")
        data = fn(**kwargs)
        return {
            "status": "success",
            "tool": tool.tool_id,
            "protocol": "python",
            "data": data,
        }

    @staticmethod
    async def _execute_cli(tool: ToolDefinition, kwargs: dict[str, Any]) -> dict[str, Any]:
        if not tool.command:
            raise ValueError(f"Tool {tool.tool_id} missing CLI command.")
        cmd = shlex.split(tool.command)
        for key in sorted(kwargs):
            cmd.extend([f"--{key.replace('_', '-')}", str(kwargs[key])])
        process = shim_run(cmd, capture_output=True, text=True, check=False)
        if process.returncode != 0:
            raise RuntimeError(process.stderr.strip() or f"Command failed with exit code {process.returncode}")
        return {
            "status": "success",
            "tool": tool.tool_id,
            "protocol": "cli",
            "data": process.stdout.strip(),
        }

    def discover_tools(self, target_path: str) -> list[ToolDefinition]:
        """Scan a path or endpoint for new tools."""
        _log.info("Starting recursive tool discovery in: %s", target_path)

        # Simulated discovery logic
        found = [
            ToolDefinition(
                tool_id="net_scanner",
                description="Scans local network for peer agents",
                parameters={"subnet": "string"},
                protocol="cli",
            ),
            ToolDefinition(
                tool_id="log_analyzer",
                description="Analyzes structured logs for anomalies",
                parameters={"log_file": "string"},
                protocol="python",
            ),
        ]

        for tool in found:
            self.discovered_tools[tool.tool_id] = tool

        return found

    def wrap_tool(self, tool_id: str) -> Callable:
        """Wrap a discovered tool into a standard execution function."""
        tool = self.discovered_tools.get(tool_id)
        if not tool:
            raise ValueError(f"Tool {tool_id} not found in discovery cache.")

        _log.info("Adapting interface for tool: %s (protocol: %s)", tool_id, tool.protocol)

        async def adapted_call(**kwargs):
            _log.info("Executing adapted tool call for %s with args: %s", tool_id, kwargs)
            validation_error = self._validate_kwargs(tool, kwargs)
            if validation_error is not None:
                return validation_error

            if tool.protocol == "mcp":
                return await self._execute_mcp(tool, kwargs)
            if tool.protocol == "rest":
                return await self._execute_rest(tool, kwargs)
            if tool.protocol == "python":
                return await self._execute_python(tool, kwargs)
            if tool.protocol == "cli":
                return await self._execute_cli(tool, kwargs)
            raise ValueError(f"Unsupported protocol: {tool.protocol}")

        return adapted_call

    def generate_binding(self, tool_id: str) -> str:
        """Generate a Python/JSON binding for the tool to be used in prompts."""
        tool = self.discovered_tools.get(tool_id)
        if not tool:
            return ""

        return f"Tool: {tool.tool_id}\nDescription: {tool.description}\nParams: {json.dumps(tool.parameters).decode()}"


# Register with unified adapter registry
from thegent.adapters.ports import AdapterRegistry


class ToolAdapterWrapper:
    """Tool adapter wrapper for registry"""

    def __init__(self):
        self._adapter = None

    def call(self, **kwargs) -> dict:
        return {"status": "tool_adapter_ready"}


AdapterRegistry.register("tool", ToolAdapterWrapper())
