prelude! {
    syn::{Punctuated, token, LitInt},
}

/// GReact `sample` operator.
pub struct Sample {
    pub sample_token: keyword::sample,
    pub paren_token: token::Paren,
    /// Input expression.
    pub expr: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Sampling period in milliseconds.
    pub period_ms: LitInt,
}
mk_new! { impl Sample =>
    new {
        sample_token: keyword::sample,
        paren_token: token::Paren,
        expr: FlowExpression = expr.into(),
        comma_token: Token![,],
        period_ms: LitInt,
    }
}

/// GReact `scan` operator.
pub struct Scan {
    pub scan_token: keyword::scan,
    pub paren_token: token::Paren,
    /// Input expression.
    pub expr: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Scanning period in milliseconds.
    pub period_ms: LitInt,
}
mk_new! { impl Scan =>
    new {
        scan_token: keyword::scan,
        paren_token: token::Paren,
        expr: FlowExpression = expr.into(),
        comma_token: Token![,],
        period_ms: LitInt,
    }
}

/// GReact `timeout` operator.
pub struct Timeout {
    pub timeout_token: keyword::timeout,
    pub paren_token: token::Paren,
    /// Input expression.
    pub expr: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Deadline in milliseconds.
    pub deadline: LitInt,
}
mk_new! { impl Timeout =>
    new {
        timeout_token: keyword::timeout,
        paren_token: token::Paren,
        expr: FlowExpression = expr.into(),
        comma_token: Token![,],
        deadline: LitInt,
    }
}

/// GReact `throttle` operator.
pub struct Throttle {
    pub throttle_token: keyword::throttle,
    pub paren_token: token::Paren,
    /// Input expression.
    pub expr: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Variation that will update the signal.
    pub delta: Constant,
}
mk_new! { impl Throttle =>
    new {
        throttle_token: keyword::throttle,
        paren_token: token::Paren,
        expr: FlowExpression = expr.into(),
        comma_token: Token![,],
        delta: Constant,
    }

}

/// GReact `on_change` operator.
pub struct OnChange {
    pub on_change_token: keyword::on_change,
    pub paren_token: token::Paren,
    /// Input expression.
    pub expr: Box<FlowExpression>,
}
mk_new! { impl OnChange =>
    new {
        on_change_token: keyword::on_change,
        paren_token: token::Paren,
        expr: FlowExpression = expr.into(),
    }

}

/// GReact `persist` operator.
pub struct Persist {
    pub persist_token: keyword::persist,
    pub paren_token: token::Paren,
    /// Input expression.
    pub expr: Box<FlowExpression>,
}
mk_new! { impl Persist =>
    new {
        persist_token: keyword::persist,
        paren_token: token::Paren,
        expr: FlowExpression = expr.into(),
    }

}

/// GReact `merge` operator.
pub struct Merge {
    pub merge_token: keyword::merge,
    pub paren_token: token::Paren,
    /// Input expressions.
    pub expr_1: Box<FlowExpression>,
    pub comma_token: Token![,],
    pub expr_2: Box<FlowExpression>,
}
mk_new! { impl Merge =>
    new {
        merge_token: keyword::merge,
        paren_token: token::Paren,
        expr_1: FlowExpression = expr_1.into(),
        comma_token: Token![,],
        expr_2: FlowExpression = expr_2.into(),
    }

}

/// GReact `time` operator.
pub struct Time {
    pub time_token: keyword::time,
    pub paren_token: token::Paren,
}
mk_new! { impl Time =>
    new {
        time_token: keyword::time,
        paren_token: token::Paren,
    }

}

/// Component call.
pub struct ComponentCall {
    /// Identifier to the component to call.
    pub ident_component: Ident,
    pub paren_token: token::Paren,
    /// Input expressions.
    pub inputs: Punctuated<FlowExpression, Token![,]>,
}

/// Flow expression kinds.
pub enum FlowExpression {
    /// GReact `sample` operator.
    Sample(Sample),
    /// GReact `scan` operator.
    Scan(Scan),
    /// GReact `timeout` operator.
    Timeout(Timeout),
    /// GReact `throttle` operator.
    Throttle(Throttle),
    /// GReact `on_change` operator.
    OnChange(OnChange),
    /// GReact `persist` operator.
    Persist(Persist),
    /// GReact `merge` operator.
    Merge(Merge),
    /// Component call.
    ComponentCall(ComponentCall),
    /// Identifier to flow.
    Ident(Ident),
    /// Time flow.
    Time(Time),
}

