//! This crate provides the [handle_tokens] function used in the [`grust!`] macro.
//! It transforms input GRust tokens into output Rust tokens, performing static analysis.
//!
//! This crate also provides the [dump_ast] function used to see the generated code
//! at a given filepath.
//!
//! [`grust!`]: ../grust/macro.grust.html

extern crate proc_macro;

use ast::Ast;
use backend::rust_ast_from_lir::project::rust_ast_from_lir;
use frontend::{hir_from_ast::HIRFromAST, lir_from_hir::LIRFromHIR, typing_analysis::TypeAnalysis};
use hir::file::File;
use lir::project::Project;
pub use proc_macro::TokenStream;
use quote::TokenStreamExt;
use symbol_table::SymbolTable;

/// GRust AST module.
pub mod ast;
/// GRust backend transformations.
pub mod backend;
/// GRust common domain or application module.
pub mod common;
pub mod conf;
/// GRust error handler module.
pub mod error;
/// GRust frontend transformations.
pub mod frontend;
/// GRust HIR module.
pub mod hir;
/// GRust LIR module.
pub mod lir;
/// GRust symbol table module.
pub mod symbol_table;

/// Compiles input GRust tokens into output Rust tokens.
pub fn handle_tokens(tokens: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(tokens as Ast);
    let tokens = into_token_stream(ast);
    if let Some(path) = conf::dump_code() {
        dump_code(&path, &tokens);
    }
    TokenStream::from(tokens)
}

/// Creates RustAST from GRust file.
pub fn into_token_stream(ast: Ast) -> proc_macro2::TokenStream {
    let mut symbol_table = SymbolTable::new();
    let mut errors = vec![];

    let mut hir: File = ast.hir_from_ast(&mut symbol_table, &mut errors).unwrap();
    hir.typing(&mut symbol_table, &mut errors).unwrap();
    hir.generate_dependency_graphs(&symbol_table, &mut errors)
        .unwrap();
    hir.causality_analysis(&symbol_table, &mut errors).unwrap();
    hir.normalize(&mut symbol_table, &mut errors).unwrap();
    let lir: Project = hir.lir_from_hir(&symbol_table);
    let rust = rust_ast_from_lir(lir);

    let mut tokens = proc_macro2::TokenStream::new();
    tokens.append_all(rust);
    tokens
}

/// Writes the generated code at the given filepath.
pub fn dump_code(path_name: &str, tokens: &proc_macro2::TokenStream) {
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
        .arg(path)
        .spawn()
        .expect("failed to spawn `rustfmt`");
    rustfmt.wait().expect("`rustfmt` failed");
}
