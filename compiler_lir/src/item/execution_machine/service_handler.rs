prelude! {
    item::execution_machine::{flows_context::FlowsContext, ArrivingFlow}
}

#[derive(Debug, PartialEq)]
pub struct ServiceHandler {
    /// The service name.
    pub service: String,
    /// Its components.
    pub components: Vec<String>,
    /// The flows handling.
    pub flows_handling: Vec<FlowHandler>,
    /// The signals context from where components will get their inputs.
    pub flows_context: FlowsContext,
}

#[derive(Debug, PartialEq)]
pub struct FlowHandler {
    pub arriving_flow: ArrivingFlow,
    pub instructions: Vec<FlowInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Let(String, Expression),
    UpdateContext(String, Expression),
    Send(String, Expression, String),
    IfThrottle(String, String, Constant, Box<FlowInstruction>),
    IfChange(String, String, Vec<FlowInstruction>, Vec<FlowInstruction>),
    ResetTimer(String, String),
    ComponentCall(Pattern, String, Vec<Option<String>>),
    HandleDelay(Vec<String>, Vec<MatchArm>),
}
mk_new! { impl FlowInstruction =>
    Let: def_let (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    UpdateContext: update_ctx (name: impl Into<String> = name.into(), expr: Expression = expr.into())
    Send: send (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
        instant: impl Into<String> = instant.into(),
    )
    IfThrottle: if_throttle (
        flow_name: impl Into<String> = flow_name.into(),
        source_name: impl Into<String> = source_name.into(),
        delta: Constant = delta,
        instr: FlowInstruction = instr.into(),
    )
    IfChange: if_change (
        old_event_name: impl Into<String> = old_event_name.into(),
        source_name: impl Into<String> = source_name.into(),
        then: Vec<FlowInstruction> = then,
        els: Vec<FlowInstruction> = els,
    )
    ResetTimer: reset (
        name: impl Into<String> = name.into(),
        instant: impl Into<String> = instant.into(),
    )
    ComponentCall: comp_call (
        pat: Pattern = pat,
        name: impl Into<String> = name.into(),
        events: impl Into<Vec<Option<String>>> = events.into(),
    )
    HandleDelay: handle_delay(
        input_names: impl Iterator<Item = String> = input_names.collect(),
        arms: impl Iterator<Item = MatchArm> = arms.collect(),
    )
}

#[derive(Debug, PartialEq)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub block: Vec<FlowInstruction>,
}
mk_new! { impl MatchArm =>
    new {
        patterns: impl Iterator<Item = Pattern> = patterns.collect(),
        block: Vec<FlowInstruction> = block,
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: String,
    },
    /// A call from the context: `ctxt.s`.
    InContext {
        /// The flow called.
        flow: String,
    },
    /// A call from the context that will take the value: `ctxt.s.take()`.
    TakeFromContext {
        /// The flow called.
        flow: String,
    },
    /// Some expression: `Some(v)`.
    Some {
        /// The value expression inside.
        expression: Box<Expression>,
    },
    /// Ok expression: `Ok(v)`.
    Ok {
        /// The value expression inside.
        expression: Box<Expression>,
    },
    /// None expression: `None`.
    None,
    /// Err expression: `Err`.
    Err,
}
mk_new! { impl Expression =>
    Literal: lit {
        literal: Constant = literal
    }
    Identifier: ident {
        identifier: impl Into<String> = identifier.into()
    }
    InContext: in_ctx {
        flow: impl Into<String> = flow.into()
    }
    TakeFromContext: take_from_ctx {
        flow: impl Into<String> = flow.into()
    }
    Some: some {
        expression: Expression = expression.into()
    }
    Ok: ok {
        expression: Expression = expression.into()
    }
    None: none {}
    Err: err {}
}
