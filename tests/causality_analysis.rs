use codespan_reporting::files::SimpleFiles;

use grustine::causality_analysis;
use grustine::error::display;

#[test]
fn causality_analysis_of_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/causality_analysis/success/counter.gr")
            .expect("unkown file"),
    );

    match causality_analysis(counter_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn causality_analysis_of_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/causality_analysis/success/blinking.gr")
            .expect("unkown file"),
    );

    match causality_analysis(blinking_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn causality_analysis_of_button_management() {
    let mut files = SimpleFiles::new();

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/causality_analysis/success/button_management.gr")
            .expect("unkown file"),
    );

    match causality_analysis(button_management_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn causality_analysis_of_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string(
            "tests/fixture/causality_analysis/success/button_management_condition_match.gr",
        )
        .expect("unkown file"),
    );

    match causality_analysis(button_management_condition_match_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn causality_analysis_of_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string(
            "tests/fixture/causality_analysis/success/button_management_using_function.gr",
        )
        .expect("unkown file"),
    );

    match causality_analysis(button_management_using_function_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn causality_analysis_of_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/causality_analysis/success/pid.gr")
            .expect("unkown file"),
    );

    match causality_analysis(pid_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn error_counter_not_causal() {
    let mut files = SimpleFiles::new();

    let counter_not_causal_id = files.add(
        "counter_not_causal.gr",
        std::fs::read_to_string("tests/fixture/causality_analysis/error/counter_not_causal.gr")
            .expect("unkown file"),
    );

    match causality_analysis(counter_not_causal_id, &mut files) {
        Ok(()) => (),
        Err(errors) => display(&errors, &files),
    }
}
