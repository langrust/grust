use crate::common::label::Label;
use crate::common::{location::Location, r#type::Type};
use crate::hir::{dependencies::Dependencies, expression::ExpressionKind};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression kind AST.
pub enum StreamExpressionKind {
    /// Expression.
    Expression {
        /// The expression kind.
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
    },
}
#[derive(Debug, PartialEq, Clone)]
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
    /// Get stream expression's type.
    pub fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }
    /// Get stream expression's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
    /// Get stream expression's dependencies.
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }

    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => expression
                .propagate_predicate(StreamExpression::no_fby, |statement| {
                    statement.expression.no_fby()
                }),
            StreamExpressionKind::FollowedBy { .. } => false,
            StreamExpressionKind::NodeApplication { inputs, .. } => {
                inputs.iter().all(|(_, expression)| expression.no_fby())
            }
        }
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => expression
                .propagate_predicate(StreamExpression::no_node_application, |statement| {
                    statement.expression.is_normal_form()
                }),
            StreamExpressionKind::FollowedBy { expression, .. } => expression.no_node_application(),
            StreamExpressionKind::NodeApplication { inputs, .. } => inputs
                .iter()
                .all(|(_, expression)| expression.no_node_application()),
        }
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => expression
                .propagate_predicate(StreamExpression::no_node_application, |statement| {
                    statement.expression.no_node_application()
                }),
            StreamExpressionKind::FollowedBy { expression, .. } => expression.no_node_application(),
            StreamExpressionKind::NodeApplication { .. } => false,
        }
    }
}
