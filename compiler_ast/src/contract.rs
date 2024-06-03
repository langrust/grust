//! Contracts for the AST.

prelude! {
    syn::{braced, parse::Parse, token, Token},
    operator::{BinaryOperator, UnaryOperator},
    expr::ParsePrec,
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

impl Implication {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![=>])
    }
    fn parse(input: syn::parse::ParseStream, left: Term) -> syn::Result<Self> {
        let arrow: Token![=>] = input.parse()?;
        let right: Term = input.parse()?;
        Ok(Implication::new(left, arrow, right))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Event presence implication term.
pub struct EventImplication {
    /// The pattern receiving the value of the event.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The event to match.
    pub event: String,
    pub question_token: Token![?],
    pub arrow: Token![=>],
    pub term: Box<Term>,
}

mk_new! { impl EventImplication =>
    new {
        pattern: Pattern,
        eq_token: Token![=],
        event: impl Into<String> = event.into(),
        question_token: Token![?],
        arrow: Token![=>],
        term: Term = term.into(),
    }
}

impl EventImplication {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input
            .fork()
            .call(|forked| {
                let _: Pattern = forked.parse()?;
                let _: token::Eq = forked.parse()?;
                let _: syn::Ident = forked.parse()?;
                Ok(())
            })
            .is_ok()
    }
}
impl Parse for EventImplication {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern: Pattern = input.parse()?;
        let eq_token: token::Eq = input.parse()?;
        let event: syn::Ident = input.parse()?;
        let question_token: token::Question = input.parse()?;
        let arrow: Token![=>] = input.parse()?;
        let term: Term = Term::parse_prec4(input)?;
        Ok(EventImplication::new(
            pattern,
            eq_token,
            event.to_string(),
            question_token,
            arrow,
            term,
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Event timeout implication term.
pub struct TimeoutImplication {
    /// The timeout.
    pub timeout_token: keyword::timeout,
    /// The event to match.
    pub event: String,
    pub arrow: Token![=>],
    pub term: Box<Term>,
}

mk_new! { impl TimeoutImplication =>
    new {
        timeout_token: keyword::timeout,
        event: impl Into<String> = event.into(),
        arrow: Token![=>],
        term: Term = term.into(),
    }
}

impl TimeoutImplication {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::timeout)
    }
}
impl Parse for TimeoutImplication {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let timeout_token: keyword::timeout = input.parse()?;
        let event: syn::Ident = input.parse()?;
        let arrow: Token![=>] = input.parse()?;
        let term: Term = Term::parse_prec4(input)?;
        Ok(TimeoutImplication::new(
            timeout_token,
            event.to_string(),
            arrow,
            term,
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Unary term.
pub struct Unary {
    pub op: UnaryOperator,
    pub term: Box<Term>,
}

mk_new! { impl Unary =>
    new {
        op: UnaryOperator,
        term: Term = term.into(),
    }
}

impl Unary {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        UnaryOperator::peek(input)
    }
}
impl Parse for Unary {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op: UnaryOperator = input.parse()?;
        let term: Term = Term::parse_term(input)?;
        Ok(Unary::new(op, term))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// Binary term.
pub struct Binary {
    pub left: Box<Term>,
    pub op: BinaryOperator,
    pub right: Box<Term>,
}

mk_new! { impl Binary =>
    new {
        left: Term = left.into(),
        op: BinaryOperator,
        right: Term = right.into(),
    }
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
    Constant(Constant),
    Identifier(String),
    Unary(Unary),
    Binary(Binary),
    Implication(Implication),
    EventImplication(EventImplication),
    TimeoutImplication(TimeoutImplication),
}

mk_new! { impl Term =>
    Constant: constant (val: Constant = val)
    Identifier: ident (val: impl Into<String> = val.into())
    Unary: unary (val: Unary = val)
    Binary: binary (val: Binary = val)
    Implication: implication (val: Implication = val)
    EventImplication: event (val: EventImplication = val)
    TimeoutImplication: timeout (val: TimeoutImplication = val)
}

impl ParsePrec for Term {
    fn parse_term(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let term = if input.fork().call(Constant::parse).is_ok() {
            Term::constant(input.parse()?)
        } else if Unary::peek(input) {
            Term::unary(input.parse()?)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Term::ident(ident.to_string())
        } else {
            return Err(input.error("expected expression"));
        };

        Ok(term)
    }

    fn parse_prec1(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut term = Term::parse_term(input)?;

        loop {
            if BinaryOperator::peek_prec1(input) {
                term = Term::binary(Binary::parse_term(Box::new(term), input)?);
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
                term = Term::binary(Binary::parse_prec1(Box::new(term), input)?);
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
                term = Term::binary(Binary::parse_prec2(Box::new(term), input)?);
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
                term = Term::binary(Binary::parse_prec3(Box::new(term), input)?);
            } else {
                break;
            }
        }

        Ok(term)
    }
}
impl Parse for Term {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut term = if TimeoutImplication::peek(input) {
            Self::timeout(input.parse()?)
        } else if EventImplication::peek(input) {
            Self::event(input.parse()?)
        } else {
            Self::parse_prec4(input)?
        };

        loop {
            if Implication::peek(input) {
                term = Term::implication(Implication::parse(input, term)?);
            } else {
                break;
            }
        }

        Ok(term)
    }
}

#[cfg(test)]
mod parse_term {
    prelude! {
        contract::*,
        operator::BinaryOperator,
    }

    #[test]
    fn should_parse_constant() {
        let term: Term = syn::parse_quote! {1};
        let control = Term::constant(Constant::int(syn::parse_quote! {1}));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_identifier() {
        let term: Term = syn::parse_quote! {x};
        let control = Term::ident("x");
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_unary_operation() {
        let term: Term = syn::parse_quote! {!x};
        let control = Term::unary(Unary::new(UnaryOperator::Not, Term::ident("x")));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_binary_operation() {
        let term: Term = syn::parse_quote! {-x + 1};
        let control = Term::binary(Binary::new(
            Term::unary(Unary::new(UnaryOperator::Neg, Term::ident("x"))),
            BinaryOperator::Add,
            Term::constant(Constant::int(syn::parse_quote! {1})),
        ));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_implication() {
        let term: Term = syn::parse_quote! { !x && y => z};
        let control = Term::implication(Implication::new(
            Term::binary(Binary::new(
                Term::unary(Unary::new(UnaryOperator::Not, Term::ident("x"))),
                BinaryOperator::And,
                Term::ident("y"),
            )),
            Default::default(),
            Term::ident("z"),
        ));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_event_implication() {
        let term: Term = syn::parse_quote! { d = p? => d > x+y};
        let control = Term::event(EventImplication::new(
            Pattern::ident("d"),
            Default::default(),
            "p",
            Default::default(),
            Default::default(),
            Term::binary(Binary::new(
                Term::ident("d"),
                BinaryOperator::Grt,
                Term::binary(Binary::new(
                    Term::ident("x"),
                    BinaryOperator::Add,
                    Term::ident("y"),
                )),
            )),
        ));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_timeout_implication() {
        let term: Term = syn::parse_quote! { timeout p => s == x+y};
        let control = Term::timeout(TimeoutImplication::new(
            Default::default(),
            "p",
            Default::default(),
            Term::binary(Binary::new(
                Term::ident("s"),
                BinaryOperator::Eq,
                Term::binary(Binary::new(
                    Term::ident("x"),
                    BinaryOperator::Add,
                    Term::ident("y"),
                )),
            )),
        ));
        assert_eq!(term, control)
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
