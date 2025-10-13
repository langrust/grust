fn main() {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/interface.proto"], &["proto"])
        .unwrap();
}
