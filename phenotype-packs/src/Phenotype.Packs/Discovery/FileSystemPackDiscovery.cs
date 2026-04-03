using System.Collections.Generic;
using System.IO;
using System.Linq;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Phenotype.Packs.Discovery
{
    /// <summary>
    /// Service for discovering packs in the filesystem.
    /// </summary>
    public interface IPackDiscoveryService
    {
        TManifest? DiscoverManifest<TManifest>(string path) where TManifest : class, IPackManifest, new();
        IEnumerable<string> DiscoverPackDirectories(string rootPath);
    }

    /// <summary>
    /// File system-based pack discovery using YAML manifests.
    /// </summary>
    public class FileSystemPackDiscovery : IPackDiscoveryService
    {
        private static readonly string[] ManifestNames = { "pack.yaml", "pack.yml", "manifest.yaml", "manifest.yml" };
        private readonly IDeserializer _yamlDeserializer;

        public FileSystemPackDiscovery()
        {
            _yamlDeserializer = new DeserializerBuilder()
                .WithNamingConvention(CamelCaseNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();
        }

        public TManifest? DiscoverManifest<TManifest>(string path) where TManifest : class, IPackManifest, new()
        {
            // Check if path is a directory or file
            if (File.Exists(path))
            {
                return TryLoadManifest<TManifest>(path);
            }

            if (!Directory.Exists(path))
            {
                return null;
            }

            // Look for manifest files in the directory
            foreach (var manifestName in ManifestNames)
            {
                var manifestPath = Path.Combine(path, manifestName);
                if (File.Exists(manifestPath))
                {
                    return TryLoadManifest<TManifest>(manifestPath);
                }
            }

            return null;
        }

        public IEnumerable<string> DiscoverPackDirectories(string rootPath)
        {
            if (!Directory.Exists(rootPath))
            {
                yield break;
            }

            foreach (var directory in Directory.GetDirectories(rootPath))
            {
                // Check if this directory contains a manifest
                foreach (var manifestName in ManifestNames)
                {
                    var manifestPath = Path.Combine(directory, manifestName);
                    if (File.Exists(manifestPath))
                    {
                        yield return directory;
                        break;
                    }
                }
            }
        }

        private TManifest? TryLoadManifest<TManifest>(string path) where TManifest : class, IPackManifest, new()
        {
            try
            {
                var yaml = File.ReadAllText(path);
                var manifest = _yamlDeserializer.Deserialize<TManifest>(yaml);
                return manifest;
            }
            catch
            {
                return null;
            }
        }
    }
}
