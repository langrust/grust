prelude! {
    BTreeMap as Map,
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
    pub instruction: FlowInstruction,
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Let(String, Expression),
    InitEvent(String),
    UpdateEvent(String, Expression),
    UpdateContext(String, Expression),
    Send(String, Expression, Option<String>),
    IfThrottle(String, String, Constant, Box<Self>),
    IfChange(String, Expression, Box<Self>),
    IfActivated(Vec<String>, Vec<String>, Box<Self>, Option<Box<Self>>),
    ResetTimer(String, String),
    ComponentCall(
        Pattern,
        String,
        Vec<(String, String)>,
        Vec<(String, Option<String>)>,
    ),
    HandleDelay(Vec<String>, Vec<MatchArm>),
    Seq(Vec<Self>),
    Para(Map<ParaMethod, Vec<Self>>),
}
mk_new! { impl FlowInstruction =>
    Let: def_let (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    InitEvent: init_event (
        name: impl Into<String> = name.into(),
    )
    UpdateEvent: update_event (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    UpdateContext: update_ctx (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    Send: send_from (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
        instant: impl Into<String> = Some(instant.into()),
    )
    Send: send (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
        instant = None,
    )
    IfThrottle: if_throttle (
        flow_name: impl Into<String> = flow_name.into(),
        source_name: impl Into<String> = source_name.into(),
        delta: Constant = delta,
        instr: FlowInstruction = instr.into(),
    )
    IfChange: if_change (
        old_event_name: impl Into<String> = old_event_name.into(),
        signal: Expression = signal,
        then: FlowInstruction = then.into(),
    )
    IfActivated: if_activated (
        events: impl Into<Vec<String>> = events.into(),
        signals: impl Into<Vec<String>> = signals.into(),
        then: FlowInstruction = then.into(),
        els: Option<FlowInstruction> = els.map(Into::into),
    )
    ResetTimer: reset (
        name: impl Into<String> = name.into(),
        instant: impl Into<String> = instant.into(),
    )
    ComponentCall: comp_call (
        pat: Pattern = pat,
        name: impl Into<String> = name.into(),
        signals: impl Into<Vec<(String, String)>> = signals.into(),
        events: impl Into<Vec<(String, Option<String>)>> = events.into(),
    )
    HandleDelay: handle_delay(
        input_names: impl Iterator<Item = String> = input_names.collect(),
        arms: impl Iterator<Item = MatchArm> = arms.collect(),
    )
    Seq: seq(
        instrs: Vec<FlowInstruction> = instrs,
    )
    Para: para(
        para_instr: Map<ParaMethod, Vec<Self>> = para_instr,
    )
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParaMethod {
    Rayon,
    Threads,
    Tokio,
    DoNotPara,
}
mk_new! { impl ParaMethod =>
    Rayon: rayon ()
    Threads: threads ()
    Tokio: tokio ()
    DoNotPara: dont_para ()
}

#[derive(Debug, PartialEq)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub instr: FlowInstruction,
}
mk_new! { impl MatchArm =>
    new {
        patterns: Vec<Pattern> = patterns,
        instr: FlowInstruction = instr,
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An event call: `x`.
    Event {
        /// The identifier.
        identifier: String,
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
    /// None expression: `None`.
    None,
}
mk_new! { impl Expression =>
    Literal: lit {
        literal: Constant = literal
    }
    Event: event {
        identifier: impl Into<String> = identifier.into()
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
    None: none {}
}
