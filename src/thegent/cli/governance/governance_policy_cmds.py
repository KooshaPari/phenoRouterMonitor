"""Governance policy, contracts, and compliance commands (WL-124).

This module handles policy configuration, contract management, drift detection, and compliance verification.
"""

# @trace WL-124
from __future__ import annotations

import hashlib
import json
import sys
import uuid
from pathlib import Path

import typer

from rich.panel import Panel
from rich.table import Table

from thegent.cli.commands._cli_shared import (
    ThegentSettings,
    _load_artifact,
    _normalize_output_format,
    console,
)


def policy_show_cmd() -> None:
    """Show active governance policies and thresholds."""
    settings = ThegentSettings()
    console.print(f"[bold]Active Governance Policies[/bold] (Environment: [cyan]{settings.environment}[/cyan])")

    table = Table(show_header=True)
    table.add_column("Policy Name")
    table.add_column("Rule / Threshold")
    table.add_column("Status")

    table.add_row("Critical Confidence", ">= 0.9", "[green]Active[/green]")
    table.add_row(
        "Production Trust",
        f">= {settings.trust_score_threshold}",
        "[green]Active[/green]" if settings.environment == "production" else "[dim]Inactive[/dim]",
    )
    table.add_row("Agent Restriction", "Block 'unknown' in Prod/Critical", "[green]Active[/green]")
    table.add_row("Audit Signing", "SHA-256 Run Signatures", "[green]Active[/green]")
    table.add_row("Override TTL (WP-3003)", f"{settings.override_ttl_seconds}s", "[green]Active[/green]")

    console.print(table)


def policy_purge_cmd(dry_run: bool = True) -> None:
    """Purge expired history based on tiered retention (WP-3006)."""
    settings = ThegentSettings()
    from thegent.execution import RunRegistry

    registry = RunRegistry(settings.session_dir)
    res = registry.purge_expired(
        default_days=settings.retention_default_days,
        by_domain=settings.retention_by_domain,
        dry_run=dry_run,
    )
    if dry_run:
        console.print(f"[yellow]Dry run: would purge {res['purged']} records (kept {res['kept']}).[/yellow]")
    else:
        console.print(f"[green]Purged {res['purged']} records (kept {res['kept']}).[/green]")


def contracts_registry_cmd(format: str | None = None) -> None:
    """Show the contract registry and compatibility matrix."""
    from rich.console import Console
    from rich.table import Table

    from thegent.contracts.registry import get_registry

    registry = get_registry()
    versions = registry.list_versions()

    console = Console()
    if format == "json":
        # Handle both Pydantic models and mock/dict objects for testing
        data = []
        for v in versions:
            model_dump = getattr(v, "model_dump", None)
            if callable(model_dump):
                data.append(model_dump())
            elif hasattr(v, "__dict__"):
                data.append(v.__dict__)
            else:
                data.append(v)
        sys.stdout.write(json.dumps(data) + "\n")
        return

    table = Table(title="Contract Registry")
    table.add_column("Contract ID", style="cyan")
    table.add_column("Version", style="green")
    table.add_column("Description", style="white")
    table.add_column("Status", style="red")

    for v in sorted(versions, key=lambda x: (x.contract_id, x.version)):
        status = "[red]DEPRECATED[/red]" if v.deprecated else "[green]ACTIVE[/green]"
        if v.migration_window_end:
            status += f"\n[dim](ends {v.migration_window_end})[/dim]"
        table.add_row(v.contract_id, v.version, v.description, status)

    console.print(table)


def migration_cmd(contract_id: str, version: str, format: str | None = None) -> None:
    """Evaluate migration status for a contract version."""
    from rich.console import Console

    from thegent.contracts.migration import MigrationController

    console = Console()
    mc = MigrationController()
    res = mc.evaluate_version(contract_id, version)

    if format == "json":
        sys.stdout.write(json.dumps(res) + "\n")
        return

    color = "green" if res["allowed"] else "red"
    if res["status"] == "deprecated":
        color = "yellow"

    panel = Panel(
        f"[bold]Status:[/bold] {res['status'].upper()}\n"
        f"[bold]Allowed:[/bold] {'YES' if res['allowed'] else 'NO'}\n"
        f"[bold]Reason:[/bold] {res['reason']}\n"
        f"[bold]Days Left:[/bold] {res.get('migration_days_left', 'N/A')}",
        title=f"Migration Evaluation: {contract_id}@{version}",
        border_style=color,
    )
    console.print(panel)


