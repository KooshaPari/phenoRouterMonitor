# Phenotype.Packs

[![NuGet](https://img.shields.io/nuget/v/Phenotype.Packs.svg)](https://www.nuget.org/packages/Phenotype.Packs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A **generic content pack system** extracted from [DINOForge](https://github.com/dinoforge/DINOForge), battle-tested in production with 1,017+ tests and real-world modding scenarios.

## What is This?

Phenotype.Packs provides a **complete, reusable content pack infrastructure** for any application that needs:

- 🎒 **Content Pack Loading** - Load packs from directories with YAML/JSON manifests
- 🔗 **Dependency Resolution** - Automatic dependency ordering with SemVer support
- ⚠️ **Conflict Detection** - Detect duplicate IDs, overrides, and version mismatches
- ✅ **Validation** - Schema validation for manifests and content definitions
- 📦 **Registry Management** - Type-safe registries with layered override priority

## Quick Start

```bash
dotnet add package Phenotype.Packs
```

### Basic Usage

```csharp
using Phenotype.Packs;

// Define your manifest type
public class GamePackManifest : IPackManifest
{
    public string Id { get; set; } = "";
    public string Name { get; set; } = "";
    public string Version { get; set; } = "0.1.0";
    public string FrameworkVersion { get; set; } = ">=1.0.0";
    public string Author { get; set; } = "";
    public string Type { get; set; } = "content";
    public string? Description { get; set; }
    public IReadOnlyList<IPackDependency> Dependencies { get; set; } = new List<IPackDependency>();
    public IReadOnlyList<string> Conflicts { get; set; } = new List<string>();
}

// Define your content definition type
public class UnitDefinition : IContentDefinition
{
    public string Id { get; set; } = "";
    public string DefinitionType => "unit";
    public string SourcePackId { get; set; } = "";
    public bool IsOverride { get; set; }
    public string UnitType { get; set; } = "";
    public int Health { get; set; } = 100;
    public int Attack { get; set; } = 10;
}

// Create and use the pack system
var packSystem = new ContentPackSystem<GamePackManifest, UnitDefinition>();

// Subscribe to events
packSystem.PackLoaded += (sender, e) =>
    Console.WriteLine($"Loaded pack: {e.Pack.Manifest.Name}");

// Load a pack
var result = packSystem.LoadPack("/path/to/mymod");

if (result.Success)
{
    Console.WriteLine($"Loaded {result.Definitions.Count} units");
}
else
{
    foreach (var error in result.Errors)
        Console.WriteLine($"Error: {error}");
}
```

### Pack Manifest Format (YAML)

```yaml
# pack.yaml
id: my-awesome-mod
name: My Awesome Mod
version: 1.2.0
framework_version: ">=0.5.0"
author: John Doe
type: content
description: Adds new units and buildings

dependencies:
  - pack_id: core-framework
    version: ">=1.0.0"
  - pack_id: optional-helper
    version: ">=0.5.0"
    optional: true

conflicts:
  - incompatible-mod
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│               Your Application                          │
│  ┌─────────────────────────────────────────────────┐  │
│  │           ContentPackSystem                       │  │
│  │             <TManifest, TDef>                     │  │
│  └────────────────┬────────────────────────────────┘  │
│                   │                                    │
│  ┌────────────────┼────────────────────────────────┐  │
│  │                ▼ Services                       │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐  │  │
│  │  │Discovery │ │ Loading  │ │ Resolution   │  │  │
│  │  │Service   │ │ Service  │ │ Service      │  │  │
│  │  └──────────┘ └──────────┘ └──────────────┘  │  │
│  │  ┌──────────┐ ┌──────────┐                   │  │
│  │  │Validation│ │ Conflict │                   │  │
│  │  │Service   │ │ Detector │                   │  │
│  │  └──────────┘ └──────────┘                   │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Features

### 1. Generic Type System

The library is fully generic over your manifest and definition types:

```csharp
IContentPackSystem<TManifest, TDefinition>
    where TManifest : IPackManifest
    where TDefinition : IContentDefinition
```

This means you can use it for:
- **Game mods** (units, buildings, items)
- **Configuration packs** (settings, presets)
- **Asset bundles** (textures, sounds, models)
- **Any domain** requiring structured content loading

### 2. Dependency Resolution

Automatic topological sorting with SemVer constraint checking:

```csharp
var graph = packSystem.ResolveDependencies(manifests);

foreach (var packId in graph.LoadOrder)
    Console.WriteLine($"Load: {packId}");

foreach (var conflict in graph.VersionConflicts)
    Console.WriteLine($"Conflict: {conflict.PackId} requires {conflict.DependencyId} {conflict.RequiredVersion}, found {conflict.FoundVersion}");
```

### 3. Conflict Detection

Detects and reports:
- **Duplicate IDs** - Multiple definitions with same ID
- **Override Conflicts** - Base definition not found for override
- **Version Mismatches** - Dependencies not satisfied

### 4. Validation Pipeline

Built-in validation for:
- Required fields (ID, version)
- Identifier format (lowercase, hyphens)
- Semantic versioning
- Dependency completeness

### 5. Event-Driven Architecture

```csharp
packSystem.PackLoaded += (s, e) => { };
packSystem.PackLoadFailed += (s, e) => { };
```

## Advanced Usage

### Custom Discovery Service

```csharp
public class ZipPackDiscovery : IPackDiscoveryService
{
    public TManifest? DiscoverManifest<TManifest>(string path) where TManifest : class, IPackManifest, new()
    {
        // Load from .zip files
    }
}

var packSystem = new ContentPackSystem<GamePackManifest, UnitDefinition>(
    discoveryService: new ZipPackDiscovery()
);
```

### Batch Loading with Conflicts

```csharp
var paths = Directory.GetDirectories("./mods");
var result = packSystem.LoadPacks(paths);

if (result.ConflictReport?.HasConflicts == true)
{
    foreach (var duplicate in result.ConflictReport.DuplicateIds)
    {
        Console.WriteLine($"Duplicate ID: {duplicate.ConflictingId}");
    }
}
```

## DINOForge Integration

This library was extracted from DINOForge. To migrate:

```bash
# Replace DINOForge SDK pack classes with Phenotype.Packs
dotnet add package Phenotype.Packs
```

Then update your using statements:

```csharp
// Before
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;

// After
using Phenotype.Packs;
using Phenotype.Packs.Resolution;
```

## Comparison with Alternatives

| Feature | Phenotype.Packs | BepInEx Patching | Manual Loading |
|---------|-----------------|------------------|----------------|
| Dependency Resolution | ✅ Built-in | ❌ Manual | ❌ Manual |
| Conflict Detection | ✅ Built-in | ❌ None | ❌ Manual |
| Generic Types | ✅ Full | ⚠️ Limited | ❌ None |
| YAML Manifests | ✅ Built-in | ❌ Custom | ❌ Custom |
| Validation | ✅ Built-in | ❌ None | ❌ Manual |
| Event System | ✅ Built-in | ❌ Limited | ❌ Manual |

## Documentation

- [API Reference](docs/API.md)
- [Migration Guide from DINOForge](docs/MIGRATION.md)
- [Custom Manifest Types](docs/CUSTOM_MANIFESTS.md)
- [Best Practices](docs/BEST_PRACTICES.md)

## Contributing

This project follows the [Phenotype Component Architecture](https://github.com/phenotype/architecture). See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) file.

## Acknowledgments

Extracted from [DINOForge](https://github.com/dinoforge/DINOForge) by devopsdinosaur, used in production for the RTS game "Diplomacy is Not an Option" with 1,017+ test cases and real-world modding community usage.
