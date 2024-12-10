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
    if let Ok(res) = res {
        res
    } else {
        panic!("fatal error during {}", desc)
    }
}

pub fn raw_from_ast(ast: Ast, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> File {
    let mut ctx = ctx::Simple::new(symbols, errors);
    unwrap("IR0 to IR1", ast.into_ir1(&mut ctx))
}

pub fn from_ast(ast: Ast, symbols: &mut SymbolTable) -> Result<File, Vec<Error>> {
    let mut errors_data = vec![];
    let errors = &mut errors_data;
    let mut ir1 = raw_from_ast(ast, symbols, errors);
    macro_rules! check_errors {
        {} => {
            if !errors.is_empty() { return Err(errors_data); }
        };
        { $desc:expr, $e:expr $(,)? } => {{
            check_errors!();
            match $e {
                Ok(()) => check_errors!(),
                Err(()) => {
                    assert!(!errors.is_empty());
                    for e in errors {
                        e.add_note_mut(Note::new(None, concat!("failure during ", $desc)))
                    }
                    return Err(errors_data);
                }
            }
        }};
    }
    check_errors!("IR1 type-checking", ir1.typ_check(symbols, errors));
    check_errors!(
        "IR1 dependency graph generation",
        ir1.generate_dependency_graphs(symbols, errors),
    );
    check_errors!(
        "IR1 causality analysis",
        ir1.causality_analysis(symbols, errors),
    );
    ir1.normalize(symbols);
    debug_assert!(errors.is_empty());
    Ok(ir1)
}
