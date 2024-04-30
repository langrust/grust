use syn::{braced, parse::Parse, token, Token};

use super::keyword;

#[derive(Debug, PartialEq, Clone)]
/// Implication term.
pub struct Implication {
    pub left: Box<Term>,
    pub arrow: Token![=>],
    pub right: Box<Term>,
}
impl Implication {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(Term::parse).is_err() {
            return false;
        }
        forked.peek(Token![=>])
    }
}
impl Parse for Implication {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left: Box<Term> = Box::new(input.parse()?);
        let arrow: Token![=>] = input.parse()?;
        let right: Box<Term> = Box::new(input.parse()?);
        Ok(Implication { left, arrow, right })
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust clause's term.
pub enum Term {
    Implication(Implication),
    Expression(syn::Expr),
}
impl Parse for Term {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if Implication::peek(input) {
            Ok(Term::Implication(input.parse()?))
        } else {
            Ok(Term::Expression(input.parse()?))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// The kind of the clause.
pub enum ClauseKind {
    Requires(keyword::requires),
    Ensures(keyword::ensures),
    Invariant(keyword::invariant),
    Assert(keyword::assert),
}
impl ClauseKind {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::requires)
            || input.peek(keyword::ensures)
            || input.peek(keyword::invariant)
            || input.peek(keyword::assert)
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust clause.
pub struct Clause {
    pub kind: ClauseKind,
    pub brace: token::Brace,
    pub term: Term,
}
impl Parse for Clause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kind = {
            if input.peek(keyword::requires) {
                Ok(ClauseKind::Requires(input.parse()?))
            } else if input.peek(keyword::ensures) {
                Ok(ClauseKind::Ensures(input.parse()?))
            } else if input.peek(keyword::invariant) {
                Ok(ClauseKind::Invariant(input.parse()?))
            } else if input.peek(keyword::assert) {
                Ok(ClauseKind::Assert(input.parse()?))
            } else {
                Err(input.error("expected 'requires', 'ensures', 'invariant', or 'assert'"))
            }
        }?;
        let content;
        let brace = braced!(content in input);
        let term = content.parse()?;

        if content.is_empty() {
            Ok(Clause { kind, brace, term })
        } else {
            Err(content.error("expected term"))
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
/// GRust contract to prove using Creusot.
pub struct Contract {
    /// Contract's clauses.
    pub clauses: Vec<Clause>,
}
impl Parse for Contract {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let clauses = {
            let mut clauses = Vec::new();
            while ClauseKind::peek(input) {
                clauses.push(input.parse()?);
            }
            clauses
        };
        Ok(Contract { clauses })
    }
}
