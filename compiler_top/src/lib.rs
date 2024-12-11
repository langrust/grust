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

prelude! {}

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
pub fn into_token_stream(ast: Ast) -> TokenStream2 {
    let mut symbol_table = SymbolTable::new();
    let ir1 = match ir1::from_ast(ast, &mut symbol_table) {
        Ok(ir1) => ir1,
        Err(errors) => {
            for error in errors {
                error.emit();
                // tokens.extend(error.into_syn_error().to_compile_error());
            }
            return parse_quote! {};
        }
    };
    let ir2 = ir1.into_ir2(symbol_table);
    let rust = ir2.into_syn();
    let mut tokens = TokenStream2::new();
    {
        use quote::TokenStreamExt;
        tokens.append_all(rust);
    }
    tokens
}

/// Writes the generated code at the given filepath.
pub fn dump_code(path_name: &str, tokens: &TokenStream2) {
    use std::{fs::OpenOptions, io::Write, path::Path, process::Command};
    let path = Path::new(path_name);

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect(&format!("failed to open `{path_name}`"));
        writeln!(&mut file, "{}", tokens).expect(&format!("failed to write to `{path_name}`"));
    }

    let mut rustfmt = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(path)
        .spawn()
        .expect("failed to spawn `rustfmt`");
    rustfmt.wait().expect("`rustfmt` failed");
}
