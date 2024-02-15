use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::hir::{dependencies::Dependencies, expression::Expression};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression kind AST.
pub enum StreamExpressionKind {
    /// Expression.
    Expression { expression: Expression },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Constant,
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
}
