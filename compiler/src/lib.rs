//! This crate provides the [handle_tokens] function used in the [`grust!`] macro. It transforms
//! input GRust tokens into output Rust tokens, performing static analysis.
//!
//! This crate also provides the [dump_code] function used to see the generated code at a given
//! filepath.
//!
//! [`grust!`]: ../grust/macro.grust.html

extern crate proc_macro;

pub use proc_macro::TokenStream;

#[macro_use]
pub mod prelude;

prelude! {
    frontend::typing_analysis::TypeAnalysis,
    lir::Project,
    quote::TokenStreamExt,
}

mod ext;

pub mod frontend;
pub mod hir;

/// Compiles input GRust tokens into output Rust tokens.
pub fn handle_tokens(tokens: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(tokens as Ast);
    let tokens = into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        dump_code(&path, &tokens);
    }
    TokenStream::from(tokens)
}

/// Creates RustAST from GRust file.
pub fn into_token_stream(ast: Ast) -> macro2::TokenStream {
    let mut symbol_table = SymbolTable::new();
    let mut errors = vec![];
    macro_rules! present_errors {
        {
            $desc:literal, $work:expr $(,)?
        } => {{
            let res = $work;
            if !errors.is_empty() {
                let desc = $desc;
                let count = errors.len();
                let plural = if count > 1 { "s" } else { "" };
                eprintln!("{count} error{plural} occurred during {desc}:");
                for err in &errors {
                    println!("- {err}")
                }
            }
            res.expect(concat!("failure during ", $desc))
        }}
    }

    let mut hir = present_errors!(
        "HIR generation from AST",
        ast.into_hir(&mut hir::ctx::Simple::new(&mut symbol_table, &mut errors))
    );

    present_errors!("HIR typing", hir.typing(&mut symbol_table, &mut errors));

    present_errors!(
        "dependency graph generation",
        hir.generate_dependency_graphs(&symbol_table, &mut errors)
    );

    present_errors!(
        "causality analysis",
        hir.causality_analysis(&symbol_table, &mut errors)
    );

    hir.normalize(&mut symbol_table);
    let lir: Project = hir.into_lir(symbol_table);
    let rust = lir.into_syn();

    let mut tokens = macro2::TokenStream::new();
    tokens.append_all(rust);
    tokens
}

/// Writes the generated code at the given filepath.
pub fn dump_code(path_name: &str, tokens: &macro2::TokenStream) {
    use std::{fs::OpenOptions, io::Write, path::Path, process::Command};
    let path = Path::new(path_name);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect(&format!("failed to open `{path_name}`"));
    writeln!(&mut file, "{}", tokens).expect(&format!("failed to write to `{path_name}`"));

    let mut rustfmt = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(path)
        .spawn()
        .expect("failed to spawn `rustfmt`");
    rustfmt.wait().expect("`rustfmt` failed");
}
