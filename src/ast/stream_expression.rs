use crate::util::{location::Location, constant::Constant};

#[derive(Debug, PartialEq)]
/// LanGRust stream expression AST.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant{
        /// The constant.
        constant: Constant,
        /// Stream expression location.
        location: Location,
    },
    /// Signal call stream expression.
    SignalCall{
        /// Signal identifier.
        id: String,
        /// Stream expression location.
        location: Location,
    },
}
