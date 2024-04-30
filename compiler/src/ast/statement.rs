use syn::parse::Parse;
use syn::Token;

use crate::ast::expression::Expression;
use crate::common::r#type::Type;

use super::ident_colon::IdentColon;

/// GRust statement AST.
pub struct Statement {
    pub let_token: Token![let],
    /// Identifier of the variable and its type.
    pub typed_ident: IdentColon<Type>,
    pub eq_token: Token![=],
    /// The expression defining the variable.
    pub expression: Expression,
    pub semi_token: Token![;],
}
impl Parse for Statement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let let_token: Token![let] = input.parse()?;
        let typed_ident: IdentColon<Type> = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: Expression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Statement {
            let_token,
            typed_ident,
            eq_token,
            expression,
            semi_token,
        })
    }
}
