use codespan_reporting::files::SimpleFiles;

use grustine::{error::display, hir_from_ast};

#[test]
fn hir_from_ast_transformation_for_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/success/counter.gr")
            .expect("unkown file"),
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
        std::fs::read_to_string("tests/fixture/hir_from_ast/success/blinking.gr")
            .expect("unkown file"),
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
        std::fs::read_to_string("tests/fixture/hir_from_ast/success/button_management.gr")
            .expect("unkown file"),
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
        std::fs::read_to_string(
            "tests/fixture/hir_from_ast/success/button_management_condition_match.gr",
        )
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
        std::fs::read_to_string(
            "tests/fixture/hir_from_ast/success/button_management_using_function.gr",
        )
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
        std::fs::read_to_string("tests/fixture/hir_from_ast/success/pid.gr").expect("unkown file"),
    );

    match hir_from_ast(pid_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_hir_from_ast_transformation_for_counter_unknown_signal() {
    let mut files = SimpleFiles::new();

    let counter_unknown_signal_id = files.add(
        "counter_unknown_signal.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/error/counter_unknown_signal.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(counter_unknown_signal_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_hir_from_ast_transformation_for_button_management_using_function_unknown_element() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_unknown_element_id = files.add(
        "button_management_using_function_unknown_element.gr",
        std::fs::read_to_string(
            "tests/fixture/hir_from_ast/error/button_management_using_function_unknown_element.gr",
        )
        .expect("unkown file"),
    );

    match hir_from_ast(
        button_management_using_function_unknown_element_id,
        &mut files,
    ) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_hir_from_ast_transformation_for_button_management_unknown_type() {
    let mut files = SimpleFiles::new();

    let button_management_unknown_type_id = files.add(
        "button_management_unknown_type.gr",
        std::fs::read_to_string(
            "tests/fixture/hir_from_ast/error/button_management_unknown_type.gr",
        )
        .expect("unkown file"),
    );

    match hir_from_ast(button_management_unknown_type_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_hir_from_ast_transformation_for_pid_unknown_field() {
    let mut files = SimpleFiles::new();

    let pid_unknown_field_id = files.add(
        "pid_unknown_field.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/error/pid_unknown_field.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(pid_unknown_field_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_pid_missing_field() {
    let mut files = SimpleFiles::new();

    let pid_missing_field_id = files.add(
        "pid_missing_field.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/error/pid_missing_field.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(pid_missing_field_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_blinking_component_call() {
    let mut files = SimpleFiles::new();

    let blinking_component_call_id = files.add(
        "blinking_component_call.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/error/blinking_component_call.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(blinking_component_call_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_blinking_already_defined_element() {
    let mut files = SimpleFiles::new();

    let blinking_already_defined_element_id = files.add(
        "blinking_already_defined_element.gr",
        std::fs::read_to_string(
            "tests/fixture/hir_from_ast/error/blinking_already_defined_element.gr",
        )
        .expect("unkown file"),
    );

    match hir_from_ast(blinking_already_defined_element_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_pid_expect_structure_type() {
    let mut files = SimpleFiles::new();

    let pid_expect_structure_type_id = files.add(
        "pid_expect_structure_type.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/error/pid_expect_structure_type.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(pid_expect_structure_type_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_pid_unknown_enumeration() {
    let mut files = SimpleFiles::new();

    let pid_unknown_enumeration_id = files.add(
        "pid_unknown_enumeration.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/error/pid_unknown_enumeration.gr")
            .expect("unkown file"),
    );

    match hir_from_ast(pid_unknown_enumeration_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn hir_from_ast_transformation_for_urban_braking() {
    let mut files = SimpleFiles::new();

    let urban_braking_id = files.add(
        "urban_braking.gr",
        std::fs::read_to_string("tests/fixture/hir_from_ast/success/urban_braking.gr").expect("unkown file"),
    );

    match hir_from_ast(urban_braking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}
