//! Contracts for the AST.

prelude! {
    syn::{Parse, token},
    expr::ParsePrec,
}

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

impl ForAll {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::forall)
    }
}
impl Parse for ForAll {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let forall_token: keyword::forall = input.parse()?;
        let ident: Ident = input.parse()?;
        let colon_token: Token![:] = input.parse()?;
        let ty: Typ = input.parse()?;
        let comma_token: Token![,] = input.parse()?;
        let term: Term = input.parse()?;
        Ok(ForAll::new(
            forall_token,
            ident.to_string(),
            colon_token,
            ty,
            comma_token,
            term,
        ))
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

impl Implication {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![=>])
    }
    fn parse(input: ParseStream, left: Term) -> syn::Res<Self> {
        let arrow: Token![=>] = input.parse()?;
        let right: Term = input.parse()?;
        Ok(Implication::new(left, arrow, right))
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

impl EventImplication {
    fn peek(input: ParseStream) -> bool {
        input.peek(keyword::when)
    }
}
impl Parse for EventImplication {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let when_token: keyword::when = input.parse()?;
        let pattern: Ident = input.parse()?;
        let eq_token: token::Eq = input.parse()?;
        let event: Ident = input.parse()?;
        let question_token: token::Question = input.parse()?;
        let arrow: Token![=>] = input.parse()?;
        let term: Term = Term::parse_prec4(input)?;
        Ok(EventImplication::new(
            when_token,
            pattern.to_string(),
            eq_token,
            event.to_string(),
            question_token,
            arrow,
            term,
        ))
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
impl Enumeration {
    pub fn peek(input: ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(Ident::parse).is_err() {
            return false;
        }
        forked.peek(Token![::])
    }
}
impl Parse for Enumeration {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let ident_enum: Ident = input.parse()?;
        let _: Token![::] = input.parse()?;
        let ident_elem: Ident = input.parse()?;
        Ok(Enumeration::new(
            ident_enum.to_string(),
            ident_elem.to_string(),
        ))
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

impl Unary {
    fn peek(input: ParseStream) -> bool {
        UOp::peek(input)
    }
}
impl Parse for Unary {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let op: UOp = input.parse()?;
        let term: Term = Term::parse_term(input)?;
        Ok(Unary::new(op, term))
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

impl Binary {
    fn parse_term(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_term(input)?);
        Ok(Binary { op, left, right })
    }
    fn parse_prec1(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec1(input)?);
        Ok(Binary { op, left, right })
    }
    fn parse_prec2(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec2(input)?);
        Ok(Binary { op, left, right })
    }
    fn parse_prec3(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
        let op = input.parse()?;
        let right = Box::new(Term::parse_prec3(input)?);
        Ok(Binary { op, left, right })
    }
}
impl Parse for Binary {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let left: Box<Term> = Box::new(input.parse()?);
        let op: BOp = input.parse()?;
        let right: Box<Term> = Box::new(input.parse()?);
        Ok(Binary { left, op, right })
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

impl ParsePrec for Term {
    fn parse_term(input: ParseStream) -> syn::Res<Self> {
        let term = if input.peek(keyword::result) {
            Term::result(input.parse()?)
        } else if input.fork().call(Constant::parse).is_ok() {
            Term::constant(input.parse()?)
        } else if Enumeration::peek(input) {
            Term::enumeration(input.parse()?)
        } else if Unary::peek(input) {
            Term::unary(input.parse()?)
        } else if input.fork().call(Ident::parse).is_ok() {
            let ident: Ident = input.parse()?;
            Term::ident(ident.to_string())
        } else {
            return Err(input.error("expected expression"));
        };

        Ok(term)
    }

    fn parse_prec1(input: ParseStream) -> syn::Res<Self> {
        let mut term = Term::parse_term(input)?;

        loop {
            if BOp::peek_prec1(input) {
                term = Term::binary(Binary::parse_term(Box::new(term), input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }

    fn parse_prec2(input: ParseStream) -> syn::Res<Self> {
        let mut term = Term::parse_prec1(input)?;

        loop {
            if BOp::peek_prec2(input) {
                term = Term::binary(Binary::parse_prec1(Box::new(term), input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }

    fn parse_prec3(input: ParseStream) -> syn::Res<Self> {
        let mut term = Term::parse_prec2(input)?;

        loop {
            if BOp::peek_prec3(input) {
                term = Term::binary(Binary::parse_prec2(Box::new(term), input)?);
            } else {
                break;
            }
        }
        Ok(term)
    }

    fn parse_prec4(input: ParseStream) -> syn::Res<Self> {
        let mut term = Term::parse_prec3(input)?;

        loop {
            if BOp::peek_prec4(input) {
                term = Term::binary(Binary::parse_prec3(Box::new(term), input)?);
            } else {
                break;
            }
        }

        Ok(term)
    }
}
impl Parse for Term {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let mut term = if ForAll::peek(input) {
            Self::forall(input.parse()?)
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
    }

    #[test]
    fn should_parse_constant() {
        let term: Term = parse_quote! {1};
        let control = Term::constant(Constant::int(parse_quote! {1}));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_identifier() {
        let term: Term = parse_quote! {x};
        let control = Term::ident("x");
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_unary_operation() {
        let term: Term = parse_quote! {!x};
        let control = Term::unary(Unary::new(UOp::Not, Term::ident("x")));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_binary_operation() {
        let term: Term = parse_quote! {-x + 1};
        let control = Term::binary(Binary::new(
            Term::unary(Unary::new(UOp::Neg, Term::ident("x"))),
            BOp::Add,
            Term::constant(Constant::int(parse_quote! {1})),
        ));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_implication() {
        let term: Term = parse_quote! { !x && y => z};
        let control = Term::implication(Implication::new(
            Term::binary(Binary::new(
                Term::unary(Unary::new(UOp::Not, Term::ident("x"))),
                BOp::And,
                Term::ident("y"),
            )),
            Default::default(),
            Term::ident("z"),
        ));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_event_implication() {
        let term: Term = parse_quote! { when d = p? => d > x+y};
        let control = Term::event(EventImplication::new(
            Default::default(),
            "d",
            Default::default(),
            "p",
            Default::default(),
            Default::default(),
            Term::binary(Binary::new(
                Term::ident("d"),
                BOp::Grt,
                Term::binary(Binary::new(Term::ident("x"), BOp::Add, Term::ident("y"))),
            )),
        ));
        assert_eq!(term, control)
    }

    #[test]
    fn should_parse_forall() {
        let term: Term = parse_quote! { forall d: int, d > x+y};
        let control = Term::forall(ForAll::new(
            Default::default(),
            "d",
            Default::default(),
            Typ::int(),
            Default::default(),
            Term::binary(Binary::new(
                Term::ident("d"),
                BOp::Grt,
                Term::binary(Binary::new(Term::ident("x"), BOp::Add, Term::ident("y"))),
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
    fn peek(input: ParseStream) -> bool {
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

mk_new! { impl Clause =>
    new {
        kind: ClauseKind,
        brace: token::Brace,
        term: Term,
    }
}

impl Parse for Clause {
    fn parse(input: ParseStream) -> syn::Res<Self> {
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
            Ok(Clause::new(kind, brace, term))
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

mk_new! { impl Contract =>
    new {
        clauses: impl Into<Vec<Clause>> = clauses.into(),
    }
}

impl Parse for Contract {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let clauses = {
            let mut clauses = Vec::new();
            while ClauseKind::peek(input) {
                clauses.push(input.parse()?);
            }
            clauses
        };
        Ok(Contract::new(clauses))
    }
}
