use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::file::File;
use grustine::error::display;
use grustine::parser::langrust;

#[test]
fn typing_counter() {
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

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn typing_blinking() {
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

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn typing_button_management() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/button_management.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn typing_button_management_condition_match() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn typing_button_management_using_function() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/button_management_using_function.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(blinking_id, &files.source(blinking_id).unwrap())
        .unwrap();
    file.typing(&mut errors).unwrap();

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn typing_pid() {
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

    insta::assert_yaml_snapshot!(file);
}

#[test]
fn error_when_typing_counter_not_well_typed() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_not_well_typed_id = files.add(
        "counter_not_well_typed.gr",
        std::fs::read_to_string("tests/fixture/counter_not_well_typed.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            counter_not_well_typed_id,
            &files.source(counter_not_well_typed_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_counter_unknown_signal() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_unknown_signal_id = files.add(
        "counter_unknown_signal.gr",
        std::fs::read_to_string("tests/fixture/counter_unknown_signal.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            counter_unknown_signal_id,
            &files.source(counter_unknown_signal_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_blinking_unknown_node() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_unknown_node_id = files.add(
        "blinking_unknown_node.gr",
        std::fs::read_to_string("tests/fixture/blinking_unknown_node.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            blinking_unknown_node_id,
            &files.source(blinking_unknown_node_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_button_management_using_function_unknown_element() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let button_management_using_function_unknown_element_id = files.add(
        "button_management_using_function_unknown_element.gr",
        std::fs::read_to_string(
            "tests/fixture/button_management_using_function_unknown_element.gr",
        )
        .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            button_management_using_function_unknown_element_id,
            &files
                .source(button_management_using_function_unknown_element_id)
                .unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_button_management_unknown_type() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let button_management_unknown_type_id = files.add(
        "button_management_unknown_type.gr",
        std::fs::read_to_string("tests/fixture/button_management_unknown_type.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            button_management_unknown_type_id,
            &files.source(button_management_unknown_type_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_pid_unknown_field() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_unknown_field_id = files.add(
        "pid_unknown_field.gr",
        std::fs::read_to_string("tests/fixture/pid_unknown_field.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_unknown_field_id,
            &files.source(pid_unknown_field_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_pid_missing_field() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_missing_field_id = files.add(
        "pid_missing_field.gr",
        std::fs::read_to_string("tests/fixture/pid_missing_field.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_missing_field_id,
            &files.source(pid_missing_field_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_blinking_component_call() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_component_call_id = files.add(
        "blinking_component_call.gr",
        std::fs::read_to_string("tests/fixture/blinking_component_call.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            blinking_component_call_id,
            &files.source(blinking_component_call_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_blinking_already_defined_element() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_already_defined_element_id = files.add(
        "blinking_already_defined_element.gr",
        std::fs::read_to_string("tests/fixture/blinking_already_defined_element.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            blinking_already_defined_element_id,
            &files.source(blinking_already_defined_element_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_blinking_incompatible_type() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let blinking_incompatible_type_id = files.add(
        "blinking_incompatible_type.gr",
        std::fs::read_to_string("tests/fixture/blinking_incompatible_type.gr")
            .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            blinking_incompatible_type_id,
            &files.source(blinking_incompatible_type_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_pid_incompatible_pattern() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_incompatible_pattern_id = files.add(
        "pid_incompatible_pattern.gr",
        std::fs::read_to_string("tests/fixture/pid_incompatible_pattern.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_incompatible_pattern_id,
            &files.source(pid_incompatible_pattern_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_button_management_using_function_incompatible_input_number() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let button_management_using_function_incompatible_input_number_id = files.add(
        "button_management_using_function_incompatible_input_number.gr",
        std::fs::read_to_string(
            "tests/fixture/button_management_using_function_incompatible_input_number.gr",
        )
        .expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            button_management_using_function_incompatible_input_number_id,
            &files
                .source(button_management_using_function_incompatible_input_number_id)
                .unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_counter_expect_number() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_expect_number_id = files.add(
        "counter_expect_number.gr",
        std::fs::read_to_string("tests/fixture/counter_expect_number.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            counter_expect_number_id,
            &files.source(counter_expect_number_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_counter_expect_option() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_expect_option_id = files.add(
        "counter_expect_option.gr",
        std::fs::read_to_string("tests/fixture/counter_expect_option.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            counter_expect_option_id,
            &files.source(counter_expect_option_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_pid_expect_structure_type() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_expect_structure_type_id = files.add(
        "pid_expect_structure_type.gr",
        std::fs::read_to_string("tests/fixture/pid_expect_structure_type.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_expect_structure_type_id,
            &files.source(pid_expect_structure_type_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}

#[test]
fn error_when_typing_pid_unknown_enumeration() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let pid_unknown_enumeration_id = files.add(
        "pid_unknown_enumeration.gr",
        std::fs::read_to_string("tests/fixture/pid_unknown_enumeration.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            pid_unknown_enumeration_id,
            &files.source(pid_unknown_enumeration_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap_err();

    display(&errors, &files);
}
