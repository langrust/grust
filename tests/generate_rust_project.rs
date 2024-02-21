use codespan_reporting::files::SimpleFiles;

use grustine::generate_rust_project;

#[test]
fn generate_rust_project_for_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/counter.gr").expect("unkown file"),
    );

    generate_rust_project(counter_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/blinking.gr").expect("unkown file"),
    );

    generate_rust_project(blinking_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_button_management() {
    let mut files = SimpleFiles::new();

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/button_management.gr").expect("unkown file"),
    );

    generate_rust_project(button_management_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        button_management_condition_match_id,
        &mut files,
        "tests/generated/",
    )
}

#[test]
fn generate_rust_project_for_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/button_management_using_function.gr")
            .expect("unkown file"),
    );

    generate_rust_project(
        button_management_using_function_id,
        &mut files,
        "tests/generated/",
    )
}

#[test]
fn generate_rust_project_for_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/pid.gr").expect("unkown file"),
    );

    generate_rust_project(pid_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_pid_function_field_access() {
    let mut files = SimpleFiles::new();

    let pid_function_field_access_id = files.add(
        "pid_function_field_access.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/pid_function_field_access.gr").expect("unkown file"),
    );

    generate_rust_project(pid_function_field_access_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_pid_field_access() {
    let mut files = SimpleFiles::new();

    let pid_function_field_access_id = files.add(
        "pid_field_access.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/pid_field_access.gr").expect("unkown file"),
    );

    generate_rust_project(pid_function_field_access_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_alarm_manager_function() {
    let mut files = SimpleFiles::new();

    let alarm_manager_function_id = files.add(
        "alarm_manager_function.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/alarm_manager_function.gr").expect("unkown file"),
    );

    generate_rust_project(alarm_manager_function_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_alarm_manager() {
    let mut files = SimpleFiles::new();

    let alarm_manager_id = files.add(
        "alarm_manager.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/alarm_manager.gr").expect("unkown file"),
    );

    generate_rust_project(alarm_manager_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_alarm_counter_function() {
    let mut files = SimpleFiles::new();

    let alarm_counter_function_id = files.add(
        "alarm_counter_function.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/alarm_counter_function.gr").expect("unkown file"),
    );

    generate_rust_project(alarm_counter_function_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_alarm_counter() {
    let mut files = SimpleFiles::new();

    let alarm_counter_id = files.add(
        "alarm_counter.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/alarm_counter.gr").expect("unkown file"),
    );

    generate_rust_project(alarm_counter_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_alarm_sort_function() {
    let mut files = SimpleFiles::new();

    let alarm_sort_function_id = files.add(
        "alarm_sort_function.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/alarm_sort_function.gr").expect("unkown file"),
    );

    generate_rust_project(alarm_sort_function_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_alarm_sort() {
    let mut files = SimpleFiles::new();

    let alarm_sort_id = files.add(
        "alarm_sort.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/alarm_sort.gr").expect("unkown file"),
    );

    generate_rust_project(alarm_sort_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_factorial() {
    let mut files = SimpleFiles::new();

    let factorial_id = files.add(
        "factorial.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/factorial.gr").expect("unkown file"),
    );

    generate_rust_project(factorial_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_map_int_to_float() {
    let mut files = SimpleFiles::new();

    let map_int_to_float_id = files.add(
        "map_int_to_float.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/map_int_to_float.gr").expect("unkown file"),
    );

    generate_rust_project(map_int_to_float_id, &mut files, "tests/generated/")
}

#[test]
fn generate_rust_project_for_adas_example() {
    let mut files = SimpleFiles::new();
    let mut files_id = vec![];

    // files_id.push(
    //     files.add(
    //         "radar_detection.gr",
    //         std::fs::read_to_string("tests/fixture/generate_rust_project/success/adas_example/radar_detection.gr")
    //             .expect("unkown file"),
    //     ),
    // );
    // files_id.push(
    //     files.add(
    //         "lidar_detection.gr",
    //         std::fs::read_to_string("tests/fixture/generate_rust_project/success/adas_example/lidar_detection.gr")
    //             .expect("unkown file"),
    //     ),
    // );
    // files_id.push(
    //     files.add(
    //         "classification.gr",
    //         std::fs::read_to_string("tests/fixture/generate_rust_project/success/adas_example/classification.gr")
    //             .expect("unkown file"),
    //     ),
    // );
    // files_id.push(files.add(
    //     "fusion.gr",
    //     std::fs::read_to_string("tests/fixture/generate_rust_project/success/adas_example/fusion.gr").expect("unkown file"),
    // ));
    // files_id.push(
    //     files.add(
    //         "object_tracking.gr",
    //         std::fs::read_to_string("tests/fixture/generate_rust_project/success/adas_example/object_tracking.gr")
    //             .expect("unkown file"),
    //     ),
    // );

    files_id.into_iter().for_each(|id| {
        generate_rust_project(id, &mut files, format!("tests/generated/adas_example/"))
    })
}

#[test]
fn generate_rust_project_for_contracts_test() {
    let mut files = SimpleFiles::new();

    let contracts_test_id = files.add(
        "contracts_test.gr",
        std::fs::read_to_string("tests/fixture/generate_rust_project/success/contracts_test.gr").expect("unkown file"),
    );

    generate_rust_project(contracts_test_id, &mut files, "tests/generated/")
}
