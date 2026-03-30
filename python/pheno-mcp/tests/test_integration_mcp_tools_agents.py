"""Integration tests for pheno-mcp with MCP, Tools, and Agents together.

Traces to: FR-MCP-004 - End-to-End Integration with Mock MCP Server
"""

import pytest
from pheno_mcp.mcp import MCPEntryPoint, MCPServer
from pheno_mcp.tools import mcp_tool, tool_registry
from pheno_mcp.agents import Agent, AgentRole, TaskDefinition, AgentOrchestrator


class TestMCPToolsIntegration:
    """Integration tests combining MCP server with registered tools."""

    def test_server_with_registered_tools(self):
        """Test MCP server with registered tools."""
        # Create server
        entry_point = MCPEntryPoint(name="tool-server")
        entry_point.set_endpoint_url("http://localhost:8000")
        server = MCPServer(entry_point)

        # Create tool registry
        registry = tool_registry.ToolRegistry()

        # Register some tools
        @mcp_tool(registry=registry, name="process_data")
        def process(data: str) -> str:
            return f"processed: {data}"

        @mcp_tool(registry=registry, name="validate_input")
        def validate(value: str) -> bool:
            return len(value) > 0

        # Add tools to server
        for tool_name in registry.list_tools():
            tool = registry.get_tool(tool_name)
            server.add_tool(tool_name, tool)

        # Verify
        assert len(server.get_tools()) == 2
        tools = {t["name"]: t["callable"] for t in server.get_tools()}
        assert "process_data" in tools
        assert "validate_input" in tools

    def test_tool_execution_through_server(self):
        """Test executing tools registered with server."""
        entry_point = MCPEntryPoint(name="exec-server")
        server = MCPServer(entry_point)

        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="add")
        def add_numbers(x: int, y: int) -> int:
            return x + y

        tool = registry.get_tool("add")
        server.add_tool("add", tool)

        # Execute through server
        result = server.get_tools()[0]["callable"](3, 4)
        assert result == 7


class TestAgentToolsIntegration:
    """Integration tests combining agents with tools."""

    def test_agent_with_mcp_tools(self):
        """Test agent using MCP-registered tools."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="analyze_data")
        def analyze(data: str) -> dict:
            return {"status": "analyzed", "data": data}

        @mcp_tool(registry=registry, name="generate_report")
        def report(analysis: dict) -> str:
            return f"Report: {analysis['status']}"

        # Create agent with tools
        analyst = Agent(
            role=AgentRole.ANALYST,
            name="data_analyst",
            tools=registry.list_tools()
        )

        assert len(analyst.get_tools()) == 2
        assert "analyze_data" in analyst.get_tools()

    def test_orchestrator_with_tool_bound_agents(self):
        """Test orchestrator with agents that have tools."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="execute_task")
        def execute(task: str) -> str:
            return f"completed: {task}"

        @mcp_tool(registry=registry, name="log_result")
        def log(result: str) -> None:
            pass

        orchestrator = AgentOrchestrator()

        executor = Agent(
            role=AgentRole.WORKER,
            name="executor",
            tools=["execute_task", "log_result"]
        )
        orchestrator.add_agent(executor)

        task = TaskDefinition(
            name="work",
            description="Do work",
            assigned_to=executor
        )
        orchestrator.add_task(task)

        result = orchestrator.execute()
        assert result["agents_count"] == 1
        assert result["tasks_count"] == 1
        assert result["status"] == "completed"


