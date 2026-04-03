using System.Collections.Generic;

namespace Phenotype.Validation;

/// <summary>
/// Generic interface for validating documents against schemas.
/// </summary>
/// <typeparam name="TSchema">The type representing a schema (e.g., string for YAML schemas).</typeparam>
/// <typeparam name="TDocument">The type representing a document to validate.</typeparam>
public interface ISchemaValidator<TSchema, TDocument>
{
    /// <summary>
    /// Validates a document against a schema.
    /// </summary>
    /// <param name="schema">The schema to validate against.</param>
    /// <param name="document">The document to validate.</param>
    /// <returns>A validation result indicating success or failure with details.</returns>
    ValidationResult Validate(TSchema schema, TDocument document);
}

/// <summary>
/// Simplified interface for string-based YAML schema validation.
/// </summary>
public interface IYamlSchemaValidator : ISchemaValidator<string, string>
{
    /// <summary>
    /// Validates YAML content against a named schema.
    /// </summary>
    /// <param name="schemaName">The logical schema name (e.g., "pack-manifest").</param>
    /// <param name="yamlContent">Raw YAML text to validate.</param>
    /// <returns>A validation result describing whether validation passed.</returns>
    ValidationResult Validate(string schemaName, string yamlContent);
}

/// <summary>
/// Factory for creating schema validators from schema sources.
/// </summary>
public interface ISchemaValidatorFactory
{
    /// <summary>
    /// Creates a validator from a dictionary of schema sources.
    /// </summary>
    /// <param name="schemaSources">Dictionary mapping schema names to their YAML schema content.</param>
    /// <returns>A configured schema validator.</returns>
    IYamlSchemaValidator CreateValidator(Dictionary<string, string> schemaSources);
}

/// <summary>
/// Converts between YAML and JSON formats.
/// </summary>
public interface IYamlJsonConverter
{
    /// <summary>
    /// Converts YAML content to JSON format.
    /// </summary>
    /// <param name="yamlContent">The YAML content to convert.</param>
    /// <returns>JSON representation of the YAML content.</returns>
    string ConvertYamlToJson(string yamlContent);
}

/// <summary>
/// Registry for managing multiple schemas.
/// </summary>
public interface ISchemaRegistry
{
    /// <summary>
    /// Registers a schema by name.
    /// </summary>
    /// <param name="name">The schema name.</param>
    /// <param name="schemaContent">The schema content (YAML).</param>
    void RegisterSchema(string name, string schemaContent);
    
    /// <summary>
    /// Retrieves a registered schema.
    /// </summary>
    /// <param name="name">The schema name.</param>
    /// <returns>The schema content, or null if not found.</returns>
    string? GetSchema(string name);
    
    /// <summary>
    /// Checks if a schema is registered.
    /// </summary>
    /// <param name="name">The schema name.</param>
    /// <returns>True if the schema exists.</returns>
    bool HasSchema(string name);
}
