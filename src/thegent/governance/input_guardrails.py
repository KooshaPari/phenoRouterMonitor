"""Input guardrails (NeMo-style) before policy checks. G-GP-02.

Validates prompt, agent, model, cwd before PolicyEngine.
See docs/governance/NEMO_GUARDRAILS_DESIGN.md.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from thegent.config import ThegentSettings


@dataclass
class GuardrailResult:
    """Result of input guardrail check."""

    passed: bool
    rail_id: str = ""
    reason: str = ""
    remediation: str = ""


@dataclass
class InputGuardrails:
    """Input validation rails before OPA/PolicyEngine. G-GP-02."""

    prompt_max_chars: int = 65536
    prompt_blocklist_patterns: list[str] = field(default_factory=list)
    agent_allowlist: list[str] = field(default_factory=list)  # Empty = allow all
    cwd_allowed_prefixes: list[str] = field(default_factory=list)  # Empty = allow all
    model_allowlist: list[str] = field(default_factory=list)  # Empty = allow all

    def check(
        self,
        prompt: str = "",
        agent: str = "",
        model: str | None = None,
        cwd: str | Path | None = None,
    ) -> GuardrailResult:
        """Validate inputs. Returns passed=True if all rails pass."""
        # prompt_length
        if prompt and len(prompt) > self.prompt_max_chars:
            return GuardrailResult(
                passed=False,
                rail_id="prompt_length",
                reason=f"Prompt exceeds {self.prompt_max_chars} chars ({len(prompt)})",
                remediation="Shorten prompt or increase THGENT_PROMPT_MAX_CHARS",
            )

        # prompt_blocklist
        for pat in self.prompt_blocklist_patterns:
            result = self._check_pattern(pat, prompt)
            if result:
                return result

        # agent_allowlist (empty = allow all)
        if self.agent_allowlist and agent and agent not in self.agent_allowlist:
            return GuardrailResult(
                passed=False,
                rail_id="agent_allowlist",
                reason=f"Agent '{agent}' not in allowlist",
                remediation=f"Use one of: {', '.join(self.agent_allowlist)}",
            )

        # cwd_restriction (empty = allow all)
        if self.cwd_allowed_prefixes and cwd:
            cwd_str = str(Path(cwd).resolve())
            if not any(cwd_str.startswith(p) for p in self.cwd_allowed_prefixes):
                return GuardrailResult(
                    passed=False,
                    rail_id="cwd_restriction",
                    reason=f"CWD {cwd_str} not under allowed prefixes",
                    remediation=f"Run from: {', '.join(self.cwd_allowed_prefixes)}",
                )

        # model_allowlist (empty = allow all)
        if self.model_allowlist and model and model not in self.model_allowlist:
            return GuardrailResult(
                passed=False,
                rail_id="model_allowlist",
                reason=f"Model '{model}' not in allowlist",
                remediation=f"Use one of: {', '.join(self.model_allowlist)}",
            )

        return GuardrailResult(passed=True)

    def _check_pattern(self, pat: str, prompt: str) -> GuardrailResult | None:
        """Helper to check a single pattern."""
        try:
            if re.search(pat, prompt or ""):
                return GuardrailResult(
                    passed=False,
                    rail_id="prompt_blocklist",
                    reason="Prompt matched blocklist pattern",
                    remediation="Remove blocked content from prompt",
                )
        except re.error:
            pass
        return None


def guardrails_from_settings(settings: ThegentSettings | None = None) -> InputGuardrails:
    """Build InputGuardrails from ThegentSettings."""
    from thegent.config import ThegentSettings

    s = settings or ThegentSettings()
    blocklist = [p.strip() for p in s.prompt_blocklist_patterns.split(",") if p.strip()]
    allowlist = [a.strip() for a in s.agent_allowlist.split(",") if a.strip()]
    cwd_prefixes = [p.strip() for p in s.cwd_allowed_prefixes.split(",") if p.strip()]
    return InputGuardrails(
        prompt_max_chars=s.prompt_max_chars,
        prompt_blocklist_patterns=blocklist,
        agent_allowlist=allowlist,
        cwd_allowed_prefixes=cwd_prefixes,
    )


def guardrails_from_env() -> InputGuardrails:
    """Deprecated: Use guardrails_from_settings() instead. Kept for backwards compatibility."""
    return guardrails_from_settings()
