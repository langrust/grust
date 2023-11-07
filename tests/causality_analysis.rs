use codespan_reporting::{
    files::{Files, SimpleFiles},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use grustine::ast::file::File;
use grustine::frontend::hir_from_ast::file::hir_from_ast;
use grustine::parser::langrust;

#[test]
fn causality_analysis_of_counter() {
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
    let file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();

    file.causality_analysis(&mut errors).unwrap();
}

#[test]
fn causality_analysis_of_blinking() {
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
    let file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();

    file.causality_analysis(&mut errors).unwrap();
}

#[test]
fn causality_analysis_of_button_management() {
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
    let file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();

    file.causality_analysis(&mut errors).unwrap();
}

#[test]
fn causality_analysis_of_button_management_condition_match() {
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
    let file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();

    file.causality_analysis(&mut errors).unwrap();
}

#[test]
fn causality_analysis_of_button_management_using_function() {
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
    let file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();

    file.causality_analysis(&mut errors).unwrap();
}

#[test]
fn error_when_typing_counter_not_causal() {
    let mut files = SimpleFiles::new();
    let mut errors = vec![];

    let counter_not_causal_id = files.add(
        "counter_not_causal.gr",
        std::fs::read_to_string("tests/fixture/counter_not_causal.gr").expect("unkown file"),
    );

    let mut file: File = langrust::fileParser::new()
        .parse(
            counter_not_causal_id,
            &files.source(counter_not_causal_id).unwrap(),
        )
        .unwrap();
    file.typing(&mut errors).unwrap();
    let file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();

    file.causality_analysis(&mut errors).unwrap_err();

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();
    for error in &errors {
        let writer = &mut writer.lock();
        let _ = term::emit(writer, &config, &files, &error.to_diagnostic());
    }
}
