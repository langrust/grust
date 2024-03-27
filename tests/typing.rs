use codespan_reporting::files::SimpleFiles;

use grustine::error::display;
use grustine::typing;

#[test]
fn typing_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/typing/success/counter.gr").expect("unkown file"),
    );

    match typing(counter_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn typing_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/typing/success/blinking.gr").expect("unkown file"),
    );

    match typing(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn typing_button_management() {
    let mut files = SimpleFiles::new();

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/typing/success/button_management.gr")
            .expect("unkown file"),
    );

    match typing(button_management_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn typing_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string(
            "tests/fixture/typing/success/button_management_condition_match.gr",
        )
        .expect("unkown file"),
    );

    match typing(button_management_condition_match_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn typing_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/typing/success/button_management_using_function.gr")
            .expect("unkown file"),
    );

    match typing(button_management_using_function_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn typing_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/typing/success/pid.gr").expect("unkown file"),
    );

    match typing(pid_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_counter_not_well_typed() {
    let mut files = SimpleFiles::new();

    let counter_not_well_typed_id = files.add(
        "counter_not_well_typed.gr",
        std::fs::read_to_string("tests/fixture/typing/error/counter_not_well_typed.gr")
            .expect("unkown file"),
    );

    match typing(counter_not_well_typed_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_blinking_unknown_node() {
    let mut files = SimpleFiles::new();

    let blinking_unknown_node_id = files.add(
        "blinking_unknown_node.gr",
        std::fs::read_to_string("tests/fixture/typing/error/blinking_unknown_node.gr")
            .expect("unkown file"),
    );

    match typing(blinking_unknown_node_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_blinking_incompatible_type() {
    let mut files = SimpleFiles::new();

    let blinking_incompatible_type_id = files.add(
        "blinking_incompatible_type.gr",
        std::fs::read_to_string("tests/fixture/typing/error/blinking_incompatible_type.gr")
            .expect("unkown file"),
    );

    match typing(blinking_incompatible_type_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_pid_incompatible_pattern() {
    let mut files = SimpleFiles::new();

    let pid_incompatible_pattern_id = files.add(
        "pid_incompatible_pattern.gr",
        std::fs::read_to_string("tests/fixture/typing/error/pid_incompatible_pattern.gr")
            .expect("unkown file"),
    );

    match typing(pid_incompatible_pattern_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_button_management_using_function_incompatible_input_number() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_incompatible_input_number_id = files.add(
        "button_management_using_function_incompatible_input_number.gr",
        std::fs::read_to_string(
            "tests/fixture/typing/error/button_management_using_function_incompatible_input_number.gr",
        )
        .expect("unkown file"),
    );

    match typing(
        button_management_using_function_incompatible_input_number_id,
        &mut files,
    ) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_counter_expect_number() {
    let mut files = SimpleFiles::new();

    let counter_expect_number_id = files.add(
        "counter_expect_number.gr",
        std::fs::read_to_string("tests/fixture/typing/error/counter_expect_number.gr")
            .expect("unkown file"),
    );

    match typing(counter_expect_number_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_counter_expect_option() {
    let mut files = SimpleFiles::new();

    let counter_expect_option_id = files.add(
        "counter_expect_option.gr",
        std::fs::read_to_string("tests/fixture/typing/error/counter_expect_option.gr")
            .expect("unkown file"),
    );

    match typing(counter_expect_option_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_alarm_manager_expect_abstraction() {
    let mut files = SimpleFiles::new();

    let alarm_manager_expect_abstraction_id = files.add(
        "alarm_manager_expect_abstraction.gr",
        std::fs::read_to_string("tests/fixture/typing/error/alarm_manager_expect_abstraction.gr")
            .expect("unkown file"),
    );

    match typing(alarm_manager_expect_abstraction_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_typing_alarm_manager_expect_array() {
    let mut files = SimpleFiles::new();

    let alarm_manager_expect_array_id = files.add(
        "alarm_manager_expect_array.gr",
        std::fs::read_to_string("tests/fixture/typing/error/alarm_manager_expect_array.gr")
            .expect("unkown file"),
    );

    match typing(alarm_manager_expect_array_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn typing_urban_braking() {
    let mut files = SimpleFiles::new();

    let urban_braking_id = files.add(
        "urban_braking.gr",
        std::fs::read_to_string("tests/fixture/typing/success/urban_braking.gr").expect("unkown file"),
    );

    match typing(urban_braking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}
