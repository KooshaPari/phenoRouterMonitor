using System.Collections.Generic;
using Xunit;
using Phenotype.Validation;

namespace Phenotype.Validation.Tests;

public class SchemaValidatorTests
{
    private static readonly Dictionary<string, string> TestSchemas = new()
    {
        ["test-schema"] = @"
type: object
required:
  - name
properties:
  name:
    type: string
  version:
    type: string
    pattern: '^\d+\.\d+\.\d+$'
"
    };

    [Fact]
    public void ValidYamlDocument_PassesValidation()
    {
        // Arrange
        var factory = new SchemaValidatorFactory();
        var validator = factory.CreateValidator(TestSchemas);

        string yaml = @"
name: MyMod
version: 1.0.0
";

        // Act
        var result = validator.Validate("test-schema", yaml);

        // Assert
        Assert.True(result.IsValid);
        Assert.Empty(result.Errors);
    }

    [Fact]
    public void MissingRequiredField_FailsValidation()
    {
        // Arrange
        var factory = new SchemaValidatorFactory();
        var validator = factory.CreateValidator(TestSchemas);

        string yaml = @"
version: 1.0.0
";

        // Act
        var result = validator.Validate("test-schema", yaml);

        // Assert
        Assert.False(result.IsValid);
        Assert.NotEmpty(result.Errors);
    }

    [Fact]
    public void PatternMismatch_FailsValidation()
    {
        // Arrange
        var factory = new SchemaValidatorFactory();
        var validator = factory.CreateValidator(TestSchemas);

        string yaml = @"
name: MyMod
version: invalid
";

        // Act
        var result = validator.Validate("test-schema", yaml);

        // Assert
        Assert.False(result.IsValid);
        Assert.Contains(result.Errors, e => e.Message.Contains("version"));
    }

    [Fact]
    public void ValidJsonDocument_PassesValidation()
    {
        // Arrange
        var factory = new SchemaValidatorFactory();
        var validator = factory.CreateValidator(TestSchemas);

        string json = @"{ ""name"": ""MyMod"", ""version"": ""1.0.0"" }";

        // Act
        var result = validator.Validate("test-schema", json);

        // Assert
        Assert.True(result.IsValid);
    }

    [Fact]
    public void UnknownSchema_ReturnsEmptyError()
    {
        // Arrange
        var factory = new SchemaValidatorFactory();
        var validator = factory.CreateValidator(TestSchemas);

        // Act
        var result = validator.Validate("unknown-schema", "{ }");

        // Assert
        Assert.False(result.IsValid);
        Assert.NotEmpty(result.Errors);
    }
}

public class SchemaRegistryTests
{
    [Fact]
    public void RegisterAndRetrieveSchema()
    {
        // Arrange
        var registry = new InMemorySchemaRegistry();
        const string schemaName = "test-schema";
        const string schemaContent = "type: object";

        // Act
        registry.RegisterSchema(schemaName, schemaContent);
        var retrieved = registry.GetSchema(schemaName);

        // Assert
        Assert.Equal(schemaContent, retrieved);
        Assert.True(registry.HasSchema(schemaName));
    }

    [Fact]
    public void GetUnknownSchema_ReturnsNull()
    {
        // Arrange
        var registry = new InMemorySchemaRegistry();

        // Act
        var result = registry.GetSchema("unknown");

        // Assert
        Assert.Null(result);
        Assert.False(registry.HasSchema("unknown"));
    }

    [Fact]
    public void Clear_RemovesAllSchemas()
    {
        // Arrange
        var registry = new InMemorySchemaRegistry();
        registry.RegisterSchema("schema1", "content1");
        registry.RegisterSchema("schema2", "content2");

        // Act
        registry.Clear();

        // Assert
        Assert.False(registry.HasSchema("schema1"));
        Assert.False(registry.HasSchema("schema2"));
    }
}
