# Phenotype.Packs

Generic content pack system with dependency resolution, schema validation, and conflict detection. Extracted from [DINOForge](https://github.com/kooshapari/Dino).

## Features

- **Generic Design** - Works with any manifest and definition types via type parameters
- **Dependency Resolution** - Semantic versioning with topological sorting
- **Conflict Detection** - Automatic detection of pack ID/name collisions
- **YAML Support** - Built-in YAML manifest loading via YamlDotNet
- **Validation** - Extensible manifest validation framework
- **Async/Await** - Fully asynchronous API
- **Event-Driven** - Load/unload events for monitoring

## Installation

```bash
dotnet add package Phenotype.Packs
```

## Quick Start

### 1. Define Your Manifest

```csharp
public class GamePackManifest : IPackManifest
{
    public string Id { get; set; } = "";
    public string Name { get; set; } = "";
    public string Version { get; set; } = "1.0.0";
    public string Author { get; set; } = "";
    public IReadOnlyList<IPackDependency> Dependencies { get; set; } = new List<IPackDependency>();
}
```

### 2. Define Your Content

```csharp
public class UnitDefinition : IPackDefinition
{
    public string DefinitionType => "unit";
    public string SourcePath { get; set; } = "";
    public string UnitName { get; set; } = "";
    public int Health { get; set; }
}
```

### 3. Create and Use the Pack System

```csharp
// Create the pack system
var packSystem = new ContentPackSystem<GamePackManifest, UnitDefinition>();

// Subscribe to events
packSystem.PackLoaded += (sender, e) =>
    Console.WriteLine($"Loaded: {e.PackId}");

// Load a pack
var context = new PackContext<GamePackManifest, UnitDefinition>
{
    PackPath = "/path/to/pack",
    Manifest = manifest,
    Definitions = new List<UnitDefinition> { unit1, unit2 }
};

var result = await packSystem.LoadPackAsync(
    new PackIdentifier("my-pack", "1.0.0"),
    context);

if (result.Success)
{
    Console.WriteLine($"Loaded in {result.LoadDuration.TotalMilliseconds}ms");
}
```

### 4. Resolve Dependencies

```csharp
var manifests = new[] { packA, packB, packC };
var resolution = await packSystem.ResolveDependenciesAsync(manifests);

if (resolution.Success)
{
    foreach (var pack in resolution.ResolvedOrder)
    {
        Console.WriteLine($"Load order: {pack}");
    }
}
```

## Architecture

```
Phenotype.Packs/
├── IContentPackSystem<TManifest, TDefinition>  # Core interface
├── ContentPackSystem                           # Production implementation
├── Discovery/
│   └── FileSystemPackDiscovery               # File system discovery
├── Loading/
│   └── YamlPackLoader                        # YAML manifest loading
├── Resolution/
│   └── SemverDependencyResolver              # Dependency resolution
└── Validation/
    └── DefaultManifestValidator              # Manifest validation
```

## Advanced Usage

### Custom Discovery

```csharp
public class HttpPackDiscovery : IPackDiscovery
{
    public async Task<IReadOnlyList<PackDiscoveryResult>> DiscoverPacksAsync(
        string url,
        DiscoveryOptions? options = null,
        CancellationToken ct = default)
    {
        // Fetch manifests from HTTP endpoint
    }
}
```

### Custom Validation

```csharp
public class StrictManifestValidator : IManifestValidator<GamePackManifest>
{
    public Task<ValidationResult> ValidateAsync(
        GamePackManifest manifest,
        CancellationToken ct = default)
    {
        var errors = new List<string>();

        if (manifest.Name.Length < 3)
            errors.Add("Pack name must be at least 3 characters");

        return Task.FromResult(errors.Any()
            ? ValidationResult.Failure(errors)
            : ValidationResult.Success());
    }
}
```

## Extracted From DINOForge

This library was extracted from [DINOForge](https://github.com/kooshapari/Dino), a mod platform for *Diplomacy is Not an Option*. The original code handled:

- 200+ content packs
- Complex dependency chains
- Hot reload during development
- Asset replacement and swapping
- Total conversion mods

The generic abstraction removes Unity-specific dependencies while preserving the battle-tested core.

## License

MIT License - See [LICENSE](LICENSE) for details.

## Related

- [DINOForge](https://github.com/kooshapari/Dino) - Original mod platform
- [Phenotype.Validation](../phenotype-validation) - Schema validation component
- [Phenotype.Registry](../phenotype-registry) - Typed registry component
