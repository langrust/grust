use crate::common::r#type::Type;
use crate::lir::statement::Statement;

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
#[derive(Debug, PartialEq)]
pub struct TimingEvent {
    /// The name of the timing event.
    pub identifier: String,
    /// Kind of timing event.
    pub kind: TimingEventKind,
}
#[derive(Debug, PartialEq)]
pub enum TimingEventKind {
    Period(u64),
    Timeout(u64),
}

#[derive(Debug, PartialEq)]
pub struct FlowHandler {
    pub arriving_flow: String,
    pub instructions: Vec<FlowInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Update(String),
    Send(String),
    Let(Statement), // todo: ComponentCall, ResetTimer, IfBlock
}
