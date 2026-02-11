use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_path = out_dir.join("agent.bin");

    tonic_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .compile_protos(&["proto/agent.proto"], &["proto"])?;   // ← 2 аргумента

    println!("cargo:rerun-if-changed=proto/agent.proto");
    println!("PROTO COMPILED SUCCESSFULLY ✅");

    Ok(())
}
