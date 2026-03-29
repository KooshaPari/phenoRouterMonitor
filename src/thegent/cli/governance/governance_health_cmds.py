"""Governance health and status commands facade (WL-124).

Re-exports split modules:
- governance_health_core_cmds: health/status/cycle/watch
- governance_policy_cmds: policy, contracts, migration, drift
- governance_audit_compliance_cmds: audit, compliance, guardrails
"""

from thegent.cli.governance.governance_health_core_cmds import (
    govern_configure_cmd,
    govern_go_health_cmd,
    govern_go_status_cmd,
    govern_go_cycle_cmd,
    govern_go_watch_cmd,
)
from thegent.cli.governance.governance_policy_cmds import (
    policy_show_cmd,
    policy_purge_cmd,
    contracts_registry_cmd,
    migration_cmd,
    drift_cmd,
    contracts_conformance_cmd,
    trust_status_cmd,
)
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

__all__ = [
    "govern_configure_cmd",
    "govern_go_health_cmd",
    "govern_go_status_cmd",
    "govern_go_cycle_cmd",
    "govern_go_watch_cmd",
    "policy_show_cmd",
    "policy_purge_cmd",
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
    "govern_cost_cmd",
    "guardrails_check_cmd",
    "guardrails_show_cmd",
]
