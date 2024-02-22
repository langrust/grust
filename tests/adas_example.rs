use codespan_reporting::files::SimpleFiles;

use grustine::generate_rust_project;

#[test]
fn generate_rust_project_for_adas_example() {
    let mut files = SimpleFiles::new();
    let mut files_id = vec![];

    files_id.push(
        files.add(
            "radar_detection.gr",
            std::fs::read_to_string("tests/fixture/adas_example/radar_detection.gr")
                .expect("unkown file"),
        ),
    );
    files_id.push(
        files.add(
            "lidar_detection.gr",
            std::fs::read_to_string("tests/fixture/adas_example/lidar_detection.gr")
                .expect("unkown file"),
        ),
    );
    files_id.push(
        files.add(
            "classification.gr",
            std::fs::read_to_string("tests/fixture/adas_example/classification.gr")
                .expect("unkown file"),
        ),
    );
    files_id.push(files.add(
        "fusion.gr",
        std::fs::read_to_string("tests/fixture/adas_example/fusion.gr").expect("unkown file"),
    ));
    files_id.push(
        files.add(
            "object_tracking.gr",
            std::fs::read_to_string("tests/fixture/adas_example/object_tracking.gr")
                .expect("unkown file"),
        ),
    );

    files_id.into_iter().for_each(|id| {
        generate_rust_project(id, &mut files, format!("tests/generated/adas_example/"))
    })
}

#[test]
fn generate_rust_project_for_contracts_test() {
    let mut files = SimpleFiles::new();

    let contracts_test_id = files.add(
        "contracts_test.gr",
        std::fs::read_to_string("tests/fixture/contracts_test.gr").expect("unkown file"),
    );

    generate_rust_project(contracts_test_id, &mut files, "tests/generated/")
}
