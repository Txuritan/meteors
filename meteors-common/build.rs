fn main() -> std::io::Result<()> {
    let mut config = prost_build::Config::new();

    config.btree_map(&["."]);

    // this is stable but not for prost (yet)
    config.protoc_arg("--experimental_allow_proto3_optional");

    config.compile_protos(&["src/models.proto"], &["src/"])?;

    Ok(())
}
