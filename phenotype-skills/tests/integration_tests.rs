#[cfg(test)]
mod tests {
    use phenotype_skills::{SkillRegistry, SkillManifest, SkillVersion};
    use phenotype_skills::domain::SkillDependency;
    
    #[test]
    fn test_skill_registry_creation() {
        let registry = SkillRegistry::new();
        let skills = registry.list();
        assert!(skills.is_empty());
    }
    
    #[test]
    fn test_skill_manifest_parsing() {
        let toml = r#"
name = "test-skill"
version = "1.0.0"
description = "A test skill"
author = "Test Author"
license = "MIT"
entry_point = "main.wasm"
dependencies = []

[metadata]
tags = ["test", "example"]
categories = ["testing"]
"#;
        
        let manifest: SkillManifest = toml::from_str(toml).expect("Failed to parse TOML");
        assert_eq!(manifest.name, "test-skill");
        assert_eq!(manifest.version.to_string(), "1.0.0");
    }
    
    #[test]
    fn test_skill_version() {
        let v = SkillVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
        
        let v2 = SkillVersion::parse("2.0.0").expect("Failed to parse version");
        assert_eq!(v2.to_string(), "2.0.0");
    }
    
    #[test]
    fn test_dependency_resolver() {
        use phenotype_skills::application::DependencyResolver;
        
        let resolver = DependencyResolver::new();
        // Test with empty skill list
        let skills = vec![];
        let order = resolver.get_execution_order(&skills);
        assert!(order.is_ok());
        assert!(order.unwrap().is_empty());
    }
}
