//! Build script for zebra-indexer-proto.
//!
//! By default this is a no-op: the generated code is committed under
//! `proto/__generated__/`. Downstream users get a pure "cargo build" experience
//! with no protoc requirement and no heavy build dependencies.
//!
//! Maintainer usage:
//!     cargo build --features regenerate
//! Requires `protoc` (the Protocol Buffers compiler) to be installed.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/indexer.proto");

    #[cfg(feature = "regenerate")]
    regenerate()?;

    Ok(())
}

#[cfg(feature = "regenerate")]
fn regenerate() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let generated_dir = manifest_dir.join("proto").join("__generated__");
    std::fs::create_dir_all(&generated_dir)?;

    let proto_file = "proto/indexer.proto";

    // Direct generation into our committed directory.
    // This produces:
    //   proto/__generated__/zebra.indexer.rpc.rs
    //   proto/__generated__/indexer_descriptor.bin
    tonic_prost_build::configure()
        .out_dir(&generated_dir)
        .file_descriptor_set_path(generated_dir.join("indexer_descriptor.bin"))
        .compile_protos(&[proto_file], &["proto/"])?;

    println!("cargo:warning=Regenerated zebra-indexer-proto bindings into proto/__generated__/");

    Ok(())
}