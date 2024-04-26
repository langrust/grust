use syn::{token, Token};

use super::keyword;

#[derive(Debug, PartialEq, Clone)]
/// GRust contract's term.
pub struct Implication {
    pub left: Box<Term>,
    pub arrow: Token![=>],
    pub right: Box<Term>,
}

#[derive(Debug, PartialEq, Clone)]
/// GRust term's kind.
pub enum Term {
    Expression(syn::Expr),
    Implication(Implication),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClauseKind {
    Requires(keyword::requires),
    Ensures(keyword::ensures),
    Invariant(keyword::invariant),
    Assert(keyword::assert),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Clause {
    pub kind: ClauseKind,
    pub brace: token::Brace,
    pub term: Term,
}

#[derive(Debug, Default, PartialEq, Clone)]
/// GRust contract to prove using Creusot.
pub struct Contract {
    /// Contract's clauses.
    pub clauses: Vec<Clause>,
}
