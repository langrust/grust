use crate::ast::expression::ExpressionKind;
use crate::common::location::Location;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression kind AST.
pub enum StreamExpressionKind {
    /// Expression.
    Expression {
        // The expression kind.
        expression: ExpressionKind<StreamExpression>,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Box<StreamExpression>,
        /// The buffered expression.
        expression: Box<StreamExpression>,
    },
}
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression AST.
pub struct StreamExpression {
    /// Stream expression kind.
    pub kind: StreamExpressionKind,
    /// Stream expression location.
    pub location: Location,
}
