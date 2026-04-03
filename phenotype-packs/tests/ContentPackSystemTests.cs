using Xunit;
using Phenotype.Packs;
using Phenotype.Packs.Resolution;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace Phenotype.Packs.Tests
{
    public class ContentPackSystemTests
    {
        // Test manifest implementation
        public class TestManifest : IPackManifest
        {
            public string Id { get; set; } = "";
            public string Name { get; set; } = "";
            public string Version { get; set; } = "0.1.0";
            public string FrameworkVersion { get; set; } = ">=1.0.0";
            public string Author { get; set; } = "";
            public string Type { get; set; } = "content";
            public string? Description { get; set; }
            public IReadOnlyList<IPackDependency> Dependencies { get; set; } = new List<IPackDependency>();
            public IReadOnlyList<string> Conflicts { get; set; } = new List<string>();
        }

        // Test definition implementation
        public class TestDefinition : IContentDefinition
        {
            public string Id { get; set; } = "";
            public string DefinitionType { get; set; } = "";
            public string SourcePackId { get; set; } = "";
            public bool IsOverride { get; set; }
        }

        [Fact]
        public void PackDependency_ShouldSatisfyVersion()
        {
            // Arrange
            var dep = new PackDependency
            {
                PackId = "test-dep",
                VersionConstraint = ">=1.0.0",
                IsOptional = false
            };

            // Act & Assert
            Assert.True(dep.IsSatisfiedBy("1.0.0"));
            Assert.True(dep.IsSatisfiedBy("1.5.0"));
            Assert.True(dep.IsSatisfiedBy("2.0.0"));
        }

        [Fact]
        public void SemverDependencyResolver_ShouldCreateCorrectLoadOrder()
        {
            // Arrange
            var resolver = new SemverDependencyResolver();
            var packs = new List<TestManifest>
            {
                new TestManifest
                {
                    Id = "core",
                    Name = "Core Pack",
                    Version = "1.0.0",
                    Dependencies = new List<IPackDependency>()
                },
                new TestManifest
                {
                    Id = "extension",
                    Name = "Extension Pack",
                    Version = "1.0.0",
                    Dependencies = new List<IPackDependency>
                    {
                        new PackDependency { PackId = "core", VersionConstraint = ">=1.0.0" }
                    }
                }
            };

            // Act
            var graph = resolver.Resolve(packs);

            // Assert
            Assert.Equal(2, graph.LoadOrder.Count);
            Assert.Equal("core", graph.LoadOrder[0]);
            Assert.Equal("extension", graph.LoadOrder[1]);
        }

        [Fact]
        public void DefaultManifestValidator_ShouldDetectMissingId()
        {
            // Arrange
            var validator = new Validation.DefaultManifestValidator<TestManifest>();
            var manifest = new TestManifest
            {
                Id = "",
                Version = "1.0.0"
            };

            // Act
            var result = validator.Validate(manifest);

            // Assert
            Assert.False(result.IsValid);
            Assert.Contains(result.Errors, e => e.Property == nameof(IPackManifest.Id));
        }

        [Fact]
        public void DefaultConflictDetector_ShouldDetectDuplicateIds()
        {
            // Arrange
            var detector = new DefaultConflictDetector<TestDefinition>();
            var definitions = new List<TestDefinition>
            {
                new TestDefinition { Id = "unit1", SourcePackId = "pack-a", IsOverride = false },
                new TestDefinition { Id = "unit1", SourcePackId = "pack-b", IsOverride = false }
            };

            // Act
            var report = detector.DetectConflicts(definitions);

            // Assert
            Assert.True(report.HasConflicts);
            Assert.Single(report.DuplicateIds);
            Assert.Equal("unit1", report.DuplicateIds[0].ConflictingId);
        }

        [Fact]
        public void DefaultConflictDetector_ShouldAllowValidOverrides()
        {
            // Arrange
            var detector = new DefaultConflictDetector<TestDefinition>();
            var definitions = new List<TestDefinition>
            {
                new TestDefinition { Id = "unit1", SourcePackId = "base", IsOverride = false },
                new TestDefinition { Id = "unit1", SourcePackId = "mod", IsOverride = true }
            };

            // Act
            var report = detector.DetectConflicts(definitions);

            // Assert
            Assert.False(report.HasConflicts);
            Assert.Empty(report.DuplicateIds);
            Assert.Single(report.OverrideConflicts);
        }

        [Fact]
        public void PackLoadResult_ShouldBeSuccessful()
        {
            // Arrange & Act
            var result = new PackLoadResult<TestManifest, TestDefinition>
            {
                Success = true,
                Manifest = new TestManifest { Id = "test", Version = "1.0.0" },
                Definitions = new List<TestDefinition>(),
                Errors = new List<string>(),
                Warnings = new List<string>(),
                LoadDuration = TimeSpan.FromSeconds(1)
            };

            // Assert
            Assert.True(result.Success);
            Assert.NotNull(result.Manifest);
            Assert.Equal("test", result.Manifest.Id);
        }

        [Fact]
        public void DependencyGraph_ShouldReportMissingDependencies()
        {
            // Arrange
            var resolver = new SemverDependencyResolver();
            var packs = new List<TestManifest>
            {
                new TestManifest
                {
                    Id = "mod-a",
                    Name = "Mod A",
                    Version = "1.0.0",
                    Dependencies = new List<IPackDependency>
                    {
                        new PackDependency { PackId = "missing-dep", VersionConstraint = ">=1.0.0", IsOptional = false }
                    }
                }
            };

            // Act
            var graph = resolver.Resolve(packs);

            // Assert
            Assert.Single(graph.MissingDependencies);
            Assert.Equal("mod-a", graph.MissingDependencies[0].PackId);
            Assert.Equal("missing-dep", graph.MissingDependencies[0].DependencyId);
        }

        [Fact]
        public void DependencyGraph_ShouldReportVersionConflicts()
        {
            // Arrange
            var resolver = new SemverDependencyResolver();
            var packs = new List<TestManifest>
            {
                new TestManifest
                {
                    Id = "dep-v1",
                    Version = "1.0.0",
                    Dependencies = new List<IPackDependency>()
                },
                new TestManifest
                {
                    Id = "mod-a",
                    Version = "1.0.0",
                    Dependencies = new List<IPackDependency>
                    {
                        new PackDependency { PackId = "dep-v1", VersionConstraint = ">=2.0.0", IsOptional = false }
                    }
                }
            };

            // Act
            var graph = resolver.Resolve(packs);

            // Assert
            Assert.Single(graph.VersionConflicts);
            Assert.Equal("mod-a", graph.VersionConflicts[0].PackId);
            Assert.Equal("dep-v1", graph.VersionConflicts[0].DependencyId);
        }
    }
}
