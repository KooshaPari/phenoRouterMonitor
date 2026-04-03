# Phenotype.Validation

Generic schema validation engine for YAML/JSON content, extracted from [DINOForge](https://github.com/kooshapari/Dino).

## Overview

Phenotype.Validation provides a clean, extensible validation framework for validating YAML/JSON documents against schemas. It's designed to be game-agnostic and reusable across different mod platforms.

## Installation

```bash
dotnet add package Phenotype.Validation
```

## Quick Start

### Basic Validation

```csharp
using Phenotype.Validation;

// Define your schemas
var schemas = new Dictionary<string, string>
{
    ["pack-manifest"] = @"
type: object
required:
  - name
  - version
properties:
  name:
    type: string
  version:
    type: string
    pattern: '^\\d+\\.\\d+\\.\\d+$'
"
};

// Create validator
var factory = new SchemaValidatorFactory();
var validator = factory.CreateValidator(schemas);

// Validate content
string yamlContent = @"
name: MyMod
version: 1.0.0
";

ValidationResult result = validator.Validate("pack-manifest", yamlContent);

if (result.IsValid)
{
    Console.WriteLine("Validation passed!");
}
else
{
    foreach (var error in result.Errors)
    {
        Console.WriteLine($"{error.Path}: {error.Message}");
    }
}
```

### Generic Interface

```csharp
// Use the generic interface for custom document types
public interface ISchemaValidator<TSchema, TDocument>
{
    ValidationResult Validate(TSchema schema, TDocument document);
}

// Example: Binary document validation
public class BinarySchemaValidator : ISchemaValidator<byte[], byte[]>
{
    public ValidationResult Validate(byte[] schema, byte[] document)
    {
        // Custom binary validation logic
    }
}
```

## Features

- **YAML Schema Validation**: Validate YAML content against YAML-defined JSON schemas
- **Generic Interfaces**: `ISchemaValidator<TSchema, TDocument>` for custom implementations
- **Schema Registry**: In-memory registry for managing multiple schemas
- **Caching**: Automatic schema caching for performance
- **Error Reporting**: Detailed validation errors with path, message, and rule information

## Architecture

```
Phenotype.Validation
├── ISchemaValidator<TSchema, TDocument>    Generic validation interface
├── IYamlSchemaValidator                     YAML-specific interface
├── ISchemaRegistry                          Schema management
├── IYamlJsonConverter                     YAML↔JSON conversion
├── NJsonSchemaValidator                    NJsonSchema implementation
├── InMemorySchemaRegistry                 In-memory schema storage
└── SchemaValidatorFactory                 Factory for creating validators
```

## Extraction from DINOForge

This library was extracted from `DINOForge.SDK.Validation`:

| Original File | New Location |
|---------------|--------------|
| `ISchemaValidator.cs` | `ISchemaValidator.cs` (generic) |
| `NJsonSchemaValidator.cs` | `NJsonSchemaValidator.cs` |
| `ValidationResult.cs` | `ValidationResult.cs` |
| `YamlSchemaConverter.cs` | `DefaultYamlJsonConverter` (internal) |

## Usage in DINOForge

```csharp
// DINOForge will reference this package
using Phenotype.Validation;

public class PackManifestValidator
{
    private readonly IYamlSchemaValidator _validator;

    public PackManifestValidator()
    {
        var factory = new SchemaValidatorFactory();
        _validator = factory.CreateValidator(LoadSchemas());
    }

    public bool ValidatePack(string manifestYaml)
    {
        var result = _validator.Validate("pack-manifest", manifestYaml);
        return result.IsValid;
    }
}
```

## License

MIT License - See [LICENSE](../LICENSE) for details.
