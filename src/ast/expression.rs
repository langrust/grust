use crate::util::{location::Location, constant::Constant};

#[derive(Debug, PartialEq)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant{
        /// The constant.
        constant: Constant,
        /// Stream expression location.
        location: Location,
    },
}