def drift_cmd(
    window: int = 50,
    format: str | None = None,
    structural_budget: float = 5.0,
    semantic_budget: float = 10.0,
) -> None:
    """Detect significant drift in contract performance and check alert budgets (G-RV-07)."""
    from rich.console import Console

    from thegent.contracts.telemetry import ContractTelemetry

    settings = ThegentSettings()
    console = Console()
    ct = ContractTelemetry(settings.session_dir)
    issues = ct.detect_drift(window_size=window)
    budget = ct.get_drift_budget_status(
        structural_budget_pct=structural_budget,
        semantic_budget_pct=semantic_budget,
    )

    if format == "json":
        out = {"issues": issues, "budget": budget}
        sys.stdout.write(json.dumps(out) + "\n")
        return

    if not issues and budget["within_budget"]:
        console.print("[green]No significant contract drift detected.[/green]")
        return

    parts = []
    if issues:
        issue_str = "\n".join([f"[red]![/red] {i}" for i in issues])
        parts.append(issue_str)
    if not budget["within_budget"]:
        parts.append(
            f"[yellow]Budget exceeded:[/yellow] structural {budget['structural_rate_pct']}% "
            f"(budget {budget['structural_budget_pct']}%), semantic {budget['semantic_rate_pct']}% "
            f"(budget {budget['semantic_budget_pct']}%)"
        )
    if parts:
        panel = Panel(
            "\n\n".join(parts),
            title=f"Contract Drift Alert (window={window})",
            border_style="red",
        )
        console.print(panel)


def contracts_conformance_cmd(
    format: str | None = None,
    check_drift: bool = False,
    drift_window: int = 50,
) -> None:
    """Run provider adapter conformance tests."""
    from rich.console import Console
    from rich.table import Table

    from thegent.contracts.conformance import run_conformance_suite

    session_dir = ThegentSettings().session_dir if check_drift else None
    report = run_conformance_suite(session_dir=session_dir, drift_window=drift_window)
    console = Console()

    if format == "json":
        sys.stdout.write(json.dumps(report) + "\n")
        import typer

        if report.get("drift_issues") or report["failed"] > 0:
            raise typer.Exit(1)
        return

    table = Table(title=f"Adapter Conformance (Passed: {report['passed']}/{report['total']})")
    table.add_column("Test")
    table.add_column("Provider")
    table.add_column("Result")
    table.add_column("Conf")
    table.add_column("Issues")

    for r in report["results"]:
        status = "[green]PASS[/green]" if r["success"] else "[red]FAIL[/red]"
        issues = ", ".join(r["issues"]) if r["issues"] else "-"
        table.add_row(r["name"], r["provider"], status, f"{r['confidence']:.2f}", issues)

    console.print(table)

    if report.get("drift_checked") and report.get("drift_issues"):
        console.print()
        panel = Panel(
            "\n".join(f"[red]![/red] {i}" for i in report["drift_issues"]),
            title="Drift Alarms",
            border_style="red",
        )
        console.print(panel)

    if report["failed"] > 0 or report.get("drift_issues"):
        import typer

        raise typer.Exit(1)


def trust_status_cmd(format: str | None = None) -> None:
    """Show last environment and trust boundary status (WP-3007)."""
    settings = ThegentSettings()
    from thegent.execution import TrustBoundaryValidator

    trust_boundary = TrustBoundaryValidator(settings.session_dir)
    last_env = trust_boundary.get_last_environment()

    res = {
        "current_environment": settings.environment,
        "last_recorded_environment": last_env,
        "session_dir": str(settings.session_dir),
    }

    fmt = _normalize_output_format(format)
    if fmt == "json":
        sys.stdout.write(json.dumps(res) + "\n")
        return

    console.print("[bold]Trust Boundary Status (WP-3007)[/bold]")
    console.print(f"Current Env: [cyan]{settings.environment}[/cyan]")
    console.print(f"Last Env:    [cyan]{last_env or 'None'}[/cyan]")

    if last_env:
        allowed, reason = trust_boundary.validate_transition(last_env, settings.environment)
        status_color = "green" if allowed else "red"
        console.print(f"Transition:  [{status_color}]{reason}[/{status_color}]")


