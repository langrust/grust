use crate::util::{constant::Constant, location::Location, type_system::Type};

#[derive(Debug, PartialEq)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Expression location.
        location: Location,
    },
    /// Call expression.
    Call {
        /// Element identifier.
        id: String,
        /// Expression location.
        location: Location,
    },
    /// Application expression.
    Application {
        /// The expression applied.
        expression: Box<Expression>,
        /// The inputs to the expression.
        inputs: Vec<Expression>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<String>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression with inputs types.
    TypedAbstraction {
        /// The inputs to the abstraction.
        inputs: Vec<(String, Type)>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression location.
        location: Location,
    },
}
