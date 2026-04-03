using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Phenotype.Packs.Loading;

/// <summary>
/// Loads pack manifests and definitions.
/// </summary>
public interface IPackLoader<TManifest, TDefinition>
    where TManifest : IPackManifest, new()
    where TDefinition : IPackDefinition
{
    Task<TManifest> LoadManifestAsync(string manifestPath, CancellationToken cancellationToken = default);
    Task<IReadOnlyList<TDefinition>> LoadDefinitionsAsync(
        PackContext<TManifest, TDefinition> context,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// YAML-based pack loader.
/// </summary>
public class YamlPackLoader<TManifest, TDefinition> : IPackLoader<TManifest, TDefinition>
    where TManifest : IPackManifest, new()
    where TDefinition : IPackDefinition
{
    private readonly IDeserializer _deserializer;

    public YamlPackLoader()
    {
        _deserializer = new DeserializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .IgnoreUnmatchedProperties()
            .Build();
    }

    public Task<TManifest> LoadManifestAsync(string manifestPath, CancellationToken cancellationToken = default)
    {
        if (!File.Exists(manifestPath))
        {
            throw new FileNotFoundException($"Manifest not found: {manifestPath}");
        }

        cancellationToken.ThrowIfCancellationRequested();

        var yaml = File.ReadAllText(manifestPath);
        var manifest = _deserializer.Deserialize<TManifest>(yaml);

        return Task.FromResult(manifest);
    }

    public Task<IReadOnlyList<TDefinition>> LoadDefinitionsAsync(
        PackContext<TManifest, TDefinition> context,
        CancellationToken cancellationToken = default)
    {
        // Default implementation returns empty list
        // Derived classes should override to load specific definition types
        return Task.FromResult<IReadOnlyList<TDefinition>>(new List<TDefinition>());
    }
}
