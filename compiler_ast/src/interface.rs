prelude! {
    syn::{Parse, Punctuated, token, LitInt},
}

/// GReact `sample` operator.
pub struct Sample {
    pub sample_token: keyword::sample,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Sampling period in milliseconds.
    pub period_ms: LitInt,
}
impl Sample {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::sample)
    }
}
impl Parse for Sample {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let sample_token: keyword::sample = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: FlowExpression = content.parse()?;
        let comma_token: Token![,] = content.parse()?;
        let period_ms: LitInt = content.parse()?;
        if content.is_empty() {
            Ok(Sample::new(
                sample_token,
                paren_token,
                flow_expression,
                comma_token,
                period_ms,
            ))
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}
mk_new! { impl Sample =>
    new {
        sample_token: keyword::sample,
        paren_token: token::Paren,
        flow_expression: FlowExpression = flow_expression.into(),
        comma_token: Token![,],
        period_ms: LitInt,
    }
}

/// GReact `scan` operator.
pub struct Scan {
    pub scan_token: keyword::scan,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Scanning period in milliseconds.
    pub period_ms: LitInt,
}
impl Scan {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::scan)
    }
}
impl Parse for Scan {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let scan_token: keyword::scan = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: FlowExpression = content.parse()?;
        let comma_token: Token![,] = content.parse()?;
        let period_ms: LitInt = content.parse()?;
        if content.is_empty() {
            Ok(Scan::new(
                scan_token,
                paren_token,
                flow_expression,
                comma_token,
                period_ms,
            ))
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}
mk_new! { impl Scan =>
    new {
        scan_token: keyword::scan,
        paren_token: token::Paren,
        flow_expression: FlowExpression = flow_expression.into(),
        comma_token: Token![,],
        period_ms: LitInt,
    }
}

/// GReact `timeout` operator.
pub struct Timeout {
    pub timeout_token: keyword::timeout,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Deadline in milliseconds.
    pub deadline: LitInt,
}
impl Timeout {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::timeout)
    }
}
impl Parse for Timeout {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let timeout_token: keyword::timeout = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: FlowExpression = content.parse()?;
        let comma_token: Token![,] = content.parse()?;
        let deadline: LitInt = content.parse()?;
        if content.is_empty() {
            Ok(Timeout::new(
                timeout_token,
                paren_token,
                flow_expression,
                comma_token,
                deadline,
            ))
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}
mk_new! { impl Timeout =>
    new {
        timeout_token: keyword::timeout,
        paren_token: token::Paren,
        flow_expression: FlowExpression = flow_expression.into(),
        comma_token: Token![,],
        deadline: LitInt,
    }
}

/// GReact `throttle` operator.
pub struct Throttle {
    pub throttle_token: keyword::throttle,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
    pub comma_token: Token![,],
    /// Variation that will update the signal.
    pub delta: Constant,
}
impl Throttle {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::throttle)
    }
}
impl Parse for Throttle {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let throttle_token: keyword::throttle = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: FlowExpression = content.parse()?;
        let comma_token: Token![,] = content.parse()?;
        let delta: Constant = content.parse()?;
        if content.is_empty() {
            Ok(Throttle::new(
                throttle_token,
                paren_token,
                flow_expression,
                comma_token,
                delta,
            ))
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}
mk_new! { impl Throttle =>
    new {
        throttle_token: keyword::throttle,
        paren_token: token::Paren,
        flow_expression: FlowExpression = flow_expression.into(),
        comma_token: Token![,],
        delta: Constant,
    }

}

/// GReact `on_change` operator.
pub struct OnChange {
    pub on_change_token: keyword::on_change,
    pub paren_token: token::Paren,
    /// Input expression.
    pub flow_expression: Box<FlowExpression>,
}
impl OnChange {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::on_change)
    }
}
impl Parse for OnChange {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let on_change_token: keyword::on_change = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression: FlowExpression = content.parse()?;
        if content.is_empty() {
            Ok(OnChange::new(on_change_token, paren_token, flow_expression))
        } else {
            Err(content.error("expected one input expression"))
        }
    }
}
mk_new! { impl OnChange =>
    new {
        on_change_token: keyword::on_change,
        paren_token: token::Paren,
        flow_expression: FlowExpression = flow_expression.into(),
    }

}

