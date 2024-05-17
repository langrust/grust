use syn::parse::Parse;
use syn::Token;

use crate::ast::{expression::Expression, pattern::Pattern};

/// GRust declaration AST.
pub struct LetDeclaration<E> {
    pub let_token: Token![let],
    /// Pattern of instanciated signals and its type.
    pub typed_pattern: Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: E,
    pub semi_token: Token![;],
}
impl<E> Parse for LetDeclaration<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let let_token: Token![let] = input.parse()?;
        let typed_pattern: Pattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: E = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(LetDeclaration {
            let_token,
            typed_pattern,
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
    pub semi_token: Token![;],
}
impl Parse for ReturnInstruction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let return_token: Token![return] = input.parse()?;
        let expression: Expression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(ReturnInstruction {
            return_token,
            expression,
            semi_token,
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
