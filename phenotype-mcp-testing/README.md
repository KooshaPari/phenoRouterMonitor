# phenotype-mcp-testing

> MCP Server for Game Testing and Validation

This MCP server provides tools for testing and validating games and applications. It handles:

- Game launching and management
- Automated testing scenarios
- Performance profiling
- State validation and assertions
- Screenshot/comparison testing
- Integration with CI/CD pipelines

## Tools

### Game Management

- `launch_game` - Launch a game with specified parameters
- `terminate_game` - Terminate a running game
- `get_game_status` - Get current game status
- `wait_for_world` - Wait for game world to be ready

### Testing & Validation

- `run_test_scenario` - Execute an automated test scenario
- `validate_state` - Validate game state against expected values
- `capture_screenshot` - Capture a screenshot for comparison
- `assert_condition` - Assert a condition is met

### Performance & Debugging

- `profile_performance` - Profile game performance
- `get_memory_usage` - Get current memory usage
- `analyze_frame_times` - Analyze frame timing data

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    phenotype-mcp-testing                               │
│                                                                       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │ Game Management  │  │ Testing & Valid. │  │ Performance      │  │
│  │                  │  │                  │  │                  │  │
│  │ - launch_game    │  │ - run_test       │  │ - profile_perf   │  │
│  │ - terminate      │  │ - validate_state │  │ - get_memory     │  │
│  │ - get_status     │  │ - capture_screen │  │ - analyze_frames │  │
│  │ - wait_for_world │  │ - assert_cond    │  │                  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                    NanoVMS Integration                            │ │
│  │                                                                   │ │
│  │  - Tier 1: WASM Sandboxes (~1ms)                                 │ │
│  │  - Tier 2: gVisor Containers (~90ms)                             │ │
│  │  - Tier 3: Firecracker MicroVMs (~125ms)                        │ │
│  │                                                                   │ │
│  └──────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────┘
```

## Installation

```bash
cargo install phenotype-mcp-testing
```

## Usage

```json
{
  "mcpServers": {
    "phenotype-testing": {
      "command": "phenotype-mcp-testing",
      "args": ["--workspace", "./test-workspace"]
    }
  }
}
```

## Integration with CI/CD

```yaml
# .github/workflows/game-tests.yml
- name: Run Game Tests
  run: |
    phenotype-mcp-testing run-tests \
      --scenario-file tests/smoke.json \
      --output results/
```

## License

MIT
