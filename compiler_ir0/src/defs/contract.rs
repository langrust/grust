//! Contracts for the AST.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// For all term.
pub struct ForAll {
    pub forall_token: keyword::forall,
    pub ident: String,
    pub colon_token: Token![:],
    pub ty: Typ,
    pub comma_token: Token![,],
    pub term: Box<Term>,
}

mk_new! { impl ForAll =>
    new {
        forall_token: keyword::forall,
        ident: impl Into<String> = ident.into(),
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
    pub pattern: String,
    pub eq_token: Token![=],
    /// The event to match.
    pub event: String,
    pub question_token: Token![?],
    pub arrow: Token![=>],
    pub term: Box<Term>,
}

mk_new! { impl EventImplication =>
    new {
        when_token: keyword::when,
        pattern: impl Into<String> = pattern.into(),
        eq_token: Token![=],
        event: impl Into<String> = event.into(),
        question_token: Token![?],
        arrow: Token![=>],
        term: Term = term.into(),
    }
}

/// Enumeration term.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration {
    /// The enumeration type name.
    pub enum_name: String,
    /// The element name.
    pub elem_name: String,
}
mk_new! { impl Enumeration =>
    new {
        enum_name: impl Into<String> = enum_name.into(),
        elem_name: impl Into<String> = elem_name.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Unary term.
pub struct Unary {
    pub op: UOp,
    pub term: Box<Term>,
}

mk_new! { impl Unary =>
    new {
        op: UOp,
        term: Term = term.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Binary term.
pub struct Binary {
    pub left: Box<Term>,
    pub op: BOp,
    pub right: Box<Term>,
}

mk_new! { impl Binary =>
    new {
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
    Identifier(String),
    Enumeration(Enumeration),
    Unary(Unary),
    Binary(Binary),
    ForAll(ForAll),
    Implication(Implication),
    EventImplication(EventImplication),
}

mk_new! { impl Term =>
    Constant: constant (val: Constant = val)
    Result: result (val: keyword::result = val)
    Identifier: ident (val: impl Into<String> = val.into())
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
