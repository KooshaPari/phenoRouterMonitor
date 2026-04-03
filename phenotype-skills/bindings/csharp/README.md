# Phenotype.Skills - C# Bindings

C# bindings for the Phenotype Skills Rust library.

## Installation

```bash
dotnet add package Phenotype.Skills
```

## Usage

```csharp
using Phenotype.Skills;

// Create a skill registry
using var registry = new SkillRegistry();

// Register a skill from manifest
registry.Register("./my-skill.toml");

// List registered skills
var skills = registry.List();
foreach (var skill in skills)
{
    Console.WriteLine($"Skill: {skill}");
}
```

## Building from Source

```bash
cd bindings/csharp
dotnet build
dotnet pack
```

## Architecture

The C# bindings use P/Invoke to call into the native Rust library. The native library must be built first:

```bash
cargo build --release
```

This produces:
- `libphenotype_skills.dylib` (macOS)
- `libphenotype_skills.so` (Linux)
- `phenotype_skills.dll` (Windows)
