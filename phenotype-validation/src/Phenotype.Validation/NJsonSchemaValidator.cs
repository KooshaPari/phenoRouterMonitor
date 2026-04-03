using System;
using System.Collections.Generic;
using System.Linq;
using NJsonSchema;
using Newtonsoft.Json.Linq;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Phenotype.Validation;

/// <summary>
/// Implements <see cref="IYamlSchemaValidator"/> using the NJsonSchema library.
/// Validates YAML content against YAML-defined JSON schemas by converting both
/// to JSON and leveraging NJsonSchema's validation capabilities.
///
/// This is a thin wrapper around NJsonSchema following the "wrap, don't handroll" principle.
/// </summary>
public class NJsonSchemaValidator : IYamlSchemaValidator
{
    private readonly Dictionary<string, string> _schemaSources;
    private readonly Dictionary<string, JsonSchema> _cachedSchemas;
    private readonly IYamlJsonConverter _yamlConverter;

    /// <summary>
    /// Initializes a new instance of <see cref="NJsonSchemaValidator"/>.
    /// </summary>
    /// <param name="schemaSources">
    /// Dictionary mapping schema names (e.g., "pack-manifest") to their YAML schema content.
    /// </param>
    public NJsonSchemaValidator(Dictionary<string, string> schemaSources)
        : this(schemaSources, new DefaultYamlJsonConverter())
    {
    }

    /// <summary>
    /// Initializes a new instance with a custom YAML converter.
    /// </summary>
    /// <param name="schemaSources">Dictionary mapping schema names to their YAML schema content.</param>
    /// <param name="yamlConverter">Converter for YAML to JSON conversion.</param>
    public NJsonSchemaValidator(Dictionary<string, string> schemaSources, IYamlJsonConverter yamlConverter)
    {
        _schemaSources = schemaSources ?? throw new ArgumentNullException(nameof(schemaSources));
        _yamlConverter = yamlConverter ?? throw new ArgumentNullException(nameof(yamlConverter));
        _cachedSchemas = new Dictionary<string, JsonSchema>();
    }

    /// <summary>
    /// Validates YAML content against a named schema.
    /// </summary>
    /// <param name="schemaName">The logical schema name (e.g., "pack-manifest").</param>
    /// <param name="yamlContent">Raw YAML text to validate.</param>
    /// <returns>A validation result describing whether validation passed and any errors.</returns>
    public ValidationResult Validate(string schemaName, string yamlContent)
    {
        if (string.IsNullOrWhiteSpace(schemaName))
            throw new ArgumentException("Schema name cannot be null or empty.", nameof(schemaName));
        if (string.IsNullOrWhiteSpace(yamlContent))
            throw new ArgumentException("YAML content cannot be null or empty.", nameof(yamlContent));

        // Load and cache the schema
        if (!_cachedSchemas.TryGetValue(schemaName, out JsonSchema? schema))
        {
            if (!_schemaSources.TryGetValue(schemaName, out string? schemaYaml))
                throw new InvalidOperationException($"Schema '{schemaName}' not found.");

            schema = LoadSchema(schemaYaml);
            _cachedSchemas[schemaName] = schema;
        }

        // Convert YAML content to JSON
        string jsonContent = _yamlConverter.ConvertYamlToJson(yamlContent);

        // Parse JSON for validation
        JToken jToken = JToken.Parse(jsonContent);

        // Perform validation
        ICollection<NJsonSchema.Validation.ValidationError> errors = schema.Validate(jToken);

        if (errors.Count == 0)
            return ValidationResult.Success();

        List<ValidationError> validationErrors = errors
            .Select(e => new ValidationError(
                path: e.Path ?? "",
                message: e.ToString(),
                rule: GetRuleKind(e)))
            .ToList();

        return ValidationResult.Failure(validationErrors.AsReadOnly());
    }

    /// <summary>
    /// Validates using a direct schema string instead of a named schema.
    /// </summary>
    public ValidationResult Validate(string schema, string document)
    {
        if (string.IsNullOrWhiteSpace(schema))
            throw new ArgumentException("Schema cannot be null or empty.", nameof(schema));
        if (string.IsNullOrWhiteSpace(document))
            throw new ArgumentException("Document cannot be null or empty.", nameof(document));

        // Convert YAML schema to JSON and load
        JsonSchema jsonSchema = LoadSchema(schema);

        // Convert YAML document to JSON
        string jsonContent = _yamlConverter.ConvertYamlToJson(document);

        // Parse and validate
        JToken jToken = JToken.Parse(jsonContent);
        var errors = jsonSchema.Validate(jToken);

        if (errors.Count == 0)
            return ValidationResult.Success();

        return ValidationResult.Failure(
            errors.Select(e => new ValidationError(e.Path ?? "", e.ToString(), GetRuleKind(e))).ToList().AsReadOnly());
    }

    /// <summary>
    /// Loads a JSON schema from YAML schema content.
    /// </summary>
    private JsonSchema LoadSchema(string yamlSchemaContent)
    {
        string jsonSchemaContent = _yamlConverter.ConvertYamlToJson(yamlSchemaContent);
        return JsonSchema.FromJsonAsync(jsonSchemaContent).GetAwaiter().GetResult();
    }

    /// <summary>
    /// Extracts the rule kind from a validation error.
    /// </summary>
    private static string GetRuleKind(object validationError)
    {
        System.Reflection.PropertyInfo? kindProperty = validationError.GetType().GetProperty("Kind");
        if (kindProperty != null)
        {
            object? kindValue = kindProperty.GetValue(validationError);
            return kindValue?.ToString() ?? "unknown";
        }
        return "unknown";
    }
}

/// <summary>
/// Default implementation of YAML to JSON converter using YamlDotNet.
/// </summary>
public class DefaultYamlJsonConverter : IYamlJsonConverter
{
    private readonly IDeserializer _yamlDeserializer;
    private readonly Newtonsoft.Json.JsonSerializer _jsonSerializer;

    public DefaultYamlJsonConverter()
    {
        _yamlDeserializer = new DeserializerBuilder()
            .WithNamingConvention(CamelCaseNamingConvention.Instance)
            .Build();
        
        _jsonSerializer = new Newtonsoft.Json.JsonSerializer
        {
            Formatting = Newtonsoft.Json.Formatting.None,
            NullValueHandling = Newtonsoft.Json.NullValueHandling.Ignore
        };
    }

    /// <summary>
    /// Converts YAML content to JSON format.
    /// </summary>
    public string ConvertYamlToJson(string yamlContent)
    {
        if (string.IsNullOrWhiteSpace(yamlContent))
            return "{}";

        // Deserialize YAML to object
        object? yamlObject = _yamlDeserializer.Deserialize<object>(yamlContent);
        
        if (yamlObject == null)
            return "{}";

        // Serialize to JSON
        using var stringWriter = new System.IO.StringWriter();
        using var jsonWriter = new Newtonsoft.Json.JsonTextWriter(stringWriter);
        _jsonSerializer.Serialize(jsonWriter, yamlObject);
        
        return stringWriter.ToString();
    }
}
