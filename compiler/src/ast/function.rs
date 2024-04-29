use syn::punctuated::Punctuated;
use syn::{token, Token};

use crate::ast::{expression::Expression, statement::Statement};
use crate::common::r#type::Type;

#[derive(Debug, PartialEq)]
/// GRust function AST.
pub struct Function {
    /// Function identifier.
    pub ident: syn::Ident,
    pub args_paren: token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: Punctuated<(syn::Ident, Token![:], Type), Token![,]>,
    pub arrow_token: Token![->],
    pub output_type: (syn::Ident, Token![:], Type),
    pub brace: token::Brace,
    /// Function's statements.
    pub statements: Vec<Statement>,
    pub return_token: Token![return],
    /// Function's returned expression and its type.
    pub returned: Expression,
}
