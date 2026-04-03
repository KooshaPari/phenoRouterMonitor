using System.Collections.Generic;

namespace Phenotype.Validation;

/// <summary>
/// In-memory implementation of <see cref="ISchemaRegistry"/>.
/// </summary>
public class InMemorySchemaRegistry : ISchemaRegistry
{
    private readonly Dictionary<string, string> _schemas;

    /// <summary>
    /// Creates a new empty schema registry.
    /// </summary>
    public InMemorySchemaRegistry()
    {
        _schemas = new Dictionary<string, string>();
    }

    /// <summary>
    /// Creates a schema registry with pre-populated schemas.
    /// </summary>
    public InMemorySchemaRegistry(Dictionary<string, string> schemas)
    {
        _schemas = new Dictionary<string, string>(schemas);
    }

    /// <inheritdoc/>
    public void RegisterSchema(string name, string schemaContent)
    {
        _schemas[name] = schemaContent;
    }

    /// <inheritdoc/>
    public string? GetSchema(string name)
    {
        return _schemas.TryGetValue(name, out string? schema) ? schema : null;
    }

    /// <inheritdoc/>
    public bool HasSchema(string name)
    {
        return _schemas.ContainsKey(name);
    }

    /// <summary>
    /// Gets all registered schema names.
    /// </summary>
    public IEnumerable<string> GetSchemaNames()
    {
        return _schemas.Keys;
    }

    /// <summary>
    /// Clears all registered schemas.
    /// </summary>
    public void Clear()
    {
        _schemas.Clear();
    }
}
