use crate::{
    common::{location::Location, r#type::Type},
    hir::statement::Statement,
};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust interface HIR.
pub struct Interface {
    /// Interface identifier.
    pub id: usize,
    /// Interface's flow statements.
    pub flow_statements: Vec<Statement<FlowExpression>>,
}

#[derive(Debug, PartialEq, Clone)]
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
