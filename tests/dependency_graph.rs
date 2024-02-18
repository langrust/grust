use codespan_reporting::files::SimpleFiles;

use grustine::dependency_graph;
use grustine::error::display;

#[test]
fn dependency_graph_of_counter() {
    let mut files = SimpleFiles::new();

    let counter_id = files.add(
        "counter.gr",
        std::fs::read_to_string("tests/fixture/dependency_graph/success/counter.gr")
            .expect("unkown file"),
    );

    match dependency_graph(counter_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn dependency_graph_of_blinking() {
    let mut files = SimpleFiles::new();

    let blinking_id = files.add(
        "blinking.gr",
        std::fs::read_to_string("tests/fixture/dependency_graph/success/blinking.gr")
            .expect("unkown file"),
    );

    match dependency_graph(blinking_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn dependency_graph_of_button_management() {
    let mut files = SimpleFiles::new();

    let button_management_id = files.add(
        "button_management.gr",
        std::fs::read_to_string("tests/fixture/dependency_graph/success/button_management.gr")
            .expect("unkown file"),
    );

    match dependency_graph(button_management_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn dependency_graph_of_button_management_condition_match() {
    let mut files = SimpleFiles::new();

    let button_management_condition_match_id = files.add(
        "button_management_condition_match.gr",
        std::fs::read_to_string(
            "tests/fixture/dependency_graph/success/button_management_condition_match.gr",
        )
        .expect("unkown file"),
    );

    match dependency_graph(button_management_condition_match_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn dependency_graph_of_button_management_using_function() {
    let mut files = SimpleFiles::new();

    let button_management_using_function_id = files.add(
        "button_management_using_function.gr",
        std::fs::read_to_string(
            "tests/fixture/dependency_graph/success/button_management_using_function.gr",
        )
        .expect("unkown file"),
    );

    match dependency_graph(button_management_using_function_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}

#[test]
fn dependency_graph_of_pid() {
    let mut files = SimpleFiles::new();

    let pid_id = files.add(
        "pid.gr",
        std::fs::read_to_string("tests/fixture/dependency_graph/success/pid.gr")
            .expect("unkown file"),
    );

    match dependency_graph(pid_id, &mut files) {
        Ok(file) => insta::assert_yaml_snapshot!(file),
        Err(errors) => display(&errors, &files),
    }
}
