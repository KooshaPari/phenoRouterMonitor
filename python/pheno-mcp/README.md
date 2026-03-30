# pheno-mcp

Standalone MCP (Model Context Protocol) tooling package for Phenotype. Provides FastMCP wrappers, tool registry, and CrewAI agent orchestration - usable with any MCP server, not atoms-specific.

## Overview

`pheno-mcp` extracts and generalizes the MCP layer from the Phenotype ecosystem, enabling:

- **MCP Entry Point Configuration**: Server-agnostic entry point management
- **Tool Registry & Decorators**: FastMCP decorator abstraction for easy tool registration
- **Agent Orchestration**: CrewAI-compatible agent and task orchestration
- **Zero atoms-specific References**: Pure generic MCP tooling for maximum reusability

## Installation

### From GitHub Packages

```bash
pip install --index-url https://npm.pkg.github.com/ pheno-mcp
```

### From source (development)

```bash
git clone https://github.com/KooshaPari/phenotype-infrakit.git
cd phenotype-infrakit/python/pheno-mcp
pip install -e ".[dev]"
```

## Quick Start

### MCP Entry Point Configuration

```python
from pheno_mcp.mcp import MCPEntryPoint, MCPServer

# Configure entry point
entry_point = MCPEntryPoint(name="my-mcp-server", version="1.0.0")
entry_point.set_endpoint_url("http://localhost:8000")
entry_point.configure_auth("sk-your-api-key")

# Create server wrapper
server = MCPServer(entry_point)
server.start()
```

### Tool Registration with Decorators

```python
from pheno_mcp.tools import mcp_tool, tool_registry

# Create registry
registry = tool_registry.ToolRegistry()

# Register tools with @mcp_tool decorator
@mcp_tool(registry=registry, name="add_numbers")
def add(x: int, y: int) -> int:
    """Add two numbers together.

    Args:
        x: First number
        y: Second number

    Returns:
        Sum of x and y
    """
    return x + y

@mcp_tool(registry=registry, name="multiply_numbers")
def multiply(x: int, y: int) -> int:
    """Multiply two numbers.

    Args:
        x: First number
        y: Second number

    Returns:
        Product of x and y
    """
    return x * y

# List all registered tools
tools = registry.list_tools()
print(tools)  # ['add_numbers', 'multiply_numbers']
```

### Agent Orchestration

```python
from pheno_mcp.agents import Agent, AgentRole, TaskDefinition, AgentOrchestrator

# Create orchestrator
orchestrator = AgentOrchestrator()

# Create agents
manager = Agent(role=AgentRole.MANAGER, name="project_manager")
analyst = Agent(role=AgentRole.ANALYST, name="data_analyst")
worker = Agent(role=AgentRole.WORKER, name="executor")

# Add agents with tools
manager.add_tool("delegate")
analyst.add_tool("analyze")
worker.add_tool("execute")

orchestrator.add_agent(manager)
orchestrator.add_agent(analyst)
orchestrator.add_agent(worker)

# Define tasks
plan_task = TaskDefinition(
    name="plan",
    description="Plan the project",
    assigned_to=manager
)
analyze_task = TaskDefinition(
    name="analyze",
    description="Analyze requirements",
    assigned_to=analyst,
    depends_on=[plan_task]
)
execute_task = TaskDefinition(
    name="execute",
    description="Execute the plan",
    assigned_to=worker,
    depends_on=[analyze_task]
)

# Add tasks
orchestrator.add_task(plan_task)
orchestrator.add_task(analyze_task)
orchestrator.add_task(execute_task)

# Execute workflow
result = orchestrator.execute()
print(result)
```

## API Reference

### MCP Module

#### `MCPEntryPoint`

Server-agnostic MCP entry point configuration.

**Methods:**
- `set_endpoint_url(url: str)` - Set MCP server endpoint
- `configure_auth(api_key: str)` - Set authentication credentials
- `get_endpoint_url() -> Optional[str]` - Get configured endpoint
- `set_metadata(key: str, value: Any)` - Store arbitrary metadata
- `to_dict() -> Dict[str, Any]` - Serialize to dictionary

#### `MCPServer`

Wrapper for MCP server implementations.

