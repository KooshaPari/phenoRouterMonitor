using System.Collections.Generic;
using System.Linq;

namespace Phenotype.Packs
{
    /// <summary>
    /// Service for detecting conflicts between content definitions.
    /// </summary>
    public interface IConflictDetector<TDefinition> where TDefinition : IContentDefinition
    {
        ConflictReport<TDefinition> DetectConflicts(IEnumerable<TDefinition> definitions);
    }

    /// <summary>
    /// Default conflict detector implementation.
    /// </summary>
    public class DefaultConflictDetector<TDefinition> : IConflictDetector<TDefinition> where TDefinition : IContentDefinition
    {
        public ConflictReport<TDefinition> DetectConflicts(IEnumerable<TDefinition> definitions)
        {
            var definitionList = definitions.ToList();
            var duplicateIds = new List<DuplicateIdConflict<TDefinition>>();
            var overrideConflicts = new List<OverrideConflict<TDefinition>>();

            // Group by ID to find duplicates
            var groupedById = definitionList.GroupBy(d => d.Id);

            foreach (var group in groupedById)
            {
                var definitionsInGroup = group.ToList();

                if (definitionsInGroup.Count > 1)
                {
                    // Check for overrides
                    var baseDefs = definitionsInGroup.Where(d => !d.IsOverride).ToList();
                    var overrideDefs = definitionsInGroup.Where(d => d.IsOverride).ToList();

                    if (baseDefs.Count == 1 && overrideDefs.Count > 0)
                    {
                        // Valid override pattern
                        overrideConflicts.Add(new OverrideConflict<TDefinition>
                        {
                            BaseId = group.Key,
                            BaseDefinition = baseDefs[0],
                            OverrideDefinitions = overrideDefs.AsReadOnly()
                        });
                    }
                    else if (baseDefs.Count > 1)
                    {
                        // Multiple base definitions - this is a conflict
                        duplicateIds.Add(new DuplicateIdConflict<TDefinition>
                        {
                            ConflictingId = group.Key,
                            Definitions = definitionsInGroup.AsReadOnly()
                        });
                    }
                }
            }

            return new ConflictReport<TDefinition>
            {
                DuplicateIds = duplicateIds.AsReadOnly(),
                OverrideConflicts = overrideConflicts.AsReadOnly()
            };
        }
    }
}
