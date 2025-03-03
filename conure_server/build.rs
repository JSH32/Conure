fn main() -> Result<(), Box<dyn std::error::Error>> {
    capnpc::CompilerCommand::new()
        .file("../protocol/conure_rpc.capnp")
        .run()?;
    Ok(())
}
