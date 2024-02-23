use crate::common::label::Label;
use crate::common::{location::Location, r#type::Type};
use crate::hir::{dependencies::Dependencies, expression::ExpressionKind};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression kind AST.
pub enum StreamExpressionKind {
    /// Expression.
    Expression {
        expression: ExpressionKind<StreamExpression>,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Box<StreamExpression>,
        /// The buffered expression.
        expression: Box<StreamExpression>,
    },
    /// Node application stream expression.
    NodeApplication {
        /// Node's id in Symbol Table.
        node_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, StreamExpression)>,
        /// Output signal's id in Symbol Table.
        output_id: usize,
    },
    /// Unitary node application stream expression.
    UnitaryNodeApplication {
        /// Unitary node's id in Symbol Table.
        node_id: usize,
        /// The inputs to the expression.
        inputs: Vec<(usize, StreamExpression)>,
        /// Output signal's id in Symbol Table.
        output_id: usize,
    },
}
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression AST.
pub struct StreamExpression {
    /// Stream expression kind.
    pub kind: StreamExpressionKind,
    /// Stream expression type.
    pub typing: Option<Type>,
    /// Stream expression location.
    pub location: Location,
    /// Stream expression dependencies.
    pub dependencies: Dependencies,
}

impl StreamExpression {
    pub fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }

    pub fn no_fby(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => {
                expression.propagate_predicate(StreamExpression::no_fby)
            }
            StreamExpressionKind::FollowedBy { .. } => false,
            StreamExpressionKind::UnitaryNodeApplication { inputs, .. } => {
                inputs.iter().all(|(_, expression)| expression.no_fby())
            }
            StreamExpressionKind::NodeApplication { inputs, .. } => {
                inputs.iter().all(|(_, expression)| expression.no_fby())
            }
        }
    }
    pub fn is_normal_form(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => {
                expression.propagate_predicate(StreamExpression::no_any_node_application)
            }
            StreamExpressionKind::FollowedBy { expression, .. } => {
                expression.no_any_node_application()
            }
            StreamExpressionKind::UnitaryNodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_any_node_application()),
            StreamExpressionKind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_any_node_application()),
        }
    }
    pub fn no_any_node_application(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => {
                expression.propagate_predicate(StreamExpression::no_any_node_application)
            }
            StreamExpressionKind::FollowedBy { expression, .. } => {
                expression.no_any_node_application()
            }
            StreamExpressionKind::UnitaryNodeApplication { .. } => false,
            StreamExpressionKind::NodeApplication { .. } => false,
        }
    }
    pub fn no_node_application(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => {
                expression.propagate_predicate(StreamExpression::no_node_application)
            }
            StreamExpressionKind::FollowedBy { expression, .. } => expression.no_node_application(),
            StreamExpressionKind::UnitaryNodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_node_application()),
            StreamExpressionKind::NodeApplication { .. } => false,
        }
    }
}
