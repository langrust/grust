prelude! {
    syn::{Punctuated, Parse, token},
    stmt::Pattern, stmt::LetDecl,
}

use super::keyword;

pub struct Instantiation<E> {
    /// Pattern of instantiated signals.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signals.
    pub expr: E,
    pub semi_token: Token![;],
}

mk_new! { impl{E} Instantiation<E> =>
    new {
        pattern: Pattern,
        eq_token: Token![=],
        expr: E,
        semi_token: Token![;],
    }
}

impl<E: Parse> Parse for Instantiation<E> {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let pattern: Pattern = input.parse()?;
        let eq: Token![=] = input.parse()?;
        let expr: E = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Instantiation::new(pattern, eq, expr, semi_token))
    }
}

/// Arm for matching expression.
pub struct Arm {
    /// The pattern to match.
    pub pattern: expr::Pattern,
    /// The optional guard.
    pub guard: Option<(Token![if], stream::Expr)>,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Eq>,
}

mk_new! { impl Arm =>
    new {
        pattern: expr::Pattern,
        guard: Option<(Token![if], stream::Expr)>,
        arrow_token: Token![=>],
        brace_token: token::Brace,
        equations: Vec<Eq>,
    }
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let pattern = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let token = input.parse()?;
                let guard = input.parse()?;
                Some((token, guard))
            } else {
                None
            }
        };
        let arrow = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let equations = {
            let mut equations = Vec::new();
            while !content.is_empty() {
                equations.push(content.parse()?);
            }
            equations
        };
        Ok(Arm::new(pattern, guard, arrow, brace, equations))
    }
}

pub struct Match {
    pub match_token: Token![match],
    /// The stream expression defining the signals.
    pub expr: stream::Expr,
    pub brace_token: token::Brace,
    /// The different matching cases.
    pub arms: Punctuated<Arm, Token![,]>,
}

mk_new! { impl Match =>
    new {
        match_token: Token![match],
        expr: stream::Expr,
        brace_token: token::Brace,
        arms: Punctuated<Arm, Token![,]>,
    }
}

impl Parse for Match {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let match_token = input.parse()?;
        let expr = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let arms: Punctuated<Arm, Token![,]> = Punctuated::parse_terminated(&content)?;

        Ok(Match::new(match_token, expr, brace, arms))
    }
}

/// GRust simpl equation AST.
pub enum Eq {
    LocalDef(LetDecl<stream::Expr>),
    OutputDef(Instantiation<stream::Expr>),
    Match(Match),
}
mk_new! { impl Eq =>
    LocalDef: local_def(e: LetDecl<stream::Expr> = e)
    OutputDef: out_def(i: Instantiation<stream::Expr> = i)
    Match: pat_match(m : Match = m)
}
impl Parse for Eq {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![match]) {
            Ok(Eq::pat_match(input.parse()?))
        } else if input.peek(Token![let]) {
            Ok(Eq::local_def(input.parse()?))
        } else {
            Ok(Eq::out_def(input.parse()?))
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct TupleEventPattern {
    pub paren_token: token::Paren,
    /// The activated patterns.
    pub patterns: Punctuated<EventPattern, Token![,]>,
}
mk_new! { impl TupleEventPattern =>
    new {
        paren_token: token::Paren,
        patterns: Punctuated<EventPattern, Token![,]>,
    }
}
impl Parse for TupleEventPattern {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let patterns = Punctuated::parse_terminated(&content)?;
        Ok(TupleEventPattern::new(paren_token, patterns))
    }
}

#[derive(PartialEq, Clone)]
pub struct LetEventPattern {
    pub let_token: Token![let],
    /// The pattern receiving the value of the event.
    pub pattern: expr::Pattern,
    pub eq_token: Token![=],
    /// The event to match.
    pub event: Ident,
    pub question_token: Token![?],
}
mk_new! { impl LetEventPattern =>
    new {
        let_token: Token![let],
        pattern: expr::Pattern,
        eq_token: Token![=],
        event: Ident,
        question_token: Token![?],
    }
}
impl Parse for LetEventPattern {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let let_token = input.parse()?;
        let pattern = input.parse()?;
        let eq_token = input.parse()?;
        let event = input.parse()?;
        let question_token = input.parse()?;
        Ok(LetEventPattern::new(
            let_token,
            pattern,
            eq_token,
            event,
            question_token,
        ))
    }
}

