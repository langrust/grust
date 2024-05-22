use crate::common::r#type::Type;
use crate::lir::statement::Statement;

#[derive(Debug, PartialEq)]
pub struct FlowRunLoop {
    /// The component name.
    pub component: String,
    /// Its state machine inputs.
    pub inputs: Vec<(String, Type)>,
    /// The input flows.
    pub input_flows: Vec<Flow>,
    /// The output flows.
    pub output_flows: Vec<Flow>,
    /// The arms of the tokio::select!.
    pub select_arms: Vec<SelectArm>,
}

/// An flow structure.
#[derive(Debug, PartialEq)]
pub struct Flow {
    /// Path of the flow.
    pub path: syn::Path,
    /// The name of the flow.
    pub identifier: String,
    /// The type of the flow.
    pub r#type: Type,
}

#[derive(Debug, PartialEq)]
pub struct SelectArm {
    pub event_ident: String,
    pub instructions: Vec<FlowInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Update(String),
    Send(String),
    Let(Statement),
}