class TestCompleteEndToEndWorkflow:
    """End-to-end workflow tests with MCP, Tools, and Agents."""

    def test_complete_workflow_with_mock_mcp_server(self):
        """Test complete workflow: Server -> Tools -> Agents -> Execution."""
        # Setup MCP Server
        ep = MCPEntryPoint(name="workflow-server", version="1.0.0")
        ep.set_endpoint_url("http://localhost:9000")
        server = MCPServer(ep)

        # Setup Tool Registry
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="fetch_data")
        def fetch(source: str) -> list:
            return [f"item_{i}" for i in range(3)]

        @mcp_tool(registry=registry, name="process_items")
        def process(items: list) -> list:
            return [f"processed_{item}" for item in items]

        @mcp_tool(registry=registry, name="generate_output")
        def generate(items: list) -> str:
            return f"output: {len(items)} items"

        # Register tools with server
        for tool_name in registry.list_tools():
            tool = registry.get_tool(tool_name)
            server.add_tool(tool_name, tool)

        # Setup Agents
        orchestrator = AgentOrchestrator()

        fetcher = Agent(
            role=AgentRole.WORKER,
            name="data_fetcher",
            tools=["fetch_data"]
        )
        processor = Agent(
            role=AgentRole.WORKER,
            name="data_processor",
            tools=["process_items"]
        )
        generator = Agent(
            role=AgentRole.ANALYST,
            name="report_generator",
            tools=["generate_output"]
        )

        orchestrator.add_agent(fetcher)
        orchestrator.add_agent(processor)
        orchestrator.add_agent(generator)

        # Setup Tasks
        fetch_task = TaskDefinition(
            name="fetch",
            description="Fetch data",
            assigned_to=fetcher
        )
        process_task = TaskDefinition(
            name="process",
            description="Process data",
            assigned_to=processor,
            depends_on=[fetch_task]
        )
        report_task = TaskDefinition(
            name="report",
            description="Generate report",
            assigned_to=generator,
            depends_on=[process_task]
        )

        orchestrator.add_task(fetch_task)
        orchestrator.add_task(process_task)
        orchestrator.add_task(report_task)

        # Verify setup
        assert server.entry_point.get_endpoint_url() == "http://localhost:9000"
        assert len(server.get_tools()) == 3
        assert len(orchestrator.agents()) == 3
        assert len(orchestrator.tasks()) == 3

        # Execute
        result = orchestrator.execute()
        assert result["status"] == "completed"
        assert result["agents_count"] == 3
        assert result["tasks_count"] == 3
        assert len(result["results"]) == 3

    def test_workflow_with_multiple_tool_registries(self):
        """Test workflow with isolated tool registries per agent type."""
        # Create separate registries for different agent types
        analyst_registry = tool_registry.ToolRegistry()
        worker_registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=analyst_registry, name="analyze")
        def analyze(data: str) -> dict:
            return {"analysis": data}

        @mcp_tool(registry=worker_registry, name="execute")
        def execute(task: str) -> str:
            return f"done: {task}"

        # Setup orchestrator
        orchestrator = AgentOrchestrator()

        analyst = Agent(
            role=AgentRole.ANALYST,
            name="analyst",
            tools=analyst_registry.list_tools()
        )
        worker = Agent(
            role=AgentRole.WORKER,
            name="worker",
            tools=worker_registry.list_tools()
        )

        orchestrator.add_agent(analyst)
        orchestrator.add_agent(worker)

        # Verify isolation
        assert "analyze" in analyst.get_tools()
        assert "execute" in worker.get_tools()
        assert "analyze" not in worker.get_tools()
        assert "execute" not in analyst.get_tools()

    def test_server_health_with_active_tools_and_agents(self):
        """Test server health check with active configuration."""
        ep = MCPEntryPoint(name="health-test", version="2.0.0")
        ep.set_endpoint_url("http://localhost:9001")
        server = MCPServer(ep)

        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="health_check")
        def health():
            return "healthy"

        for tool_name in registry.list_tools():
            server.add_tool(tool_name, registry.get_tool(tool_name))

        health_status = server.health_check()
        assert health_status["status"] == "healthy"
        assert health_status["name"] == "health-test"
        assert health_status["version"] == "2.0.0"
        assert health_status["tools_registered"] == 1

    def test_workflow_validation_with_tool_binding(self):
        """Test workflow validation including tool binding checks."""
        orchestrator = AgentOrchestrator()
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="task_tool")
        def task_tool():
            return "result"

        agent = Agent(
            role=AgentRole.WORKER,
            name="worker",
            tools=["task_tool"]
        )
        orchestrator.add_agent(agent)

        task = TaskDefinition(
            name="task",
            description="A task",
            assigned_to=agent
        )
        orchestrator.add_task(task)

        validation = orchestrator.validate_workflow()
        assert validation["valid"] is True
        assert len(validation["errors"]) == 0


class TestErrorHandling:
    """Tests for error handling in integrated workflows."""

    def test_missing_tool_handling(self):
        """Test handling of missing tools in registry."""
        registry = tool_registry.ToolRegistry()

        @mcp_tool(registry=registry, name="existing_tool")
        def existing():
            return "exists"

        # Try to get non-existent tool
        missing = registry.get_tool("nonexistent")
        assert missing is None

        # Existing tool should work
        tool = registry.get_tool("existing_tool")
        assert tool is not None
        assert tool() == "exists"

    def test_unassigned_task_warning(self):
        """Test detection of unassigned tasks."""
        orchestrator = AgentOrchestrator()

        # Add task without assigning to agent
        task = TaskDefinition(name="orphan", description="No agent")
        orchestrator.add_task(task)

        validation = orchestrator.validate_workflow()
        assert len(validation["warnings"]) > 0
        assert any("orphan" in w for w in validation["warnings"])

    def test_server_start_without_endpoint(self):
        """Test that server requires endpoint before starting."""
        ep = MCPEntryPoint(name="no-endpoint")
        server = MCPServer(ep)

        # Should raise ValueError
        with pytest.raises(ValueError):
            server.start()
