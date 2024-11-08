prelude! {
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

/// Arm for matching expression.
pub struct Arm {
    /// The pattern to match.
    pub pattern: expr::Pattern,
    /// The optional guard.
    pub guard: Option<(Token![if], stream::Expr)>,
    pub arrow_token: Token![=>],
    pub brace_token: syn::token::Brace,
    /// The equations.
    pub equations: Vec<Eq>,
}

mk_new! { impl Arm =>
    new {
        pattern: expr::Pattern,
        guard: Option<(Token![if], stream::Expr)>,
        arrow_token: Token![=>],
        brace_token: syn::token::Brace,
        equations: Vec<Eq>,
    }
}

pub struct Match {
    pub match_token: Token![match],
    /// The stream expression defining the signals.
    pub expr: stream::Expr,
    pub brace_token: syn::token::Brace,
    /// The different matching cases.
    pub arms: syn::Punctuated<Arm, Token![,]>,
}

mk_new! { impl Match =>
    new {
        match_token: Token![match],
        expr: stream::Expr,
        brace_token: syn::token::Brace,
        arms: syn::Punctuated<Arm, Token![,]>,
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

#[derive(PartialEq, Clone)]
pub struct TupleEventPattern {
    pub paren_token: syn::token::Paren,
    /// The activated patterns.
    pub patterns: syn::Punctuated<EventPattern, Token![,]>,
}
mk_new! { impl TupleEventPattern =>
    new {
        paren_token: syn::token::Paren,
        patterns: syn::Punctuated<EventPattern, Token![,]>,
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

#[derive(PartialEq, Clone)]
pub enum EventPattern {
    Tuple(TupleEventPattern),
    Let(LetEventPattern),
    RisingEdge(Box<stream::Expr>),
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
    pub brace_token: syn::token::Brace,
    /// The equations.
    pub equations: Vec<Eq>,
}
mk_new! { impl EventArmWhen =>
    new {
        pattern: EventPattern,
        guard: Option<(Token![if], stream::Expr)>,
        arrow_token: Token![=>],
        brace_token: syn::token::Brace,
        equations: Vec<Eq>,
    }
}

pub struct MatchWhen {
    pub when_token: keyword::when,
    pub brace_token: syn::token::Brace,
    /// The different matching cases.
    pub arms: Vec<EventArmWhen>,
}
mk_new! { impl MatchWhen =>
    new {
        when_token: keyword::when,
        brace_token: syn::token::Brace,
        arms: Vec<EventArmWhen>,
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
impl std::fmt::Debug for Eq {
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
                    l0.pattern == r0.pattern && l0.guard == r0.guard && l0.equations == r0.equations
                })
            }
            _ => false,
        }
    }
}
impl std::fmt::Debug for ReactEq {
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