**Methods:**
- `add_tool(tool_name: str, tool_callable: Any)` - Register a tool
- `get_tools() -> List[Dict]` - List registered tools
- `health_check() -> Dict[str, Any]` - Get server health status
- `start()` - Start the server
- `stop()` - Stop the server
- `is_running() -> bool` - Check if server is running

### Tools Module

#### `@mcp_tool` Decorator

Register a function as an MCP tool.

```python
@mcp_tool(registry=registry, name="my_tool", description="Tool description")
def my_tool(param: str) -> str:
    return f"processed: {param}"
```

#### `ToolRegistry`

Manage tool registration and discovery.

**Methods:**
- `register(name: str, callable_obj: Callable, **metadata)` - Register a tool
- `get_tool(name: str) -> Optional[Callable]` - Retrieve a tool
- `has_tool(name: str) -> bool` - Check if tool exists
- `list_tools() -> List[str]` - List all tool names
- `get_tool_info(name: str) -> Optional[Dict]` - Get tool metadata

### Agents Module

#### `Agent`

Represents an agent with a role and capabilities.

**Properties:**
- `role: AgentRole` - Agent's role
- `name: str` - Agent name
- `tools: List[str]` - Assigned tools

**Methods:**
- `add_tool(tool_name: str)` - Assign a tool
- `remove_tool(tool_name: str)` - Remove a tool
- `get_tools() -> List[str]` - Get assigned tools

#### `TaskDefinition`

Represents a task that can be assigned to agents.

**Properties:**
- `name: str` - Task name
- `description: str` - Task description
- `assigned_to: Optional[Agent]` - Assigned agent
- `depends_on: List[TaskDefinition]` - Task dependencies

#### `AgentOrchestrator`

Orchestrates agents and tasks in a workflow.

**Methods:**
- `add_agent(agent: Agent)` - Add agent to workflow
- `add_task(task: TaskDefinition)` - Add task to workflow
- `agents() -> List[Agent]` - Get all agents
- `tasks() -> List[TaskDefinition]` - Get all tasks
- `execute() -> Dict[str, Any]` - Execute the workflow
- `validate_workflow() -> Dict[str, Any]` - Validate configuration

## Architecture

```
pheno-mcp/
├── src/pheno_mcp/
│   ├── mcp/              # MCP entry points and server abstraction
│   │   ├── entry_points.py
│   │   └── __init__.py
│   ├── tools/            # Tool registry and decorators
│   │   ├── decorators.py
│   │   ├── tool_registry.py
│   │   └── __init__.py
│   ├── agents/           # Agent orchestration
│   │   ├── orchestration.py
│   │   └── __init__.py
│   └── __init__.py
├── tests/                # Test suite
├── pyproject.toml       # Package configuration
└── README.md
```

## Design Principles

- **Server-Agnostic**: Works with any MCP-compatible server
- **Zero Atoms-Specific Code**: Pure generic implementations
- **Modular**: Each component (MCP, Tools, Agents) is independent
- **Testable**: Comprehensive test coverage (80%+)
- **Type-Safe**: Full type annotations with mypy support

## Testing

Run the test suite:

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run with coverage
pytest --cov=src/pheno_mcp

# Run specific test file
pytest tests/test_mcp_entry_points.py
```

## Contributing

1. Create a feature branch: `git checkout -b feat/your-feature`
2. Write tests first (TDD mandate)
3. Implement the feature
4. Ensure all tests pass: `pytest`
5. Check coverage: `pytest --cov`
6. Format code: `black src/ tests/`
7. Run linter: `ruff check src/ tests/`
8. Commit with conventional messages: `git commit -m "feat: your feature description"`

## License

MIT License - See LICENSE file for details

## References

- [Model Context Protocol Spec](https://spec.modelcontextprotocol.io)
- [FastMCP Documentation](https://github.com/jmorganca/fastmcp)
- [CrewAI Documentation](https://github.com/joaomdmoura/crewai)
- [Phenotype Infrakit](https://github.com/KooshaPari/phenotype-infrakit)

## Support

For issues, questions, or contributions, please open an issue on the [GitHub repository](https://github.com/KooshaPari/phenotype-infrakit).
