use syn::parse::Parse;
use syn::Token;

use crate::ast::stream_expression::StreamExpression;
use crate::common::r#type::Type;

use super::ident_colon::IdentColon;

pub struct LetDeclaration {
    pub let_token: Token![let],
    /// Identifier of the signal and its type.
    pub typed_ident: IdentColon<Type>,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    pub semi_token: Token![;],
}

pub struct Instanciation {
    /// Identifier of the signal.
    pub ident: syn::Ident,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    pub semi_token: Token![;],
}

/// GRust equation AST.
pub enum Equation {
    LocalDef(LetDeclaration),
    OutputDef(Instanciation),
}
impl Parse for Equation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let]) {
            let let_token: Token![let] = input.parse()?;
            let typed_ident: IdentColon<Type> = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let expression: StreamExpression = input.parse()?;
            let semi_token: Token![;] = input.parse()?;

            Ok(Equation::LocalDef(LetDeclaration {
                let_token,
                typed_ident,
                eq_token,
                expression,
                semi_token,
            }))
        } else {
            let ident: syn::Ident = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let expression: StreamExpression = input.parse()?;
            let semi_token: Token![;] = input.parse()?;

            Ok(Equation::OutputDef(Instanciation {
                ident,
                eq_token,
                expression,
                semi_token,
            }))
        }
    }
}
