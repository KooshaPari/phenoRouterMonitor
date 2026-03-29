"""Re-export facade for governance compliance commands (WL-124).

This module consolidates imports from:
- governance_trust_sigs_cmds: Trust boundary and signature verification
- governance_compliance_guardrails_cmds: Compliance checks and guardrails
"""

from __future__ import annotations

# Re-export from governance_trust_sigs_cmds
from thegent.cli.governance.governance_trust_sigs_cmds import (
    trust_status_cmd,
    signatures_list_cmd,
    signatures_verify_cmd,
)

# Re-export from governance_compliance_guardrails_cmds
from thegent.cli.governance.governance_compliance_guardrails_cmds import (
    compliance_siem_test_cmd,
    compliance_plugin_check_cmd,
    compliance_redact_cmd,
    govern_cost_cmd,
    guardrails_check_cmd,
    guardrails_show_cmd,
    policy_check_cmd,
)

__all__ = [
    "compliance_plugin_check_cmd",
    "compliance_redact_cmd",
    "compliance_siem_test_cmd",
    "govern_cost_cmd",
    "guardrails_check_cmd",
    "guardrails_show_cmd",
    "policy_check_cmd",
    "signatures_list_cmd",
    "signatures_verify_cmd",
    "trust_status_cmd",
]
