mod causality_analysis;
mod dependency_graph;
mod into_hir;
mod normalizing;
mod typing_analysis;

pub use self::{into_hir::IntoHir, typing_analysis::TypeAnalysis};

prelude! {}

fn present_errors(blah: &str, errors: &mut Vec<Error>) -> bool {
    if !errors.is_empty() {
        let count = errors.len();
        eprintln!("{} error{} occurred during {}:", count, plural(count), blah);
        for e in errors.drain(0..) {
            eprintln!("- {}", e);
        }
        true
    } else {
        false
    }
}

fn handle_result<T>(res: TRes<T>) -> T {
    match res {
        Ok(res) => res,
        Err(e) => {
            panic!("a fatal error occurred: {}", e)
        }
    }
}

fn raw_hir_with(ast: Ast, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> hir::File {
    handle_result(ast.into_hir(&mut hir::ctx::Simple::new(symbols, errors)))
}

fn raw_hir(ast: Ast, symbols: &mut SymbolTable) -> (hir::File, Vec<Error>) {
    let mut errors = vec![];
    let hir = raw_hir_with(ast, symbols, &mut errors);
    (hir, errors)
}

pub fn hir_analysis(ast: Ast, symbols: &mut SymbolTable) -> hir::File {
    let (mut hir, mut errors) = raw_hir(ast, symbols);
    let errors = &mut errors;
    present_errors("HIR generation from AST", errors);
    handle_result(hir.typing(symbols, errors));
    present_errors("HIR type-checking", errors);
    handle_result(hir.generate_dependency_graphs(symbols, errors));
    present_errors("HIR dependency graph generation", errors);
    handle_result(hir.causality_analysis(symbols, errors));
    present_errors("HIR causality analysis", errors);
    hir.normalize(symbols);
    hir
}
