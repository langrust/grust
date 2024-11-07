prelude! {
    syn::{Parse, punctuated::Punctuated, token},
}

/// Typed pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct Typed {
    /// The ident.
    pub ident: Ident,
    /// The colon token.
    pub colon_token: Token![:],
    /// The type.
    pub typ: Typ,
}
mk_new! { impl Typed =>
    new {
        ident: Ident = ident,
        colon_token: Token![:],
        typ: Typ,
    }
}
impl Typed {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![:])
    }

    pub fn parse(ident: Ident, input: ParseStream) -> syn::Res<Self> {
        let colon_token: Token![:] = input.parse()?;
        let typ = input.parse()?;
        Ok(Typed {
            ident,
            colon_token,
            typ,
        })
    }
}

/// Tuple pattern that matches tuples.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
mk_new! { impl Tuple =>
    new { elements: Vec<Pattern> }
}
impl Tuple {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(token::Paren)
    }
}
impl Parse for Tuple {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Tuple {
            elements: elements.into_iter().collect(),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub enum Pattern {
    /// Identifier pattern.
    Identifier(Ident),
    /// Typed pattern.
    Typed(Typed),
    /// Tuple pattern that matches tuples.
    Tuple(Tuple),
}
impl Pattern {
    mk_new! {
        Identifier: ident(ident: Ident = ident)
        Typed: typed(t: Typed = t)
        Tuple: tuple(t: Tuple = t)
    }
}
impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let pattern = if Tuple::peek(input) {
            Pattern::Tuple(input.parse()?)
        } else {
            let ident: Ident = input.parse()?;
            if Typed::peek(input) {
                Pattern::Typed(Typed::parse(ident, input)?)
            } else {
                Pattern::ident(ident)
            }
        };

        Ok(pattern)
    }
}

/// GRust declaration AST.
pub struct LetDecl<E> {
    pub let_token: Token![let],
    /// Pattern of instantiated signals and their type.
    pub typed_pattern: Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expr: E,
    pub semi_token: Token![;],
}

mk_new! { impl{E} LetDecl<E> =>
    new {
        let_token: Token![let],
        typed_pattern: Pattern,
        eq_token: Token![=],
        expr: E,
        semi_token: Token![;],
    }
}

impl<E> Parse for LetDecl<E>
where
    E: Parse,
{
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let let_token: Token![let] = input.parse()?;
        let typed_pattern: Pattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let expr: E = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(LetDecl {
            let_token,
            typed_pattern,
            eq_token,
            expr,
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
    fn parse(input: ParseStream) -> syn::Res<Self> {
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
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![let]) {
            Ok(Stmt::Declaration(input.parse()?))
        } else {
            Ok(Stmt::Return(input.parse()?))
        }
    }
}
