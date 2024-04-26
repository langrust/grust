use syn::Token;

use crate::ast::stream_expression::StreamExpression;

#[derive(Debug, PartialEq, Clone)]
pub struct LetDeclaration {
    pub let_token: Token![let],
    /// Identifier of the signal and its type.
    pub typed_ident: syn::PatType,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Instanciation {
    /// Identifier of the signal.
    pub ident: syn::Ident,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
}

#[derive(Debug, PartialEq, Clone)]
/// GRust equation AST.
pub enum Equation {
    LocalDef(LetDeclaration),
    OutputDef(Instanciation),
}
