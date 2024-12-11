//! Contracts for the AST.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// For all term.
pub struct ForAll {
    pub forall_token: keyword::forall,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Typ,
    pub comma_token: Token![,],
    pub term: Box<Term>,
}
impl HasLoc for ForAll {
    fn loc(&self) -> Loc {
        Loc::from(self.forall_token.span).join(self.term.loc())
    }
}

mk_new! { impl ForAll =>
    new {
        forall_token: keyword::forall,
        ident: impl Into<Ident> = ident.into(),
        colon_token: Token![:],
        ty: Typ,
        comma_token: Token![,],
        term: Term = term.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Implication term.
pub struct Implication {
    pub left: Box<Term>,
    pub arrow: Token![=>],
    pub right: Box<Term>,
}
impl HasLoc for Implication {
    fn loc(&self) -> Loc {
        self.left.loc().join(self.right.loc())
    }
}

mk_new! { impl Implication =>
    new {
        left: Term = left.into(),
        arrow: Token![=>],
        right: Term = right.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Event presence implication term.
pub struct EventImplication {
    pub when_token: keyword::when,
    /// The pattern receiving the value of the event.
    pub pattern: Ident,
    pub eq_token: Token![=],
    /// The event to match.
    pub event: Ident,
    pub question_token: Token![?],
    pub arrow: Token![=>],
    pub term: Box<Term>,
}
impl HasLoc for EventImplication {
    fn loc(&self) -> Loc {
        Loc::from(self.when_token.span).join(self.term.loc())
    }
}

mk_new! { impl EventImplication =>
    new {
        when_token: keyword::when,
        pattern: impl Into<Ident> = pattern.into(),
        eq_token: Token![=],
        event: impl Into<Ident> = event.into(),
        question_token: Token![?],
        arrow: Token![=>],
        term: Term = term.into(),
    }
}

/// Enumeration term.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration {
    /// The enumeration type name.
    pub enum_name: Ident,
    /// The element name.
    pub elem_name: Ident,
}
impl HasLoc for Enumeration {
    fn loc(&self) -> Loc {
        self.enum_name.loc().join(self.elem_name.loc())
    }
}
mk_new! { impl Enumeration =>
    new {
        enum_name: impl Into<Ident> = enum_name.into(),
        elem_name: impl Into<Ident> = elem_name.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Unary term.
pub struct Unary {
    pub op_loc: Loc,
    pub op: UOp,
    pub term: Box<Term>,
}
impl HasLoc for Unary {
    fn loc(&self) -> Loc {
        self.op_loc.join(self.term.loc())
    }
}

mk_new! { impl Unary =>
    new {
        op_loc: impl Into<Loc> = op_loc.into(),
        op: UOp,
        term: Term = term.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Binary term.
pub struct Binary {
    pub op_loc: Loc,
    pub left: Box<Term>,
    pub op: BOp,
    pub right: Box<Term>,
}
impl HasLoc for Binary {
    fn loc(&self) -> Loc {
        self.left.loc().join(self.right.loc())
    }
}

mk_new! { impl Binary =>
    new {
        op_loc: impl Into<Loc> = op_loc.into(),
        left: Term = left.into(),
        op: BOp,
        right: Term = right.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust clause's term.
pub enum Term {
    Constant(Constant),
    Result(keyword::result),
    Identifier(Ident),
    Enumeration(Enumeration),
    Unary(Unary),
    Binary(Binary),
    ForAll(ForAll),
    Implication(Implication),
    EventImplication(EventImplication),
}
impl HasLoc for Term {
    fn loc(&self) -> Loc {
        match self {
            Self::Constant(c) => c.loc(),
            Self::Result(r) => r.span.into(),
            Self::Identifier(i) => i.loc(),
            Self::Enumeration(e) => e.loc(),
            Self::Unary(u) => u.loc(),
            Self::Binary(b) => b.loc(),
            Self::ForAll(f) => f.loc(),
            Self::Implication(i) => i.loc(),
            Self::EventImplication(ei) => ei.loc(),
        }
    }
}

mk_new! { impl Term =>
    Constant: constant (val: Constant = val)
    Result: result (val: keyword::result = val)
    Identifier: ident (val: impl Into<Ident> = val.into())
    Identifier: test_ident (
        val: impl AsRef<str> = Ident::new(val.as_ref(), Loc::test_dummy().into()),
    )
    Enumeration: enumeration (val: Enumeration = val)
    Unary: unary (val: Unary = val)
    Binary: binary (val: Binary = val)
    ForAll: forall (val: ForAll = val)
    Implication: implication (val: Implication = val)
    EventImplication: event (val: EventImplication = val)
}

#[derive(Debug, PartialEq, Clone)]
/// The kind of the clause.
pub enum ClauseKind {
    Requires(keyword::requires),
    Ensures(keyword::ensures),
    Invariant(keyword::invariant),
    Assert(keyword::assert),
}

#[derive(Debug, PartialEq, Clone)]
/// GRust clause.
pub struct Clause {
    pub kind: ClauseKind,
    pub brace: syn::token::Brace,
    pub term: Term,
}
mk_new! { impl Clause =>
    new {
        kind: ClauseKind,
        brace: syn::token::Brace,
        term: Term,
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
/// GRust contract to prove using Creusot.
pub struct Contract {
    /// Contract's clauses.
    pub clauses: Vec<Clause>,
}
mk_new! { impl Contract =>
    new {
        clauses: impl Into<Vec<Clause>> = clauses.into(),
    }
}
