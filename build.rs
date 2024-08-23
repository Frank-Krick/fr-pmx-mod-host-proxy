fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/proxy.proto"], &["."])?;
    tonic_build::configure().compile(
        &["../fr-pmx-registry/proto/registry.proto"],
        &["../fr-pmx-registry/"],
    )?;
    Ok(())
}
