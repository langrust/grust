prelude! {
    syn::{parse::Parse, Token},
}

/// GRust declaration AST.
pub struct LetDecl<E> {
    pub let_token: Token![let],
    /// Pattern of instantiated signals and their type.
    pub typed_pattern: Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expression: E,
    pub semi_token: Token![;],
}

mk_new! { impl{E} LetDecl<E> =>
    new {
        let_token: Token![let],
        typed_pattern: Pattern,
        eq_token: Token![=],
        expression: E,
        semi_token: Token![;],
    }
}

impl<E> Parse for LetDecl<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let let_token: Token![let] = input.parse()?;
        let typed_pattern: Pattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expression: E = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(LetDecl {
            let_token,
            typed_pattern,
            eq_token,
            expression,
            semi_token,
        })
    }
}

/// GRust return statement AST.
pub struct Return {
    pub return_token: Token![return],
    /// The expression defining the variable.
    pub expression: Expr,
    pub semi_token: Token![;],
}
impl Parse for Return {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let return_token: Token![return] = input.parse()?;
        let expression: Expr = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Return {
            return_token,
            expression,
            semi_token,
        })
    }
}

/// GRust statement AST.
pub enum Stmt {
    /// GRust declaration AST.
    Declaration(LetDecl<Expr>),
    /// GRust return statement AST.
    Return(Return),
}
impl Parse for Stmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let]) {
            Ok(Stmt::Declaration(input.parse()?))
        } else {
            Ok(Stmt::Return(input.parse()?))
        }
    }
}
