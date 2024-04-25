//! This crate provides the [handle_tokens] function used in the [`grust!`] macro.
//! It transforms input GRust tokens into output Rust tokens, performing static analysis.
//!
//! This crate also provides the [dump_ast] function used to see the generated code
//! at a given filepath.
//!
//! [`grust!`]: ../grust/macro.grust.html

extern crate proc_macro;

pub use proc_macro::TokenStream;

pub mod ast;
pub mod conf;

/// Compiles input GRust tokens into output Rust tokens.
pub fn handle_tokens(_tokens: TokenStream) -> TokenStream {
    todo!()
}

/// Writes the generated code at the given filepath.
pub fn dump_ast(path_name: &str, tokens: &proc_macro2::TokenStream) {
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
