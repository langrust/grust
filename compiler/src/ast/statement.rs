use syn::parse::Parse;
use syn::Token;

use crate::ast::{expression::Expression, ident_colon::IdentColon};
use crate::common::r#type::Type;

/// GRust declaration AST.
pub struct LetDeclaration<E> {
    pub let_token: Token![let],
    /// Identifier of the signal and its type.
    pub typed_ident: IdentColon<Type>,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: E,
    pub semi_token: Token![;],
}
impl<E> LetDeclaration<E> {
    pub fn get_ident(&self) -> &syn::Ident {
        &self.typed_ident.ident
    }
}
impl<E> Parse for LetDeclaration<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let let_token: Token![let] = input.parse()?;
        let typed_ident: IdentColon<Type> = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: E = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(LetDeclaration {
            let_token,
            typed_ident,
            eq_token,
            expression,
            semi_token,
        })
    }
}

/// GRust return statement AST.
pub struct ReturnInstruction {
    pub return_token: Token![return],
    /// The expression defining the variable.
    pub expression: Expression,
}
impl Parse for ReturnInstruction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let return_token: Token![return] = input.parse()?;
        let expression: Expression = input.parse()?;

        Ok(ReturnInstruction {
            return_token,
            expression,
        })
    }
}

/// GRust statement AST.
pub enum Statement {
    /// GRust declaration AST.
    Declaration(LetDeclaration<Expression>),
    /// GRust return statement AST.
    Return(ReturnInstruction),
}
impl Parse for Statement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let]) {
            Ok(Statement::Declaration(input.parse()?))
        } else {
            Ok(Statement::Return(input.parse()?))
        }
    }
}
