prelude! {}

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

/// Tuple pattern that matches tuples.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
mk_new! { impl Tuple =>
    new { elements: Vec<Pattern> }
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

/// GRust return statement AST.
pub struct Return {
    pub return_token: Token![return],
    /// The expression defining the variable.
    pub expression: Expr,
    pub semi_token: Token![;],
}

/// GRust statement AST.
pub enum Stmt {
    /// GRust declaration AST.
    Declaration(LetDecl<Expr>),
    /// GRust return statement AST.
    Return(Return),
}
