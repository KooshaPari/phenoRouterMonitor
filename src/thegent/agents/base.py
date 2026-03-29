"""Base agent runner interface.

# @trace WL-038
"""

from __future__ import annotations

import logging
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from thegent.agents.sub_agent_dispatcher import SubAgentDispatcher

_log = logging.getLogger(__name__)


@dataclass
class RunResult:
    """Result of an agent run."""

    exit_code: int
    stdout: str
    stderr: str
    timed_out: bool = False
    context_tokens_used: int | None = None
    context_window_max: int | None = None
    context_usage_ratio: float | None = None
    audio_transcript: str | None = None
    grounding_sources: list[str] | None = None


class AgentRunner:
    """Base interface for agent runners.

    An optional :attr:`sub_dispatcher` hook point is available for WL-081 through
    WL-084 to wire in :class:`~thegent.agents.sub_agent_dispatcher.SubAgentDispatcher`
    so that agent output containing tool calls or ``$defer`` directives can be
    forwarded to the appropriate sub-agent runner.

    # @trace WL-038
    # @trace WL-080
    """

    #: Hook point for SubAgentDispatcher (wired by WL-081–084). When set,
    #: subclass runners should delegate sub-agent tool calls to this dispatcher.
    sub_dispatcher: SubAgentDispatcher | None = None

    def __init__(self) -> None:
        #: Activated skills (skill name -> content) for this runner instance.
        #: Subclasses can use this to inject skill instructions into prompts.
        self.activated_skills: dict[str, str] = {}

    def _ensure_activated_skills(self) -> dict[str, str]:
        """Return the per-instance activated_skills dict, initializing it if needed.

        Subclasses that override ``__init__`` without calling ``super().__init__()``
        will not have ``activated_skills`` set.  This method ensures the attribute
        always exists before use.
        """
        if not isinstance(getattr(self, "activated_skills", None), dict):
            self.activated_skills = {}
        return self.activated_skills

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
        env: dict[str, str] | None = None,
        image_paths: list[str] | None = None,
    ) -> RunResult:
        """Run the agent with the given prompt and options."""
        raise TypeError(
            f"{self.__class__.__name__}.run() is abstract and must be implemented by a concrete AgentRunner subclass"
        )

    def activate_skill(self, name: str) -> str:
        """Activate a skill by name, loading its instructions into the runner.

        Discovers skills from both ``~/.thegent/skills/`` and ``.thegent/skills/``
        directories using :class:`~thegent.skills.discovery.SkillDiscovery`, loads
        the SKILL.md content, stores it in :attr:`activated_skills` for injection
        into prompts, and returns the skill content.

        # @trace WL-101

        Args:
            name: Exact skill name to activate (must match the skill's manifest name).

        Returns:
            The skill's instruction content string.

        Raises:
            KeyError: If no skill with the given name can be found in any search dir.
        """
        from thegent.skills.discovery import SkillDiscovery

        discovery = SkillDiscovery()
        manifest = discovery.find(name)  # raises KeyError if not found
        content = manifest.instructions
        skills = self._ensure_activated_skills()
        skills[name] = content
        _log.info("Activated skill: %s", name)
        return content

    def get_skill_prompt_suffix(self) -> str:
        """Get the prompt suffix containing all activated skill instructions.

        This method should be called when building prompts to include
        activated skill instructions in the system prompt.

        Returns:
            A string containing formatted skill instructions, or empty string
            if no skills are activated.
        """
        skills = self._ensure_activated_skills()
        if not skills:
            return ""

        sections = ["\n\n# Activated Skills\n"]
        for name, content in skills.items():
            sections.append(f"\n## {name}\n{content}")

        return "".join(sections)

    def _process_output_deferrals(
        self,
        result: RunResult,
        cwd: Path | None = None,
        project: str | None = None,
    ) -> RunResult:
        """Post-process *result* to extract ``$defer`` directives and inject them into the queue.

        Scans ``result.stdout`` and ``result.stderr`` for ``$defer <task>`` lines.
        Any found tasks are appended to the Unified Prompt Queue as ``pending`` entries.
        The result object is returned unchanged so callers can chain this call.

        # @trace WL-038

        Args:
            result:  The RunResult from a completed agent run.
            cwd:     Working directory used for the run (used as project root).
            project: Project identifier for queue entries.  Defaults to
                     the string representation of *cwd* (or ``"unknown"``).

        Returns:
            The original *result* object (unchanged).
        """
        combined = (result.stdout or "") + "\n" + (result.stderr or "")
        try:
            from thegent.orchestration.resilience.deferral import extract_deferred_tasks

            tasks = extract_deferred_tasks(combined)
        except Exception as exc:  # noqa: BLE001
            _log.warning("$defer extraction failed: %s", exc)
            return result

        if not tasks:
            return result

        effective_project = project or (str(cwd) if cwd else "unknown")
        try:
            from thegent.config import ThegentSettings
            from thegent.orchestration.resilience.deferral import inject_deferred_tasks

            settings = ThegentSettings()
            queue_path = settings.session_dir / "prompt_queue.jsonl"
            injected = inject_deferred_tasks(tasks, queue_path, project=effective_project)
            _log.info("Runner deferred %d task(s) into queue (project=%s)", injected, effective_project)
        except Exception as exc:  # noqa: BLE001
            _log.warning("$defer queue injection failed: %s", exc)

        return result
