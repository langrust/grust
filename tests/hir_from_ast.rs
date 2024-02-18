use codespan_reporting::files::SimpleFiles;

use grustine::{error::display, hir_from_ast};

#[test]
fn hir_from_ast_transformation_for_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/counter.gr").expect("unkown file"),
    );

    match hir_from_ast(counter_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn hir_from_ast_transformation_for_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/blinking.gr").expect("unkown file"),
    );

    match hir_from_ast(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn hir_from_ast_transformation_for_button_management() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/button_management.gr").expect("unkown file"),
    );

    match hir_from_ast(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn hir_from_ast_transformation_for_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn hir_from_ast_transformation_for_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/button_management_using_function.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn hir_from_ast_transformation_for_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/pid.gr").expect("unkown file"),
    );

    match hir_from_ast(pid_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}
