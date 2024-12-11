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
impl HasLoc for Typed {
    fn loc(&self) -> Loc {
        self.ident.loc()
    }
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
    pub loc: Loc,
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
impl HasLoc for Tuple {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl Tuple => new {
    loc: impl Into<Loc> = loc.into(),
    elements: Vec<Pattern>,
} }

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
impl HasLoc for Pattern {
    fn loc(&self) -> Loc {
        match self {
            Self::Identifier(id) => id.loc(),
            Self::Typed(t) => t.loc(),
            Self::Tuple(t) => t.loc(),
        }
    }
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
impl<E> HasLoc for LetDecl<E> {
    fn loc(&self) -> Loc {
        Loc::from(self.let_token.span).join(self.semi_token.span)
    }
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
