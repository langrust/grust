use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::file::File;
use grustine::frontend::hir_from_ast::file::hir_from_ast;
use grustine::frontend::mir_from_hir::file::mir_from_hir;
use grustine::parser::langrust;

#[test]
fn mir_from_hir_transformation_for_counter() {
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

    let project = mir_from_hir(file);
    insta::assert_yaml_snapshot!(project);
}

