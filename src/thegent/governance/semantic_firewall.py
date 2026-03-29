"""WP-28002: Semantic Firewall for Model Output.
Analyzes model outputs for semantic violations, alignment drift, and forbidden patterns.
Sits between the model and the final execution environment.
"""

import logging
import re

from pydantic import BaseModel

_log = logging.getLogger(__name__)


class FirewallRule(BaseModel):
    """Definition of a semantic firewall rule."""

    rule_id: str
    pattern: str  # Regex or keyword
    action: str  # 'block', 'warn', 'redact'
    reason: str


class SemanticFirewall:
    """Protects the agent environment from unsafe model outputs."""

    def __init__(self) -> None:
        self.rules: list[FirewallRule] = [
            FirewallRule(
                rule_id="R1",
                pattern=r"password\s*=\s*['\"].*['\"]",
                action="redact",
                reason="Sensitive credential detection.",
            ),
            FirewallRule(
                rule_id="R2", pattern=r"rm\s+-rf\s+/", action="block", reason="Destructive root command detected."
            ),
            FirewallRule(
                rule_id="R3",
                pattern=r"I\s+cannot\s+perform\s+this\s+action",
                action="warn",
                reason="Possible model refusal detected.",
            ),
        ]

    def inspect_output(self, output: str) -> tuple[str, list[str]]:
        """Inspect model output and apply firewall rules."""
        _log.info("Firewall inspecting model output (Length: %d)", len(output))

        violations = []
        modified_output = output

        for rule in self.rules:
            if re.search(rule.pattern, output, re.IGNORECASE):
                violations.append(f"{rule.rule_id}: {rule.reason}")

                if rule.action == "block":
                    _log.error("FIREWALL BLOCK: %s", rule.reason)
                    return "ERROR: BLOCK BY SEMANTIC FIREWALL", [f"CRITICAL: {rule.reason}"]

                if rule.action == "redact":
                    _log.warning("FIREWALL REDACT: %s", rule.reason)
                    modified_output = re.sub(
                        rule.pattern, "[REDACTED BY FIREWALL]", modified_output, flags=re.IGNORECASE
                    )

                if rule.action == "warn":
                    _log.warning("FIREWALL WARNING: %s", rule.reason)

        return modified_output, violations
