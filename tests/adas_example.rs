use codespan_reporting::files::SimpleFiles;

use grustine::generate_rust_project;

#[test]
fn generate_rust_project_for_lidar_detection() {
    let mut files = SimpleFiles::new();

    let lidar_detection_id = files.add(
        "lidar_detection.gr",
        std::fs::read_to_string("tests/fixture/adas_example/lidar_detection.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        lidar_detection_id,
        &mut files,
        "tests/generated/adas_example/",
    )
}

#[test]
fn generate_rust_project_for_radar_detection() {
    let mut files = SimpleFiles::new();

    let radar_detection_id = files.add(
        "radar_detection.gr",
        std::fs::read_to_string("tests/fixture/adas_example/radar_detection.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        radar_detection_id,
        &mut files,
        "tests/generated/adas_example/",
    )
}

#[test]
fn generate_rust_project_for_classification() {
    let mut files = SimpleFiles::new();

    let classification_id = files.add(
        "classification.gr",
        std::fs::read_to_string("tests/fixture/adas_example/classification.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        classification_id,
        &mut files,
        "tests/generated/adas_example/",
    )
}

#[test]
fn generate_rust_project_for_fusion() {
    let mut files = SimpleFiles::new();

    let fusion_id = files.add(
        "fusion.gr",
        std::fs::read_to_string("tests/fixture/adas_example/fusion.gr").expect("unkown file"),
    );

    generate_rust_project(fusion_id, &mut files, "tests/generated/adas_example/")
}

#[test]
fn generate_rust_project_for_object_tracking() {
    let mut files = SimpleFiles::new();

    let object_tracking_id = files.add(
        "object_tracking.gr",
        std::fs::read_to_string("tests/fixture/adas_example/object_tracking.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        object_tracking_id,
        &mut files,
        "tests/generated/adas_example/",
    )
}

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
