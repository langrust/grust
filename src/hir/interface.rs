use crate::common::location::Location;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust interface HIR.
pub struct Interface {
    /// Interface identifier.
    pub id: usize,
    /// Interface's imports and exports combined as system flows.
    pub system_flows: Vec<usize>,
    /// Interface's flow statements.
    pub flow_statements: Vec<FlowStatement>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Flow statement HIR.
pub struct FlowStatement {
    /// Identifier of the new flow.
    pub id: usize,
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Flow expression kinds.
pub enum FlowExpressionKind {
    /// Flow identifier call.
    Ident {
        /// The identifier of the flow to call.
        id: usize,
    },
    /// GReact `tiemout` operator.
    Timeout {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        /// Time of the timeout in milliseconds.
        timeout_ms: u64,
    },
    /// GReact `merge` operator.
    Merge {
        /// Input expression 1.
        flow_expression_1: Box<FlowExpression>,
        /// Input expression 2.
        flow_expression_2: Box<FlowExpression>,
    },
    /// GReact `zip` operator.
    Zip {
        /// Input expression 1.
        flow_expression_1: Box<FlowExpression>,
        /// Input expression 2.
        flow_expression_2: Box<FlowExpression>,
    },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        component_id: usize,
        /// Input expressions.
        inputs: Vec<FlowExpression>,
        /// Identifier to the component output signal to call.
        signal_id: usize,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Flow expression HIR.
pub struct FlowExpression {
    /// Flow expression's kind.
    pub kind: FlowExpressionKind,
    /// Flow expression location.
    pub location: Location,
}
