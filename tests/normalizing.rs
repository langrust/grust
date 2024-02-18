use codespan_reporting::files::SimpleFiles;

use grustine::error::display;
use grustine::normalizing;

#[test]
fn normalize_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/normalizing/success/counter.gr").expect("unkown file"),
    );

    match normalizing(counter_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn normalize_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/normalizing/success/blinking.gr").expect("unkown file"),
    );

    match normalizing(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn normalize_button_management() {
    let mut files = SimpleFiles::new();

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/normalizing/success/button_management.gr").expect("unkown file"),
    );

    match normalizing(button_management_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn normalize_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string("tests/fixture/normalizing/success/button_management_condition_match.gr")
            .expect("unkown file"),
    );

    match normalizing(button_management_condition_match_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn normalize_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string("tests/fixture/normalizing/success/button_management_using_function.gr")
            .expect("unkown file"),
    );

    match normalizing(button_management_using_function_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn normalize_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/normalizing/success/pid.gr").expect("unkown file"),
    );

    match normalizing(pid_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_when_normalize_pid_unused_signal() {
    let mut files = SimpleFiles::new();

    let pid_unused_signal_id = files.add(
        "pid_unused_signal.gr",
        std::fs::read_to_string("tests/fixture/normalizing/error/pid_unused_signal.gr").expect("unkown file"),
    );

    match normalizing(pid_unused_signal_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}
