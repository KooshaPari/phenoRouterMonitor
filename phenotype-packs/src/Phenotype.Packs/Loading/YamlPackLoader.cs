using System.Collections.Generic;
using System.IO;
using System.Linq;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Phenotype.Packs.Loading
{
    /// <summary>
    /// Service for loading pack definitions.
    /// </summary>
    public interface IPackLoader<TManifest, TDefinition>
        where TManifest : IPackManifest
        where TDefinition : IContentDefinition
    {
        IEnumerable<TDefinition> LoadDefinitions(string packPath, TManifest manifest);
    }

    /// <summary>
    /// YAML-based pack loader for content definitions.
    /// </summary>
    public class YamlPackLoader<TManifest, TDefinition> : IPackLoader<TManifest, TDefinition>
        where TManifest : IPackManifest
        where TDefinition : class, IContentDefinition, new()
    {
        private readonly IDeserializer _yamlDeserializer;

        public YamlPackLoader()
        {
            _yamlDeserializer = new DeserializerBuilder()
                .WithNamingConvention(CamelCaseNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();
        }

        public IEnumerable<TDefinition> LoadDefinitions(string packPath, TManifest manifest)
        {
            var definitions = new List<TDefinition>();

            if (!Directory.Exists(packPath))
            {
                return definitions;
            }

            // Discover definition files
            var definitionFiles = DiscoverDefinitionFiles(packPath);

            foreach (var file in definitionFiles)
            {
                try
                {
                    var yaml = File.ReadAllText(file);
                    var definition = _yamlDeserializer.Deserialize<TDefinition>(yaml);

                    // Set source pack ID if property exists
                    if (definition is { } def)
                    {
                        // Note: In real implementation, we'd use reflection or interface method
                        // to set the source pack ID on the definition
                    }

                    definitions.Add(definition);
                }
                catch
                {
                    // Log error but continue loading other definitions
                    continue;
                }
            }

            return definitions;
        }

        private IEnumerable<string> DiscoverDefinitionFiles(string packPath)
        {
            // Look for definition files in common subdirectories
            var searchDirs = new[] { "definitions", "content", "data", "" };

            foreach (var dir in searchDirs)
            {
                var searchPath = string.IsNullOrEmpty(dir) ? packPath : Path.Combine(packPath, dir);

                if (!Directory.Exists(searchPath))
                {
                    continue;
                }

                // Look for YAML files
                foreach (var file in Directory.GetFiles(searchPath, "*.yaml", SearchOption.AllDirectories))
                {
                    // Skip manifest files
                    var fileName = Path.GetFileName(file).ToLowerInvariant();
                    if (fileName is "pack.yaml" or "manifest.yaml")
                    {
                        continue;
                    }

                    yield return file;
                }

                foreach (var file in Directory.GetFiles(searchPath, "*.yml", SearchOption.AllDirectories))
                {
                    var fileName = Path.GetFileName(file).ToLowerInvariant();
                    if (fileName is "pack.yml" or "manifest.yml")
                    {
                        continue;
                    }

                    yield return file;
                }
            }
        }
    }
}
