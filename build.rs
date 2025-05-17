use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_files = [
        "proto/common.proto",
        "proto/chain_engine_router_core.proto",
        "proto/rag_manager_persona_layer.proto",
        "proto/memory_chain_engine.proto",
        "proto/router_core_model_registry.proto",
    ];

    let proto_dir = PathBuf::from("proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&proto_files, &[proto_dir])?;

    // Tell cargo to rerun this build script if the proto files change
    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={}", proto_file);
    }

    Ok(())
}
