use codespan_reporting::files::SimpleFiles;

use grustine::rust_ast_from_lir;

#[test]
fn rust_ast_from_lir_transformation_for_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/rust_ast_from_lir/success/counter.gr").expect("unkown file"),
    );

    let project = rust_ast_from_lir(counter_id, &mut files);
    insta::assert_debug_snapshot!(project)
}

#[test]
fn rust_ast_from_lir_transformation_for_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/rust_ast_from_lir/success/blinking.gr").expect("unkown file"),
    );

    let project = rust_ast_from_lir(blinking_id, &mut files);
    insta::assert_debug_snapshot!(project)
}

#[test]
fn rust_ast_from_lir_transformation_for_button_management() {
    let mut files = SimpleFiles::new();

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/rust_ast_from_lir/success/button_management.gr").expect("unkown file"),
    );

    let project = rust_ast_from_lir(button_management_id, &mut files);
    insta::assert_debug_snapshot!(project)
}

#[test]
fn rust_ast_from_lir_transformation_for_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/rust_ast_from_lir/success/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    let project = rust_ast_from_lir(button_management_condition_match_id, &mut files);
    insta::assert_debug_snapshot!(project)
}

#[test]
fn rust_ast_from_lir_transformation_for_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/rust_ast_from_lir/success/button_management_using_function.gr")
            .expect("unkown file"),
    );

    let project = rust_ast_from_lir(button_management_using_function_id, &mut files);
    insta::assert_debug_snapshot!(project)
}

#[test]
fn rust_ast_from_lir_transformation_for_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/rust_ast_from_lir/success/pid.gr").expect("unkown file"),
    );

    let project = rust_ast_from_lir(pid_id, &mut files);
    insta::assert_debug_snapshot!(project)
}
