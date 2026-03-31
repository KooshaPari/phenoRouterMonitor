fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto files live in the shared proto/ directory at the workspace root
    // (three levels up from this build script: crates/agileplus-agent-service → workspace root).
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()   // crates/
        .and_then(|p| p.parent()) // agileplus-agents/
        .and_then(|p| p.parent()) // repo root
        .map(|p| p.join("proto"))
        .expect("could not compute proto root path");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[proto_root.join("agileplus/v1/agents.proto")],
            &[&proto_root],
        )?;

    Ok(())
}