/// GReact `merge` operator.
pub struct Merge {
    pub merge_token: keyword::merge,
    pub paren_token: token::Paren,
    /// Input expressions.
    pub flow_expression_1: Box<FlowExpression>,
    pub comma_token: Token![,],
    pub flow_expression_2: Box<FlowExpression>,
}
impl Merge {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::merge)
    }
}
impl Parse for Merge {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let merge_token: keyword::merge = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let flow_expression_1: FlowExpression = content.parse()?;
        let comma_token = content.parse()?;
        let flow_expression_2: FlowExpression = content.parse()?;
        if content.is_empty() {
            Ok(Merge::new(
                merge_token,
                paren_token,
                flow_expression_1,
                comma_token,
                flow_expression_2,
            ))
        } else {
            Err(content.error("expected two input expressions"))
        }
    }
}
mk_new! { impl Merge =>
    new {
        merge_token: keyword::merge,
        paren_token: token::Paren,
        flow_expression_1: FlowExpression = flow_expression_1.into(),
        comma_token: Token![,],
        flow_expression_2: FlowExpression = flow_expression_2.into(),
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
impl Parse for ComponentCall {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let ident_component: Ident = input.parse()?;
        let content;
        let paren_token: token::Paren = parenthesized!(content in input);
        let inputs: Punctuated<FlowExpression, Token![,]> = Punctuated::parse_terminated(&content)?;
        // let ident_signal: Option<(Token![.], Ident)> = {
        //     if input.peek(Token![.]) {
        //         Some((input.parse()?, input.parse()?))
        //     } else {
        //         None
        //     }
        // };
        Ok(ComponentCall {
            ident_component,
            paren_token,
            inputs,
        })
    }
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
    /// GReact `merge` operator.
    Merge(Merge),
    /// Component call.
    ComponentCall(ComponentCall),
    /// Identifier to flow.
    Ident(String),
}
impl Parse for FlowExpression {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if Sample::peek(input) {
            Ok(Self::sample(input.parse()?))
        } else if Scan::peek(input) {
            Ok(Self::scan(input.parse()?))
        } else if Timeout::peek(input) {
            Ok(Self::timeout(input.parse()?))
        } else if Throttle::peek(input) {
            Ok(Self::throttle(input.parse()?))
        } else if OnChange::peek(input) {
            Ok(Self::on_change(input.parse()?))
        } else if Merge::peek(input) {
            Ok(Self::merge(input.parse()?))
        } else if input.fork().call(ComponentCall::parse).is_ok() {
            Ok(Self::comp_call(input.parse()?))
        } else {
            let ident: Ident = input.parse()?;
            Ok(Self::ident(ident.to_string()))
        }
    }
}

mk_new! { impl FlowExpression =>
    Ident: ident (val: impl Into<String> = val.into())
    Sample: sample (val: Sample = val)
    Scan: scan (val: Scan = val)
    Timeout: timeout (val: Timeout = val)
    Throttle: throttle (val: Throttle = val)
    OnChange: on_change (val: OnChange = val)
    Merge: merge (val: Merge = val)
    ComponentCall: comp_call (val: ComponentCall = val)
}

#[derive(Clone)]
pub enum FlowKind {
    Signal(keyword::signal),
    Event(keyword::event),
}
impl FlowKind {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::signal) || input.peek(keyword::event)
    }

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
impl Parse for FlowKind {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(keyword::signal) {
            Ok(FlowKind::Signal(input.parse()?))
        } else if input.peek(keyword::event) {
            Ok(FlowKind::Event(input.parse()?))
        } else {
            Err(input.error("expected 'signal' or 'event'"))
        }
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
impl Parse for FlowPattern {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(token::Paren) {
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let patterns: Punctuated<FlowPattern, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(FlowPattern::Tuple {
                paren_token,
                patterns,
            })
        } else if FlowKind::peek(input) {
            let kind: FlowKind = input.parse()?;
            let ident: Ident = input.parse()?;
            let colon_token: Token![:] = input.parse()?;
            let ty: Typ = input.parse()?;
            Ok(FlowPattern::SingleTyped {
                kind,
                ident,
                colon_token,
                ty,
            })
        } else {
            let ident: Ident = input.parse()?;
            Ok(FlowPattern::Single { ident })
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
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
impl FlowDeclaration {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![let])
    }
}
impl Parse for FlowDeclaration {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let let_token: Token![let] = input.parse()?;
        let typed_pattern: FlowPattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let flow_expression: FlowExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowDeclaration {
            let_token,
            typed_pattern,
            eq_token,
            flow_expression,
            semi_token,
        })
    }
}

