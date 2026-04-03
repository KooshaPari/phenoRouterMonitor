# Hexagon Template Registry

Unified hexagonal architecture templates for multiple languages.

## Templates

| Language | Template | Status |
|----------|----------|--------|
| Go | `templates/go/` | Production-ready |
| Rust | `templates/rust/` | Production-ready |
| Zig | `templates/zig/` | Production-ready |
| C# | `templates/csharp/` | Planned |
| Elixir | `templates/elixir/` | Planned |
| Java | `templates/java/` | Planned |
| Kotlin | `templates/kotlin/` | Planned |
| Swift | `templates/swift/` | Planned |

## Usage

```bash
# Copy template for your language
cp -r hexagon/templates/go ./my-project
cd my-project && ./scripts/init.sh
```

## Migration from Individual Repos

Previous separate repositories have been consolidated:
- hexagon-go → templates/go/
- hexagon-rust → templates/rust/
- hexagon-zig → templates/zig/

See individual template READMEs for architecture details.
