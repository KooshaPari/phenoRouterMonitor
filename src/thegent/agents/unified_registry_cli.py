"""CLI commands for Unified Agent Registry."""

import typer
from rich.console import Console
from rich.table import Table

from thegent.agents.unified_registry import Agent, AgentCapability, AgentRegistryService, AgentStatus, ProjectAssignment

app = typer.Typer(help="Manage unified agent registry")
console = Console()

# In-memory service for now (should be singleton or use persistent storage)
_service = AgentRegistryService()


@app.command("list")
def list_agents(
    status: AgentStatus | None = typer.Option(None, "--status", help="Filter by status"),
    project: str | None = typer.Option(None, "--project", help="Filter by project ID"),
    capability: AgentCapability | None = typer.Option(None, "--capability", help="Filter by capability"),
):
    """List all agents in the registry."""
    agents = _service.list_agents(status=status, project_id=project, capability=capability)

    if not agents:
        console.print("[yellow]No agents found matching criteria.[/yellow]")
        return

    table = Table(title="Unified Agent Registry")
    table.add_column("ID", style="cyan")
    table.add_column("Name", style="green")
    table.add_column("Status", style="magenta")
    table.add_column("Capabilities", style="yellow")
    table.add_column("Projects", style="blue")

    for agent in agents:
        caps = ", ".join([c.value for c in agent.capabilities])
        projs = ", ".join([p.project_id for p in agent.projects])
        table.add_row(agent.id, agent.name, agent.status.value, caps, projs)

    console.print(table)


@app.command("show")
def get_agent(agent_id: str = typer.Argument(..., help="Agent ID")):
    """Show details for a specific agent."""
    agent = _service.get_agent(agent_id)
    if not agent:
        console.print(f"[red]Agent '{agent_id}' not found.[/red]")
        return

    console.print(f"[bold cyan]Agent: {agent.name} ({agent.id})[/bold cyan]")
    console.print(f"Description: {agent.description or 'N/A'}")
    console.print(f"Status: {agent.status.value}")
    console.print(f"Version: {agent.version}")
    console.print(f"Capabilities: {', '.join([c.value for c in agent.capabilities])}")
    console.print(f"Tools: {', '.join(agent.tools)}")
    console.print(f"Models: {', '.join(agent.models)}")

    if agent.projects:
        console.print("\n[bold blue]Project Assignments:[/bold blue]")
        for p in agent.projects:
            console.print(f"  - {p.project_id} (Role: {p.role})")

    console.print("\n[bold green]Performance Metrics:[/bold blue]")
    console.print(f"  Success Rate: {agent.metrics.success_rate:.1%}")
    console.print(f"  Tasks: {agent.metrics.completed_tasks}/{agent.metrics.total_tasks}")
    console.print(f"  Avg Response Time: {agent.metrics.average_response_time:.2f}s")


@app.command("register")
def register_agent(
    agent_id: str = typer.Argument(..., help="Unique agent ID"),
    name: str = typer.Argument(..., help="Display name"),
    capabilities: list[AgentCapability] = typer.Option([], "--capability", "-c", help="Agent capabilities"),
):
    """Register a new agent."""
    agent = Agent(id=agent_id, name=name, capabilities=capabilities, status=AgentStatus.ACTIVE)
    _service.register_agent(agent)
    console.print(f"[green]Agent '{name}' ({agent_id}) registered successfully.[/green]")


@app.command("assign")
def assign_project(
    agent_id: str = typer.Argument(..., help="Agent ID"),
    project_id: str = typer.Argument(..., help="Project ID"),
    role: str = typer.Option("contributor", "--role", help="Role in project"),
):
    """Assign agent to a project."""
    assignment = ProjectAssignment(project_id=project_id, role=role)
    agent = _service.assign_to_project(agent_id, assignment)
    if agent:
        console.print(f"[green]Agent '{agent_id}' assigned to project '{project_id}' as {role}.[/green]")
    else:
        console.print(f"[red]Agent '{agent_id}' not found.[/red]")


@app.command("discover")
def discover(
    description: str = typer.Argument(..., help="Task description"),
    capabilities: list[AgentCapability] = typer.Option(..., "--capability", "-c", help="Required capabilities"),
    project: str | None = typer.Option(None, "--project", help="Project context"),
):
    """Discover best agent for a task."""
    agent = _service.discover_best_agent(description, capabilities, project_id=project)
    if agent:
        console.print(f"[green]Best agent found: [bold]{agent.name}[/bold] ({agent.id})[/green]")
        console.print(f"Success Rate: {agent.metrics.success_rate:.1%}")
    else:
        console.print("[yellow]No suitable agent found.[/yellow]")
