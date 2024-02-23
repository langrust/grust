use codespan_reporting::files::SimpleFiles;
use grustine::generate_rust_project;

#[test]
fn generate_rust_project_for_veh_speed_odometer() {
    let mut files = SimpleFiles::new();

    let veh_speed_odometer_id = files.add(
        "veh_speed_odometer.gr",
        std::fs::read_to_string("tests/fixture/from_baptiste/veh_speed_odometer.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        veh_speed_odometer_id,
        &mut files,
        "tests/generated/from_baptiste/",
    )
}
