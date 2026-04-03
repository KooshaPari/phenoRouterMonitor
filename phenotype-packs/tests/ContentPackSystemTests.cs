using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;

namespace Phenotype.Packs.Tests;

// Test manifest implementation
public class TestManifest : IPackManifest
{
    public string Id { get; set; } = "";
    public string Name { get; set; } = "";
    public string Version { get; set; } = "1.0.0";
    public string Author { get; set; } = "";
    public IReadOnlyList<IPackDependency> Dependencies { get; set; } = new List<TestDependency>();
}

// Test dependency implementation
public class TestDependency : IPackDependency
{
    public string PackId { get; set; } = "";
    public string VersionConstraint { get; set; } = "*";
    public bool IsRequired { get; set; } = true;
}

// Test definition implementation
public class TestDefinition : IPackDefinition
{
    public string DefinitionType { get; set; } = "";
    public string SourcePath { get; set; } = "";
}

public class ContentPackSystemTests
{
    private readonly ContentPackSystem<TestManifest, TestDefinition> _system;

    public ContentPackSystemTests()
    {
        _system = new ContentPackSystem<TestManifest, TestDefinition>();
    }

    [Fact]
    public async Task LoadPack_Success_WhenValid()
    {
        // Arrange
        var manifest = new TestManifest
        {
            Id = "test-pack",
            Name = "Test Pack",
            Version = "1.0.0",
            Author = "Test Author"
        };

        var context = new PackContext<TestManifest, TestDefinition>
        {
            PackPath = "/test/path",
            Manifest = manifest
        };

        var packId = new PackIdentifier("test-pack", "1.0.0");

        // Act
        var result = await _system.LoadPackAsync(packId, context);

        // Assert
        Assert.True(result.Success);
        Assert.Equal(packId, result.PackId);
        Assert.True(_system.IsPackLoaded(packId));
    }

    [Fact]
    public async Task LoadPack_Fails_WhenIdInvalid()
    {
        // Arrange
        var manifest = new TestManifest
        {
            Id = "",  // Invalid - empty ID
            Name = "Test Pack",
            Version = "1.0.0"
        };

        var context = new PackContext<TestManifest, TestDefinition>
        {
            PackPath = "/test/path",
            Manifest = manifest
        };

        var packId = new PackIdentifier("test-pack", "1.0.0");

        // Act
        var result = await _system.LoadPackAsync(packId, context);

        // Assert
        Assert.False(result.Success);
        Assert.Contains(result.Errors, e => e.Contains("ID"));
    }

    [Fact]
    public async Task UnloadPack_Success_WhenPackLoaded()
    {
        // Arrange
        var manifest = new TestManifest
        {
            Id = "unload-test",
            Name = "Unload Test",
            Version = "1.0.0"
        };

        var context = new PackContext<TestManifest, TestDefinition>
        {
            PackPath = "/test/path",
            Manifest = manifest
        };

        var packId = new PackIdentifier("unload-test", "1.0.0");
        await _system.LoadPackAsync(packId, context);

        // Act
        var result = await _system.UnloadPackAsync(packId);

        // Assert
        Assert.True(result);
        Assert.False(_system.IsPackLoaded(packId));
    }

    [Fact]
    public async Task UnloadPack_Fails_WhenPackNotLoaded()
    {
        // Act
        var result = await _system.UnloadPackAsync(new PackIdentifier("not-loaded", "1.0.0"));

        // Assert
        Assert.False(result);
    }

    [Fact]
    public async Task ResolveDependencies_Success_WithNoDeps()
    {
        // Arrange
        var manifest = new TestManifest
        {
            Id = "standalone",
            Name = "Standalone Pack",
            Version = "1.0.0",
            Dependencies = new List<TestDependency>()
        };

        // Act
        var result = await _system.ResolveDependenciesAsync(new[] { manifest });

        // Assert
        Assert.True(result.Success);
        Assert.Single(result.ResolvedOrder);
        Assert.Empty(result.Conflicts);
    }

