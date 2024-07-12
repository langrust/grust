prelude! {
    syn::{
        punctuated::Punctuated, braced, Token, parse::Parse, token,
    },
    Pattern, stmt::LetDecl,
}

use super::keyword;

pub struct Instantiation {
    /// Pattern of instantiated signals.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signals.
    pub expression: stream::Expr,
    pub semi_token: Token![;],
}

mk_new! { impl Instantiation =>
    new {
        pattern: Pattern,
        eq_token: Token![=],
        expression: stream::Expr,
        semi_token: Token![;],
    }
}

impl Parse for Instantiation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern: Pattern = input.parse()?;
        let eq: Token![=] = input.parse()?;
        let expr: stream::Expr = input.parse()?;
        let semi_token: Token![;] = input.parse()?;

        Ok(Instantiation::new(pattern, eq, expr, semi_token))
    }
}

/// Arm for matching expression.
pub struct Arm {
    /// The pattern to match.
    pub pattern: Pattern,
    /// The optional guard.
    pub guard: Option<(Token![if], stream::Expr)>,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}

mk_new! { impl Arm =>
    new {
        pattern: Pattern,
        guard: Option<(Token![if], stream::Expr)>,
        arrow_token: Token![=>],
        brace_token: token::Brace,
        equations: Vec<Equation>,
    }
}

impl Parse for Arm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
    pub expression: stream::Expr,
    pub brace_token: token::Brace,
    /// The different matching cases.
    pub arms: Punctuated<Arm, Token![,]>,
}

mk_new! { impl Match =>
    new {
        match_token: Token![match],
        expression: stream::Expr,
        brace_token: token::Brace,
        arms: Punctuated<Arm, Token![,]>,
    }
}

impl Parse for Match {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_token = input.parse()?;
        let expr = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let arms: Punctuated<Arm, Token![,]> = Punctuated::parse_terminated(&content)?;

        Ok(Match::new(match_token, expr, brace, arms))
    }
}

/// EventArmWhen for matching event.
pub struct EventArmWhen {
    /// The pattern receiving the value of the event.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The event to match.
    pub event: syn::Ident,
    pub question_token: Token![?],
    /// The optional guard.
    pub guard: Option<(Token![if], stream::Expr)>,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}

mk_new! { impl EventArmWhen =>
    new {
        pattern: Pattern,
        eq_token: Token![=],
        event: syn::Ident,
        question_token: Token![?],
        guard: Option<(Token![if], stream::Expr)>,
        arrow_token: Token![=>],
        brace_token: token::Brace,
        equations: Vec<Equation>,
    }
}

impl Parse for EventArmWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pat = input.parse()?;
        let eq = input.parse()?;
        let event = input.parse()?;
        let question_token = input.parse()?;
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
        Ok(EventArmWhen::new(
            pat,
            eq,
            event,
            question_token,
            guard,
            arrow,
            brace,
            equations,
        ))
    }
}

/// EventArmWhen for matching event.
pub struct TimeoutArmWhen {
    /// The timeout.
    pub timeout_token: keyword::timeout,
    /// The event to match.
    pub event: syn::Ident,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}

mk_new! { impl TimeoutArmWhen =>
    new {
        timeout_token: keyword::timeout,
        event: syn::Ident,
        arrow_token: Token![=>],
        brace_token: token::Brace,
        equations: Vec<Equation>,
    }
}

impl Parse for TimeoutArmWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let timeout = input.parse()?;
        let event = input.parse()?;
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
        Ok(TimeoutArmWhen::new(timeout, event, arrow, brace, equations))
    }
}

/// DefaultArmWhen for absence of events.
pub struct DefaultArmWhen {
    pub otherwise_token: keyword::otherwise,
    pub arrow_token: Token![=>],
    pub brace_token: token::Brace,
    /// The equations.
    pub equations: Vec<Equation>,
}

mk_new! { impl DefaultArmWhen =>
    new {
        otherwise_token: keyword::otherwise,
        arrow_token: Token![=>],
        brace_token: token::Brace,
        equations: Vec<Equation>,
    }
}

impl Parse for DefaultArmWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let otherwise = input.parse()?;
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
        Ok(DefaultArmWhen::new(otherwise, arrow, brace, equations))
    }
}

