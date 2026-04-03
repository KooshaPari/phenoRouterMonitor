using System.Collections.Generic;

namespace Phenotype.Validation;

/// <summary>
/// Factory for creating NJsonSchema-based validators.
/// </summary>
public class SchemaValidatorFactory : ISchemaValidatorFactory
{
    private readonly IYamlJsonConverter? _yamlConverter;

    /// <summary>
    /// Creates a factory with default YAML converter.
    /// </summary>
    public SchemaValidatorFactory()
    {
    }

    /// <summary>
    /// Creates a factory with a custom YAML converter.
    /// </summary>
    public SchemaValidatorFactory(IYamlJsonConverter yamlConverter)
    {
        _yamlConverter = yamlConverter;
    }

    /// <inheritdoc/>
    public IYamlSchemaValidator CreateValidator(Dictionary<string, string> schemaSources)
    {
        return _yamlConverter != null
            ? new NJsonSchemaValidator(schemaSources, _yamlConverter)
            : new NJsonSchemaValidator(schemaSources);
    }
}
