use syn::punctuated::Punctuated;
use syn::{token, LitInt, Token};

use super::keyword;
use crate::ast::{contract::Contract, equation::Equation};
use crate::common::r#type::Type;

#[derive(Debug, PartialEq, Clone)]
/// GRust component AST.
pub struct Component {
    pub node_token: keyword::component,
    /// Component identifier.
    pub ident: syn::Ident,
    pub args_paren: token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: Punctuated<(syn::Ident, Token![:], Type), Token![,]>,
    pub arrow_token: Token![->],
    pub outs_paren: token::Paren,
    /// Component's outputs identifiers and their types.
    pub outs: Punctuated<(syn::Ident, Token![:], Type), Token![,]>,
    /// Component's computation period.
    pub period: Option<(Token![@], LitInt, keyword::ms)>,
    /// Component's contract.
    pub contract: Contract,
    pub brace: token::Brace,
    /// Component's equations.
    pub equations: Vec<Equation>,
}
