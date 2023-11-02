use codespan_reporting::files::{Files, SimpleFiles};

use grustine::ast::file::File;
use grustine::frontend::hir_from_ast::file::hir_from_ast;
use grustine::parser::langrust;

#[test]
fn dependency_graph_of_counter() {
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

    insta::assert_yaml_snapshot!(file);
}
