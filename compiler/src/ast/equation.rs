use syn::parse::Parse;
use syn::Token;

use crate::ast::{statement::LetDeclaration, stream_expression::StreamExpression};

pub struct Instanciation {
    /// Identifier of the signal.
    pub ident: syn::Ident,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    pub semi_token: Token![;],
}
impl Parse for Instanciation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: StreamExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Instanciation {
            ident,
            eq_token,
            expression,
            semi_token,
        })
    }
}

/// GRust equation AST.
pub enum Equation {
    LocalDef(LetDeclaration<StreamExpression>),
    OutputDef(Instanciation),
}
impl Parse for Equation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let]) {
            Ok(Equation::LocalDef(input.parse()?))
        } else {
            Ok(Equation::OutputDef(input.parse()?))
        }
    }
}
