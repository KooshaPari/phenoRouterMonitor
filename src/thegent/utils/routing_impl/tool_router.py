"""Central Tool Router for thegent.
Implements the "Routed Toolset" pattern to bypass LLM tool limits (e.g., 128 tools).
Semantically selects and injects only relevant tools into the active context.
"""

import json
import logging
from pathlib import Path
from typing import Any

from pydantic import BaseModel, TypeAdapter

_log = logging.getLogger(__name__)


class ToolDefinition(BaseModel):
    """Metadata for a registered tool."""

    name: str
    description: str
    parameters: dict[str, Any]
    protocol: str  # 'mcp', 'rest', 'python', 'cli', 'wasm'
    tags: list[str] = []
    category: str = "general"


# Pre-compile the TypeAdapter for ToolDefinition
tool_adapter = TypeAdapter(ToolDefinition)


class ToolRouter:
    """Manages tool discovery and semantic routing."""

    def __init__(self, registry_path: Path | None = None) -> None:
        self.registry_path = registry_path or Path("tools_registry.json")
        self.tools: dict[str, ToolDefinition] = {}
        self._load_registry()

    def _load_registry(self) -> None:
        """Load registered tools from the filesystem."""
        if not self.registry_path.exists():
            # Initialize with default tools if registry doesn't exist
            self._initialize_defaults()
            return

        try:
            with open(self.registry_path) as f:
                data = json.load(f)
                for tool_data in data:
                    tool = tool_adapter.validate_python(tool_data)
                    self.tools[tool.name] = tool
        except Exception as e:
            _log.error("Failed to load tool registry: %s", e)

    def _initialize_defaults(self) -> None:
        """Populate registry with core thegent tools."""
        defaults = [
            ToolDefinition(
                name="deep_research",
                description="Performs aggressive web research across Reddit, GitHub, Arxiv, and DDG.",
                parameters={"query": "string", "output": "string"},
                protocol="python",
                tags=["research", "web", "scraping"],
                category="research",
            ),
            ToolDefinition(
                name="shell_execute",
                description="Executes a shell command in a secure environment.",
                parameters={"command": "string"},
                protocol="cli",
                tags=["shell", "terminal", "system"],
                category="utility",
            ),
            ToolDefinition(
                name="wasm_linter",
                description="Fast-path code linting using a Wasm sandbox.",
                parameters={"code": "string", "language": "string"},
                protocol="wasm",
                tags=["lint", "code", "wasm"],
                category="dev",
            ),
        ]
        for tool in defaults:
            self.tools[tool.name] = tool
        self.save_registry()

    def save_registry(self) -> None:
        """Persist current tool registry to the filesystem."""
        try:
            with open(self.registry_path, "w") as f:
                json.dump([t.model_dump() for t in self.tools.values()], f, indent=2)
        except Exception as e:
            _log.error("Failed to save tool registry: %s", e)

    def register_tool(self, tool: ToolDefinition) -> None:
        """Register a new tool in the router."""
        self.tools[tool.name] = tool
        self.save_registry()
        _log.info("Registered tool: %s", tool.name)

    def route(self, prompt: str, limit: int = 5) -> list[ToolDefinition]:
        """Perform keyword-based routing to select relevant tools.
        (Future: Upgrade to semantic search with embeddings)
        """
        prompt_lower = prompt.lower()
        scored_tools = []

        for tool in self.tools.values():
            score = 0
            # Check name
            if tool.name.lower() in prompt_lower:
                score += 5
            # Check tags
            for tag in tool.tags:
                if tag.lower() in prompt_lower:
                    score += 2
            # Check description keywords
            for word in tool.description.lower().split():
                if len(word) > 3 and word in prompt_lower:
                    score += 1

            if score > 0:
                scored_tools.append((score, tool))

        # Sort by score descending
        scored_tools.sort(key=lambda x: x[0], reverse=True)

        # Return top N tools
        return [t for _, t in scored_tools[:limit]]

    def get_tool_prompt_injection(self, prompt: str) -> str:
        """Generate a string of tool definitions to inject into the LLM context."""
        relevant_tools = self.route(prompt)
        if not relevant_tools:
            return ""

        injection = "\n\n### RELEVANT TOOLS FOR THIS TASK ###\n"
        for tool in relevant_tools:
            injection += f"- **{tool.name}**: {tool.description}\n"
            injection += f"  Params: {json.dumps(tool.parameters)}\n"
        return injection
