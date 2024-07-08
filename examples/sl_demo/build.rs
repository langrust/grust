extern crate tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "src/interface.proto";
    let proto_path: &std::path::Path = proto.as_ref();

    // directory the main .proto file resides in
    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    tonic_build::configure()
        .type_attribute(
            "interface.Input",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.Output",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.Speed",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.SetSpeed",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.VdcState",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.ActivationResquest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.VacuumBrakeState",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.Kickdown",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.Failure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.SlState",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.Input.message",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "interface.Output.message",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&[proto_path], &[proto_dir])?;
    Ok(())
}
