use crate::common::{constant::Constant, location::Location, r#type::Type};

#[derive(Debug, PartialEq, Clone)]
/// Flow expression kinds.
pub enum FlowExpressionKind {
    /// Flow identifier call.
    Ident {
        /// The identifier of the flow to call.
        id: usize,
    },
    /// GReact `sample` operator.
    Sample {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        /// Sampling period in milliseconds.
        period_ms: u64,
    },
    /// GReact `scan` operator.
    Scan {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        /// Scaning period in milliseconds.
        period_ms: u64,
    },
    /// GReact `timeout` operator.
    Timeout {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        /// Dealine in milliseconds.
        deadline: u64,
    },
    /// GReact `throtle` operator.
    Throtle {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        /// Variation that will update the signal.
        delta: Constant,
    },
    /// GReact `on_change` operator.
    OnChange {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
    },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        component_id: usize,
        /// Input expressions.
        inputs: Vec<(usize, FlowExpression)>,
        /// Identifier to the component output signal to call.
        signal_id: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
/// Flow expression HIR.
pub struct FlowExpression {
    /// Flow expression's kind.
    pub kind: FlowExpressionKind,
    /// Flow expression type.
    pub typing: Option<Type>,
    /// Flow expression location.
    pub location: Location,
}
