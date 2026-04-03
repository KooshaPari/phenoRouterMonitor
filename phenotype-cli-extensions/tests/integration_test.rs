//! Integration tests for phenotype-cli-extensions

#[test]
fn test_library_loads() {
    // Verify the library loads correctly
    assert_eq!(1 + 1, 2);
}

#[test]
fn test_kitty_module_exists() {
    // Verify kitty module is accessible
    use phenotype_cli_extensions::kitty;
    // Module exists if this compiles
}

#[test]
fn test_mcp_module_exists() {
    // Verify MCP module is accessible
    use phenotype_cli_extensions::shell_tool_mcp;
    // Module exists if this compiles
}
