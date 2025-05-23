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

/// Compiles input GRust tokens into output Rust tokens using nightly function.
#[cfg(not(feature = "no_diagnostics"))]
pub fn handle_tokens(tokens: TokenStream) -> TokenStream {
    let top = parse_macro_input!(tokens as ir0::Top);
    let (ast, mut ctx) = top.init();
    let tokens = into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code.as_ref() {
        let res = dump_code(path, &tokens);
        if let Err(e) = res {
            e.emit()
        }
    }
    TokenStream::from(tokens)
}

/// Compiles input GRust tokens into output Rust tokens using non nightly function.
#[cfg(feature = "no_diagnostics")]
pub fn handle_tokens(tokens: TokenStream) -> TokenStream {
    let top = parse_macro_input!(tokens as ir0::Top);
    let (ast, mut ctx) = top.init();
    let tokens = into_token_stream(ast, &mut ctx);
    if let Some(path) = ctx.conf.dump_code.as_ref() {
        let res = dump_code(path, &tokens);
        if let Err(e) = res {
            panic!("compilation error detected: {}", e.0);
        }
    }
    TokenStream::from(tokens)
}

/// Creates RustAST from GRust file using nightly funtion.
#[cfg(not(feature = "no_diagnostics"))]
pub fn into_token_stream(ast: Ast, ctx: &mut ir0::Ctx) -> TokenStream2 {
    let mut stats = Stats::new();
    let ir1 = match ir1::from_ast_timed(ast, ctx, stats.as_mut()) {
        Ok(pair) => pair,
        Err(errors) => {
            for error in errors {
                error.emit();
            }
            return parse_quote! {};
        }
    };
    if let Some(filepath) = &ctx.conf.dump_graph {
        ir1.dump_graph(filepath.value(), ctx);
    }
    let ir2 = stats.timed("ir1 → ir2", || ir1.into_ir2(ctx));
    let rust = stats.timed("codegen (ir2 → rust tokens)", || {
        ir2.prepare_tokens(ctx).to_token_stream()
    });
    if let Some(stats) = stats.pretty(&ctx.conf) {
        println!("Stats:\n\n{}", stats);
    }
    let mut tokens = TokenStream2::new();
    {
        use quote::TokenStreamExt;
        tokens.append_all(rust);
    }
    tokens
}

/// Creates RustAST from GRust file using nightly funtion.
#[cfg(feature = "no_diagnostics")]
pub fn into_token_stream(ast: Ast, ctx: &mut ir0::Ctx) -> TokenStream2 {
    let mut stats = Stats::new();
    let ir1 = match ir1::from_ast_timed(ast, ctx, stats.as_mut()) {
        Ok(pair) => pair,
        Err(errors) => {
            for error in errors {
                panic!("compilation error detected: {}", error.0);
            }
            return parse_quote! {};
        }
    };
    if let Some(filepath) = &ctx.conf.dump_graph {
        ir1.dump_graph(filepath.value(), ctx);
    }
    let ir2 = stats.timed("ir1 → ir2", || ir1.into_ir2(ctx));
    let rust = stats.timed("codegen (ir2 → rust tokens)", || {
        ir2.prepare_tokens(ctx).to_token_stream()
    });
    if let Some(stats) = stats.pretty(&ctx.conf) {
        println!("Stats:\n\n{}", stats);
    }
    let mut tokens = TokenStream2::new();
    {
        use quote::TokenStreamExt;
        tokens.append_all(rust);
    }
    tokens
}

/// Writes the generated code at the given filepath.
pub fn dump_code(path_lit: &syn::LitStr, tokens: &TokenStream2) -> Res<()> {
    use std::{fs::OpenOptions, io::Write, path::Path, process::Command};
    let path_str = path_lit.value();
    let path = Path::new(&path_str);

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| error!( @path_lit.span() => "failed to open this file: {}", e ))?;
        let content = tokens.to_string();
        for line in content.lines() {
            writeln!(&mut file, "{}", line).map_err(|e| {
                error!( @path_lit.span() =>
                    "failed to write to this file: {}", e
                )
            })?;
        }
        file.flush().map_err(|e| {
            error!( @path_lit.span() =>
                "failed to write to this file: {}", e
            )
        })?;
    }

    let mut rustfmt = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(path)
        .spawn()
        .map_err(|e| {
            error!( @path_lit.span() =>
                "rust fmt failed to spawn: {}", e
            )
        })?;
    rustfmt.wait().map_err(|e| {
        error!( @path_lit.span() =>
            "rust fmt terminated with an error: {}", e
        )
    })?;

    Ok(())
}