/// ArmWhen for matching expression.
pub enum ArmWhen {
    EventArmWhen(EventArmWhen),
    TimeoutArmWhen(TimeoutArmWhen),
    Default(DefaultArmWhen),
}

mk_new! { impl ArmWhen =>
    EventArmWhen: event (e : EventArmWhen = e)
    TimeoutArmWhen: timeout (e : TimeoutArmWhen = e)
    Default: default (e : DefaultArmWhen = e)
}

impl Parse for ArmWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(keyword::otherwise) {
            Ok(ArmWhen::default(input.parse()?))
        } else if input.peek(keyword::timeout) {
            Ok(ArmWhen::timeout(input.parse()?))
        } else {
            Ok(ArmWhen::event(input.parse()?))
        }
    }
}

pub struct MatchWhen {
    pub when_token: keyword::when,
    pub brace_token: token::Brace,
    /// The different matching cases.
    pub arms: Punctuated<ArmWhen, Token![,]>,
}
mk_new! { impl MatchWhen =>
    new {
        when_token: keyword::when,
        brace_token: token::Brace,
        arms: Punctuated<ArmWhen, Token![,]>,
    }
}
impl Parse for MatchWhen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let when_token = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let arms: Punctuated<ArmWhen, Token![,]> = Punctuated::parse_terminated(&content)?;

        Ok(MatchWhen::new(when_token, brace, arms))
    }
}

/// GRust equation AST.
pub enum Equation {
    LocalDef(LetDecl<stream::Expr>),
    OutputDef(Instantiation),
    Match(Match),
    MatchWhen(MatchWhen),
}

mk_new! { impl Equation =>
    LocalDef: local_def(e: LetDecl<stream::Expr> = e)
    OutputDef: out_def(i: Instantiation = i)
    Match: pat_match(m : Match = m)
    MatchWhen: match_when(m : MatchWhen = m)
}

impl Parse for Equation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![match]) {
            Ok(Equation::pat_match(input.parse()?))
        } else if input.peek(keyword::when) {
            Ok(Equation::match_when(input.parse()?))
        } else if input.peek(Token![let]) {
            Ok(Equation::local_def(input.parse()?))
        } else {
            Ok(Equation::out_def(input.parse()?))
        }
    }
}

#[cfg(test)]
mod parse_equation {
    use std::fmt::Debug;

    use super::*;

    prelude! { just
        expr::{Binop, IfThenElse, Tuple},
        pattern::{Pattern, Tuple as PatTuple, Typed},
        operator::BinaryOperator,
    }

    use super::Equation;

