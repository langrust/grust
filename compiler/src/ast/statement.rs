use syn::Token;

use crate::ast::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
/// GRust statement AST.
pub struct Statement {
    pub let_token: Token![let],
    /// Identifier of the variable and its type.
    pub typed_ident: syn::PatType,
    pub eq_token: Token![=],
    /// The expression defining the variable.
    pub expression: Expression,
    pub semi_token: Token![;],
}
