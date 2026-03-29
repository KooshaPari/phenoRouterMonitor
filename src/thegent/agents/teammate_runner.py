"""Teammate agent runner (WP-16001)."""

import logging
from collections.abc import Callable
from pathlib import Path

from thegent.agents.base import AgentRunner, RunResult
from thegent.agents.codex_proxy import CodexProxyRunner
from thegent.config import ThegentSettings
from thegent.governance.teammates import TeammateManager

logger = logging.getLogger(__name__)


class TeammateRunner(AgentRunner):
    """Runner that executes a teammate persona by delegating to its underlying model/provider."""

    def __init__(
        self,
        teammate_id: str,
        settings: ThegentSettings | None = None,
    ) -> None:
        self.teammate_id = teammate_id
        self._settings = settings or ThegentSettings()

        # Discover teammate metadata
        mgr = TeammateManager(self._settings.cache_dir / "teammates.json")
        personas = mgr.list_personas()
        self.persona = next((p for p in personas if p.id == teammate_id), None)

        if not self.persona:
            raise ValueError(f"Unknown teammate persona: {teammate_id}")

        # Resolve underlying runner based on model
        # For now, we assume all teammates use the proxy (CodexProxyRunner)
        # unless it's a specific known direct agent.
        self.underlying_runner = CodexProxyRunner(
            "claude", settings=self._settings
        )  # Default to claude proxy for haiku/opus/sonnet

    def run(
        self,
        prompt: str,
        cwd: Path | None,
        mode: str,
        timeout: int,
        *,
        use_stream: bool = True,
        live_output: bool = False,
        on_stdout: Callable[[str], None] | None = None,
        on_stderr: Callable[[str], None] | None = None,
        agent_model: str | None = None,
        enable_search: bool = True,
        run_id: str | None = None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        # Load system prompt from persona definition
        system_prompt = self._load_system_prompt()

        # Inject system prompt into the user prompt for runners that don't support system roles yet
        # or use a more structured approach if supported.
        # For CodexProxyRunner (which uses 'codex' CLI), we can't easily pass system prompt
        # as a separate argument yet, so we prepend it.
        enhanced_prompt = f"{system_prompt}\n\nUSER TASK: {prompt}"

        model = agent_model or self.persona.default_model

        return self.underlying_runner.run(
            enhanced_prompt,
            cwd,
            mode,
            timeout,
            use_stream=use_stream,
            live_output=live_output,
            on_stdout=on_stdout,
            on_stderr=on_stderr,
            agent_model=model,
            enable_search=enable_search,
            run_id=run_id,
            env=env,
        )

    def _load_system_prompt(self) -> str:
        """Load the full markdown content of the agent definition as system prompt."""
        # Find the .md file for this teammate
        agents_dir = Path("agents")
        if not agents_dir.exists():
            return f"You are {self.persona.role}: {self.persona.description}"

        # The persona ID is name-model or name
        # We need to find the file that matches the name part
        name_part = self.persona.id
        if "-" in name_part and name_part.endswith(self.persona.default_model):
            name_part = name_part[: -(len(self.persona.default_model) + 1)]

        md_file = agents_dir / f"{name_part}.md"
        if md_file.exists():
            # Return content after frontmatter
            content = md_file.read_text()
            if content.startswith("---"):
                parts = content.split("---", 2)
                if len(parts) > 2:
                    return parts[2].strip()
            return content.strip()

        return f"You are {self.persona.role}: {self.persona.description}"
