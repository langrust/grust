use crate::common::{location::Location, r#type::Type};

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
