"""Governance domain subpackage for CLI.

Phase 2 of CLI decomposition: extracted governance, compliance, policy, and
audit-related commands and services from the CLI god package.

Modules:
- governance_cmds: Main governance command aggregator
- governance_policy_cmds: Policy-related commands
- governance_health_cmds: Health assessment and monitoring
- governance_compliance_cmds: Compliance and guardrails
- governance_audit_compliance_cmds: Audit and compliance tracking
- governance_escalation_hitl_cmds: Human-in-the-loop escalations
- governance_discovery_guardrails_cmds: Discovery guardrails
- governance_compliance_guardrails_cmds: Compliance guardrails
- governance_data_protection_cmds: Data protection commands
- governance_agileplus_cmds: Agile+ governance extensions
- governance_trust_sigs_cmds: Trust signatures
- governance_policy_core_cmds: Core policy implementations
- governance_policy_contracts_cmds: Policy contracts
- governance_policy_health_cmds: Policy health assessment
- governance_health_core_cmds: Core health implementations
- governance_health_helpers: Health assessment helpers
- cli_git_worktree_governance: Git worktree governance
- governance: Services and utilities
"""

from __future__ import annotations