def signatures_list_cmd(limit: int = 50, format: str | None = None) -> None:
    """List signed MAIF artifacts (WP-3002)."""
    settings = ThegentSettings()
    artifacts_dir = settings.session_dir / "artifacts"

    artifacts = []
    if artifacts_dir.exists():
        for p in sorted(artifacts_dir.glob("maif.json"), key=lambda x: x.stat().st_mtime, reverse=True)[:limit]:
            _load_artifact(artifacts, p)

    fmt = _normalize_output_format(format)
    if fmt == "json":
        sys.stdout.write(json.dumps(artifacts) + "\n")
        return

    if not artifacts:
        console.print("[dim]No signed artifacts found.[/dim]")
        return

    table = Table(title="Signed Action Artifacts (MAIF v1.0)")
    table.add_column("Artifact ID")
    table.add_column("Root Hash (Short)")
    table.add_column("Blocks")
    table.add_column("Timestamp (us)")

    for a in artifacts:
        header = a.get("header", {})
        blocks = a.get("blocks", [])
        table.add_row(
            header.get("artifact_id", "?"),
            (header.get("root_hash") or "")[:12] + "...",
            str(len(blocks)),
            str(header.get("timestamp_us", "?")),
        )
    console.print(table)


def signatures_verify_cmd(run_id: str) -> None:
    """Verify a signed MAIF artifact (WP-3002)."""
    settings = ThegentSettings()
    artifact_path = settings.session_dir / "artifacts" / f"{run_id}.maif.json"

    if not artifact_path.exists():
        console.print(f"[red]Artifact not found for run_id={run_id}[/red]")
        raise typer.Exit(1)

    try:
        artifact_data = json.loads(artifact_path.read_text(encoding="utf-8"))
        header = artifact_data.get("header", {})
        blocks = artifact_data.get("blocks", [])
        chain = artifact_data.get("provenance_chain", [])

        console.print(f"[bold cyan]Verifying MAIF Artifact: {header.get('artifact_id')}[/bold cyan]")

        # 1. Verify Blocks
        all_blocks_valid = True
        for block in blocks:
            # Re-calculate hash (simplified for CLI check)
            payload = block.get("payload")
            body = json.dumps(payload, sort_keys=True, separators=(",", ":"))
            actual_hash = hashlib.sha256(body.encode()).hexdigest()

            if actual_hash != block.get("payload_hash"):
                console.print(f"  [red]✗ Block {block.get('block_id')} payload hash mismatch![/red]")
                all_blocks_valid = False
            else:
                console.print(f"  [green]✓ Block {block.get('block_id')} verified.[/green]")

        # 2. Verify Chain
        chain_valid = True
        if chain:
            prev_hash = "0" * 64
            for i, block in enumerate(blocks):
                link_data = f"{prev_hash}|{block.get('payload_hash')}"
                expected_link_hash = hashlib.sha256(link_data.encode()).hexdigest()
                if chain[i] != expected_link_hash:
                    console.print(f"  [red]✗ Provenance chain broken at block {i}![/red]")
                    chain_valid = False
                    break
                prev_hash = expected_link_hash

        # 3. Verify Root Hash
        root_valid = False
        if chain and chain[-1] == header.get("root_hash"):
            root_valid = True
            console.print(f"  [green]✓ Root hash {header.get('root_hash')[:12]}... matches chain.[/green]")
        else:
            console.print("  [red]✗ Root hash mismatch![/red]")

        if all_blocks_valid and chain_valid and root_valid:
            console.print(f"\n[bold green]RESULT: Artifact for {run_id} is VALID.[/bold green]")
        else:
            console.print(f"\n[bold red]RESULT: Artifact for {run_id} is INVALID.[/bold red]")
            raise typer.Exit(1)

    except Exception as e:
        console.print(f"[red]Failed to verify artifact: {e}[/red]")
        raise typer.Exit(1)


def compliance_siem_test_cmd(message: str, severity: str = "low") -> None:
    """Test SIEM event egress (WP-15001)."""
    from thegent.observability.egress import EgressEvent, SIEMEgress

    egress = SIEMEgress(endpoint_url="http://simulated-siem.internal")
    event = EgressEvent(
        id=str(uuid.uuid4()),
        severity=severity,
        event_type="test_event",
        source="thegent-cli",
        payload={"message": message},
    )

    success = egress.push_event(event)
    if success:
        console.print("[green]SIEM test event pushed successfully (simulated).[/green]")
        console.print(f"Format: {egress.format_for_syslog(event)}")
    else:
        console.print("[yellow]SIEM egress not configured or failed.[/yellow]")


def compliance_plugin_check_cmd(plugin_id: str, signature: str) -> None:
    """Verify a plugin contract (WP-15003)."""
    from thegent.contracts.marketplace import PluginContract, PluginVerifier

    verifier = PluginVerifier()
    contract = PluginContract(
        plugin_id=plugin_id,
        version="1.0.0",
        author="unknown",
        capabilities=["read"],
        signature=signature,
    )

    if verifier.verify_contract(contract):
        console.print(f"[green]Plugin {plugin_id} VERIFIED successfully.[/green]")
    else:
        console.print(f"[red]Plugin {plugin_id} verification FAILED. Invalid signature.[/red]")


