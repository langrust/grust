use codespan_reporting::{
    files::{Files, SimpleFiles},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use grustine::ast::file::File;
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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
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

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
}
