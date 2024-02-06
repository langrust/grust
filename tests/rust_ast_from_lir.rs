use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::file::File;
use grustine::backend::rust_ast_from_lir::project::rust_ast_from_lir;
use grustine::frontend::hir_from_ast::file::hir_from_ast;
use grustine::frontend::lir_from_hir::file::lir_from_hir;
use grustine::parser::langrust;

#[test]
fn rust_ast_from_lir_transformation_for_counter() {
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
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);

    let project = rust_ast_from_lir(project);
    insta::assert_debug_snapshot!(project);
}

#[test]
fn rust_ast_from_lir_transformation_for_blinking() {
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
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);

    let project = rust_ast_from_lir(project);
    insta::assert_debug_snapshot!(project);
}

#[test]
fn rust_ast_from_lir_transformation_for_button_management() {
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
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);

    let project = rust_ast_from_lir(project);
    insta::assert_debug_snapshot!(project);
}

#[test]
fn rust_ast_from_lir_transformation_for_button_management_condition_match() {
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
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);

    let project = rust_ast_from_lir(project);
    insta::assert_debug_snapshot!(project);
}

#[test]
fn rust_ast_from_lir_transformation_for_button_management_using_function() {
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
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);

    let project = rust_ast_from_lir(project);
    insta::assert_debug_snapshot!(project);
}

#[test]
fn rust_ast_from_lir_transformation_for_pid() {
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
    let mut file = hir_from_ast(file);
    file.generate_dependency_graphs(&mut errors).unwrap();
    file.causality_analysis(&mut errors).unwrap();
    file.normalize(&mut errors).unwrap();
    let project = lir_from_hir(file);

    let project = rust_ast_from_lir(project);
    insta::assert_debug_snapshot!(project);
}