#[derive(PartialEq, Clone)]
pub enum EventPattern {
    Tuple(TupleEventPattern),
    Let(LetEventPattern),
    RisingEdge(Box<stream::Expr>),
}
impl Parse for EventPattern {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(token::Paren) {
            Ok(EventPattern::Tuple(input.parse()?))
        } else if input.peek(Token![let]) {
            Ok(EventPattern::Let(input.parse()?))
        } else {
            let forked = input.fork();
            let is_event = forked
                .parse::<Ident>()
                .is_ok_and(|_| forked.parse::<token::Question>().is_ok());
            if is_event {
                let event: Ident = input.parse()?;
                let question_token: token::Question = input.parse()?;
                let span = event.span();
                let let_token = token::Let { span };
                let pattern = expr::Pattern::ident(event.to_string());
                let eq_token = token::Eq { spans: [span] };
                let pat = LetEventPattern::new(let_token, pattern, eq_token, event, question_token);
                Ok(EventPattern::Let(pat))
            } else {
                Ok(EventPattern::RisingEdge(input.parse()?))
            }
        }
    }
}
impl std::fmt::Debug for EventPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tuple(arg0) => f
                .debug_tuple("Tuple")
                .field(&arg0.patterns.iter().collect::<Vec<_>>())
                .finish(),
            Self::Let(arg0) => f
                .debug_tuple("Let")
                .field(&(&arg0.pattern, &arg0.event))
                .finish(),
            Self::RisingEdge(arg0) => f.debug_tuple("RisingEdge").field(&arg0).finish(),
        }
    }
}

/// EventArmWhen for matching event.
pub struct EventArmWhen {
    pub pattern: EventPattern,
    /// The optional guard.
    pub guard: Option<(Token![if], stream::Expr)>,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Eq>,
}
mk_new! { impl EventArmWhen =>
    new {
        pattern: EventPattern,
        guard: Option<(Token![if], stream::Expr)>,
        arrow_token: Token![=>],
        brace_token: token::Brace,
        equations: Vec<Eq>,
    }
}

impl Parse for EventArmWhen {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let pat = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let token = input.parse()?;
                let guard = input.parse()?;
                Some((token, guard))
            } else {
                None
            }
        };
        let arrow = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let equations = {
            let mut equations = Vec::new();
            while !content.is_empty() {
                equations.push(content.parse()?);
            }
            equations
        };
        Ok(EventArmWhen::new(pat, guard, arrow, brace, equations))
    }
}

pub struct MatchWhen {
    pub when_token: keyword::when,
    pub brace_token: token::Brace,
    /// The different matching cases.
    pub arms: Vec<EventArmWhen>,
}
mk_new! { impl MatchWhen =>
    new {
        when_token: keyword::when,
        brace_token: token::Brace,
        arms: Vec<EventArmWhen>,
    }
}
impl Parse for MatchWhen {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let when_token = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let mut arms: Vec<EventArmWhen> = vec![];
        while !content.is_empty() {
            arms.push(content.parse()?);
        }

        Ok(MatchWhen::new(when_token, brace, arms))
    }
}

/// GRust reactive equation AST.
pub enum ReactEq {
    LocalDef(LetDecl<stream::ReactExpr>),
    OutputDef(Instantiation<stream::ReactExpr>),
    MatchWhen(MatchWhen),
    Match(Match),
}
mk_new! { impl ReactEq =>
    LocalDef: local_def(e: LetDecl<stream::ReactExpr> = e)
    OutputDef: out_def(i: Instantiation<stream::ReactExpr> = i)
    MatchWhen: match_when(m : MatchWhen = m)
    Match: pat_match(m : Match = m)
}
impl Parse for ReactEq {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![match]) {
            Ok(ReactEq::pat_match(input.parse()?))
        } else if input.peek(keyword::when) {
            Ok(ReactEq::match_when(input.parse()?))
        } else if input.peek(Token![let]) {
            Ok(ReactEq::local_def(input.parse()?))
        } else {
            Ok(ReactEq::out_def(input.parse()?))
        }
    }
}