/// Flow statement AST.
pub struct FlowInstantiation {
    /// Pattern of instantiated flows.
    pub pattern: FlowPattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    pub semi_token: Token![;],
}
impl FlowInstantiation {
    pub fn peek(input: ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(FlowPattern::parse).is_err() {
            return false;
        }
        forked.peek(Token![=])
    }
}
impl Parse for FlowInstantiation {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let pattern: FlowPattern = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let flow_expression: FlowExpression = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowInstantiation {
            pattern,
            eq_token,
            flow_expression,
            semi_token,
        })
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
impl FlowImport {
    pub fn peek(input: ParseStream) -> bool {
        let forked = input.fork();
        forked
            .parse::<keyword::import>()
            .and_then(|_| forked.parse::<FlowKind>())
            .is_ok()
    }
}
impl Parse for FlowImport {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let import_token: keyword::import = input.parse()?;
        let kind: FlowKind = input.parse()?;
        let typed_path: Colon<syn::Path, Typ> = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowImport {
            import_token,
            kind,
            typed_path,
            semi_token,
        })
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
impl FlowExport {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::export)
    }
}
impl Parse for FlowExport {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let export_token: keyword::export = input.parse()?;
        let kind: FlowKind = input.parse()?;
        let typed_path: Colon<syn::Path, Typ> = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(FlowExport {
            export_token,
            kind,
            typed_path,
            semi_token,
        })
    }
}

pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instantiation(FlowInstantiation),
}
impl FlowStatement {
    pub fn peek(input: ParseStream) -> bool {
        FlowDeclaration::peek(input) || FlowInstantiation::peek(input)
    }
}
impl Parse for FlowStatement {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if FlowDeclaration::peek(input) {
            Ok(FlowStatement::Declaration(input.parse()?))
        } else if FlowInstantiation::peek(input) {
            Ok(FlowStatement::Instantiation(input.parse()?))
        } else {
            Err(input.error("expected flow declaration or instantiation"))
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
impl TimeRange {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![@])
    }
}
impl Parse for TimeRange {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let at_token: token::At = input.parse()?;
        let content;
        let bracket_token: token::Bracket = syn::bracketed!(content in input);
        let min: LitInt = content.parse()?;
        let comma_token: token::Comma = content.parse()?;
        let max: LitInt = content.parse()?;
        if content.is_empty() {
            Ok(TimeRange {
                at_token,
                bracket_token,
                min,
                comma_token,
                max,
            })
        } else {
            Err(content.error("expected something like `@ [min, max]`"))
        }
    }
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
impl Service {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::service)
    }
}
impl Parse for Service {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        let service_token: keyword::service = input.parse()?;
        let ident: Ident = input.parse()?;
        let time_range = if TimeRange::peek(input) {
            Some(input.parse()?)
        } else {
            None
        };
        let content;
        let brace: token::Brace = syn::braced!(content in input);
        let flow_statements: Vec<FlowStatement> = {
            let mut flow_statements = vec![];
            while !content.is_empty() {
                flow_statements.push(content.parse()?)
            }
            flow_statements
        };
        Ok(Service {
            service_token,
            ident,
            time_range,
            brace,
            flow_statements,
        })
    }
}

#[cfg(test)]
mod parse_service {
    use super::*;

    #[test]
    fn should_parse_service() {
        let _: Service = parse_quote! {
            service aeb {
                let event pedestrian: float = merge(pedestrian_l, pedestrian_r);
                let event timeout_pedestrian: unit = timeout(pedestrian, 2000);
                brakes = braking_state(pedestrian, timeout_pedestrian, speed_km_h);
            }
        };
    }
}
