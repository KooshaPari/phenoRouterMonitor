"""Checker Agent (Head LLM) for Lifecycle loops."""

import json
import logging
from enum import StrEnum
from typing import Any

from pydantic import BaseModel

from thegent.agents.registry import get_runner
from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


class CheckerDecision(StrEnum):
    """Possible decisions by the Checker Agent."""

    RE_PROMPT = "re_prompt"
    CONTINUE = "continue"
    KILL = "kill"


class CheckerResult(BaseModel):
    """Result of a Checker Agent decision."""

    decision: CheckerDecision
    prompt: str | None = None
    reason: str


CHECKER_SYSTEM_PROMPT = """You are the Head Checker Agent for 'thegent' orchestration system.
Your role is to oversee a Worker Agent and decide its next move.

INPUTS:
1. Governance Report: Any policy denials, warnings, or overrides.
2. Todo Spec: The original task list.
3. WBS Status: Current progress, blocked items, and completion percentages.
4. Agent Response: The most recent output from the Worker Agent.

STRUCTURAL TRIGGERS:
Look for XML tags in the 'Agent Response' to guide your decision:
- <COMPLETE> or <FEATURE_COMPLETE>: Worker signals task completion.
- <GOVERNANCE_NEEDED> or <CHECK_GATES>: Worker requests policy/audit check.
- <VALIDATE_TESTS>: Worker suggests checking coverage or running tests.
- <ADD_POLISH>: Worker suggests a cleanup/refactoring phase.
- <WRITE_TESTS> / <WRITE_DOCS>: Specific sub-task requests.

DECISION RULES:
- RE_PROMPT: If the worker needs a specific new instruction or correction.
  * TIP: If you see a trigger tag, your 'prompt' can simply repeat that tag or use keywords like 'test', 'doc', 'polish', 'audit' to trigger optimized presets.
- CONTINUE: If the worker is on the right track and should proceed.
- KILL: If the task is completed (<COMPLETE> received and verified), failed irrecoverably, or policy prevents further action.

OUTPUT FORMAT:
Return a JSON object with:
{
  "decision": "re_prompt" | "continue" | "kill",
  "prompt": "The next instruction (only if re_prompt)",
  "reason": "Brief explanation of your decision"
}
"""


class CheckerAgent:
    """Head LLM that decides the next action in a loop."""

    def __init__(self, settings: ThegentSettings, agent_name: str = "antigravity") -> None:
        self.settings = settings
        self.agent_name = agent_name
        self.runner = get_runner(agent_name)

    def decide(
        self,
        governance_report: dict[str, Any],
        todo_spec: str,
        wbs_status: dict[str, Any],
        agent_response: str,
    ) -> CheckerResult:
        """Invoke the Checker Agent to make a decision."""
        if not self.runner:
            return CheckerResult(
                decision=CheckerDecision.KILL,
                reason=f"Runner not found for checker agent: {self.agent_name}",
            )

        context = {
            "governance_report": governance_report,
            "todo_spec": todo_spec,
            "wbs_status": wbs_status,
            "agent_response": agent_response,
        }

        prompt = f"Context:\n{json.dumps(context, indent=2)}\n\n{CHECKER_SYSTEM_PROMPT}\n\nDecision:"

        try:
            result = self.runner.run(
                prompt=prompt,
                cwd=self.settings.cwd,
                mode="write",
                timeout=self.settings.default_timeout,
            )

            if result.exit_code != 0:
                return CheckerResult(
                    decision=CheckerDecision.KILL,
                    reason=f"Checker agent failed with exit code {result.exit_code}: {result.stderr}",
                )

            # Extract JSON from response (could have markdown blocks)
            output = result.stdout.strip()
            if "```json" in output:
                output = output.split("```json")[1].split("```")[0].strip()
            elif "```" in output:
                output = output.split("```")[1].split("```")[0].strip()

            data = json.loads(output)
            return CheckerResult(**data)

        except Exception as e:
            _log.exception("Checker agent decision failed")
            return CheckerResult(
                decision=CheckerDecision.KILL,
                reason=f"Exception in checker agent: {e!s}",
            )