def compliance_redact_cmd(text: str) -> None:
    """Test PII/Secret redaction (WP-15005)."""
    from thegent.governance.support import SupportRedactor

    redactor = SupportRedactor()
    redacted = redactor.redact_text(text)

    console.print("[bold]Original:[/bold]")
    console.print(text)
    console.print("\n[bold]Redacted:[/bold]")
    console.print(redacted)


def govern_cost_cmd(owner: str | None = None, days: int = 1, format: str | None = None) -> None:
    """Show daily cost aggregation (FR-GOV-002)."""
    settings = ThegentSettings()
    from thegent.cost.aggregator import CostAggregator

    agg = CostAggregator(settings.session_dir)
    total = agg.daily_total(owner=owner, days=days)

    res = {
        "owner": owner or "all",
        "days": days,
        "total_usd": total,
        "currency": "USD",
    }

    fmt = _normalize_output_format(format)
    if fmt == "json":
        sys.stdout.write(json.dumps(res) + "\n")
        return

    console.print("[bold]Daily Cost Aggregation (FR-GOV-002)[/bold]")
    console.print(f"Owner: [cyan]{owner or 'All Owners'}[/cyan]")
    console.print(f"Days:  [cyan]{days}[/cyan]")
    console.print(f"Total: [green]${total:.4f} USD[/green]")


def guardrails_check_cmd(prompt: str, agent: str | None = None, model: str | None = None) -> None:
    """Check a prompt against active guardrails (FR-GOV-003..006)."""
    from thegent.governance.input_guardrails import InputGuardrails

    rails = InputGuardrails()
    result = rails.check(prompt, agent=agent or "", model=model)

    if result.passed:
        console.print("[green]Prompt passed all guardrails.[/green]")
        return

    console.print("[red]Prompt FAILED guardrail checks:[/red]")
    console.print(f"- [bold]{result.rail_id}[/bold]: {result.reason}")
    if result.remediation:
        console.print(f"  [dim]Remediation: {result.remediation}[/dim]")


def guardrails_show_cmd() -> None:
    """Show active guardrail configuration (FR-GOV-007)."""
    from thegent.governance.input_guardrails import InputGuardrails

    rails = InputGuardrails()

    table = Table(title="Input Guardrails Configuration")
    table.add_column("Parameter")
    table.add_column("Value")

    table.add_row("Max Chars", str(rails.prompt_max_chars))
    table.add_row("Blocklist Patterns", str(len(rails.prompt_blocklist_patterns)))
    table.add_row("Agent Allowlist", ", ".join(rails.agent_allowlist) if rails.agent_allowlist else "None")
    table.add_row("Model Allowlist", ", ".join(rails.model_allowlist) if rails.model_allowlist else "None")
    table.add_row(
        "CWD Allowed Prefixes", ", ".join(rails.cwd_allowed_prefixes) if rails.cwd_allowed_prefixes else "None"
    )

    console.print(table)


def policy_check_cmd(agent: str, model: str | None = None, lane: str = "standard", confidence: float = 1.0) -> None:
    """Evaluate a hypothetical run against governance policies (WP-3001)."""
    settings = ThegentSettings()
    from thegent.execution import PolicyEngine, RunMeta, RunRegistry

    engine = PolicyEngine(settings)
    registry = RunRegistry(settings.session_dir)

    # Create a mock RunMeta
    run = RunMeta(
        run_id="policy-check-" + str(uuid.uuid4())[:8],
        correlation_id="check",
        agent=agent,
        model=model or "default",
        mode="write",
        prompt="test prompt",
        cwd=str(Path.cwd()),
        owner="operator",
        lane=lane,
        confidence=confidence,
    )

    result, reason = engine.evaluate(run, registry=registry)

    color = "green" if result == "allow" else "yellow" if result == "warn" else "red"
    console.print(f"Policy Result: [{color}]{result.upper()}[/{color}]")
    console.print(f"Reason: {reason}")


__all__ = [
    "compliance_plugin_check_cmd",
    "compliance_redact_cmd",
    "compliance_siem_test_cmd",
    "contracts_conformance_cmd",
    "contracts_registry_cmd",
    "drift_cmd",
    "govern_cost_cmd",
    "guardrails_check_cmd",
    "guardrails_show_cmd",
    "migration_cmd",
    "policy_check_cmd",
    "policy_purge_cmd",
    "policy_show_cmd",
    "signatures_list_cmd",
    "signatures_verify_cmd",
    "trust_status_cmd",
]