#[cfg(test)]
mod parse_equation {
    use std::fmt::Debug;

    use super::*;

    prelude! { just
        expr::{Binop, IfThenElse, Tuple},
    }

    use super::{Eq, ReactEq};

    impl PartialEq for Eq {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::LocalDef(l0), Self::LocalDef(r0)) => {
                    l0.expr == r0.expr && l0.typed_pattern == r0.typed_pattern
                }
                (Self::OutputDef(l0), Self::OutputDef(r0)) => {
                    l0.expr == r0.expr && l0.pattern == r0.pattern
                }
                (Self::Match(l0), Self::Match(r0)) => {
                    l0.expr == r0.expr
                        && l0.arms.iter().zip(r0.arms.iter()).all(|(l0, r0)| {
                            l0.pattern == r0.pattern
                                && l0.guard == r0.guard
                                && l0.equations == r0.equations
                        })
                }
                _ => false,
            }
        }
    }
    impl Debug for Eq {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::LocalDef(arg0) => f
                    .debug_tuple("LocalDef")
                    .field(&arg0.typed_pattern)
                    .field(&arg0.expr)
                    .finish(),
                Self::OutputDef(arg0) => f
                    .debug_tuple("OutputDef")
                    .field(&arg0.pattern)
                    .field(&arg0.expr)
                    .finish(),
                Self::Match(arg0) => f
                    .debug_tuple("Match")
                    .field(&arg0.expr)
                    .field(
                        &arg0
                            .arms
                            .iter()
                            .map(|arm| {
                                (
                                    &arm.pattern,
                                    arm.guard.as_ref().map(|(_, expr)| expr),
                                    &arm.equations,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .finish(),
            }
        }
    }

    impl PartialEq for ReactEq {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::LocalDef(l0), Self::LocalDef(r0)) => {
                    l0.expr == r0.expr && l0.typed_pattern == r0.typed_pattern
                }
                (Self::OutputDef(l0), Self::OutputDef(r0)) => {
                    l0.expr == r0.expr && l0.pattern == r0.pattern
                }
                (Self::Match(l0), Self::Match(r0)) => {
                    l0.expr == r0.expr
                        && l0.arms.iter().zip(r0.arms.iter()).all(|(l0, r0)| {
                            l0.pattern == r0.pattern
                                && l0.guard == r0.guard
                                && l0.equations == r0.equations
                        })
                }
                (Self::MatchWhen(l0), Self::MatchWhen(r0)) => {
                    l0.arms.iter().zip(r0.arms.iter()).all(|(l0, r0)| {
                        l0.pattern == r0.pattern
                            && l0.guard == r0.guard
                            && l0.equations == r0.equations
                    })
                }
                _ => false,
            }
        }
    }
    impl Debug for ReactEq {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::LocalDef(arg0) => f
                    .debug_tuple("LocalDef")
                    .field(&arg0.typed_pattern)
                    .field(&arg0.expr)
                    .finish(),
                Self::OutputDef(arg0) => f
                    .debug_tuple("OutputDef")
                    .field(&arg0.pattern)
                    .field(&arg0.expr)
                    .finish(),
                Self::Match(arg0) => f
                    .debug_tuple("Match")
                    .field(&arg0.expr)
                    .field(
                        &arg0
                            .arms
                            .iter()
                            .map(|arm| {
                                (
                                    &arm.pattern,
                                    arm.guard.as_ref().map(|(_, expr)| expr),
                                    &arm.equations,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .finish(),
                Self::MatchWhen(arg0) => f
                    .debug_tuple("MatchWhen")
                    .field(
                        &arg0
                            .arms
                            .iter()
                            .map(|arm| {
                                (
                                    Some((&arm.pattern, arm.guard.as_ref().map(|(_, expr)| expr))),
                                    &arm.equations,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                    .finish(),
            }
        }
    }

    #[test]
    fn should_parse_output_definition() {
        let equation: ReactEq = parse_quote! {o = if res then 0 else (last o init 0) + inc;};
        let control = ReactEq::out_def(Instantiation {
            pattern: parse_quote! {o},
            eq_token: parse_quote! {=},
            expr: stream::ReactExpr::expr(stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::cst(Constant::int(parse_quote! {0})),
                stream::Expr::binop(Binop::new(
                    BOp::Add,
                    stream::Expr::last(stream::Last::new(
                        parse_quote! {o},
                        Some(stream::Expr::cst(Constant::int(parse_quote! {0}))),
                    )),
                    stream::Expr::ident("inc"),
                )),
            ))),
            semi_token: parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_tuple_instantiation() {
        let equation: ReactEq = parse_quote! {
            (o1, o2) = if res then (0, 0) else ((last o1 init 0) + inc1, last o2 + inc2);
        };
        let control = ReactEq::out_def(Instantiation {
            pattern: stmt::Pattern::tuple(stmt::Tuple::new(vec![
                parse_quote! {o1},
                parse_quote! {o2},
            ])),
            eq_token: parse_quote! {=},
            expr: stream::ReactExpr::expr(stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                ])),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::binop(Binop::new(
                        BOp::Add,
                        stream::Expr::last(stream::Last::new(
                            parse_quote! {o1},
                            Some(stream::Expr::cst(Constant::int(parse_quote! {0}))),
                        )),
                        stream::Expr::ident("inc1"),
                    )),
                    stream::Expr::binop(Binop::new(
                        BOp::Add,
                        stream::Expr::last(stream::Last::new(parse_quote! {o2}, None)),
                        stream::Expr::ident("inc2"),
                    )),
                ])),
            ))),
            semi_token: parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_local_definition() {
        let equation: ReactEq = parse_quote! {
            let o: int = if res then 0 else last o + inc;
        };
        let control = ReactEq::local_def(LetDecl::new(
            parse_quote!(let),
            stmt::Pattern::typed(stmt::Typed {
                ident: parse_quote!(o),
                colon_token: parse_quote!(:),
                typing: Typ::int(),
            }),
            parse_quote!(=),
            stream::ReactExpr::expr(stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::cst(Constant::int(parse_quote! {0})),
                stream::Expr::binop(Binop::new(
                    BOp::Add,
                    stream::Expr::last(stream::Last::new(parse_quote! {o}, None)),
                    stream::Expr::ident("inc"),
                )),
            ))),
            parse_quote! {;},
        ));
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_multiple_definitions() {
        let equation: ReactEq = parse_quote! {
            let (o1: int, o2: int) =
                if res then (0, 0) else ((last o1 init 0) + inc1, last o2 + inc2);
        };
        let control = ReactEq::local_def(LetDecl::new(
            parse_quote!(let),
            stmt::Pattern::tuple(stmt::Tuple::new(vec![
                stmt::Pattern::Typed(stmt::Typed {
                    ident: parse_quote!(o1),
                    colon_token: parse_quote!(:),
                    typing: Typ::int(),
                }),
                stmt::Pattern::Typed(stmt::Typed {
                    ident: parse_quote!(o2),
                    colon_token: parse_quote!(:),
                    typing: Typ::int(),
                }),
            ])),
            parse_quote!(=),
            stream::ReactExpr::expr(stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                ])),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::binop(Binop::new(
                        BOp::Add,
                        stream::Expr::last(stream::Last::new(
                            parse_quote! {o1},
                            Some(stream::Expr::cst(Constant::int(parse_quote! {0}))),
                        )),
                        stream::Expr::ident("inc1"),
                    )),
                    stream::Expr::binop(Binop::new(
                        BOp::Add,
                        stream::Expr::last(stream::Last::new(parse_quote! {o2}, None)),
                        stream::Expr::ident("inc2"),
                    )),
                ])),
            ))),
            parse_quote! {;},
        ));
        assert_eq!(equation, control)
    }
}
