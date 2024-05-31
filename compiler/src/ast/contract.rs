use syn::{braced, parse::Parse, token, Token};

use crate::common::{
    constant::Constant,
    operator::{BinaryOperator, UnaryOperator},
};

use super::{expression::ParsePrec, keyword};

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
/// Unary term.
pub struct Unary {
    pub op: UnaryOperator,
    pub term: Box<Term>,
}
impl Unary {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        UnaryOperator::peek(input)
    }
}
impl Parse for Unary {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op: UnaryOperator = input.parse()?;
        let term: Box<Term> = Box::new(input.parse()?);
        Ok(Unary { op, term })
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Binary term.
pub struct Binary {
    pub left: Box<Term>,
    pub op: BinaryOperator,
    pub right: Box<Term>,
}
impl Binary {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        BinaryOperator::peek(input)
    }
    pub fn parse_term(left: Box<Term>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_term(input)?);
        Ok(Binary { op, left, right })
    }
    pub fn parse_prec1(left: Box<Term>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec1(input)?);
        Ok(Binary { op, left, right })
    }
    pub fn parse_prec2(left: Box<Term>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec2(input)?);
        Ok(Binary { op, left, right })
    }
    pub fn parse_prec3(left: Box<Term>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec3(input)?);
        Ok(Binary { op, left, right })
    }
    pub fn parse_prec4(left: Box<Term>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec4(input)?);
        Ok(Binary { op, left, right })
    }
}
impl Parse for Binary {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left: Box<Term> = Box::new(input.parse()?);
        let op: BinaryOperator = input.parse()?;
        let right: Box<Term> = Box::new(input.parse()?);
        Ok(Binary { left, op, right })
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust clause's term.
pub enum Term {
    Implication(Implication),
    Unary(Unary),
    Binary(Binary),
    Constant(Constant),
    Identifier(syn::Ident),
}
impl ParsePrec for Term {
    fn parse_term(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let term = if input.fork().call(Constant::parse).is_ok() {
            Term::Constant(input.parse()?)
        } else if Unary::peek(input) {
            Term::Unary(input.parse()?)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            Term::Identifier(input.parse()?)
        } else {
            return Err(input.error("expected expression"));
        };

        Ok(term)
    }

    fn parse_prec1(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut term = Term::parse_term(input)?;

        loop {
            if BinaryOperator::peek_prec1(input) {
                term = Term::Binary(Binary::parse_term(Box::new(term), input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }

    fn parse_prec2(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut term = Term::parse_prec1(input)?;

        loop {
            if BinaryOperator::peek_prec2(input) {
                term = Term::Binary(Binary::parse_prec1(Box::new(term), input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }

    fn parse_prec3(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut term = Term::parse_prec2(input)?;

        loop {
            if BinaryOperator::peek_prec3(input) {
                term = Term::Binary(Binary::parse_prec2(Box::new(term), input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }

    fn parse_prec4(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut term = Term::parse_prec3(input)?;

        loop {
            if BinaryOperator::peek_prec4(input) {
                term = Term::Binary(Binary::parse_prec3(Box::new(term), input)?);
            } else {
                break;
            }
        }

        loop {
            if input.peek(Token![=>]) {
                term = Term::Implication(Implication::parse(input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }
}
impl Parse for Term {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Term::parse_prec4(input)
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
