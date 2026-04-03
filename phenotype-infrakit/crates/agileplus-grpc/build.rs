fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "../../proto/agileplus/v1/core.proto",
        "../../proto/agileplus/v1/agents.proto",
        "../../proto/agileplus/v1/common.proto",
        "../../proto/agileplus/v1/integrations.proto",
    ];

    let includes = &["../../proto"];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}