mk_new! { impl FlowExpression =>
    Ident: ident (val: impl Into<Ident> = val.into())
    Sample: sample (val: Sample = val)
    Scan: scan (val: Scan = val)
    Timeout: timeout (val: Timeout = val)
    Throttle: throttle (val: Throttle = val)
    OnChange: on_change (val: OnChange = val)
    Persist: persist (val: Persist = val)
    Merge: merge (val: Merge = val)
    Time: time (val: Time = val)
    ComponentCall: comp_call (val: ComponentCall = val)
}

#[derive(Clone)]
pub enum FlowKind {
    Signal(keyword::signal),
    Event(keyword::event),
}
impl FlowKind {
    #[inline]
    pub fn is_signal(&self) -> bool {
        match self {
            Self::Signal(_) => true,
            Self::Event(_) => false,
        }
    }
    #[inline]
    pub fn is_event(&self) -> bool {
        !self.is_signal()
    }
}

pub enum FlowPattern {
    Tuple {
        paren_token: token::Paren,
        patterns: Punctuated<FlowPattern, Token![,]>,
    },
    SingleTyped {
        kind: FlowKind,
        ident: Ident,
        colon_token: Token![:],
        ty: Typ,
    },
    Single {
        ident: Ident,
    },
}
impl HasLoc for FlowPattern {
    fn loc(&self) -> Loc {
        match self {
            Self::Tuple { paren_token, .. } => paren_token.span.join().into(),
            Self::SingleTyped { ident, .. } | Self::Single { ident } => ident.loc().into(),
        }
    }
}

/// Flow statement AST.
pub struct FlowDeclaration {
    pub let_token: Token![let],
    /// Pattern of declared flows and their type.
    pub typed_pattern: FlowPattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub expr: FlowExpression,
    pub semi_token: Token![;],
}
impl HasLoc for FlowDeclaration {
    fn loc(&self) -> Loc {
        Loc::from(self.let_token.span).join(self.semi_token.span)
    }
}

/// Flow statement AST.
pub struct FlowInstantiation {
    /// Pattern of instantiated flows.
    pub pattern: FlowPattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub expr: FlowExpression,
    pub semi_token: Token![;],
}
impl HasLoc for FlowInstantiation {
    fn loc(&self) -> Loc {
        self.pattern.loc().join(self.semi_token.span)
    }
}

/// Flow statement AST.
pub struct FlowImport {
    pub import_token: keyword::import,
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_path: Colon<syn::Path, Typ>,
    pub semi_token: Token![;],
}
impl HasLoc for FlowImport {
    fn loc(&self) -> Loc {
        Loc::from(self.import_token.span).join(self.semi_token.span)
    }
}

/// Flow statement AST.
pub struct FlowExport {
    pub export_token: keyword::export,
    /// Flow's kind.
    pub kind: FlowKind,
    /// Identifier of the flow and its type.
    pub typed_path: Colon<syn::Path, Typ>,
    pub semi_token: Token![;],
}
impl HasLoc for FlowExport {
    fn loc(&self) -> Loc {
        Loc::from(self.export_token.span).join(self.semi_token.span)
    }
}

pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instantiation(FlowInstantiation),
}
impl HasLoc for FlowStatement {
    fn loc(&self) -> Loc {
        match self {
            Self::Declaration(d) => d.loc(),
            Self::Instantiation(i) => i.loc(),
        }
    }
}

/// Service's time range.
pub struct TimeRange {
    pub at_token: Token![@],
    pub bracket_token: token::Bracket,
    pub min: LitInt,
    pub comma_token: Token![,],
    pub max: LitInt,
}

/// GRust service AST.
pub struct Service {
    pub service_token: keyword::service,
    /// Service identifier.
    pub ident: Ident,
    /// Service's time range.
    pub time_range: Option<TimeRange>,
    pub brace: token::Brace,
    /// Service's flow statements.
    pub flow_statements: Vec<FlowStatement>,
}
impl HasLoc for Service {
    fn loc(&self) -> Loc {
        Loc::from(self.service_token.span).join(self.brace.span.join())
    }
}