    [Fact]
    public async Task ResolveDependencies_Success_WithDependency()
    {
        // Arrange
        var dep = new TestManifest
        {
            Id = "dependency",
            Name = "Dependency Pack",
            Version = "1.0.0"
        };

        var main = new TestManifest
        {
            Id = "main-pack",
            Name = "Main Pack",
            Version = "1.0.0",
            Dependencies = new List<TestDependency>
            {
                new() { PackId = "dependency", VersionConstraint = ">=1.0.0" }
            }
        };

        // Act
        var result = await _system.ResolveDependenciesAsync(new[] { main, dep });

        // Assert
        Assert.True(result.Success);
        Assert.Equal(2, result.ResolvedOrder.Count);
        // Dependency should come before main pack
        Assert.Equal("dependency", result.ResolvedOrder[0].Id);
        Assert.Equal("main-pack", result.ResolvedOrder[1].Id);
    }

    [Fact]
    public async Task ResolveDependencies_Fails_WhenMissingRequiredDep()
    {
        // Arrange
        var main = new TestManifest
        {
            Id = "main-pack",
            Name = "Main Pack",
            Version = "1.0.0",
            Dependencies = new List<TestDependency>
            {
                new() { PackId = "missing-dep", VersionConstraint = ">=1.0.0", IsRequired = true }
            }
        };

        // Act
        var result = await _system.ResolveDependenciesAsync(new[] { main });

        // Assert
        Assert.False(result.Success);
        Assert.Contains("missing-dep", result.MissingDependencies);
    }

    [Fact]
    public async Task PackLoaded_Event_Raised()
    {
        // Arrange
        var manifest = new TestManifest
        {
            Id = "event-test",
            Name = "Event Test",
            Version = "1.0.0"
        };

        var context = new PackContext<TestManifest, TestDefinition>
        {
            PackPath = "/test/path",
            Manifest = manifest
        };

        PackIdentifier? loadedId = null;
        _system.PackLoaded += (sender, e) => loadedId = e.PackId;

        // Act
        var packId = new PackIdentifier("event-test", "1.0.0");
        await _system.LoadPackAsync(packId, context);

        // Assert
        Assert.NotNull(loadedId);
        Assert.Equal(packId, loadedId.Value);
    }

    [Fact]
    public async Task PackUnloaded_Event_Raised()
    {
        // Arrange
        var manifest = new TestManifest
        {
            Id = "unload-event-test",
            Name = "Unload Event Test",
            Version = "1.0.0"
        };

        var context = new PackContext<TestManifest, TestDefinition>
        {
            PackPath = "/test/path",
            Manifest = manifest
        };

        PackIdentifier? unloadedId = null;
        _system.PackUnloaded += (sender, e) => unloadedId = e.PackId;

        var packId = new PackIdentifier("unload-event-test", "1.0.0");
        await _system.LoadPackAsync(packId, context);

        // Act
        await _system.UnloadPackAsync(packId);

        // Assert
        Assert.NotNull(unloadedId);
        Assert.Equal(packId, unloadedId.Value);
    }

    [Fact]
    public async Task DuplicatePackId_Detected()
    {
        // Arrange
        var manifest1 = new TestManifest { Id = "duplicate", Name = "Pack 1", Version = "1.0.0" };
        var manifest2 = new TestManifest { Id = "duplicate", Name = "Pack 2", Version = "2.0.0" };

        await _system.LoadPackAsync(
            new PackIdentifier("duplicate", "1.0.0"),
            new PackContext<TestManifest, TestDefinition> { PackPath = "/p1", Manifest = manifest1 });

        // Act
        var result = await _system.LoadPackAsync(
            new PackIdentifier("duplicate", "2.0.0"),
            new PackContext<TestManifest, TestDefinition> { PackPath = "/p2", Manifest = manifest2 });

        // Assert
        Assert.False(result.Success);
        Assert.Contains(result.Errors, e => e.Contains("already loaded"));
    }

    [Fact]
    public void GetLoadedPacks_ReturnsAllLoaded()
    {
        // Arrange
        var ids = new List<PackIdentifier>();
        for (int i = 0; i < 3; i++)
        {
            var manifest = new TestManifest
            {
                Id = $"pack-{i}",
                Name = $"Pack {i}",
                Version = "1.0.0"
            };

            var packId = new PackIdentifier($"pack-{i}", "1.0.0");
            ids.Add(packId);

            _system.LoadPackAsync(
                packId,
                new PackContext<TestManifest, TestDefinition> { PackPath = $"/p{i}", Manifest = manifest }).Wait();
        }

        // Act
        var loaded = _system.GetLoadedPacks();

        // Assert
        Assert.Equal(3, loaded.Count);
        foreach (var id in ids)
        {
            Assert.Contains(id, loaded);
        }
    }
}
