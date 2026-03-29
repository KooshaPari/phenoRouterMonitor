"""Governance policy and health commands facade - re-exports from submodules."""

# @trace WL-124
from __future__ import annotations

# Health and governance commands
from thegent.cli.governance.governance_health_cmds import (
    govern_configure_cmd,
    govern_go_health_cmd,
    govern_go_status_cmd,
    govern_go_cycle_cmd,
    govern_go_watch_cmd,
    govern_cost_cmd,
    guardrails_check_cmd,
    guardrails_show_cmd,
)

# Policy and compliance commands
from thegent.cli.governance.governance_policy_cmds import (
    policy_show_cmd,
    policy_purge_cmd,
    policy_check_cmd,
    contracts_registry_cmd,
    migration_cmd,
    drift_cmd,
    contracts_conformance_cmd,
    trust_status_cmd,
    signatures_list_cmd,
    signatures_verify_cmd,
    compliance_siem_test_cmd,
    compliance_plugin_check_cmd,
    compliance_redact_cmd,
)

__all__ = [
    # Health and governance
    "govern_configure_cmd",
    "govern_go_health_cmd",
    "govern_go_status_cmd",
    "govern_go_cycle_cmd",
    "govern_go_watch_cmd",
    "govern_cost_cmd",
    "guardrails_check_cmd",
    "guardrails_show_cmd",
    # Policy and compliance
    "policy_show_cmd",
    "policy_purge_cmd",
    "policy_check_cmd",
    "contracts_registry_cmd",
    "migration_cmd",
    "drift_cmd",
    "contracts_conformance_cmd",
    "trust_status_cmd",
    "signatures_list_cmd",
    "signatures_verify_cmd",
    "compliance_siem_test_cmd",
    "compliance_plugin_check_cmd",
    "compliance_redact_cmd",
]
