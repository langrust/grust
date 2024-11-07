pub(crate) mod component;
pub(crate) mod dependencies;
pub(crate) mod file;
pub(crate) mod function;
pub(crate) mod identifier_creator;
pub(crate) mod once_cell;

pub mod contract;
pub mod ctx;
pub mod expr;
pub mod memory;
pub mod pattern;
pub mod stmt;
pub mod stream;
pub mod typedef;

pub mod flow;

pub mod interface;

prelude! {}

fn unwrap<D: Display, T>(desc: D, res: TRes<T>) -> T {
    match res {
        Ok(res) => res,
        Err(e) => panic!("fatal error during {}: {}", desc, e),
    }
}

pub fn raw_from_ast(ast: Ast, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> File {
    let mut ctx = ctx::Simple::new(symbols, errors);
    unwrap("AST to HIR", ast.into_hir(&mut ctx))
}

pub fn from_ast(ast: Ast, symbols: &mut SymbolTable) -> Result<File, Vec<Error>> {
    let mut errors_data = vec![];
    let errors = &mut errors_data;
    let mut hir = raw_from_ast(ast, symbols, errors);
    macro_rules! check_errors {
        {} => { if !errors.is_empty() { return Err(errors_data); } }
    }
    check_errors!();
    unwrap("HIR type-checking", hir.typing(symbols, errors));
    check_errors!();
    unwrap(
        "HIR dependency graph generation",
        hir.generate_dependency_graphs(symbols, errors),
    );
    check_errors!();
    unwrap(
        "HIR causality analysis",
        hir.causality_analysis(symbols, errors),
    );
    check_errors!();
    hir.normalize(symbols);
    Ok(hir)
}
