//! This crate provides the [grust!] macro for generating Rust code
//! from GRust programs.

use compiler::{handle_tokens, TokenStream};

/// The GRust compiler as an integrated Rust macro.
///
/// Calls the GRust compiler and performs analysis that guarantee
/// the correct execution of the program.
///
/// The generated structures and functions are accessible immediately
/// in the Rust program, which eases the interface with other programs.
#[proc_macro]
pub fn grust(tokens: TokenStream) -> TokenStream {
    handle_tokens(tokens)
}
