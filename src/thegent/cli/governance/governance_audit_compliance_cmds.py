"""Governance audit, signatures, and compliance commands (WL-124).

Signed artifacts, audit, SIEM, plugin verification, and guardrails.
"""

from __future__ import annotations

import hashlib
import json
import sys
import uuid

import typer

from rich.table import Table

from thegent.cli.commands._cli_shared import (
    ThegentSettings,
    _load_artifact,
    _normalize_output_format,
    console,
)


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


__all__ = [
    "signatures_list_cmd",
    "signatures_verify_cmd",
    "compliance_siem_test_cmd",
    "compliance_plugin_check_cmd",
    "compliance_redact_cmd",
    "govern_cost_cmd",
    "guardrails_check_cmd",
    "guardrails_show_cmd",
]
