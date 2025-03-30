fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .include_file("mod.rs")
        .compile_protos(
            &["src/api.proto"],
            &["src/"],
        )?;
    // tonic_build::compile_protos(, )?;

    // fn main() -> Result<()> {
    //     prost_build::compile_protos(&["src/api.proto"], )?;
    //     Ok(())
    // }
    Ok(())
}