    impl PartialEq for Equation {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::LocalDef(l0), Self::LocalDef(r0)) => {
                    l0.expression == r0.expression && l0.typed_pattern == r0.typed_pattern
                }
                (Self::OutputDef(l0), Self::OutputDef(r0)) => {
                    l0.expression == r0.expression && l0.pattern == r0.pattern
                }
                _ => false,
            }
        }
    }
    impl Debug for Equation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Equation::LocalDef(arg0) => f
                    .debug_tuple("LocalDef")
                    .field(&arg0.typed_pattern)
                    .field(&arg0.expression)
                    .finish(),
                Equation::OutputDef(arg0) => f
                    .debug_tuple("OutputDef")
                    .field(&arg0.pattern)
                    .field(&arg0.expression)
                    .finish(),
                Equation::Match(arg0) => f
                    .debug_tuple("Match")
                    .field(&arg0.expression)
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
                Equation::MatchWhen(arg0) => f
                    .debug_tuple("MatchWhen")
                    .field(
                        &arg0
                            .arms
                            .iter()
                            .map(|arm| match arm {
                                super::ArmWhen::EventArmWhen(arm) => (
                                    Some((
                                        &arm.pattern,
                                        &arm.event,
                                        arm.guard.as_ref().map(|(_, expr)| expr),
                                    )),
                                    &arm.equations,
                                ),
                                super::ArmWhen::TimeoutArmWhen(arm) => {
                                    (Some((&Pattern::Default, &arm.event, None)), &arm.equations)
                                }
                                super::ArmWhen::Default(arm) => (None, &arm.equations),
                            })
                            .collect::<Vec<_>>(),
                    )
                    .finish(),
            }
        }
    }

    #[test]
    fn should_parse_output_definition() {
        let equation: Equation = syn::parse_quote! {o = if res then 0 else (0 fby o) + inc;};
        let control = Equation::out_def(Instantiation {
            pattern: syn::parse_quote! {o},
            eq_token: syn::parse_quote! {=},
            expression: stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                stream::Expr::binop(Binop::new(
                    BinaryOperator::Add,
                    stream::Expr::fby(stream::Fby::new(
                        stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                        stream::Expr::ident("o"),
                    )),
                    stream::Expr::ident("inc"),
                )),
            )),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_tuple_instantiation() {
        let equation: Equation = syn::parse_quote! {
            (o1, o2) = if res then (0, 0) else ((0 fby o1) + inc1, (0 fby o2) + inc2);
        };
        let control = Equation::out_def(Instantiation {
            pattern: Pattern::tuple(PatTuple::new(vec![
                syn::parse_quote! {o1},
                syn::parse_quote! {o2},
            ])),
            eq_token: syn::parse_quote! {=},
            expression: stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                    stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                ])),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::binop(Binop::new(
                        BinaryOperator::Add,
                        stream::Expr::fby(stream::Fby::new(
                            stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                            stream::Expr::ident("o1"),
                        )),
                        stream::Expr::ident("inc1"),
                    )),
                    stream::Expr::binop(Binop::new(
                        BinaryOperator::Add,
                        stream::Expr::fby(stream::Fby::new(
                            stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                            stream::Expr::ident("o2"),
                        )),
                        stream::Expr::ident("inc2"),
                    )),
                ])),
            )),
            semi_token: syn::parse_quote! {;},
        });
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_local_definition() {
        let equation: Equation =
            syn::parse_quote! {let o: int = if res then 0 else (0 fby o) + inc;};
        let control = Equation::local_def(LetDecl::new(
            syn::parse_quote!(let),
            Pattern::typed(Typed {
                pattern: syn::parse_quote!(o),
                colon_token: syn::parse_quote!(:),
                typing: Typ::int(),
            }),
            syn::parse_quote!(=),
            stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                stream::Expr::binop(Binop::new(
                    BinaryOperator::Add,
                    stream::Expr::fby(stream::Fby::new(
                        stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                        stream::Expr::ident("o"),
                    )),
                    stream::Expr::ident("inc"),
                )),
            )),
            syn::parse_quote! {;},
        ));
        assert_eq!(equation, control)
    }

    #[test]
    fn should_parse_multiple_definitions() {
        let equation: Equation = syn::parse_quote! {
            let (o1: int, o2: int) = if res then (0, 0) else ((0 fby o1) + inc1, (0 fby o2) + inc2);
        };
        let control = Equation::local_def(LetDecl::new(
            syn::parse_quote!(let),
            Pattern::tuple(PatTuple::new(vec![
                Pattern::Typed(Typed {
                    pattern: syn::parse_quote!(o1),
                    colon_token: syn::parse_quote!(:),
                    typing: Typ::int(),
                }),
                Pattern::Typed(Typed {
                    pattern: syn::parse_quote!(o2),
                    colon_token: syn::parse_quote!(:),
                    typing: Typ::int(),
                }),
            ])),
            syn::parse_quote!(=),
            stream::Expr::ite(IfThenElse::new(
                stream::Expr::ident("res"),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                    stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                ])),
                stream::Expr::tuple(Tuple::new(vec![
                    stream::Expr::binop(Binop::new(
                        BinaryOperator::Add,
                        stream::Expr::fby(stream::Fby::new(
                            stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                            stream::Expr::ident("o1"),
                        )),
                        stream::Expr::ident("inc1"),
                    )),
                    stream::Expr::binop(Binop::new(
                        BinaryOperator::Add,
                        stream::Expr::fby(stream::Fby::new(
                            stream::Expr::cst(Constant::int(syn::parse_quote! {0})),
                            stream::Expr::ident("o2"),
                        )),
                        stream::Expr::ident("inc2"),
                    )),
                ])),
            )),
            syn::parse_quote! {;},
        ));
        assert_eq!(equation, control)
    }
}
