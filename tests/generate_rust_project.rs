use std::path::Path;

use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::file::File;
use grustine::backend::rust_ast_from_lir::project::rust_ast_from_lir;
use grustine::error::{display, TerminationError};
use grustine::frontend::hir_from_ast::file::hir_from_ast;
use grustine::frontend::lir_from_hir::file::lir_from_hir;
use grustine::parser::langrust;

#[test]
fn generate_rust_project_for_counter() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/counter.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(counter_id, &files.source(counter_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/counter/");

    project.generate()
}

#[test]
fn generate_rust_project_for_blinking() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/blinking.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/blinking/");

    project.generate()
}

#[test]
fn generate_rust_project_for_button_management() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/button_management.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            button_management_id,
            &files.source(button_management_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/button_management/");

    project.generate()
}

#[test]
fn generate_rust_project_for_button_management_condition_match() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            button_management_condition_match_id,
            &files.source(button_management_condition_match_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/button_management_condition_match/");

    project.generate()
}

#[test]
fn generate_rust_project_for_button_management_using_function() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/button_management_using_function.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            button_management_using_function_id,
            &files.source(button_management_using_function_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/button_management_using_function/");

    project.generate()
}

#[test]
fn generate_rust_project_for_pid() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/pid.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(pid_id, &files.source(pid_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/pid/");

    project.generate()
}

#[test]
fn generate_rust_project_for_pid_function_field_access() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_function_field_access_id = files.add(
        "pid_function_field_access.gr",
        std::fs::read_to_string("tests/fixture/pid_function_field_access.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_function_field_access_id,
            &files.source(pid_function_field_access_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/pid_function_field_access/");

    project.generate()
}

#[test]
fn generate_rust_project_for_pid_field_access() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_function_field_access_id = files.add(
        "pid_field_access.gr",
        std::fs::read_to_string("tests/fixture/pid_field_access.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_function_field_access_id,
            &files.source(pid_function_field_access_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/pid_field_access/");

    project.generate()
}

#[test]
fn generate_rust_project_for_alarm_manager_function() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let alarm_manager_function_id = files.add(
        "alarm_manager_function.gr",
        std::fs::read_to_string("tests/fixture/alarm_manager_function.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            alarm_manager_function_id,
            &files.source(alarm_manager_function_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/alarm_manager_function/");

    project.generate()
}

#[test]
fn generate_rust_project_for_alarm_manager() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let alarm_manager_id = files.add(
        "alarm_manager.gr",
        std::fs::read_to_string("tests/fixture/alarm_manager.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(alarm_manager_id, &files.source(alarm_manager_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/alarm_manager/");

    project.generate()
}

#[test]
fn generate_rust_project_for_alarm_counter_function() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let alarm_counter_function_id = files.add(
        "alarm_counter_function.gr",
        std::fs::read_to_string("tests/fixture/alarm_counter_function.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            alarm_counter_function_id,
            &files.source(alarm_counter_function_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/alarm_counter_function/");

    project.generate()
}

#[test]
fn generate_rust_project_for_alarm_counter() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let alarm_counter_id = files.add(
        "alarm_counter.gr",
        std::fs::read_to_string("tests/fixture/alarm_counter.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(alarm_counter_id, &files.source(alarm_counter_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/alarm_counter/");

    project.generate()
}

#[test]
fn generate_rust_project_for_alarm_sort_function() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let alarm_sort_function_id = files.add(
        "alarm_sort_function.gr",
        std::fs::read_to_string("tests/fixture/alarm_sort_function.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            alarm_sort_function_id,
            &files.source(alarm_sort_function_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/alarm_sort_function/");

    project.generate()
}

#[test]
fn generate_rust_project_for_alarm_sort() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let alarm_sort_id = files.add(
        "alarm_sort.gr",
        std::fs::read_to_string("tests/fixture/alarm_sort.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(alarm_sort_id, &files.source(alarm_sort_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/alarm_sort/");

    project.generate()
}

#[test]
fn generate_rust_project_for_factorial() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let factorial_id = files.add(
        "factorial.gr",
        std::fs::read_to_string("tests/fixture/factorial.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(factorial_id, &files.source(factorial_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/factorial/");

    project.generate()
}

#[test]
fn generate_rust_project_for_map_int_to_float() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let map_int_to_float_id = files.add(
        "map_int_to_float.gr",
        std::fs::read_to_string("tests/fixture/map_int_to_float.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(map_int_to_float_id, &files.source(map_int_to_float_id).unwrap())
        .unwrap();
    file.typing(&mut errors);
    display(&errors, &files);
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/map_int_to_float/");

    project.generate()
}

#[test]
fn generate_rust_project_for_adas_example() {
    let mut files = SimpleFiles::new();
    let mut files_id = vec![];
    let mut errors = vec![];

    // files_id.push(
    //     files.add(
    //         "radar_detection.gr",
    //         std::fs::read_to_string("tests/fixture/adas_example/radar_detection.gr")
    //             .expect("unkown file"),
    //     ),
    // );
    // files_id.push(
    //     files.add(
    //         "lidar_detection.gr",
    //         std::fs::read_to_string("tests/fixture/adas_example/lidar_detection.gr")
    //             .expect("unkown file"),
    //     ),
    // );
    // files_id.push(
    //     files.add(
    //         "classification.gr",
    //         std::fs::read_to_string("tests/fixture/adas_example/classification.gr")
    //             .expect("unkown file"),
    //     ),
    // );
    // files_id.push(files.add(
    //     "fusion.gr",
    //     std::fs::read_to_string("tests/fixture/adas_example/fusion.gr").expect("unkown file"),
    // ));
    // files_id.push(
    //     files.add(
    //         "object_tracking.gr",
    //         std::fs::read_to_string("tests/fixture/adas_example/object_tracking.gr")
    //             .expect("unkown file"),
    //     ),
    // );

    let test = files_id
        .into_iter()
        .map(|id| {
            let file_name = Path::new(files.name(id).unwrap())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            let file_content = files.source(id).unwrap();

            let mut file_ast: File = langrust::fileParser::new().parse(id, file_content).unwrap();
            file_ast.typing(&mut errors)?;

            let mut file_hir = hir_from_ast(file_ast);
            file_hir.generate_dependency_graphs(&mut errors)?;
            file_hir.causality_analysis(&mut errors)?;
            file_hir.normalize(&mut errors)?;

            let project_lir = lir_from_hir(file_hir);

            let mut project_rust = rust_ast_from_lir(project_lir);
            project_rust.set_parent(format!("tests/generated/adas_example/{file_name}/"));

            Ok(project_rust.generate())
        })
        .collect::<Vec<Result<(), TerminationError>>>();

    display(&errors, &files);

    test.into_iter().collect::<Result<(), _>>().unwrap()
}

#[test]
fn generate_rust_project_for_contracts_test() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let contracts_test_id = files.add(
        "contracts_test.gr",
        std::fs::read_to_string("tests/fixture/contracts_test.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(contracts_test_id, &files.source(contracts_test_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);
    let mut project = rust_ast_from_lir(project);
    project.set_parent("tests/generated/contracts_test/");

    project.generate()
}
