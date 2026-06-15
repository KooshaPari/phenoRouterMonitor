// Example: Python runtime configuration
//
// Run: cargo run --example python_runtime --features "python serde"

use phenotype_skills::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Phenotype.Skills — Python Runtime Configuration");
    println!("================================================\n");

    // Python runtime with safe defaults
    let config = RuntimeConfig {
        max_memory_mb: 256,
        timeout_ms: 30000,
        allowed_syscalls: vec!["read".to_string()],
        network_policy: NetworkPolicy::Blocked,
    };
    println!("Python runtime config:");
    println!("  Memory:  {}MB", config.max_memory_mb);
    println!("  Timeout: {}ms", config.timeout_ms);
    println!("  Network: {:?}", config.network_policy);

    // Skill targeting Python runtime
    let skill = SkillDefinition {
        id: SkillId("python-hello".to_string()),
        name: "Python Hello".to_string(),
        version: SemVer::parse("1.0.0")?,
        language: Language::Python,
        dependencies: vec![
            DependencySpec {
                name: "requests".to_string(),
                version: VersionConstraint::Exact(SemVer::parse("2.31.0")?),
            }
        ],
        sandbox_policy: SandboxPolicy::default(),
    };
    println!("\nPython skill: {:?}", skill);

    println!("\nExample complete.");
    Ok(())
}
