"""Thegent CLI governance commands facade - routes to specialized modules (WL-124).

This module re-exports all governance commands from:
- governance_audit_compliance_cmds: Audit, compliance, data protection
- governance_escalation_hitl_cmds: Escalation and HITL approval handling
- governance_policy_health_cmds: Policies, contracts, health scoring, drift
- governance_discovery_guardrails_cmds: Discovery and guardrails

Direct imports from submodules preserve all public names for CLI registration.
"""

# @trace WL-124
from __future__ import annotations

# Re-export all audit & compliance commands
from thegent.cli.governance.governance_audit_compliance_cmds import (
    signatures_list_cmd,
    signatures_verify_cmd,
    compliance_siem_test_cmd,
    compliance_plugin_check_cmd,
    compliance_redact_cmd,
    govern_cost_cmd,
    guardrails_check_cmd,
    guardrails_show_cmd,
)

# Re-export all escalation & HITL commands
from thegent.cli.governance.governance_escalation_hitl_cmds import (
    escalate_add_cmd,
    escalate_approve_cmd,
    escalate_list_cmd,
    escalate_resolve_cmd,
    govern_approve_cmd,
    govern_list_pending_cmd,
    govern_reject_cmd,
    sweep_cmd,
)

# Re-export all policy & health commands
from thegent.cli.governance.governance_policy_health_cmds import (
    govern_configure_cmd,
    govern_go_health_cmd,
    govern_go_status_cmd,
    govern_go_cycle_cmd,
    govern_go_watch_cmd,
    policy_show_cmd,
    policy_purge_cmd,
    policy_check_cmd,
    contracts_registry_cmd,
    migration_cmd,
    drift_cmd,
    contracts_conformance_cmd,
    trust_status_cmd,
)

# Re-export discovery & guardrails commands
from thegent.cli.governance.governance_discovery_guardrails_cmds import (
    discovery_parse_cmd,
    discovery_register_cmd,
    discovery_scan_cmd,
)

__all__ = [
    "signatures_list_cmd",
    "signatures_verify_cmd",
    "compliance_siem_test_cmd",
    "compliance_plugin_check_cmd",
    "compliance_redact_cmd",
    "govern_cost_cmd",
    "guardrails_check_cmd",
    "guardrails_show_cmd",
    "escalate_add_cmd",
    "escalate_approve_cmd",
    "escalate_list_cmd",
    "escalate_resolve_cmd",
    "govern_approve_cmd",
    "govern_list_pending_cmd",
    "govern_reject_cmd",
    "sweep_cmd",
    "govern_configure_cmd",
    "govern_go_health_cmd",
    "govern_go_status_cmd",
    "govern_go_cycle_cmd",
    "govern_go_watch_cmd",
    "policy_show_cmd",
    "policy_purge_cmd",
    "policy_check_cmd",
    "contracts_registry_cmd",
    "migration_cmd",
    "drift_cmd",
    "contracts_conformance_cmd",
    "trust_status_cmd",
    "discovery_parse_cmd",
    "discovery_register_cmd",
    "discovery_scan_cmd",
]
