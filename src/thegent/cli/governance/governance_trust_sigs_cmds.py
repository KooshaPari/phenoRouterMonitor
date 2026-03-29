"""Governance policy, contracts, and compliance commands (WL-124).

This module handles policy configuration, contract management, drift detection, and compliance verification.
"""

# @trace WL-124
from __future__ import annotations

import hashlib
import json
import sys

import typer

from rich.table import Table

from thegent.cli.commands._cli_shared import (
    ThegentSettings,
    _load_artifact,
    _normalize_output_format,
    console,
)


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


__all__ = [
    "trust_status_cmd",
    "signatures_list_cmd",
    "signatures_verify_cmd",
]
