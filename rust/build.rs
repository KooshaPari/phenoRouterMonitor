fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../proto/agileplus/v1/common.proto",
                "../proto/agileplus/v1/core.proto",
                "../proto/agileplus/v1/agents.proto",
                "../proto/agileplus/v1/integrations.proto",
            ],
            &["../proto"],
        )?;
    Ok(())
}
