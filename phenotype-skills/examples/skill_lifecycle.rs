// Example: Skill lifecycle management with WASM runtime
//
// Run: cargo run --example skill_lifecycle --features "serde wasm"

use phenotype_skills::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Phenotype.Skills — Skill Lifecycle Example");
    println!("============================================\n");

    // 1. Create a skill definition
    let skill = SkillDefinition {
        id: SkillId("hello-world".to_string()),
        name: "Hello World".to_string(),
        version: SemVer::parse("1.0.0")?,
        language: Language::Rust,
        dependencies: vec![],
        sandbox_policy: SandboxPolicy::default(),
    };
    println!("1. Created skill: {:?}", skill);

    // 2. Create runtime configuration
    let config = RuntimeConfig {
        max_memory_mb: 512,
        timeout_ms: 5000,
        allowed_syscalls: vec!["read".to_string(), "write".to_string()],
        network_policy: NetworkPolicy::default(),
    };
    println!("2. Runtime config: memory={}MB, timeout={}ms", 
        config.max_memory_mb, config.timeout_ms);

    // 3. Create execution context
    let ctx = ExecutionContext {
        skill_id: skill.id.clone(),
        runtime_config: config,
        input: SkillInput(vec![1, 2, 3, 4, 5]),
    };
    println!("3. Execution context prepared");

    // 4. Simulate lifecycle stages
    println!("\n4. Skill lifecycle:");
    println!("   [DISCOVER] → [VALIDATE] → [LOAD] → [EXECUTE] → [UNLOAD]");
    
    let result = SkillExecutionResult {
        success: true,
        output: vec![1, 2, 3, 4, 5, 6],
        metrics: ExecutionMetrics {
            duration_ms: 42,
            memory_peak_mb: 64,
            cpu_time_ms: 12,
        },
    };
    println!("   Result: success={}, duration={}ms", result.success, result.metrics.duration_ms);

    println!("\nExample complete.");
    Ok(())
}
