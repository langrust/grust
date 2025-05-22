prelude! {}

use super::keyword;

#[derive(Debug, PartialEq)]
pub struct Instantiation<E> {
    /// Pattern of instantiated signals.
    pub pattern: stmt::Pattern,
    pub eq_token: syn::token::Eq,
    /// The stream expression defining the signals.
    pub expr: E,
    pub semi_token: syn::token::Semi,
}
impl<E> HasLoc for Instantiation<E> {
    fn loc(&self) -> Loc {
        self.pattern.loc().join(self.semi_token.span)
    }
}
mk_new! { impl{E} Instantiation<E> =>
    new {
        pattern: stmt::Pattern,
        eq_token: Token![=],
        expr: E,
        semi_token: Token![;],
    }
}

/// Arm for matching expression.
#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct MatchEq {
    pub match_token: Token![match],
    /// The stream expression defining the signals.
    pub expr: stream::Expr,
    pub brace_token: syn::token::Brace,
    /// The different matching cases.
    pub arms: syn::Punctuated<Arm, Option<Token![,]>>,
}
impl HasLoc for MatchEq {
    fn loc(&self) -> Loc {
        Loc::from(self.match_token.span).join(self.brace_token.span.join())
    }
}
mk_new! { impl MatchEq =>
    new {
        match_token: Token![match],
        expr: stream::Expr,
        brace_token: syn::token::Brace,
        arms: syn::Punctuated<Arm, Option<Token![,]>>,
    }
}

/// GRust simpl equation AST.
#[derive(Debug, PartialEq)]
pub enum Eq {
    LocalDef(LetDecl<stream::Expr>),
    OutputDef(Instantiation<stream::Expr>),
    MatchEq(MatchEq),
}
impl HasLoc for Eq {
    fn loc(&self) -> Loc {
        match self {
            Self::LocalDef(ld) => ld.loc(),
            Self::OutputDef(od) => od.loc(),
            Self::MatchEq(m) => m.loc(),
        }
    }
}
mk_new! { impl Eq =>
    LocalDef: local_def(e: LetDecl<stream::Expr> = e)
    OutputDef: out_def(i: Instantiation<stream::Expr> = i)
    MatchEq: match_eq(m : MatchEq = m)
}

#[derive(PartialEq, Clone)]
pub struct TupleEventPattern {
    pub paren_token: syn::token::Paren,
    /// The activated patterns.
    pub patterns: syn::Punctuated<EventPattern, Token![,]>,
}
impl TupleEventPattern {
    pub fn loc(&self) -> Loc {
        self.paren_token.span.join().into()
    }
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
impl LetEventPattern {
    pub fn loc(&self) -> Loc {
        self.let_token.span.into()
    }
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
impl EventPattern {
    pub fn loc(&self) -> Loc {
        match self {
            Self::Tuple(t) => t.loc(),
            Self::Let(l) => l.loc(),
            Self::RisingEdge(r) => r.loc(),
        }
    }
}
impl std::fmt::Debug for EventPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tuple(arg0) => f.debug_tuple("Tuple").field(&arg0.patterns).finish(),
            Self::Let(arg0) => f
                .debug_tuple("Let")
                .field(&(&arg0.pattern, &arg0.event))
                .finish(),
            Self::RisingEdge(arg0) => f.debug_tuple("RisingEdge").field(&arg0).finish(),
        }
    }
}

/// EventArmWhen for matching event.
#[derive(Debug, PartialEq)]
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

/// Init arm for when stream expression.
#[derive(Debug, PartialEq)]
pub struct InitArmWhen {
    pub init_token: keyword::init,
    pub arrow_token: Token![=>],
    pub brace_token: syn::token::Brace,
    /// The initial equations.
    pub equations: Vec<Instantiation<stream::Expr>>,
}
mk_new! { impl InitArmWhen =>
    new {
        init_token: keyword::init,
        arrow_token: Token![=>],
        brace_token: syn::token::Brace,
        equations: Vec<Instantiation<stream::Expr>>,
    }
}

#[derive(Debug, PartialEq)]
pub struct WhenEq {
    pub when_token: keyword::when,
    pub brace_token: syn::token::Brace,
    /// The optional init arm.
    pub init: Option<InitArmWhen>,
    /// The different event cases.
    pub arms: syn::Punctuated<EventArmWhen, Option<Token![,]>>,
}
impl HasLoc for WhenEq {
    fn loc(&self) -> Loc {
        Loc::from(self.when_token.span).join(self.brace_token.span.join())
    }
}
mk_new! { impl WhenEq =>
    new {
        when_token: keyword::when,
        brace_token: syn::token::Brace,
        init: Option<InitArmWhen>,
        arms: syn::Punctuated<EventArmWhen, Option<Token![,]>>,
    }
}

#[derive(Debug, PartialEq)]
pub struct InitSignal {
    pub init_token: keyword::init,
    /// Pattern of instantiated signals.
    pub pattern: stmt::Pattern,
    pub eq_token: Token![=],
    /// The stream expression defining the signal.
    pub expr: stream::Expr,
    pub semi_token: Token![;],
}
impl HasLoc for InitSignal {
    fn loc(&self) -> Loc {
        Loc::from(self.init_token.span).join(self.semi_token.span)
    }
}
mk_new! { impl InitSignal =>
    new {
        init_token: keyword::init,
        pattern: stmt::Pattern,
        eq_token: Token![=],
        expr: stream::Expr,
        semi_token: Token![;],
    }
}

/// GRust reactive equation AST.
#[derive(Debug, PartialEq)]
pub enum ReactEq {
    LocalDef(LetDecl<stream::ReactExpr>),
    OutputDef(Instantiation<stream::ReactExpr>),
    WhenEq(WhenEq),
    MatchEq(MatchEq),
    Init(InitSignal),
    Log(LogStmt),
}
impl HasLoc for ReactEq {
    fn loc(&self) -> Loc {
        match self {
            Self::LocalDef(ld) => ld.loc(),
            Self::OutputDef(od) => od.loc(),
            Self::WhenEq(mw) => mw.loc(),
            Self::MatchEq(m) => m.loc(),
            Self::Init(i) => i.loc(),
            Self::Log(log) => log.loc(),
        }
    }
}
mk_new! { impl ReactEq =>
    LocalDef: local_def(e: LetDecl<stream::ReactExpr> = e)
    OutputDef: out_def(i: Instantiation<stream::ReactExpr> = i)
    WhenEq: when_eq(m : WhenEq = m)
    MatchEq: match_eq(m : MatchEq = m)
    Init: init(i: InitSignal = i)
    Log: log(l : LogStmt = l)
}
