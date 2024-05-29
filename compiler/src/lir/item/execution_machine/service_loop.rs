use crate::common::constant::Constant;
use crate::common::r#type::Type;
use crate::lir::pattern::Pattern;

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
#[derive(Debug, PartialEq)]
pub struct InterfaceFlow {
    /// Path of the flow.
    pub path: syn::Path,
    /// The name of the flow.
    pub identifier: String,
    /// The type of the flow.
    pub r#type: Type,
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
    pub instructions: Vec<FlowInstruction>,
}
#[derive(Debug, PartialEq)]
pub enum ArrivingFlow {
    Channel(String),
    TimingEvent(String),
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Let(String, Expression),
    UpdateContext(String, Expression),
    Send(String, Expression),
    IfThrotle(String, String, Constant, Box<FlowInstruction>),
    IfChange(String, String, Vec<FlowInstruction>, Vec<FlowInstruction>),
    ResetTimer(String, u64),
    EventComponentCall(Pattern, String, Option<String>),
    ComponentCall(Pattern, String),
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
