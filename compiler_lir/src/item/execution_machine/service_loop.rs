prelude! {}

#[derive(Debug, PartialEq)]
pub struct ServiceLoop {
    /// The service name.
    pub service: String,
    /// Its components.
    pub components: Vec<String>,
    /// The input flows.
    pub input_flows: Vec<InterfaceFlow>,
    /// The timing events.
    pub timing_events: Vec<TimingEvent>,
    /// The output flows.
    pub output_flows: Vec<InterfaceFlow>,
    /// The flows handling.
    pub flows_handling: Vec<FlowHandler>,
}

/// A flow structure.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceFlow {
    /// Path of the flow.
    pub path: syn::Path,
    /// The name of the flow.
    pub identifier: String,
    /// The type of the flow.
    pub r#type: Typ,
}

/// A timing event structure.
#[derive(Clone, Debug, PartialEq)]
pub struct TimingEvent {
    /// The name of the timing event.
    pub identifier: String,
    /// Kind of timing event.
    pub kind: TimingEventKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TimingEventKind {
    Period(u64),
    Timeout(u64),
}

#[derive(Debug, PartialEq)]
pub struct FlowHandler {
    pub arriving_flow: ArrivingFlow,
    pub deadline_args: Vec<String>,
    pub instructions: Vec<FlowInstruction>,
}
#[derive(Debug, PartialEq)]
pub enum ArrivingFlow {
    Channel(String, Typ),
    Period(String),
    Deadline(String),
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Let(String, Expression),
    UpdateContext(String, Expression),
    Send(String, Expression),
    IfThrottle(String, String, Constant, Box<FlowInstruction>),
    IfChange(String, String, Vec<FlowInstruction>, Vec<FlowInstruction>),
    ResetTimer(String, u64),
    EventComponentCall(Pattern, String, Option<(String, String)>),
    ComponentCall(Pattern, String),
}
mk_new! { impl FlowInstruction =>
    Let: def_let (name: impl Into<String> = name.into(), expr: Expression = expr.into())
    UpdateContext: update_ctx (name: impl Into<String> = name.into(), expr: Expression = expr.into())
    Send: send (name: impl Into<String> = name.into(), expr: Expression = expr.into())
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
    ResetTimer: reset (name: impl Into<String> = name.into(), val: u64 = val)
    ComponentCall: comp_call (
        pat: Pattern = pat,
        name: impl Into<String> = name.into(),
    )
    EventComponentCall: event_comp_call (
        pat: Pattern = pat,
        name: impl Into<String> = name.into(),
        event: Option<(String, String)> = event,
    )
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
